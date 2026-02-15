# Render環境変数設定例（バックエンド用）

このファイルは、Renderでバックエンドをデプロイする際に設定する環境変数の例です。

## 📋 必要な環境変数一覧

Renderのダッシュボードで、以下の環境変数を設定してください。

---

## 🔧 環境変数の設定方法

1. Renderダッシュボードで `payloadexchange-backend` サービスを開く
2. 左メニューから **"Environment"** をクリック
3. **"Add Environment Variable"** をクリック
4. 以下の変数を1つずつ追加

---

## 📝 環境変数の詳細

### 1. DATABASE_URL（必須）

**Key:** `DATABASE_URL`

**Value:** PostgreSQLのInternal Database URL

**取得方法:**
1. RenderダッシュボードでPostgreSQLデータベースを開く
2. **"Connections"** タブをクリック
3. **"Internal Database URL"** をコピー

**例:**
```
postgres://payloadexchange_user:your_password@dpg-xxxxxxxxxxxx-a.oregon-postgres.render.com/payloadexchange
```

**⚠️ 重要:**
- **Internal Database URL** を使用してください（External URLではありません）
- パスワード部分は実際のパスワードに置き換えてください

---

### 2. PUBLIC_BASE_URL（必須）

**Key:** `PUBLIC_BASE_URL`

**Value:** Renderが発行するバックエンドのURL

**取得方法:**
1. Webサービスのデプロイが完了したら、画面右上に表示されるURLをコピー
2. そのURLをそのまま使用

**例（デプロイ前の仮の値）:**
```
https://payloadexchange-backend.onrender.com
```

**例（デプロイ後の実際の値）:**
```
https://payloadexchange-backend-xxxx.onrender.com
```

**⚠️ 重要:**
- デプロイ完了後に実際のURLに更新してください
- プロトコル（`https://`）を含めてください
- 末尾のスラッシュ（`/`）は不要です

---

### 3. RUST_LOG（推奨）

**Key:** `RUST_LOG`

**Value:** ログレベル

**例:**
```
info
```

**オプション:**
- `error`: エラーのみ
- `warn`: 警告とエラー
- `info`: 情報、警告、エラー（推奨）
- `debug`: 詳細なデバッグ情報
- `trace`: 最も詳細な情報

---

### 4. PORT（設定不要）

**Key:** `PORT`

**Value:** （設定不要）

**説明:**
- Renderが自動的に `PORT` 環境変数を設定します
- 手動で設定する必要はありません
- コード内で `std::env::var("PORT")` で取得できます

---

## 📋 Renderダッシュボードでの設定例

Renderの環境変数設定画面では、以下のように表示されます：

```
Environment Variables
┌─────────────────────┬──────────────────────────────────────────────┐
│ Key                 │ Value                                         │
├─────────────────────┼──────────────────────────────────────────────┤
│ DATABASE_URL        │ postgres://user:pass@host:5432/dbname        │
│ PUBLIC_BASE_URL     │ https://payloadexchange-backend.onrender.com │
│ RUST_LOG            │ info                                          │
└─────────────────────┴──────────────────────────────────────────────┘
```

---

## 🔄 設定後の手順

1. すべての環境変数を設定したら、**"Save Changes"** をクリック
2. Renderが自動的に再デプロイを開始します
3. デプロイが完了するまで待ちます（5-10分）
4. デプロイ完了後、`/health` エンドポイントで動作確認

---

## ✅ 動作確認

デプロイ完了後、以下のURLにアクセスして確認：

```
https://your-backend-url.onrender.com/health
```

成功すると、以下のJSONが返ってきます：

```json
{"message":"ok"}
```

---

## 🆘 トラブルシューティング

### エラー: "Postgres not configured; set DATABASE_URL"

**原因:** `DATABASE_URL` が正しく設定されていない

**対処法:**
1. Renderの環境変数で `DATABASE_URL` が設定されているか確認
2. Internal Database URLを使用しているか確認（External URLではない）
3. URLにスペースや改行が含まれていないか確認

### エラー: デプロイが失敗する

**原因:** 環境変数の設定ミスやビルドエラー

**対処法:**
1. Renderのログを確認（サービスページの「Logs」タブ）
2. 環境変数の値に誤りがないか確認
3. `PUBLIC_BASE_URL` に実際のURLが設定されているか確認

---

## 📝 チェックリスト

設定前に以下を確認：

- [ ] PostgreSQLデータベースが作成済み
- [ ] Internal Database URLをコピー済み
- [ ] Webサービスのデプロイが完了し、URLが確定している
- [ ] すべての環境変数を正しく設定した
- [ ] "Save Changes" をクリックした
- [ ] 再デプロイが完了した
- [ ] `/health` エンドポイントで動作確認した

---

## 🔗 関連ドキュメント

- 詳細なデプロイ手順: `BACKEND_DEPLOY.md`
- クイックスタート: `RENDER_QUICK_START.md`
- デプロイ確認: `RENDER_DEPLOY_CHECK.md`

