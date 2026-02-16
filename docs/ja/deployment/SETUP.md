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

### 4. MCP サーバーを起動

新しいターミナルで以下を実行：

```bash
cd /path/to/SubsidyPayment/mcp-server
npm install
npm run dev
```

#### 環境変数

| 変数名 | 必須 | 説明 |
|--------|------|------|
| `RUST_BACKEND_URL` | はい | Rust バックエンド URL（デフォルト: `http://localhost:3000`） |
| `MCP_INTERNAL_API_KEY` | はい | バックエンド通信用 API キー |
| `AUTH0_DOMAIN` | いいえ | Auth0 テナントドメイン（例: `your-tenant.auth0.com`） |
| `AUTH0_AUDIENCE` | いいえ | Auth0 API オーディエンス |
| `PUBLIC_URL` | はい | MCP サーバーの公開 URL（デフォルト: `http://localhost:3001`） |
| `AUTH_ENABLED` | いいえ | OAuth 認証の ON/OFF 切替（`true`/`false`）。省略時は Auth0 設定の有無で自動判定 |

#### AUTH_ENABLED の判定ロジック

| `AUTH_ENABLED` | `AUTH0_DOMAIN` + `AUTH0_AUDIENCE` | 結果 |
|---|---|---|
| `true` | 任意 | OAuth **有効** |
| `false` / `0` / `no` | 任意 | OAuth **無効** |
| 未設定 | 両方あり | OAuth **有効** |
| 未設定 | どちらか欠落 | OAuth **無効** |

Auth0 設定なしでローカル開発する場合：

```bash
AUTH_ENABLED=false npm run dev
```

起動ログに `OAuth authentication is DISABLED (MVP mode)` が表示されることを確認してください。

### 5. フロントエンドサーバー（既に起動中）
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
