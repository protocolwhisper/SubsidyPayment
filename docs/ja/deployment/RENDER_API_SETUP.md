# Render APIキー取得手順

RenderのAPIを使用して環境変数を自動設定するために、APIキーを取得する必要があります。

## 🔑 APIキーの取得方法

### ステップ1: Renderダッシュボードにログイン
1. https://dashboard.render.com にアクセス
2. GitHubアカウントでログイン

### ステップ2: APIキーページにアクセス
1. 右上のアカウントアイコンをクリック
2. **"Account Settings"** をクリック
3. 左メニューから **"API Keys"** をクリック
   - または直接 https://dashboard.render.com/account/api-keys にアクセス

### ステップ3: APIキーを作成
1. **"New API Key"** ボタンをクリック
2. **Name**: `SubsidyPayment Backend Setup` など、わかりやすい名前を入力
3. **"Create API Key"** をクリック
4. **⚠️ 重要**: 表示されたAPIキーをコピーしてください
   - このキーは一度しか表示されません
   - メモ帳などに保存してください

### ステップ4: APIキーを提供
コピーしたAPIキーを私に提供してください。以下の形式で提供していただければOKです：

```
rnd_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

---

## 🔒 セキュリティ注意事項

- APIキーは機密情報です。他人に共有しないでください
- 使用後は必要に応じて削除できます
- APIキーは Renderダッシュボード → Account Settings → API Keys で管理できます

---

## 📋 次のステップ

APIキーを取得したら、以下の情報も一緒に提供していただけるとスムーズです：

1. **RenderのAPIキー**（必須）
2. **PostgreSQLデータベースの名前**（例: `payloadexchange-db`）
3. **Webサービスの名前**（例: `payloadexchange-backend`）

これらの情報があれば、環境変数の取得から設定まで自動で行えます。

