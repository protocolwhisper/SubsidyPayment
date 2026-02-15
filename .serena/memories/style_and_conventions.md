# Style and Conventions

## 全体
- 思考は英語、プロジェクト内のMarkdownドキュメントは日本語で記述
- Conventional Commits を使用（`feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `chore:`）
- 既存方針（モノリシック構成）を尊重しつつ、変更は小さく安全に行う

## Rustバックエンド
- `rustfmt` と `clippy` を前提にする
- エラーは `ApiError` + `ApiResult<T>` で一元化
- 状態共有は `SharedState`（`Arc<RwLock<AppState>>`）を使う
- 設定値は `AppConfig::from_env()` 経由で取得し、ハードコードしない
- DBアクセスは SQLx の生SQLで実装し、型変換は `types.rs` 側で吸収

## GPT API 実装規約
- ルートは `/gpt/*` に集約し、`build_gpt_router()` 内で定義
- `GPT_ACTIONS_API_KEY` が設定される場合は Bearer 認証必須
- レート制限は `RateLimiter` ミドルウェアで適用
- セッションは `gpt_sessions` テーブルで管理し、有効期限を必ず検証
- 嗜好フィルタは `user_task_preferences` を参照し、`preferred/neutral/avoided` を厳密に扱う

## フロントエンド
- React 18 + TypeScript + Vite
- 現状は `frontend/src/App.tsx` 中心の単一ファイル構成
- CSSフレームワークは使わず `styles.css` を利用

## DB・API
- マイグレーションは連番SQL（`migrations/0001...`）で追加し、既存番号は変更しない
- 主要APIは JSON 入出力を維持し、互換性を崩す変更は避ける
- x402 関連ヘッダー（`payment-signature`, `payment-required`, `payment-response`, `x402-version`）を維持
