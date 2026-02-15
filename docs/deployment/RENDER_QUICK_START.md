# Render バックエンドデプロイ - クイックスタート

## ステップ1: PostgreSQLデータベースを作成

1. https://render.com にログイン
2. ダッシュボードで **"New +"** → **"PostgreSQL"** をクリック
3. 設定：
   - **Name**: `payloadexchange-db`
   - **Database**: `payloadexchange`
   - **User**: `payloadexchange_user`
   - **Region**: お好みのリージョン（例: `Oregon (US West)`）
   - **Plan**: `Free` を選択
4. **"Create Database"** をクリック
5. 作成後、**"Connections"** タブを開く
6. **"Internal Database URL"** をコピー（後で使用します）
   - 例: `postgres://payloadexchange_user:password@dpg-xxxxx-a/payloadexchange`

## ステップ2: Webサービスを作成

1. ダッシュボードで **"New +"** → **"Web Service"** をクリック
2. **"Build and deploy from a Git repository"** を選択
3. GitHubリポジトリ `cruujon/SubsidyPayment` を接続（まだ接続していない場合）
4. リポジトリを選択
5. 設定を入力：
   - **Name**: `payloadexchange-backend`
   - **Region**: データベースと同じリージョンを選択
   - **Branch**: `deploy-test` を選択
   - **Root Directory**: （空のまま）
   - **Environment**: `Rust` を選択
   - **Build Command**: `cargo build --release`
   - **Start Command**: `./target/release/payloadexchange_mvp`
   - **Plan**: `Free` を選択

## ステップ3: 環境変数を設定

**"Environment"** セクションで以下を追加：

1. **DATABASE_URL**
   - Key: `DATABASE_URL`
   - Value: ステップ1でコピーしたPostgreSQLのInternal Database URL

2. **PUBLIC_BASE_URL**
   - Key: `PUBLIC_BASE_URL`
   - Value: `https://payloadexchange-backend.onrender.com`（後で実際のURLに置き換えます）

3. **PORT**
   - Key: `PORT`
   - Value: `3000`

4. **RUST_LOG**（オプション）
   - Key: `RUST_LOG`
   - Value: `info`

## ステップ4: デプロイ

1. **"Create Web Service"** をクリック
2. デプロイが開始されます（5-10分かかります）
3. デプロイ完了後、Renderが発行するURLをコピー
   - 例: `https://payloadexchange-backend.onrender.com`

## ステップ5: PUBLIC_BASE_URLを更新

1. デプロイ完了後、実際のURLが確定します
2. Renderのダッシュボードで環境変数 `PUBLIC_BASE_URL` を実際のURLに更新
3. **"Save Changes"** をクリック
4. 自動的に再デプロイが開始されます

## ステップ6: 動作確認

1. ブラウザで `https://payloadexchange-backend.onrender.com/health` にアクセス
2. `{"message":"ok"}` が返ってくれば成功です

## ステップ7: Vercelの環境変数を更新

1. Vercelダッシュボードで `subsidy-payment` プロジェクトを開く
2. **"Settings"** → **"Environment Variables"** を開く
3. `VITE_API_URL` の値をRenderのバックエンドURLに更新
4. **"Save"** をクリック
5. 自動的に再デプロイが開始されます

## トラブルシューティング

### デプロイが失敗する場合

- ビルドログを確認してエラーメッセージを確認
- `DATABASE_URL` が正しく設定されているか確認
- Rustのバージョンが正しいか確認（Renderは自動的に最新の安定版を使用）

### データベース接続エラー

- PostgreSQLデータベースが起動しているか確認
- `DATABASE_URL` がInternal Database URLであることを確認（External URLではない）

### 404エラー

- `/health` エンドポイントにアクセスしてサーバーが起動しているか確認
- デプロイが完了しているか確認（初回デプロイは時間がかかります）

