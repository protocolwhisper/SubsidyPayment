# Project Overview — SubsidyPayment

## 目的
x402 の HTTP 402 ペイウォールをプロキシで仲介し、スポンサー補助（タスク実行やデータ提供と交換）または直接支払いで上流有料APIへアクセス可能にする。  
加えて、`/gpt/*` と `/agent|/claude|/openclaw/discovery/*` を通じて AI クライアントからの利用導線を提供する。

## 主要ドメイン
- Resource / Proxy: x402 保護リソースへのアクセス仲介
- Sponsor / Campaign: 補助条件・予算・タスクを管理する配信単位
- Sponsored API: API 単位の補助予算付き実行エントリ
- Task Completion: タスク実行証跡
- Payment: 補助支払い・実行履歴
- Consent: 同意記録
- GPT Session / Task Preference: GPT Apps 連携認証と嗜好フィルタ
- Agent Discovery: Claude/Openclaw 等向けサービス探索 API

## 現在の実装フェーズ
- MVP〜P1 拡張フェーズ
- P0 E2E（402 → タスク/同意 → 補助支払い → リソース返却）は実装済み
- GPT Apps と Agent Discovery 向け API が稼働状態

## アクティブ仕様（.kiro/specs）
- `gpt-apps-integration`（`language: ja`, `phase: tasks-generated`）
- `smart-service-suggestion`（`language: ja`, `phase: tasks-generated`）
- `autonomous-agent-execution`（`language: ja`, `phase: tasks-generated`）

## 主な利用者
- ToB: キャンペーン設計・配信・費用対効果を管理するスポンサー
- ToC: GPT/Claude 等からスポンサー付きサービスを利用するユーザー
