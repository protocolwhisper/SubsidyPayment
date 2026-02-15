# Style and Conventions

## 全体
- 思考は英語、プロジェクト内Markdownは日本語で記述（Kiro spec の `language: ja` に準拠）
- コミットは Conventional Commits（`feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `chore:`）
- 既存のモノリシック構成を維持しつつ、小さく安全な変更を優先

## Rustバックエンド
- `rustfmt` と `clippy` を前提にする
- エラーは `ApiError` + `ApiResult<T>` で一元化
- 共有状態は `SharedState`（`Arc<RwLock<AppState>>`）を利用
- 設定値は `AppConfig::from_env()` 経由で取得し、ハードコードしない
- DBアクセスは SQLx の生SQLを利用し、型変換は `types.rs` 側で吸収

## GPT / Agent API 実装規約
- GPT ルートは `/gpt/*` に集約し、`build_gpt_router()` 内で定義
- `GPT_ACTIONS_API_KEY` 設定時は Bearer 認証を必須にする
- Discovery ルートは `/agent|/claude|/openclaw/discovery/*` に公開
- `AGENT_DISCOVERY_API_KEY` は設定時のみ必須（未設定時は開発用に許可）
- レート制限は `RateLimiter` ミドルウェアを使い、設定値で調整可能にする

## フロントエンド
- React 18 + TypeScript + Vite
- 現状は `frontend/src/App.tsx` 中心の単一ファイル構成
- CSSフレームワークは使わず `styles.css` を利用
- ビルド時に OG 画像生成スクリプト（`frontend/scripts/generate-og-image-png.mjs`）を実行

## DB・API
- マイグレーションは連番SQL（`migrations/0001...`）で追加し、既存番号は変更しない
- 主要APIは JSON 入出力を維持し、互換性破壊を避ける
- x402 関連ヘッダー（`payment-signature`, `payment-required`, `payment-response`, `x402-version`）を維持
- 仕様変更時は `openapi.yaml` と実装ルートの同期を必ず確認
