# Render環境変数手動設定ガイド

RenderのAPI経由での環境変数設定がうまくいかないため、**Renderダッシュボードから手動で設定する方法**をご案内します。

## 📋 現在の状況

- **サービス名**: SubsidyPayment
- **サービスID**: srv-d65pl3esb7us73fb96tg
- **サービスURL**: https://subsidypayment.onrender.com
- **現在の環境変数**: 未設定

## 🔧 手動設定手順

### ステップ1: Renderダッシュボードを開く

1. https://dashboard.render.com にログイン
2. 左側のメニューから **"Services"** をクリック
3. **"SubsidyPayment"** サービスをクリック

### ステップ2: 環境変数セクションを開く

1. サービスページの左メニューから **"Environment"** をクリック
2. **"Add Environment Variable"** ボタンがあることを確認

### ステップ3: 環境変数を1つずつ追加

以下の環境変数を順番に追加してください：

#### 環境変数1: PUBLIC_BASE_URL

1. **"Add Environment Variable"** をクリック
2. **Key**: `PUBLIC_BASE_URL`
3. **Value**: `https://subsidypayment.onrender.com`
4. **"Save Changes"** をクリック

#### 環境変数2: RUST_LOG

1. **"Add Environment Variable"** をクリック
2. **Key**: `RUST_LOG`
3. **Value**: `info`
4. **"Save Changes"** をクリック

#### 環境変数3: DATABASE_URL（後で設定）

**⚠️ 重要**: この環境変数は、PostgreSQLデータベースを作成してから設定します。

1. まず、PostgreSQLデータベースを作成（下記参照）
2. データベースの **"Connections"** タブから **"Internal Database URL"** をコピー
3. **Key**: `DATABASE_URL`
4. **Value**: コピーしたInternal Database URLを貼り付け
5. **"Save Changes"** をクリック

## 🗄️ PostgreSQLデータベースの作成

### ステップ1: データベース作成

1. Renderダッシュボードの左上の **"New +"** をクリック
2. **"PostgreSQL"** を選択

### ステップ2: データベース設定

以下の設定を入力：

- **Name**: `payloadexchange-db`（任意の名前）
- **Database**: `payloadexchange`
- **User**: `payloadexchange_user`
- **Region**: `Oregon (US West)`（お好みのリージョン）
- **Plan**: `Free`

### ステップ3: データベース作成

1. **"Create Database"** をクリック
2. 作成完了まで待つ（1-2分）

### ステップ4: Internal Database URLを取得

1. データベースページの **"Connections"** タブをクリック
2. **"Internal Database URL"** をコピー
   - 例: `postgres://payloadexchange_user:password@dpg-xxxxx-a.oregon-postgres.render.com/payloadexchange`

### ステップ5: DATABASE_URLを設定

1. **SubsidyPayment** サービスに戻る
2. **"Environment"** タブを開く
3. **"Add Environment Variable"** をクリック
4. **Key**: `DATABASE_URL`
5. **Value**: コピーしたInternal Database URLを貼り付け
6. **"Save Changes"** をクリック

## ✅ 設定後の確認

### 1. 環境変数の確認

**SubsidyPayment** サービスの **"Environment"** タブで、以下の3つの環境変数が設定されていることを確認：

- ✅ `PUBLIC_BASE_URL` = `https://subsidypayment.onrender.com`
- ✅ `RUST_LOG` = `info`
- ✅ `DATABASE_URL` = `postgres://...`（PostgreSQLのURL）

### 2. 再デプロイの確認

環境変数を保存すると、Renderが自動的に再デプロイを開始します。

1. **"Events"** タブでデプロイの進行状況を確認
2. ステータスが **"Live"** になるまで待つ（5-10分）

### 3. 動作確認

デプロイ完了後、以下のURLにアクセスして確認：

```
https://subsidypayment.onrender.com/health
```

成功すると、以下のJSONが返ってきます：

```json
{"message":"ok"}
```

## 🔄 次のステップ

環境変数の設定が完了したら：

1. **Vercelの環境変数を更新**
   - Vercelダッシュボードで `subsidy-payment` プロジェクトを開く
   - **"Settings"** → **"Environment Variables"** を開く
   - `VITE_API_URL` の値を `https://subsidypayment.onrender.com` に設定
   - **"Save"** をクリック

2. **フロントエンドの動作確認**
   - https://subsidy-payment.vercel.app にアクセス
   - キャンペーン作成が動作することを確認

## 🆘 トラブルシューティング

### エラー: "Postgres not configured"

- `DATABASE_URL` が設定されているか確認
- Internal Database URLを使用しているか確認（External URLではない）

### エラー: デプロイが失敗する

- Renderの **"Logs"** タブでエラーメッセージを確認
- 環境変数の値に誤りがないか確認

### エラー: 404 Not Found

- デプロイが完了しているか確認
- URLが正しいか確認

---

**注意**: この手順は、RenderのAPIが直接使えない場合の代替方法です。APIが使えるようになったら、自動化スクリプトを使用できます。

