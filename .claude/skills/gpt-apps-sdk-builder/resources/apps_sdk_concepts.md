---
title: Apps SDK 基本概念
---

# Apps SDK 基本概念

## 主要コンポーネント

| 要素 | 役割 | 要点 |
|---|---|---|
| MCPサーバー | ChatGPTからの呼び出しを受ける | /mcp でHTTPを受ける |
| Tool | モデルが呼ぶ機能 | 入力スキーマと出力の一貫性 |
| Resource | UIなどの静的リソース | resourceUri で関連付け |
| UIブリッジ | UIとモデル間通信 | postMessage JSON-RPC |

## UIブリッジの最小フロー

1. ui/initialize を呼ぶ
2. ui/notifications/initialized を通知する
3. tools/call でツールを呼ぶ
4. ui/notifications/tool-result を受けてUIを更新する

## structuredContent 設計の指針

- UIに必要な状態を小さくまとめる
- UIレンダリングは structuredContent から再構築できる形にする
- エラー時も structuredContent を空配列などで返す
