# 調査ログ: refactor-to-gpt-app-sdk

## サマリー

ChatGPT Actions（OpenAPI）から OpenAI Apps SDK（MCP）への移行に関する技術調査。既存コードベース分析、Apps SDK 仕様調査、OAuthプロバイダー評価、アーキテクチャパターン検討を実施。

**調査範囲**: フル・ディスカバリー（Complex Integration）

---

## 調査ログ

### トピック1: OpenAI Apps SDK 仕様

**ソース**: developers.openai.com/apps-sdk/, npmjs.com, github.com/modelcontextprotocol/ext-apps

**主要発見事項**:
- `@modelcontextprotocol/sdk` v1.26.0 + `@modelcontextprotocol/ext-apps` v1.0.1 が最新
- MCP サーバーは Streamable HTTP トランスポートで `/mcp` エンドポイントを公開
- ツール結果は3パート構成: `structuredContent`（モデル+ウィジェット）、`content`（モデル専用テキスト）、`_meta`（ウィジェット専用）
- `_meta` 内の `openai/toolInvocation/invoking` / `invoked` は OpenAI 拡張として動作（MCP 標準外だが ChatGPT で機能する）
- UIウィジェットは `text/html;profile=mcp-app` MIMEタイプの MCP リソースとして登録
- `window.openai` ブリッジ API が ChatGPT 互換レイヤー、`App` クラスが MCP 標準クライアント

**設計への影響**: Node.js MCPサーバーをアダプター層として構築し、既存Rustバックエンドへ HTTP 委譲する方式が最適。

### トピック2: OAuth 2.1 要件と Auth0 選定

**ソース**: developers.openai.com/apps-sdk/build/auth/, auth0.com, github.com/openai/openai-mcpkit

**主要発見事項**:
- Apps SDK は OAuth 2.1 + PKCE (S256) + DCR を必須とする
- `/.well-known/oauth-protected-resource` と `/.well-known/oauth-authorization-server` の公開が必要
- Auth0 は `openai-mcpkit` という GitHub scaffold を提供しており、ChatGPT 統合実績が最も豊富
- Stytch も対応ガイドがあるが、Auth0 のほうが DCR サポートが成熟
- 開発フェーズでは `noauth` のみで動作可能（OAuth 実装を後回しにできる）
- リダイレクトURI: 本番 `https://chatgpt.com/connector_platform_oauth_redirect`、レビュー `https://platform.openai.com/apps-manage/oauth`

**設計への影響**: Auth0 を採用。開発初期は `noauth` で進め、Phase 2 で OAuth を統合。

### トピック3: Actions と Apps の共存制約

**ソース**: help.openai.com, skywork.ai/blog

**主要発見事項**:
- ChatGPT Actions と Apps SDK は同一 GPT 内で**相互排他的**
- 既存の Custom GPT（Actions）はそのまま動作し続ける
- 新しい GPT App（MCP）は別個のエントリとして作成される
- 両方を ChatGPT 上で並行して公開可能（ただし別々の GPT/App として）

**設計への影響**: 要件6.1を修正し、別個の GPT として並行運用する方式を採用。

### トピック4: 既存コードベースの MCP 適合性

**ソース**: src/gpt.rs, src/types.rs, src/error.rs, src/main.rs

**主要発見事項**:
- 8つのハンドラすべてが JSON レスポンスを返し、`structuredContent` として直接利用可能
- `message` フィールドが全レスポンスに存在し、MCP `content` テキストに転用可能
- `GptRunServiceResponse.output` は大きくなる可能性があり、`_meta` に移動すべき
- `verify_gpt_api_key` ミドルウェアは MCP→Rust 内部通信認証としてそのまま利用可能
- `resolve_session` は OAuth トークンの `sub` → email → user_id の新フローに置換可能
- `ApiError` は7バリアントのenum で、MCP エラーレスポンスへの変換が明確

**設計への影響**: Rust 側の変更は最小限。MCP→Rust 通信は既存 `GPT_ACTIONS_API_KEY` で認証。

### トピック5: デプロイ戦略

**ソース**: render.com, developers.openai.com/apps-sdk/deploy/

**主要発見事項**:
- Render が既存 Rust バックエンドのデプロイ先であり、Node.js サービスも追加可能
- サーバーレス（Vercel等）はコールドスタートが MCP Streamable HTTP と相性が悪い
- Render の常時起動サービスならレイテンシ問題を回避可能
- App Directory 提出には組織認証（Owner ロール）が必要

**設計への影響**: MCP サーバーも Render にデプロイ。`render.yaml` に新サービスを追加。

---

## アーキテクチャパターン評価

| パターン | 利点 | 欠点 | 採用 |
|---|---|---|---|
| Node.js MCPアダプター + 既存Rust API | 既存ロジック完全保持、SDK公式サポート | 2サービス運用、ネットワークホップ | ✅ 採用 |
| Rust MCP (rmcp crate) 直接実装 | 単一サービス、ネットワーク不要 | ext-apps 非対応、自前実装多い | ❌ |
| Rust API をリファクタリングして MCP 化 | 単一言語 | ビジネスロジック大幅変更、リスク高 | ❌ |

---

## 設計判断記録

| 判断 | 選択 | 理由 |
|---|---|---|
| MCPサーバー言語 | TypeScript (Node.js) | ext-apps SDK 公式サポート、型安全 |
| OAuth プロバイダー | Auth0 | openai-mcpkit scaffold、DCR 成熟、実績豊富 |
| session_token の扱い | ハイブリッド（OAuth優先、フォールバックあり） | 移行期間中の後方互換性確保 |
| MCP→Rust 認証 | 既存 GPT_ACTIONS_API_KEY | 追加実装不要、十分なセキュリティ |
| ウィジェット技術 | バニラ HTML/JS/CSS + Vite バンドル | 非ゴールで React 除外、シンプル |
| デプロイ先 | Render | 既存インフラとの一致、常時起動 |
| 移行方式 | 別個 GPT App として並行運用 | Actions/Apps 共存不可の制約 |

---

## リスク一覧

| リスク | 確率 | 影響 | 緩和策 |
|---|---|---|---|
| Auth0 DCR が ChatGPT の要件を満たさない | 低 | 高 | openai-mcpkit の実績で検証済み。Stytch をバックアップ |
| ext-apps SDK の破壊的変更 | 中 | 中 | v1.0.1 固定、changelog 監視 |
| MCP→Rust 通信のレイテンシ | 低 | 中 | 同一 Render リージョンにデプロイ |
| ウィジェットのモバイル表示崩れ | 中 | 低 | displayMode/maxHeight 活用、レスポンシブデザイン |
