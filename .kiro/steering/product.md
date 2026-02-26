# Product Context — SnapFuel (Payload Exchange Extended)

## ビジョン

x402 プロトコルの 402 Paywall をプロキシ経由でインターセプトし、スポンサーがユーザーのタスク実行やデータ提供と引き換えに支払いを肩代わりする仕組みを提供する。スポンサーはキャンペーンを作成・配信でき、ユーザーは主要な AI / 開発者 UI を通じて自然にこの仕組みを利用できる。

## コアコンセプト

| 概念 | 説明 |
|---|---|
| **Resource** | x402 で保護された上流の有料エンドポイント |
| **Proxy** | 402 レスポンスをインターセプトし、Paywall とタスクフローを提示 |
| **Sponsor** | 支払いを肩代わりするエンティティ（企業、将来的にはエージェントも） |
| **Campaign** | スポンサーが作成する募集単位（ターゲット、目的、予算、タスク、データ要求、同意条件） |
| **Offer** | リソースレベルでのスポンサー条件（割引率、上限、必要タスク、収集データ） |
| **Action Plugin** | タスクやデータ収集を追加するための拡張プラグインレイヤー |
| **Consent Vault** | 明示的同意、利用目的、保持期間、連絡許可を管理するレイヤー |
| **MCP Server** | GPT Apps / MCP からの呼び出しを受ける仲介サーバー |

## 優先度

- **P0 (Must)**: x402 Proxy → Paywall → Action → Sponsor Payment → Resource Delivery のE2Eフロー
- **P1 (Core)**: Campaign Builder (ToB) + Service Discovery / Profile Vault (ToC)
- **P2 (Scale)**: 推薦エンジン、不正対策、マルチクライアント SDK、Analytics

## マイルストーン

| マイルストーン | スコープ |
|---|---|
| M0 | E2Eフロー + 上流互換性 (P0) |
| M1 | サービス検索、スポンサー可視化 (P1 ToC 前半) |
| M2 | Campaign Builder Chat、公開、Data Inbox (P1 ToB) |
| M3 | Profile Vault + Consent 完成 (P1 運用要件) |
| M4 | 通知 + マルチクライアント API/SDK (P1 後半) |

## 現在のフェーズ

MVP / プロトタイプ段階。P0 の E2E フローは実装済み。P1 の中でも GPT Apps 向けの MCP サーバー統合、OAuth 対応、ウィジェット UI、嗜好ベースのサービス提案、zkPassport 検証フローまでを実装済みで、運用強化と拡張フェーズに入っている。

## アクティブ仕様（.kiro/specs）

| feature | phase | language | updated_at |
|---|---|---|---|
| `gpt-apps-integration` | `tasks-generated` | `ja` | 2026-02-11T13:07:00Z |
| `smart-service-suggestion` | `tasks-generated` | `ja` | 2026-02-14T01:44:00Z |
| `autonomous-agent-execution` | `tasks-generated` | `ja` | 2026-02-14T11:00:00Z |
| `refactor-to-gpt-app-sdk` | `tasks-generated` | `ja` | 2026-02-15T03:00:00Z |

## ターゲットユーザー

- **スポンサー (ToB)**: x402 対応サービスへのアクセスを提供し、ユーザーデータ/タスク完了を取得したい企業
- **エンドユーザー (ToC)**: ChatGPT / Claude / 開発者ツールから x402 リソースにアクセスしたいユーザー

## 非ゴール（初期フェーズ）

- フル KYC / 重い本人確認
- 初期段階からの高度な推薦モデル（ルール/タグから開始）
- 全クライアントへのネイティブ UI 統合（MCP + HTTP で吸収）
- 重いデータインフラ統合（エクスポート + 監査ログから開始）

## Sync Notes（2026-02-25）

- GPT Apps のタスク完了フローでは、同意チェックに加えて「プロダクトフィードバック入力」を必須化する実装方針を採用。
- フィードバックは「プロダクトリンク」「5段階評価」「評価タグ（複数）」「理由テキスト」を構造化して `details` に格納する。
- 今後のタスク種別追加時も、同意取得とフィードバック取得を分離し、後方互換（既存 `complete_task` 契約）を維持する。
