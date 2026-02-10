# クイックデプロイガイド

## ✅ 完了したこと

1. ✅ `deploy-test` ブランチを作成（mainブランチは保護されています）
2. ✅ フロントエンドの環境変数対応を追加
3. ✅ Vercel設定ファイルを作成
4. ✅ GitHubにプッシュ完了

## 🚀 次のステップ：Vercelでデプロイ

### 1. Vercelにログイン/登録

https://vercel.com にアクセスしてGitHubアカウントでログイン

### 2. プロジェクトをインポート

1. Vercelダッシュボードで "Add New..." → "Project" をクリック
2. GitHubリポジトリ `cruujon/SubsidyPayment` を選択
3. **重要**: ブランチを `deploy-test` に変更（デフォルトは `main` なので注意！）
4. **Root Directory**: `frontend` に設定
5. **Framework Preset**: Vite を選択（自動検出される場合もあります）

### 3. 環境変数の設定

**Environment Variables** セクションで以下を追加：

- **Key**: `VITE_API_URL`
- **Value**: バックエンドのURL（後で設定、例: `https://your-backend.onrender.com`）

**注意**: バックエンドをデプロイする前に、一時的に空欄でもOKです（後で更新できます）

### 4. デプロイ

"Deploy" ボタンをクリック

### 5. デプロイ完了後

- フロントエンドのURLが表示されます（例: `https://subsidy-payment.vercel.app`）
- このURLを他の人に共有できます

## 🔧 バックエンドのデプロイ（オプション）

フロントエンドだけでも動作確認できますが、完全に動作させるにはバックエンドも必要です。

### Renderを使用（推奨）

1. https://render.com にログイン
2. "New" → "Web Service"
3. GitHubリポジトリを接続
4. **ブランチ**: `deploy-test`
5. **Build Command**: `cargo build --release`
6. **Start Command**: `./target/release/payloadexchange_mvp`
7. **Environment Variables**:
   ```
   DATABASE_URL=postgres://user:pass@host:5432/dbname
   PUBLIC_BASE_URL=https://your-backend.onrender.com
   PORT=3000
   ```

### データベースのセットアップ

RenderでPostgreSQLサービスを作成し、接続URLを `DATABASE_URL` に設定

## ✅ デプロイ後の確認

1. フロントエンドURLにアクセス
2. ページが表示されることを確認
3. バックエンドがデプロイ済みの場合、API呼び出しが動作することを確認

## ⚠️ 重要な注意事項

- ✅ `deploy-test` ブランチのみを使用（mainブランチは触らない）
- ✅ デプロイ後、動作確認が完了したらブランチを削除可能
- ✅ mainブランチへのマージは不要（テスト用）

## 🔗 参考リンク

- Vercel: https://vercel.com/docs
- Render: https://render.com/docs
- 詳細な手順: `DEPLOY.md` を参照

