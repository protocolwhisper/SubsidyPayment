# エージェント発見コントラクト

スキーマバージョン: `2026-02-14`
エンドポイント: `GET /agent/discovery/services`
エイリアス: `GET /claude/discovery/services`, `GET /openclaw/discovery/services`

## 必須メタデータ項目
- `capabilities`
- `price_cents`
- `sla`
- `required_task`
- `sponsor`

## Capability 正規化
バックエンドはランキングと返却前に capability 名を正規化します。
- `scrape`, `web-scrape`, `web-scraping` -> `scraping`
- `ui-design`, `designing` -> `design`
- `data-tool`, `data-tools` -> `data-tooling`
- アンダースコアとスペースはハイフンに変換

## SLA tier 値
- `best_effort`
- `standard`

## ランキングシグナル
`ranking_score` は次から導出されます。
- `subsidy_score`
- `budget_health_score`
- `relevance_score`

## 認証とレート制限
- `AGENT_DISCOVERY_API_KEY` による任意の Bearer 認証
- `AGENT_DISCOVERY_RATE_LIMIT_PER_MIN` による共有メモリ型レート制限

## 安定性メモ
1. `schema_version` を互換性キーとして扱ってください。
2. `schema_version` が変わる場合、ロールアウト前にフィールド互換性を検証してください。
3. 未知の capability はエージェント側で致命的エラーにしないでください。
