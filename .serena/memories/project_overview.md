# Project Overview — SubsidyPayment

## 目的
x402 の HTTP 402 ペイウォールをプロキシで扱い、スポンサー補助とユーザータスクを組み合わせて有料APIアクセスを成立させる。加えて、GPT Actions からの検索・認証・タスク完了・実行までの導線を提供する。

## 主要ドメイン
- Resource: x402 保護対象の上流リソース
- Proxy: 402 応答を受けて支払いフローを仲介
- Sponsor: ユーザー負担分を補助する支払い主体
- Campaign: スポンサー配信単位（対象・予算・必須タスク・タグ）
- Sponsored API: サービスキー単位で予算管理される実行対象API
- Consent: データ共有や連絡などの同意記録
- GPT Session: GPT Actions 用のセッショントークン
- Task Preference: ユーザーのタスク嗜好（preferred/neutral/avoided）

## 現在の実装フェーズ
- MVP〜P1拡張フェーズ
- P0のE2E（402→補助支払い→結果返却）は実装済み
- GPT Apps統合向けに `/gpt/*` 系APIと嗜好ベース検索が実装済み

## アクティブ仕様（.kiro/specs）
- `gpt-apps-integration`（phase: `tasks-generated`）
- `smart-service-suggestion`（phase: `tasks-generated`）
- `autonomous-agent-execution`（phase: `tasks-generated`）

## 主な利用者
- ToB: キャンペーン作成・配信・効果計測を行うスポンサー
- ToC: GPT等からスポンサー付きサービスを利用するユーザー
