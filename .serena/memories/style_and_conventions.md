# Style and Conventions（更新: 2026-02-20）

## 全体
- 思考は英語、プロジェクト内 Markdown は日本語を維持
- コミットは Conventional Commits（`feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `chore:`）
- 互換性を重視し、既存エンドポイントの振る舞いを壊さない

## Rust バックエンド
- `rustfmt`/`clippy` 前提
- `ApiError` / `ApiResult<T>` でエラーハンドリングを統一
- 設定は `AppConfig::from_env()` 経由（ハードコード禁止）
- ルート追加時は `src/main.rs` と `openapi.yaml` を同期
- x402 ヘッダー・検証フローを維持

## GPT / Discovery API
- GPT系は `/gpt/*` に集約
- Discovery系は `/agent|/claude|/openclaw/discovery/*` の互換提供
- APIキー認証は「設定時のみ必須」の方針を維持
- レート制限ミドルウェア適用を崩さない

## MCP サーバー（TypeScript）
- ESM + TypeScript + Express + MCP SDK 構成
- ツールは `registerAppTool` + Zod スキーマで定義
- 返却形式は `structuredContent` / `content` / `_meta` を基本に統一
- ウィジェットは `ui://widget/*.html` リソースURIで登録
- 認証有効時は OAuth スコープ整合を保つ（`search_services` は noauth 維持）

## DB / Migration
- `migrations/` は連番追加のみ（既存番号の変更・再利用禁止）
- スキーマ変更時はテストと型定義を同時更新

## フロントエンド
- 現行は `App.tsx` 主体の構成を前提
- 破壊的なUI変更時は最小差分で進める
- ビルド時 OG画像生成（`generate-og-image`）を壊さない