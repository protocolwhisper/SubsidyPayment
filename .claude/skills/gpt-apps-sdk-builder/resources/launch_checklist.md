---
title: 公開前チェックリスト
---

# 公開前チェックリスト

## MCP サーバー接続

- [ ] `/mcp` が 200 または適切なレスポンスを返す
- [ ] `OPTIONS` の CORS プリフライトが成功する（`Access-Control-Allow-Origin: *`）
- [ ] `tools/call` が期待する structuredContent を返す
- [ ] UI がある場合は `resourceUri` が一致している
- [ ] ヘルスチェックエンドポイント (`GET /`) が応答する
- [ ] StreamableHTTP transport のセッション管理が正常

## React Widget ビルド

- [ ] `npm run build` がエラーなく完了する
- [ ] ビルド出力の HTML が MCP サーバーから正しく読み込まれる
- [ ] Tailwind CSS + `@openai/apps-sdk-ui/css` が正しくインポートされている
- [ ] `createRoot` のルートエレメント ID が HTML と一致している
- [ ] 本番ビルドで sourcemap が必要に応じて生成される

## window.openai 連携

- [ ] `window.openai` の存在チェックを行ってからメソッドを呼んでいる
- [ ] `useOpenAiGlobal` / `useWidgetProps` でリアクティブにデータを購読している
- [ ] `toolOutput` が null の場合のフォールバック表示がある
- [ ] `theme` ("light" / "dark") に対応している（最低限テキストカラー）

## Widget State

- [ ] `useWidgetState` でセッション永続状態を管理している
- [ ] `widgetState` のデフォルト値が設定されている
- [ ] `widgetState` に機密情報を含めていない
- [ ] ツール呼び出し間で状態が正しく保持される

## Display Mode

- [ ] 選択した表示モード (inline / fullscreen / pip) の制約を遵守している
- [ ] inline: 最大 2 アクション、ネストスクロールなし、タブなし
- [ ] fullscreen: ChatGPT コンポーザーとの共存を考慮
- [ ] pip: モバイルでの fullscreen 強制変換を考慮
- [ ] `requestDisplayMode` 呼び出し時にホストが拒否する場合を処理

## Tool Metadata

- [ ] `openai/toolInvocation/invoking` メッセージが 64 文字以内
- [ ] `openai/toolInvocation/invoked` メッセージが 64 文字以内
- [ ] `annotations` が正しく設定されている（readOnly / destructive / openWorld）
- [ ] `openai/outputTemplate` が正しい resourceUri を指している
- [ ] Decoupled パターンの場合、データツールに outputTemplate がない

## セキュリティ

- [ ] API キー・トークン・秘密情報が `structuredContent` / `content` / `_meta` / `widgetState` に含まれていない
- [ ] 認証が必要な場合は認可フローがサーバー側で機能する
- [ ] ログに個人情報が出ない
- [ ] ツール入力はサーバー側で Zod バリデーション済み
- [ ] ハンドラは冪等に設計されている（リトライ対応）
- [ ] 破壊的操作には確認ステップがある
- [ ] `_meta["openai/locale"]` / `_meta["openai/userAgent"]` を認可に使用していない
- [ ] CSP (`csp.connectDomains`) が必要なドメインだけに限定されている

## アクセシビリティ

- [ ] WCAG AA コントラスト比を満たしている
- [ ] タップターゲットが 44x44px 以上
- [ ] テキスト 200% リサイズでレイアウトが崩壊しない
- [ ] セマンティック HTML を使用している（button, main, section, etc.）
- [ ] 画像に alt テキストがある
- [ ] キーボード操作（Tab / Enter / Escape）で全機能にアクセス可能
- [ ] ARIA ラベルが適切に設定されている

## UX ガイドライン準拠

- [ ] UI は会話を補完している（置き換えていない）
- [ ] カスタムフォントを使用していない（プラットフォームネイティブを継承）
- [ ] アプリロゴを Widget 内に配置していない（ChatGPT が自動付与）
- [ ] ブランドカラーはロゴ・アイコン・プライマリ CTA のみ
- [ ] ツール名と説明が短く明確
- [ ] エラーメッセージが理解しやすい
- [ ] UI 操作が少ない手順で完結する
- [ ] フォローアップのサジェストが自然

## 体験品質

- [ ] ローディング状態が表示される
- [ ] エラー状態のフォールバック UI がある
- [ ] 空状態（データ 0 件）の表示がある
- [ ] `structuredContent` のエラー時フォールバック（空配列等）がある
- [ ] 画像にアスペクト比が設定されている（CLS 防止）

## 提出準備

- [ ] ChatGPT Apps ガイドラインを確認した
- [ ] 必要なスクリーンショットや説明文を準備した
- [ ] `_meta.ui.domain` が本番ドメインに設定されている
- [ ] HTTPS URL + `/mcp` でアクセス可能
- [ ] MCP Inspector での最終動作確認が完了した
