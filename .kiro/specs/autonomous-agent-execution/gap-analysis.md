# Autonomous Agent Execution — ギャップ分析

## 分析概要

本ドキュメントは、`autonomous-agent-execution` 要件と既存コードベースのギャップを分析し、設計フェーズへのインプットを提供する。

---

## 1. 既存資産の評価

### 1.1 再利用可能なコンポーネント（高再利用度）

| コンポーネント | 場所 | 再利用方法 |
|---|---|---|
| **GPTルーターパターン** | `src/main.rs:28-51` `build_gpt_router()` | `/agent/*` ルーターを同パターンで構築。ミドルウェアスタック（認証 + レート制限）がそのまま参考になる |
| **RateLimiter** | `src/gpt.rs:25-85` | トークンバケットアルゴリズム実装済み。ただしグローバル共有なので、エージェントセッション単位への拡張が必要 |
| **APIキー認証ミドルウェア** | `src/gpt.rs:87-117` `verify_gpt_api_key()` | Bearer トークン検証ロジックを流用。エージェント用環境変数 `AGENT_API_KEY` を追加する形 |
| **セッション解決** | `src/gpt.rs:119-132` `resolve_session()` | `gpt_sessions` テーブルのパターンを `agent_sessions` に適用 |
| **サービス検索** | `src/gpt.rs:134-223` `gpt_search_services()` | campaigns + sponsored_apis の統合検索ロジックを拡張（スコアリング、ページネーション追加） |
| **タスク完了フロー** | `src/gpt.rs:409-493` `gpt_complete_task()` | 同意記録 + タスク完了記録のパターンをそのまま流用。メタデータフィールド追加のみ |
| **スポンサー決済フロー** | `src/gpt.rs:495-627` `gpt_run_service()` | キャンペーンマッチング → 予算差引 → 決済記録の全フローが再利用可能 |
| **エラー型** | `src/error.rs` | `ApiError` enum に新バリアントを追加する形で拡張可能。`RateLimited` は既に実装済み |
| **Prometheusメトリクス** | `src/types.rs:82-139` | `IntCounterVec` パターンにエージェント用ラベルを追加 |
| **x402決済ロジック** | `src/utils.rs:83-149`, `src/onchain.rs` | 上流リソースへの決済検証・決済処理がそのまま再利用可能 |

### 1.2 部分的に再利用可能（拡張が必要）

| コンポーネント | 現状 | 必要な拡張 |
|---|---|---|
| **`gpt_sessions` テーブル** | `token`, `user_id`, `created_at`, `expires_at` の4カラム | スコープ（許可タスク種別、予算上限、データ提供範囲）、`agent_id`、`execution_mode` が不足 |
| **`consents` テーブル** | `consent_type` は `data_sharing/contact/retention` の3種 | `granted_via` (agent/human)、`agent_session_id` メタデータが不足 |
| **`task_completions` テーブル** | `id`, `campaign_id`, `user_id`, `task_name`, `details`, `created_at` | `execution_mode` (agent/human)、`agent_id` が不足 |
| **`payments` テーブル** | `tx_hash`, `campaign_id`, `service`, `amount_cents`, `payer`, `source`, `status` | `agent_id`、`agent_session_id` が不足 |
| **エラーレスポンス構造** | `{ error: { code, message, details } }` | `retry_allowed`、`next_actions` フィールドが不足（要件 9.1） |
| **`GptSearchResponse`** | `services`, `total_count`, `message` | `relevance_score`、ページネーション（`offset`/`limit`）、`next_actions` が不足 |

### 1.3 新規実装が必要

| コンポーネント | 要件参照 | 概要 |
|---|---|---|
| **`agent_sessions` テーブル** | 2.2, 2.3 | スコープ付きエージェントセッション管理。`gpt_sessions` の拡張版 |
| **`agent_audit_logs` テーブル** | 8.1, 8.2 | エージェント全操作の監査ログ |
| **`/agent/*` ルーター** | 1.1 | 専用APIルート群（6〜8エンドポイント） |
| **セッションスコープ検証ミドルウェア** | 2.3, 2.4, 4.4 | リクエストごとにスコープとの整合性を検証 |
| **セッション単位レート制限** | 7.1, 7.4, 7.5 | 現在のグローバルレート制限をセッション単位に拡張 |
| **`next_actions` レスポンスビルダー** | 1.3 | フロー状態に応じた次アクションヒント生成 |
| **予算上限チェックロジック** | 5.5, 7.3 | セッション累計決済額の追跡・検証 |
| **重複リクエスト検知** | 7.4 | 短時間内の同一サービスへの重複リクエスト検知 |
| **異常パターン検知** | 7.5 | 急激なリクエスト増加、スコープ外操作の連続試行の検出 |
| **監査履歴APIエンドポイント** | 8.2 | `/agent/audit/history` |
| **フローステータスエンドポイント** | 1.4 | `/agent/flow/status` |

---

## 2. データベーススキーマギャップ

### 2.1 新規テーブル

```
agent_sessions (新規)
├─ id (uuid) PK
├─ token (uuid) UNIQUE, DEFAULT gen_random_uuid()
├─ user_id (uuid) FK→users
├─ agent_id (text) NOT NULL          -- エージェント識別子
├─ scopes (jsonb) NOT NULL           -- { allowed_task_types, allowed_data_types, budget_limit_cents, allowed_categories }
├─ budget_spent_cents (bigint) DEFAULT 0
├─ active (boolean) DEFAULT true
├─ created_at (timestamptz)
├─ expires_at (timestamptz)          -- DEFAULT NOW() + interval '24 hours'
└─ revoked_at (timestamptz)          -- NULL = 有効

agent_audit_logs (新規)
├─ id (uuid) PK
├─ agent_session_id (uuid) FK→agent_sessions
├─ agent_id (text) NOT NULL
├─ user_id (uuid) FK→users
├─ operation (text) NOT NULL         -- 'search', 'get_tasks', 'complete_task', 'run_service', etc.
├─ endpoint (text) NOT NULL
├─ request_summary (jsonb)           -- リクエストの要約（機密情報除く）
├─ response_status (smallint)
├─ created_at (timestamptz)
```

### 2.2 既存テーブルへのカラム追加

```
task_completions:
  + execution_mode (text) DEFAULT 'human'   -- 'human' | 'agent'
  + agent_session_id (uuid) NULLABLE FK→agent_sessions

payments:
  + agent_session_id (uuid) NULLABLE FK→agent_sessions

consents:
  + granted_via (text) DEFAULT 'human'      -- 'human' | 'agent'
  + agent_session_id (uuid) NULLABLE FK→agent_sessions
```

---

## 3. APIエンドポイントギャップ

### 3.1 新規エンドポイント一覧

| メソッド | パス | 要件 | 類似既存エンドポイント |
|---|---|---|---|
| POST | `/agent/auth` | 2.1, 2.2 | `POST /gpt/auth` — スコープ設定を追加 |
| GET | `/agent/services` | 3.1-3.5 | `GET /gpt/services` — スコアリング、ページネーション、嗜好フィルタ追加 |
| GET | `/agent/tasks/{campaign_id}` | 4.1, 4.2 | `GET /gpt/tasks/{campaign_id}` — JSON Schema形式のタスクスキーマ返却 |
| POST | `/agent/tasks/{campaign_id}/complete` | 4.3-4.6 | `POST /gpt/tasks/{campaign_id}/complete` — スコープ検証、メタデータ追加 |
| POST | `/agent/services/{service}/run` | 5.1-5.5 | `POST /gpt/services/{service}/run` — 予算上限チェック、構造化レスポンス |
| GET | `/agent/flow/status` | 1.4 | `GET /gpt/user/status` — フロー全体の進捗を1回で返却 |
| GET | `/agent/audit/history` | 8.2 | 新規 — 操作履歴の取得 |
| DELETE | `/agent/session` | 6.3 | 新規 — セッション無効化 |

### 3.2 レスポンス構造の変更

**現在のGPTレスポンス形式:**
```json
{
  "services": [...],
  "total_count": 5,
  "message": "Found 5 service(s)."
}
```

**エージェントAPIの必要形式（要件 1.2, 1.3）:**
```json
{
  "status": "success",
  "data": {
    "services": [...],
    "total_count": 5,
    "offset": 0,
    "limit": 50
  },
  "next_actions": [
    {
      "action": "get_tasks",
      "endpoint": "/agent/tasks/{campaign_id}",
      "params": { "campaign_id": "..." },
      "description": "タスク詳細を取得"
    }
  ]
}
```

---

## 4. 実装アプローチの選択肢

### オプション A: `/gpt/*` を拡張（共通化アプローチ）

**概要**: 既存の `/gpt/*` ハンドラを抽象化し、GPTとエージェント両方から呼び出せる共通ロジック層を作成。`/agent/*` ルーターは薄いアダプタ層のみ。

**利点**:
- コード重複を最小化
- GPTとエージェントのビジネスロジック整合性を保証
- 修正が一箇所で済む

**欠点**:
- 既存GPTハンドラのリファクタリングが必要
- 抽象化の設計に工数がかかる
- GPT側に影響を与えるリスク

**工数目安**: 中〜大

### オプション B: `/agent/*` を独立実装（並行開発アプローチ）

**概要**: `/agent/*` を `src/agent.rs` として独立モジュールで実装。GPTコードとは独立に開発し、共通部分は `utils.rs` の関数を利用。

**利点**:
- GPT側への影響ゼロ
- 独立して開発・テスト可能
- エージェント固有の要件（スコープ、監査、`next_actions`）を自由に実装

**欠点**:
- ビジネスロジック（キャンペーンマッチング、決済フロー）の部分的重複
- 将来的な保守コストが増加

**工数目安**: 中

### オプション C: ハイブリッド（推奨）

**概要**: コアロジック（キャンペーンマッチング、決済、タスク完了）を `utils.rs` に抽出し、`/gpt/*` と `/agent/*` がそれぞれのアダプタ層から呼び出す。

**利点**:
- コアロジックの一元化
- エージェント固有のレスポンス形式を自由に設計
- GPT側への影響は最小限（ロジック抽出のみ）
- テストはコアロジック単位で記述可能

**欠点**:
- 初期のリファクタリング工数
- コアロジックのインターフェース設計が重要

**工数目安**: 中

**推奨理由**: 既存の `utils.rs` には `user_matches_campaign()`, `has_completed_task()`, `verify_x402_payment()` 等のコア関数が既にあり、この方向性と整合する。

---

## 5. 技術的課題と調査事項

### 5.1 セッション単位レート制限

**現状**: `RateLimiter` はグローバル共有（`Arc<Mutex<RateLimiter>>`）で `/gpt/*` 全体に1つ。

**課題**: 要件 7.1 はセッション単位で毎分30リクエスト。`HashMap<Uuid, RateLimiter>` でセッションごとのバケットを管理する必要がある。メモリリークを防ぐため、期限切れセッションのバケットを定期的にクリーンアップする仕組みが必要。

**アプローチ**: `DashMap<Uuid, RateLimiter>` または `Arc<RwLock<HashMap<...>>>` + バックグラウンドクリーナータスク。

### 5.2 JSON Schema によるタスク入力バリデーション

**現状**: `campaigns.task_schema` は `jsonb` カラムとして存在するが、サーバー側でのバリデーションは行っていない（GPTハンドラではスキーマを返すのみ）。

**課題**: 要件 4.2, 4.3 ではエージェントが送信したデータをJSON Schemaに対してバリデーションする必要がある。

**アプローチ**: `jsonschema` crateの利用を検討。Cargo.toml に依存追加が必要。

### 5.3 重複リクエスト検知

**現状**: 重複検知の仕組みは存在しない。

**課題**: 要件 7.4 では短時間内の同一サービスへの重複リクエストを拒否する必要がある。

**アプローチ**: セッション単位で `(service, timestamp)` の最近リクエストをインメモリキャッシュに保持。`ttl_cache` 的なパターン。

### 5.4 異常パターン検知

**現状**: なし。

**課題**: 要件 7.5 は高度な要件。MVP段階では簡易ルール（N分間にスコープ外操作がM回連続 → セッション一時停止）で対応し、将来的に精緻化。

### 5.5 `next_actions` の動的生成

**現状**: レスポンスに次アクションのヒントは含まれていない。

**課題**: 要件 1.3 ではフロー状態に応じた `next_actions` を動的生成する必要がある。

**アプローチ**: フロー状態マシン（未認証 → 認証済み → サービス選択 → タスク完了 → サービス実行可能）を定義し、状態に応じたアクションリストを返す。

---

## 6. マイグレーション影響評価

| 変更 | 影響範囲 | リスク |
|---|---|---|
| `agent_sessions` テーブル新規作成 | 新規のみ | 低 |
| `agent_audit_logs` テーブル新規作成 | 新規のみ | 低 |
| `task_completions` にカラム追加 | 既存データは `DEFAULT 'human'` で埋まる | 低 |
| `payments` にカラム追加 | 既存データは `NULL` | 低 |
| `consents` にカラム追加 | 既存データは `DEFAULT 'human'` / `NULL` | 低 |

全マイグレーションが後方互換であり、既存機能への影響は最小限。

---

## 7. 依存関係の追加候補

| crate | 用途 | 理由 |
|---|---|---|
| `jsonschema` | タスク入力バリデーション（要件 4.2, 4.3） | JSON Schemaに対するバリデーション |
| `dashmap` | セッション単位レート制限（要件 7.1） | 並行アクセス対応HashMap（オプション） |

いずれも軽量で、メンテナンスが活発な crate。

---

## 8. サマリー

### 主要ギャップ

1. **エージェントセッション管理**: スコープ・予算上限付きセッションが未実装（DB + API両方）
2. **レスポンス構造**: `next_actions` による自律ナビゲーション機能が不在
3. **監査ログ**: エージェント操作の追跡可能性がゼロ
4. **セッション単位制御**: レート制限、重複検知、異常検知がすべてセッション非対応
5. **JSON Schemaバリデーション**: タスク入力のサーバー側検証が未実装

### 強み（既存資産）

1. **GPTルーターパターン**: `/agent/*` ルーターの雛形として直接利用可能
2. **コアビジネスロジック**: キャンペーンマッチング、決済フロー、タスク完了が `utils.rs` に部分的に抽出済み
3. **エラーハンドリング**: `ApiError` enum が拡張しやすい構造
4. **DB設計**: 既存スキーマが拡張に適した設計（FKベース、NULLABLE追加が容易）
5. **同意管理**: `consents` テーブルとフローが既に存在

### 推奨実装戦略

**ハイブリッドアプローチ（オプション C）** を推奨。コアロジックを共通化しつつ、`src/agent.rs` としてエージェント固有のハンドラ・型・ミドルウェアを独立実装する。

---
