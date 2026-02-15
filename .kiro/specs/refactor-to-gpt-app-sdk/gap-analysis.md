# ギャップ分析: refactor-to-gpt-app-sdk

## エグゼクティブサマリー

既存のChatGPT Actions実装（Rust/Axum + OpenAPI）からOpenAI Apps SDK（MCPプロトコル）への移行について、コードベース調査とApps SDK技術調査を実施した。

### 主要発見事項

1. **既存実装は十分に成熟**: `src/gpt.rs`（1,165行）に8つのハンドラ、20以上の型定義、5つのDBマイグレーションが実装済み。ビジネスロジックの書き換えは不要
2. **MCPアダプター層は新規構築が必要**: Node.js MCPサーバー（`@modelcontextprotocol/sdk` + `@modelcontextprotocol/ext-apps`）のプロジェクト構造が皆無
3. **OAuth 2.1は最大のギャップ**: 現在の単純なAPIキー認証からOAuth 2.1 + PKCE + DCRへの移行は、第三者OAuthプロバイダー（Auth0/Stytch）の導入が必要
4. **重要な制約発見**: ChatGPT Actions と Apps SDK は同一GPT内で**共存不可**。移行期間中は別個のGPT（Custom GPT + App）として並行運用する必要がある
5. **UIウィジェットは完全新規**: リッチUI機能は現在存在せず、バニラHTML/JSで3種のウィジェットを新規作成する必要がある

---

## 1. 既存実装の詳細マッピング

### 1.1 ハンドラ ↔ MCPツール対応表

| # | 既存ハンドラ | MCPツール名 | メソッド/パス | annotations | 備考 |
|---|---|---|---|---|---|
| 1 | `gpt_search_services` | `search_services` | `GET /gpt/services` | `readOnlyHint: true` | 高度なフィルタリング（budget, intent, preferences）+スコアリング実装済み |
| 2 | `gpt_auth` | `authenticate_user` | `POST /gpt/auth` | `readOnlyHint: false` | ユーザー作成+セッション発行。OAuth移行で大幅変更が必要 |
| 3 | `gpt_get_tasks` | `get_task_details` | `GET /gpt/tasks/{id}` | `readOnlyHint: true` | task_schema JSONBからの動的フォーマット構築 |
| 4 | `gpt_complete_task` | `complete_task` | `POST /gpt/tasks/{id}/complete` | `readOnlyHint: false` | 同意3種類の記録+タスク完了 |
| 5 | `gpt_run_service` | `run_service` | `POST /gpt/services/{service}/run` | `openWorldHint: true` | キャンペーンマッチング→予算減算→決済記録→サービス実行 |
| 6 | `gpt_user_status` | `get_user_status` | `GET /gpt/user/status` | `readOnlyHint: true` | 完了タスク+利用可能サービス一覧 |
| 7 | `gpt_get_preferences` | `get_preferences` | `GET /gpt/preferences` | `readOnlyHint: true` | preferred/neutral/avoided |
| 8 | `gpt_set_preferences` | `set_preferences` | `POST /gpt/preferences` | `readOnlyHint: false` | DELETE→INSERT で全置換 |

### 1.2 レスポンス型の再利用性

既存のレスポンス型はすべて `Serialize` を実装しており、MCP `structuredContent` として**そのまま再利用可能**:

| 型 | フィールド数 | structuredContent適合度 | 備考 |
|---|---|---|---|
| `GptSearchResponse` | 5 | ✅ 高 | `services`, `total_count`, `message`, `applied_filters`, `available_categories` |
| `GptAuthResponse` | 5 | ✅ 高 | `session_token`, `user_id`, `email`, `is_new_user`, `message` |
| `GptTaskResponse` | 9 | ✅ 高 | `task_input_format` がネスト構造 |
| `GptCompleteTaskResponse` | 5 | ✅ 高 | フラットなJSON |
| `GptRunServiceResponse` | 6 | ✅ 高 | `output` が大きくなる可能性 → `_meta` に移動検討 |
| `GptUserStatusResponse` | 5 | ⚠️ 中 | `completed_tasks`, `available_services` 配列が大きくなる可能性 |
| `GptPreferencesResponse` | 4 | ✅ 高 | フラットなJSON |
| `GptSetPreferencesResponse` | 4 | ✅ 高 | フラットなJSON |

### 1.3 認証・セッション管理の現状

```
現在のフロー:
[ChatGPT] → Authorization: Bearer {GPT_ACTIONS_API_KEY} → [Axum /gpt/*]
                                                           ↓
                                                     verify_gpt_api_key ミドルウェア
                                                           ↓
                                                     各ハンドラで resolve_session(session_token)
                                                           ↓
                                                     user_id を取得してビジネスロジック実行

問題点:
- GPT_ACTIONS_API_KEY は全ユーザー共通（GPT全体で1つ）
- session_token はリクエストボディ/クエリで手動渡し
- OAuth 2.1 では個別ユーザーのトークンが発行される → session_token の仕組みが変わる
```

### 1.4 データベーススキーマの現状

| テーブル | 目的 | 移行影響 |
|---|---|---|
| `users` | ユーザー管理（`source` カラムで `gpt_apps` 識別） | ✅ 変更不要 |
| `campaigns` | キャンペーン管理（`task_schema` JSONBカラム含む） | ✅ 変更不要 |
| `task_completions` | タスク完了記録 | ✅ 変更不要 |
| `payments` | 支払い記録 | ✅ 変更不要 |
| `consents` | 同意記録（data_sharing, contact, retention） | ✅ 変更不要 |
| `gpt_sessions` | セッショントークン管理（30日有効期限） | ⚠️ OAuth移行後は不要になる可能性 |
| `user_task_preferences` | タスク設定（preferred/neutral/avoided） | ✅ 変更不要 |
| `sponsored_apis` | Sponsored API管理 | ✅ 変更不要 |

---

## 2. 要件別ギャップ分析

### 要件1: MCPサーバー基盤

| 項目 | 現状 | ギャップ | 対応難易度 |
|---|---|---|---|
| Node.js プロジェクト | ❌ 存在しない | Node.js MCPサーバープロジェクトを新規作成 | 中 |
| `@modelcontextprotocol/sdk` | ❌ 未導入 | npm install で導入（v1.26.0） | 低 |
| `@modelcontextprotocol/ext-apps` | ❌ 未導入 | npm install で導入（v1.0.1） | 低 |
| Streamable HTTP トランスポート | ❌ 未実装 | Express + `StreamableHTTPServerTransport` で実装 | 中 |
| `/mcp` エンドポイント | ❌ 未実装 | Express ルートとして追加 | 低 |
| Rust→MCP通信 | ❌ 未実装 | fetch/axios で既存 `/gpt/*` エンドポイントを呼び出し | 低 |
| CORS | ✅ 既存（`*` or カスタム） | `chatgpt.com`, `cdn.oaistatic.com` を追加 | 低 |

**推奨アプローチ**: プロジェクトルートに `mcp-server/` ディレクトリを作成し、独立したNode.jsプロジェクトとして構築。

**必要パッケージ**:
```json
{
  "dependencies": {
    "@modelcontextprotocol/sdk": "^1.26.0",
    "@modelcontextprotocol/ext-apps": "^1.0.1",
    "express": "^4.21.0",
    "cors": "^2.8.5",
    "zod": "^3.25.0"
  }
}
```

### 要件2: MCPツール定義

| 項目 | 現状 | ギャップ | 対応難易度 |
|---|---|---|---|
| `registerAppTool` によるツール登録 | ❌ 未実装 | 8ツールの登録コードを作成 | 中 |
| Zodスキーマ定義 | ❌ 未実装 | 既存のRust型からZodスキーマを転写 | 低 |
| `structuredContent` / `content` / `_meta` 分離 | ❌ 未実装 | レスポンス変換ロジックを実装 | 中 |
| Tool annotations | ❌ 未実装 | 8ツールに適切なアノテーションを設定 | 低 |
| ツール呼び出しステータス | ❌ 未実装 | `_meta` に `invoking`/`invoked` メッセージ追加 | 低 |

**既存資産の活用ポイント**:
- Rust側の型定義（`GptSearchParams`, `GptSearchResponse` 等）をZodスキーマに1:1で変換可能
- 各ハンドラのJSONレスポンスをそのまま `structuredContent` として利用可能
- `message` フィールドを `content` テキストに転用可能
- 大きなデータ（`GptRunServiceResponse.output` 等）を `_meta` に移動

### 要件3: 認証・認可の移行

| 項目 | 現状 | ギャップ | 対応難易度 |
|---|---|---|---|
| OAuth 2.1 認可サーバー | ❌ 未実装 | 第三者プロバイダー（Auth0/Stytch）の導入 | **高** |
| `/.well-known/oauth-protected-resource` | ❌ 未実装 | JSONメタデータエンドポイント追加 | 低 |
| `/.well-known/oauth-authorization-server` | ❌ 未実装 | OAuthプロバイダーの設定に依存 | 中 |
| Dynamic Client Registration (DCR) | ❌ 未実装 | OAuthプロバイダーの機能に依存 | 中 |
| PKCE (S256) | ❌ 未実装 | OAuthプロバイダーの機能に依存 | 中 |
| トークン検証 | ❌ 未実装 | JWKS検証+スコープチェック | 中 |
| `noauth` ツールサポート | ❌ 未実装 | `search_services` のみ認証不要に設定 | 低 |
| session_token → OAuth統合 | ⚠️ 要設計 | OAuthトークンからuser_idを解決する新しいフロー | **高** |

**重要な設計判断ポイント**:

1. **OAuthプロバイダー選択**:
   - **Option A: Auth0** — GitHub scaffold (`openai-mcpkit`) が存在。DCR/PKCEサポート済み
   - **Option B: Stytch** — Apps SDK専用のガイドあり。比較的新しい
   - **Option C: 自前実装** — 開発コスト大。非推奨

2. **session_token の扱い**:
   - **Option A: 廃止** — OAuthトークンの `sub` クレームから直接user_idを解決。`gpt_sessions` テーブル不要に
   - **Option B: 維持** — OAuthトークン検証後に既存の `resolve_session` を呼び出し。後方互換性高
   - **Option C: ハイブリッド** — OAuth認証済みリクエストはOAuthトークンから、未認証リクエストはsession_tokenから

3. **開発フェーズの認証**:
   - Apps SDK は開発中に認証なしでの動作が可能（`noauth` のみ）
   - 本番では OAuth 2.1 必須（APIキー認証の代替は存在しない）

### 要件4: UIウィジェット

| 項目 | 現状 | ギャップ | 対応難易度 |
|---|---|---|---|
| サービス検索結果ウィジェット | ❌ 未実装 | HTML/JS/CSSで新規作成 | 中 |
| タスク完了フォームウィジェット | ❌ 未実装 | HTML/JS/CSSで新規作成、同意チェックボックス含む | 中 |
| ユーザーステータスダッシュボード | ❌ 未実装 | HTML/JS/CSSで新規作成 | 中 |
| `registerAppResource` | ❌ 未実装 | MIMEタイプ `text/html;profile=mcp-app` で登録 | 低 |
| `window.openai` ブリッジ対応 | ❌ 未実装 | `toolOutput`, `callTool`, `setWidgetState` 使用 | 中 |
| ウィジェットバンドル | ❌ 未実装 | Vite + `vite-plugin-singlefile` でインライン化 | 低 |

**既存フロントエンド資産**:
- `frontend/` に React + Vite プロジェクトが存在するが、これはWebフロントエンド用
- ウィジェットはサンドボックスiframe内で動作するため、別のHTML/JSファイルとして作成する必要がある
- `vite-plugin-singlefile` でCSS/JSをインラインバンドルし、`registerAppResource` で配信

### 要件5: 既存コードのリファクタリング

| 項目 | 現状 | ギャップ | 対応難易度 |
|---|---|---|---|
| `/gpt/*` エンドポイント維持 | ✅ 動作中 | 変更不要 | なし |
| `openapi.yaml` 削除 | ✅ 存在 | 移行完了後に削除 | 低 |
| `verify_gpt_api_key` 転用 | ✅ 実装済み | MCP→Rust内部通信用に維持 or 削除 | 低 |
| レスポンス型の互換性 | ✅ Serialize済み | 変更不要 | なし |

**影響が小さい**: Rust側はほぼ変更不要。MCP→Rust通信の認証方式のみ決定が必要。

### 要件6: 後方互換性・移行戦略

| 項目 | 現状 | ギャップ | 対応難易度 |
|---|---|---|---|
| Actions + Apps 並行運用 | ❌ | **重要: 同一GPT内で共存不可** | 要設計変更 |
| 非GPTエンドポイント維持 | ✅ 変更なし | ギャップなし | なし |
| DBスキーマ維持 | ✅ 変更なし | ギャップなし | なし |
| 移行手順書 | ❌ 未作成 | 移行完了後に作成 | 低 |

**⚠️ 要件6.1への重大な影響**:

調査の結果、ChatGPT Actions と Apps SDK は同一GPT内で**相互排他的**であることが判明した。要件6.1「移行期間中は両方が並行して動作しなければならない」を満たすには:

- **Option A: 別個のGPTとして運用** — 既存Custom GPT（Actions）を維持しつつ、新しいApp（MCP）を作成。ユーザーは段階的に移行
- **Option B: 段階的切り替え** — 開発完了後に一括でActions → Appに切り替え。ダウンタイム最小化
- **Option C: Rust API直接利用** — Actions GPTはそのまま、新しいAppは `/gpt/*` 経由ではなく同じビジネスロジックを直接呼び出し

### 要件7: テスト・検証

| 項目 | 現状 | ギャップ | 対応難易度 |
|---|---|---|---|
| MCPツールユニットテスト | ❌ 未実装 | Jest/Vitest でMCPツールのテスト作成 | 中 |
| MCP→Rust統合テスト | ❌ 未実装 | HTTPモックまたは実際のRustサーバーに対するテスト | 中 |
| E2Eテスト | ❌ 未実装 | MCP Inspector + 手動テスト | 中 |
| Rust既存テスト | ✅ `cargo test` | 変更なし | なし |
| ウィジェット手動テスト | ❌ 未実装 | ngrok + ChatGPT開発者モード | 低 |

**テストツール**:
- `npx @modelcontextprotocol/inspector@latest` — MCP Inspectorでローカルテスト
- ngrok — ChatGPT開発者モードへの接続

### 要件8: デプロイ・運用

| 項目 | 現状 | ギャップ | 対応難易度 |
|---|---|---|---|
| MCPサーバーデプロイ | ❌ 未実装 | Render/Vercel/Fly.io いずれかに追加 | 中 |
| `.env.example` 更新 | ✅ 既存 | OAuth設定、バックエンドURL等を追加 | 低 |
| App Directoryメタデータ | ❌ 未準備 | アプリ名、説明、アイコン、プライバシーURL | 低 |
| ヘルスチェック | ❌ 未実装 | Express ルートとして追加 | 低 |
| 構造化ログ | ❌ 未実装 | Node.js側にログライブラリ導入 | 低 |

**デプロイ構成案**:
```
[ChatGPT] ←→ [MCP Server (Render/Vercel)]
                    ↓ HTTP
              [Rust/Axum Backend (Render)] ←→ [PostgreSQL]
```

---

## 3. リスクと不確実性

### 高リスク

| リスク | 影響 | 緩和策 |
|---|---|---|
| OAuth 2.1 の実装複雑性 | 認証フロー全体の再設計が必要 | Auth0/Stytch等の第三者プロバイダーで複雑性を軽減 |
| Actions/Apps 共存不可 | 移行中のサービス断絶リスク | 別個のGPTとして並行運用 |
| `_meta` の `toolInvocation/invoking` 非対応 | MCPスタンダードでは未対応（OpenAI拡張） | `window.openai` 互換レイヤーで対応可能性あり。設計フェーズで詳細調査 |

### 中リスク

| リスク | 影響 | 緩和策 |
|---|---|---|
| MCP サーバーのコールドスタート | サーバーレス環境でのレイテンシ | 常時起動の環境（Render, Fly.io）を選択 |
| ウィジェットのモバイル対応 | レイアウト崩れ | `displayMode` と `maxHeight` を活用した適応的デザイン |
| ext-apps SDK の安定性 | v1.0.1 — 初期バージョン | API変更への追従コストを考慮 |

### 要調査事項（設計フェーズで解決）

1. **Auth0 vs Stytch の選定**: DCR対応、コスト、ChatGPT統合実績の比較
2. **OAuthトークン→user_id マッピング**: 既存の `users` テーブルとの統合方法
3. **`toolInvocation` ステータスメッセージ**: MCP標準での対応状況の最新確認
4. **ウィジェットCSP設定**: `connectDomains` にRustバックエンドURLを含める必要性
5. **App Directory 審査基準**: アノテーション精度の要件詳細

---

## 4. 推奨アーキテクチャ概要

```
┌─────────────────────────────────────────────────────────┐
│                     ChatGPT                              │
│  ┌────────────────────────────────────────────────────┐  │
│  │              GPT App (MCP Client)                  │  │
│  │   ┌──────────────────┐  ┌───────────────────────┐ │  │
│  │   │  UIウィジェット   │  │  AIモデル             │ │  │
│  │   │  (iframe)        │  │  (ツール呼び出し判断)   │ │  │
│  │   └────────┬─────────┘  └───────────┬───────────┘ │  │
│  └────────────┼────────────────────────┼─────────────┘  │
│               │ window.openai          │ MCP Protocol    │
└───────────────┼────────────────────────┼────────────────┘
                │                        │
                │    ┌───────────────────┼───────────────┐
                │    │  Streamable HTTP  │               │
                ▼    ▼                   ▼               │
┌───────────────────────────────────────────────────────┐│
│            Node.js MCP Server (新規)                   ││
│                                                        ││
│  ┌────────────────────────────────────────────────┐   ││
│  │  OAuth 2.1 Token Verification                  │   ││
│  │  (Auth0/Stytch → user_id 解決)                 │   ││
│  └────────────────────────────────────────────────┘   ││
│                                                        ││
│  ┌────────────────────────────────────────────────┐   ││
│  │  MCP Tools (8ツール)                           │   ││
│  │  search_services, authenticate_user, ...       │   ││
│  │  → HTTP fetch → Rust Backend /gpt/*            │   ││
│  └────────────────────────────────────────────────┘   ││
│                                                        ││
│  ┌────────────────────────────────────────────────┐   ││
│  │  MCP Resources (3ウィジェット)                  │   ││
│  │  services-list, task-form, user-dashboard      │   ││
│  └────────────────────────────────────────────────┘   ││
└───────────────────────────────────────────────────────┘│
                │                                        │
                │ HTTP (内部通信)                         │
                ▼                                        │
┌───────────────────────────────────────────────────────┐│
│         Rust/Axum Backend (既存・変更最小)              ││
│                                                        ││
│  /gpt/* エンドポイント群 (既存ハンドラ8本)              ││
│  verify_gpt_api_key → 内部通信認証に転用              ││
│  resolve_session → OAuth統合 or 維持                  ││
│                                                        ││
│  /campaigns, /tasks/complete, /proxy/* (変更なし)      ││
└───────────────────────┬───────────────────────────────┘│
                        │                                │
                        ▼                                │
              ┌──────────────────┐                       │
              │   PostgreSQL     │                       │
              │   (変更なし)      │                       │
              └──────────────────┘                       │
```

---

## 5. 作業量見積もり

| カテゴリ | 新規ファイル数 | 変更ファイル数 | 難易度 | 依存関係 |
|---|---|---|---|---|
| MCPサーバー基盤 | ~5 | 0 | 中 | なし |
| MCPツール定義 | ~2 | 0 | 中 | MCPサーバー基盤 |
| OAuth 2.1 | ~3 | ~2 | 高 | 第三者プロバイダー選定 |
| UIウィジェット | ~4 | 0 | 中 | MCPツール定義 |
| Rustリファクタリング | 0 | ~2 | 低 | OAuth方式決定 |
| テスト | ~3 | 0 | 中 | 全体完了後 |
| デプロイ設定 | ~3 | ~1 | 低 | 全体完了後 |
| **合計** | **~20** | **~5** | - | - |

---

## 6. 要件への影響・修正提案

### 要件6.1の修正提案

**現在の要件**:
> 移行期間中は、既存のChatGPT Actions（OpenAPI）とApps SDK（MCP）の両方が並行して動作しなければならない

**修正案**:
> 移行期間中は、既存のCustom GPT（ChatGPT Actions）と新規のGPT App（Apps SDK）を**別個のGPTとして**並行運用し、ユーザーが段階的に移行できるようにしなければならない

**理由**: ChatGPT Actions と Apps SDK は同一GPT内で共存できないため、別個のGPTとしての並行運用に修正。

### 要件3（認証）への補足提案

OAuth 2.1 の実装に関して、以下を検討事項として追加:

- **3.8**: 開発フェーズでは、認証なし（`noauth`）のみでMCPサーバーが動作可能であること（EARS: State-driven）
- **3.9**: OAuthプロバイダーとして、Auth0 または Stytch のいずれかを採用し、自前実装は行わないこと（EARS: Ubiquitous）

---

## 7. 設計フェーズへの入力事項

設計フェーズで決定すべき項目:

1. **OAuthプロバイダーの選定**（Auth0 vs Stytch）
2. **MCPサーバーのデプロイ先**（Render vs Vercel vs Fly.io）
3. **session_tokenの扱い**（廃止 vs 維持 vs ハイブリッド）
4. **MCP→Rust内部通信の認証方式**（APIキー維持 vs 新方式）
5. **ウィジェットの技術スタック**（バニラHTML/JS vs 軽量フレームワーク）
6. **移行の段階分け**（Phase 1: MCP基盤 → Phase 2: OAuth → Phase 3: ウィジェット）
