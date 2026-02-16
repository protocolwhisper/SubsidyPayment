# ローカル環境セットアップ手順

## 前提条件
- Docker Desktopがインストールされ、起動していること
- RustとCargoがインストールされていること

## セットアップ手順

### 1. Docker Desktopを起動
Docker Desktopアプリケーションを起動してください。

### 2. Postgresコンテナを起動
```bash
cd /path/to/SubsidyPayment
docker compose -f docker-compose.postgres.yml up -d
```

### 3. バックエンドサーバーを起動
新しいターミナルで以下を実行：
```bash
cd /path/to/SubsidyPayment
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/payloadexchange
export PUBLIC_BASE_URL=http://localhost:3000
export PORT=3000
RUST_LOG=info cargo run
```

または、スクリプトを使用：
```bash
./scripts/start-backend.sh
```

### 4. フロントエンドサーバー（既に起動中）
フロントエンドは `http://localhost:5173` で既に起動しています。

## 動作確認
1. ブラウザで `http://localhost:5173` を開く
2. ログインして「Create Campaign」をクリック
3. フォームに記入して「Create Campaign」ボタンをクリック
4. エラーが発生しなければ成功です

## トラブルシューティング

### Postgresに接続できない場合
- Dockerコンテナが起動しているか確認: `docker ps`
- コンテナのログを確認: `docker logs payloadexchange-postgres`

### バックエンドサーバーが起動しない場合
- `DATABASE_URL`が正しく設定されているか確認
- Postgresコンテナが起動しているか確認
- ポート3000が使用されていないか確認
