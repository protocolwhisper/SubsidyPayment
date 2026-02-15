# Render デプロイ確認チェックリスト

## バックエンドが「Not Found」を返す場合の確認事項

### 1. Renderでサービスが作成されているか確認

1. Renderダッシュボードにアクセス
2. 「Services」を確認
3. `payloadexchange-backend` が存在するか確認

### 2. デプロイの状態を確認

1. `payloadexchange-backend` サービスを開く
2. 「Events」タブで最新のデプロイの状態を確認
   - ✅ **Live**: デプロイ成功
   - ❌ **Failed**: デプロイ失敗（ログを確認）

### 3. ビルドログを確認

1. 「Events」タブで最新のデプロイを開く
2. 「Build Logs」を確認
   - `cargo build --release` が成功しているか
   - エラーメッセージがないか

### 4. ランタイムログを確認

1. 「Logs」タブを開く
2. 以下のメッセージが表示されているか確認：
   - `Starting server on port XXXX`
   - `Server started successfully on 0.0.0.0:XXXX`
   - `payloadexchange-mvp listening on http://0.0.0.0:XXXX`

### 5. 環境変数を確認

「Environment」タブで以下が設定されているか確認：

- ✅ **DATABASE_URL**: PostgreSQLのInternal Database URL
- ✅ **PUBLIC_BASE_URL**: `https://payloadexchange-backend.onrender.com`
- ✅ **RUST_LOG**: `info`（オプション）
- ⚠️ **PORT**: 設定不要（Renderが自動設定）

### 6. よくある問題と解決方法

#### 問題: ビルドが失敗する
- **原因**: 依存関係のエラー、コンパイルエラー
- **解決**: ビルドログを確認してエラーを修正

#### 問題: サーバーが起動しない
- **原因**: データベース接続エラー、ポート設定エラー
- **解決**: ランタイムログを確認してエラーメッセージを確認

#### 問題: `/health` が404を返す
- **原因**: サーバーが起動していない、ルーティングエラー
- **解決**: ランタイムログでサーバーが起動しているか確認

### 7. 手動で再デプロイ

1. サービスページで「Manual Deploy」をクリック
2. 「Deploy latest commit」を選択
3. デプロイが完了するまで待つ（5-10分）

### 8. デプロイ後の確認

1. `/health` エンドポイントにアクセス
   - `https://payloadexchange-backend.onrender.com/health`
   - `{"message":"ok"}` が返ってくれば成功

2. フロントエンドからAPIを呼び出して確認

