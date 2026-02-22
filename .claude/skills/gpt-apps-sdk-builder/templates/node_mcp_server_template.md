---
runtime: node
language: typescript
---

# MCPサーバーテンプレート（Node / TypeScript）

## 依存関係

```json
{
  "dependencies": {
    "@modelcontextprotocol/sdk": "^1.26.0",
    "@modelcontextprotocol/ext-apps": "^1.0.1",
    "zod": "^3.25.0"
  },
  "devDependencies": {
    "typescript": "^5.9.2",
    "tsx": "^4.20.4"
  }
}
```

## サーバー実装

```ts
import { createServer } from "node:http";
import { readFileSync } from "node:fs";
import {
  registerAppResource,
  registerAppTool,
  RESOURCE_MIME_TYPE,
} from "@modelcontextprotocol/ext-apps/server";
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/streamableHttp.js";
import { z } from "zod";

const PORT: number = Number(process.env.PORT ?? 8787);
const MCP_PATH: string = "/mcp";
const APP_NAME: string = "[APP_NAME]";
const APP_VERSION: string = "[APP_VERSION]";

// UI Widget の URI と HTML
const WIDGET_URI: string = "ui://widget/[WIDGET_FILE]";
const widgetHtml: string = readFileSync("[PATH_TO_BUILT_WIDGET_HTML]", "utf8");

// ── App Server Factory ──────────────────────────────────────────────

const createAppServer = (): McpServer => {
  const server = new McpServer(
    { name: APP_NAME, version: APP_VERSION },
    { capabilities: { tools: {}, resources: {} } }
  );

  // ── UI Resource 登録 ──────────────────────────────────────────────

  registerAppResource(server, "widget", WIDGET_URI, {}, async () => ({
    contents: [
      {
        uri: WIDGET_URI,
        mimeType: RESOURCE_MIME_TYPE, // "text/html;profile=mcp-app"
        text: widgetHtml,
        _meta: {
          ui: {
            prefersBorder: true,
            domain: "[PRODUCTION_DOMAIN]",    // 提出時に必須
            csp: {
              connectDomains: ["[API_DOMAIN]"],  // Widget → API 通信先
              resourceDomains: [],
            },
          },
        },
      },
    ],
  }));

  // ── データ取得ツール（Decoupled: outputTemplate なし）──────────────

  registerAppTool(
    server,
    "[DATA_TOOL_NAME]",
    {
      title: "[DATA_TOOL_TITLE]",
      description: "[DATA_TOOL_DESCRIPTION]",
      inputSchema: {
        query: z.string().min(1).describe("[入力の説明]"),
      },
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: false,
      },
    },
    async ({ query }) => {
      // バックエンドからデータ取得
      const results = await fetchFromBackend(query);
      return {
        structuredContent: {
          items: results,
          totalCount: results.length,
          hasMore: false,
        },
        content: [
          { type: "text", text: `${results.length} 件の結果が見つかりました。` },
        ],
      };
    }
  );

  // ── レンダーツール（Decoupled: outputTemplate あり）────────────────

  registerAppTool(
    server,
    "[RENDER_TOOL_NAME]",
    {
      title: "[RENDER_TOOL_TITLE]",
      description: "[RENDER_TOOL_DESCRIPTION]. [DATA_TOOL_NAME] の後に呼ぶ。",
      inputSchema: {
        items: z.array(z.object({
          id: z.string(),
          name: z.string(),
          // ... 追加フィールド
        })),
        totalCount: z.number(),
        hasMore: z.boolean(),
      },
      _meta: {
        ui: {
          resourceUri: WIDGET_URI,
          visibility: ["model", "app"],
        },
        "openai/outputTemplate": WIDGET_URI,
        "openai/toolInvocation/invoking": "[読み込み中メッセージ <=64文字]",
        "openai/toolInvocation/invoked": "[完了メッセージ <=64文字]",
        "openai/widgetDescription": "[ウィジェットの説明（モデルコンテキスト用）]",
      },
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: false,
      },
    },
    async ({ items, totalCount, hasMore }) => ({
      structuredContent: { items, totalCount, hasMore },
      content: [
        { type: "text", text: `${totalCount} 件を表示しています。` },
      ],
    })
  );

  // ── 単体ツール（UI 付き、Decoupled でない場合）────────────────────

  registerAppTool(
    server,
    "[TOOL_NAME]",
    {
      title: "[TOOL_TITLE]",
      description: "[TOOL_DESCRIPTION]",
      inputSchema: {
        value: z.string().min(1),
      },
      _meta: {
        ui: { resourceUri: WIDGET_URI },
        "openai/outputTemplate": WIDGET_URI,
        "openai/toolInvocation/invoking": "[読み込み中...]",
        "openai/toolInvocation/invoked": "[完了]",
      },
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: false,
      },
    },
    async ({ value }) => {
      const result = await processValue(value);
      return {
        // structuredContent: Model + Widget 両方に可視
        structuredContent: { result },
        // content: テキスト説明文
        content: [{ type: "text", text: `処理しました: ${value}` }],
        // _meta: Widget のみに可視（大量/機密データ用）
        _meta: {
          "openai/toolInvocation/invoking": "処理中...",
          "openai/toolInvocation/invoked": "完了しました",
        },
      };
    }
  );

  return server;
};

// ── HTTP Server ─────────────────────────────────────────────────────

const handleRequest = async (
  req: Parameters<typeof createServer>[0],
  res: Parameters<typeof createServer>[1]
) => {
  if (!req.url) {
    res.writeHead(400).end("Missing URL");
    return;
  }

  const url = new URL(req.url, `http://${req.headers.host ?? "localhost"}`);

  // CORS プリフライト
  if (req.method === "OPTIONS" && url.pathname === MCP_PATH) {
    res.writeHead(204, {
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "POST, GET, OPTIONS, DELETE",
      "Access-Control-Allow-Headers": "content-type, mcp-session-id",
      "Access-Control-Expose-Headers": "Mcp-Session-Id",
    });
    res.end();
    return;
  }

  // ヘルスチェック
  if (req.method === "GET" && url.pathname === "/") {
    res.writeHead(200, { "content-type": "text/plain" }).end("MCP server OK");
    return;
  }

  // MCP エンドポイント
  const mcpMethods = new Set(["POST", "GET", "DELETE"]);
  if (url.pathname === MCP_PATH && req.method && mcpMethods.has(req.method)) {
    res.setHeader("Access-Control-Allow-Origin", "*");
    res.setHeader("Access-Control-Expose-Headers", "Mcp-Session-Id");

    const server = createAppServer();
    const transport = new StreamableHTTPServerTransport({
      sessionIdGenerator: undefined,
      enableJsonResponse: true,
    });

    res.on("close", () => {
      transport.close();
      server.close();
    });

    try {
      await server.connect(transport);
      await transport.handleRequest(req, res);
    } catch (error) {
      console.error("MCP request error:", error);
      if (!res.headersSent) {
        res.writeHead(500).end("Internal server error");
      }
    }
    return;
  }

  res.writeHead(404).end("Not Found");
};

const startServer = (): void => {
  const httpServer = createServer((req, res) => {
    void handleRequest(req, res);
  });

  httpServer.listen(PORT, () => {
    console.log(`MCP server listening on http://localhost:${PORT}${MCP_PATH}`);
  });
};

startServer();
```

## 注意事項

- `RESOURCE_MIME_TYPE` は `"text/html;profile=mcp-app"` に解決される
- Decoupled Data + Render パターンでは、データツールに `outputTemplate` を設定しない
- `_meta` の `openai/toolInvocation/*` メッセージは 64 文字以内
- `annotations` を正しく設定して承認プロンプトを制御
- 本番環境では `domain` フィールドが提出時に必須
