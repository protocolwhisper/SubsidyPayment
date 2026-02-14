---
runtime: node
language: typescript
---

# MCPサーバーテンプレート（Node / TypeScript）

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
const RESOURCE_URI: string = "ui://widget/[WIDGET_FILE]";
const RESOURCE_PATH: string = "[PATH_TO_WIDGET_FILE]";

const widgetHtml: string = readFileSync(RESOURCE_PATH, "utf8");

const toolInputSchema = z.object({
  value: z.string().min(1),
});

type ToolInput = z.infer<typeof toolInputSchema>;

const buildResponse = (items: string[], message?: string) => {
  return {
    content: message ? [{ type: "text", text: message }] : [],
    structuredContent: { items },
  };
};

const createAppServer = (): McpServer => {
  const server = new McpServer({ name: APP_NAME, version: APP_VERSION });

  registerAppResource(server, "widget", RESOURCE_URI, {}, async () => ({
    contents: [
      {
        uri: RESOURCE_URI,
        mimeType: RESOURCE_MIME_TYPE,
        text: widgetHtml,
      },
    ],
  }));

  registerAppTool(
    server,
    "[TOOL_NAME]",
    {
      title: "[TOOL_TITLE]",
      description: "[TOOL_DESCRIPTION]",
      inputSchema: toolInputSchema,
      _meta: { ui: { resourceUri: RESOURCE_URI } },
    },
    async (args: ToolInput) => {
      const value = args?.value?.trim?.() ?? "";
      if (!value) return buildResponse([], "入力が空です。");
      const items = [value];
      return buildResponse(items, "処理しました。");
    }
  );

  return server;
};

const handleRequest = async (
  req: Parameters<typeof createServer>[0],
  res: Parameters<typeof createServer>[1]
) => {
  if (!req.url) {
    res.writeHead(400).end("Missing URL");
    return;
  }

  const url = new URL(req.url, `http://${req.headers.host ?? "localhost"}`);

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

  if (req.method === "GET" && url.pathname === "/") {
    res.writeHead(200, { "content-type": "text/plain" }).end("MCP server");
    return;
  }

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
