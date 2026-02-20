# Project Overview — SubsidyPayment（更新: 2026-02-20）

## 目的
x402 の HTTP 402 ペイウォールをプロキシで仲介し、
- スポンサー補助（タスク実行/データ提供との交換）
- 直接支払い（フォールバック）
の両方で上流有料APIへアクセス可能にする。

## 現在の主要コンポーネント
- Rust バックエンド（`src/`）
  - Core API（`/campaigns`, `/proxy/{service}/run`, `/sponsored-apis` など）
  - GPT API（`/gpt/services`, `/gpt/auth`, `/gpt/tasks/*`, `/gpt/preferences`）
  - Agent/Claude/OpenClaw Discovery API（`/agent|/claude|/openclaw/discovery/services`）
- MCP サーバー（`mcp-server/`）
  - Streamable HTTP `/mcp`
  - OAuth メタデータ公開（`.well-known/oauth-*`）
  - 10ツール + 5ウィジェットを Rust `/gpt/*` と接続
- Frontend（`frontend/`）
  - React + Vite のWeb UI（`App.tsx` 中心）
- サンプル x402 サーバー（`x402server/`）
  - 動作検証用の別Nodeプロジェクト

## ドメイン（現行）
- Campaign / Sponsored API / Task Completion / Payment / Consent
- GPT Session / User Task Preferences / GPT Service Runs
- Sponsor Dashboard / Creator Metrics

## 現在のフェーズ
- MVP〜P1 拡張フェーズ
- P0 E2E（402→タスク/同意→補助支払い→リソース返却）は稼働済み
- GPT Apps 連携と MCP（GPT App SDKベース）実装済み

## .kiro/specs の進捗
- `gpt-apps-integration`: 33/33 完了
- `smart-service-suggestion`: 32/32 完了
- `refactor-to-gpt-app-sdk`: 27/27 完了
- `autonomous-agent-execution`: 0/41（未着手）

## 想定ユーザー
- ToB: スポンサー（キャンペーン作成・運用・効果測定）
- ToC: GPT/Claude等からスポンサー付きサービスを利用するユーザー