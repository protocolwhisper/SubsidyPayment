# Autonomous Agent Execution — リサーチログ

## サマリー

### ディスカバリスコープ
拡張型（Extension）ディスカバリを実施。既存のGPT Apps統合（`/gpt/*`）パターンを基盤とし、自律型エージェント固有の要件（スコープ付きセッション、監査、`next_actions`、セッション単位制御）に焦点を当てた。

### 主要な発見

1. **既存GPTルーターパターンが直接再利用可能**: `build_gpt_router()` パターン（認証 + レート制限ミドルウェアスタック）をそのまま `/agent/*` に適用可能
2. **コアビジネスロジックの共通化が容易**: `utils.rs` に `user_matches_campaign()`, `has_completed_task()`, `verify_x402_payment()` が既に抽出されており、ハイブリッドアプローチの基盤が整っている
3. **DBスキーマ拡張は低リスク**: 新規テーブル2つ + NULLABLEカラム追加で後方互換を完全維持

---

## リサーチログ

### トピック1: セッション単位レート制限

**調査内容**: グローバル `RateLimiter`（`src/gpt.rs:25-65`）をセッション単位に拡張する方法

**発見**:
- 現在の `Arc<Mutex<RateLimiter>>` はグローバル共有で `/gpt/*` 全体に1つ
- セッション単位にするには `HashMap<Uuid, RateLimiter>` が必要
- `DashMap` crate は並行HashMap だが、標準の `Arc<RwLock<HashMap>>` でも十分（エージェントセッション数は数十〜数百規模）
- 期限切れエントリのクリーンアップにはバックグラウンドタスクが必要

**結論**: 初期実装は `Arc<RwLock<HashMap<Uuid, RateLimiter>>>` で十分。バックグラウンドクリーナーは `tokio::spawn` + `tokio::time::interval` で実装。

### トピック2: JSON Schemaバリデーション

**調査内容**: Rustでのタスク入力バリデーション用JSON Schema crate

**発見**:
- `jsonschema` crate（v0.28+）が最も広く使用されている
- Draft 2020-12をサポート
- `jsonschema::is_valid(&schema, &instance)` でシンプルにバリデーション可能
- エラー詳細は `jsonschema::validate(&schema, &instance)` で取得可能
- `campaigns.task_schema` カラムが既に `jsonb` で存在

**結論**: `jsonschema` crate を `Cargo.toml` に追加。タスク完了ハンドラ内で入力データをバリデーション。

### トピック3: `next_actions` レスポンスパターン

**調査内容**: エージェントフレンドリーなAPIレスポンス設計

**発見**:
- HATEOAS（Hypermedia as the Engine of Application State）パターンが適用可能
- フロー状態に応じて次アクションを動的生成する方式が一般的
- 状態マシン: `unauthenticated → authenticated → service_selected → task_completed → service_ready`
- 各状態で利用可能なアクションを `next_actions` 配列に含める

**結論**: フロー状態をセッション + DB状態から推論し、`AgentNextAction` 型で次アクションリストを動的生成。

### トピック4: 重複リクエスト検知

**調査内容**: 短時間の重複リクエスト防止パターン

**発見**:
- Idempotency Key パターン（`Idempotency-Key` ヘッダー）が一般的
- 代替案: セッション + サービス + タイムウィンドウの組み合わせでインメモリキャッシュ
- TTLキャッシュ: `HashMap<(Uuid, String), Instant>` で最近のリクエストを追跡

**結論**: `Idempotency-Key` ヘッダーを推奨。サーバー側でキー重複を検知し、409 Conflict を返却。補助的にタイムウィンドウベースの検知も実装。

### トピック5: 異常パターン検知

**調査内容**: エージェントセッションの異常行動検知

**発見**:
- MVP段階ではルールベースで十分
- 検知ルール候補:
  - 5分間にスコープ外操作が3回以上 → セッション一時停止
  - 1分間にレート制限到達が3回以上 → セッション一時停止
- セッション一時停止 = `agent_sessions.active = false` に更新
- 復旧はユーザーが手動でセッションを再作成

**結論**: MVP段階はカウンターベースの簡易ルール。`agent_sessions` に `violation_count` カラムを追加し、閾値超過でセッションを停止。

---

## アーキテクチャパターン評価

| パターン | 適合度 | 理由 |
|---|---|---|
| A: GPT拡張（共通化） | △ | リファクタリングリスクが高い |
| B: 独立実装 | ○ | GPTへの影響ゼロだがロジック重複 |
| **C: ハイブリッド** | **◎** | コア共通化 + 独立モジュール。既存 `utils.rs` パターンと整合 |

**採用**: オプション C（ハイブリッド）

---

## リスクと軽減策

| リスク | 影響 | 軽減策 |
|---|---|---|
| セッション単位レート制限のメモリ増加 | 低 | バックグラウンドクリーナーで期限切れエントリを定期削除 |
| JSON Schemaバリデーションのパフォーマンス | 低 | スキーマのコンパイル結果をキャッシュ |
| 監査ログのDB負荷 | 中 | インデックス最適化、将来的にはバッチ挿入を検討 |
| `next_actions` の複雑化 | 低 | 状態マシンをシンプルに保ち、アクション数を最小限に |

---
