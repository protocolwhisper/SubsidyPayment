# デプロイ手順

このブランチ（`deploy-test`）はテストデプロイ用です。mainブランチには影響しません。

## デプロイ構成

- **フロントエンド**: Vercelでデプロイ
- **バックエンド**: RenderまたはFly.ioでデプロイ（Rustアプリケーション）

## フロントエンドのデプロイ（Vercel）

### 1. Vercelにプロジェクトをインポート

1. [Vercel](https://vercel.com)にログイン
2. "Add New..." → "Project" をクリック
3. GitHubリポジトリを選択
4. **ブランチ**: `deploy-test` を選択
5. **Root Directory**: `frontend` に設定
6. **Framework Preset**: Vite を選択
7. **Build Command**: `npm run build`
8. **Output Directory**: `dist`

### 2. 環境変数の設定

Vercelのプロジェクト設定で以下の環境変数を追加：

```
VITE_API_URL=https://your-backend-url.com
```

（バックエンドのURLは後で設定）

### 3. デプロイ

Vercelが自動的にデプロイを開始します。

## バックエンドのデプロイ（RenderまたはFly.io）

### Renderを使用する場合

1. [Render](https://render.com)にログイン
2. "New" → "Web Service" を選択
3. GitHubリポジトリを接続
4. **ブランチ**: `deploy-test` を選択
5. **Build Command**: `cargo build --release`
6. **Start Command**: `./target/release/payloadexchange_mvp`
7. **Environment Variables**:
   ```
   DATABASE_URL=postgres://...
   PUBLIC_BASE_URL=https://your-backend-url.onrender.com
   PORT=3000
   ```

### Fly.ioを使用する場合

1. [Fly.io](https://fly.io)にログイン
2. `fly launch` を実行
3. `fly.toml` を設定
4. 環境変数を設定: `fly secrets set DATABASE_URL=...`

## データベースのセットアップ

Postgresデータベースが必要です：

- Render: PostgreSQLサービスを作成
- Fly.io: `fly postgres create` で作成
- または、SupabaseなどのマネージドPostgresを使用

## デプロイ後の確認

1. フロントエンドのURLにアクセス
2. バックエンドの `/health` エンドポイントにアクセスして確認
3. フロントエンドからバックエンドAPIが呼び出せることを確認

## 注意事項

- このブランチ（`deploy-test`）はテスト用です
- mainブランチにはマージしないでください
- デプロイ後、動作確認が完了したらブランチを削除できます

