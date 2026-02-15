---
title: UI付きTodoアプリの最短構成
---

# UI付きTodoアプリの最短構成

## 目的

UIとツール連携を最小構成で確認する。

## 構成

```
todo-app/
├── public/
│   └── todo-widget.html
├── server.js
└── package.json
```

## 依存関係

```bash
npm install @modelcontextprotocol/sdk @modelcontextprotocol/ext-apps zod
```

## UI

ui_widget_template.html を todo-widget.html に置き換え、[TOOL_NAME] を add_todo にする。

## MCPサーバー

node_mcp_server_template.md の内容を server.js に適用し、RESOURCE_PATH を public/todo-widget.html にする。

## 起動

```bash
node server.js
```

## 検証

```bash
npx @modelcontextprotocol/inspector@latest --server-url http://localhost:8787/mcp --transport http
```
