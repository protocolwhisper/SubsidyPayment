---
title: ツールのみ翻訳アプリ
---

# ツールのみ翻訳アプリ

## 目的

UIなしでツールだけを公開する。

## ツール仕様の例

| Tool | Purpose | Input | Output |
|---|---|---|---|
| translate_text | テキスト翻訳 | text, targetLanguage | translatedText |

## structuredContent の例

```json
{
  "structuredContent": {
    "translatedText": "こんにちは"
  }
}
```

## 検証ポイント

- UI resource を登録しない
- ChatGPTでツール実行を確認する
