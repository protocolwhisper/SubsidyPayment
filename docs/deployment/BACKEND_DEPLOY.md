# バックエンドデプロイ手順（Render）

## 1. Renderにアカウント作成・ログイン

https://render.com にアクセスしてGitHubアカウントでログイン

## 2. PostgreSQLデータベースを作成

1. "New" → "PostgreSQL" をクリック
2. データベース名を入力（例: `payloadexchange`）
3. リージョンを選択
4. "Create Database" をクリック
5. 作成後、**Internal Database URL** をコピー（後で使用）

## 3. Webサービスを作成

1. "New" → "Web Service" をクリック
2. GitHubリポジトリ `cruujon/SubsidyPayment` を接続
3. **ブランチ**: `deploy-test` を選択
4. **Name**: `payloadexchange-backend`
5. **Environment**: `Rust` を選択
6. **Build Command**: `cargo build --release`
7. **Start Command**: `./target/release/payloadexchange_mvp`
8. **Plan**: Free を選択

## 4. 環境変数を設定

**Environment Variables** セクションで以下を追加：

- **DATABASE_URL**: ステップ2でコピーしたPostgreSQLのInternal Database URL
- **PUBLIC_BASE_URL**: Renderが発行するURL（例: `https://payloadexchange-backend.onrender.com`）
- **PORT**: `3000`

## 5. デプロイ

"Create Web Service" をクリック

## 6. デプロイ完了後

- Renderが発行するURLをコピー（例: `https://payloadexchange-backend.onrender.com`）
- このURLをVercelの環境変数 `VITE_API_URL` に設定

## 7. Vercelで環境変数を設定

1. Vercelダッシュボードで `subsidy-payment` プロジェクトを開く
2. "Settings" → "Environment Variables" を開く
3. **Key**: `VITE_API_URL`
4. **Value**: RenderのバックエンドURL（例: `https://payloadexchange-backend.onrender.com`）
5. "Save" をクリック
6. 自動的に再デプロイが開始されます

## 注意事項

- Renderの無料プランは15分間の非アクセス後にスリープします
- 初回アクセス時に起動するまで数秒かかります
- 本番環境では有料プランを使用することを推奨します

