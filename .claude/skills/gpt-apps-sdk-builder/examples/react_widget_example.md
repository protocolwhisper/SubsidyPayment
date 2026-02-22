---
title: React ウィジェットの完全構成例
---

# React ウィジェットの完全構成例

## 概要

React + Tailwind CSS + `@openai/apps-sdk-ui` + window.openai hooks を使ったサービス検索ウィジェットの完全構成。

## プロジェクト構造

```
my-gpt-app/
├── server/
│   ├── package.json
│   ├── tsconfig.json
│   └── src/
│       └── server.ts          # MCP Server
├── web/
│   ├── package.json
│   ├── tsconfig.json
│   ├── tsconfig.app.json
│   ├── vite.config.mts
│   └── src/
│       ├── index.css           # Tailwind + apps-sdk-ui CSS
│       ├── types.ts            # OpenAI globals type declarations
│       ├── use-openai-global.ts
│       ├── use-widget-state.ts
│       ├── use-widget-props.ts
│       ├── use-display-mode.ts
│       ├── use-max-height.ts
│       └── service-search/
│           ├── index.tsx       # Entry point (createRoot)
│           └── ServiceSearch.tsx  # Main component
└── README.md
```

## web/package.json

```json
{
  "name": "my-gpt-app-web",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build"
  },
  "dependencies": {
    "@openai/apps-sdk-ui": "^0.2.1",
    "react": "^19.1.1",
    "react-dom": "^19.1.1",
    "lucide-react": "^0.536.0",
    "clsx": "^2.1.1"
  },
  "devDependencies": {
    "@tailwindcss/vite": "^4.1.11",
    "@types/react": "^19.1.0",
    "@types/react-dom": "^19.1.0",
    "@vitejs/plugin-react": "^4.5.2",
    "tailwindcss": "4.1.11",
    "typescript": "^5.9.2",
    "vite": "^7.1.1"
  }
}
```

## web/vite.config.mts

```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [tailwindcss(), react()],
  server: {
    port: 4444,
    strictPort: true,
    cors: true,
  },
  esbuild: {
    jsx: "automatic",
    jsxImportSource: "react",
    target: "es2022",
  },
  build: {
    target: "es2022",
    sourcemap: true,
    minify: "esbuild",
    outDir: "dist",
    rollupOptions: {
      input: { "service-search": "src/service-search/index.tsx" },
    },
  },
});
```

## web/src/index.css

```css
@import "tailwindcss";
@import "@openai/apps-sdk-ui/css";
@source "../node_modules/@openai/apps-sdk-ui";
@source ".";
```

## web/src/service-search/index.tsx

```tsx
import { createRoot } from "react-dom/client";
import ServiceSearch from "./ServiceSearch";
import "../index.css";

const rootElement = document.getElementById("service-search-root");
if (!rootElement) throw new Error("Missing service-search-root element");

createRoot(rootElement).render(<ServiceSearch />);
```

## web/src/service-search/ServiceSearch.tsx

```tsx
import { useState, useEffect, useRef } from "react";
import { Search, MapPin, ArrowRight, Loader2 } from "lucide-react";
import { Button } from "@openai/apps-sdk-ui/components/Button";
import { Badge } from "@openai/apps-sdk-ui/components/Badge";
import { Input } from "@openai/apps-sdk-ui/components/Input";
import { useOpenAiGlobal } from "../use-openai-global";
import { useWidgetState } from "../use-widget-state";

type Service = {
  id: string;
  name: string;
  description: string;
  category: string;
  status: "active" | "pending" | "closed";
};

type WidgetState = {
  searchQuery: string;
  selectedServiceId: string | null;
};

type ToolOutput = {
  services: Service[];
  totalCount: number;
  hasMore: boolean;
};

export default function ServiceSearch() {
  const toolOutput = useOpenAiGlobal("toolOutput") as ToolOutput | null;
  const theme = useOpenAiGlobal("theme") ?? "light";
  const displayMode = useOpenAiGlobal("displayMode") ?? "inline";

  const [widgetState, setWidgetState] = useWidgetState<WidgetState>({
    searchQuery: "",
    selectedServiceId: null,
  });

  const [isSearching, setIsSearching] = useState(false);
  const [searchInput, setSearchInput] = useState(widgetState?.searchQuery ?? "");

  const services = toolOutput?.services ?? [];
  const totalCount = toolOutput?.totalCount ?? 0;

  const handleSearch = async () => {
    if (!searchInput.trim() || !window.openai?.callTool) return;

    setIsSearching(true);
    try {
      await window.openai.callTool("search_services", {
        query: searchInput.trim(),
      });
      setWidgetState((prev) => ({
        ...prev,
        searchQuery: searchInput.trim(),
      }));
    } finally {
      setIsSearching(false);
    }
  };

  const handleSelect = (service: Service) => {
    setWidgetState((prev) => ({
      ...prev,
      selectedServiceId: service.id,
    }));
    // モデルに詳細を表示させる
    window.openai?.sendFollowUpMessage?.({
      prompt: `${service.name} の詳細を教えてください`,
    });
  };

  const handleViewMore = () => {
    window.openai?.sendFollowUpMessage?.({
      prompt: "さらに結果を表示してください",
    });
  };

  const statusColor = (status: Service["status"]) => {
    switch (status) {
      case "active": return "primary";
      case "pending": return "info";
      case "closed": return "secondary";
    }
  };

  const statusLabel = (status: Service["status"]) => {
    switch (status) {
      case "active": return "受付中";
      case "pending": return "準備中";
      case "closed": return "終了";
    }
  };

  return (
    <div className="bg-surface text-primary p-4 flex flex-col gap-4">
      {/* 検索バー */}
      <div className="flex gap-2">
        <div className="flex-1 relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-secondary" />
          <Input
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSearch()}
            placeholder="サービスを検索..."
            className="pl-9"
          />
        </div>
        <Button
          color="primary"
          variant="solid"
          onClick={handleSearch}
          disabled={isSearching || !searchInput.trim()}
        >
          {isSearching ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            "検索"
          )}
        </Button>
      </div>

      {/* 結果ヘッダー */}
      {services.length > 0 && (
        <div className="flex items-center justify-between text-sm">
          <span className="text-secondary">
            {totalCount} 件のサービスが見つかりました
          </span>
        </div>
      )}

      {/* サービスカード一覧 */}
      <div className="flex flex-col gap-3">
        {services.map((service) => (
          <div
            key={service.id}
            className={`
              border rounded-xl p-4 flex flex-col gap-2 cursor-pointer
              transition-colors
              ${
                widgetState?.selectedServiceId === service.id
                  ? "border-primary/40 bg-subtle"
                  : "border-default hover:border-primary/20 hover:bg-subtle/50"
              }
            `}
            onClick={() => handleSelect(service)}
            role="button"
            tabIndex={0}
            onKeyDown={(e) => e.key === "Enter" && handleSelect(service)}
          >
            <div className="flex items-start justify-between gap-2">
              <div className="flex-1 min-w-0">
                <h3 className="text-sm font-semibold truncate">
                  {service.name}
                </h3>
                <p className="text-xs text-secondary mt-1 line-clamp-2">
                  {service.description}
                </p>
              </div>
              <Badge
                variant="soft"
                color={statusColor(service.status)}
                pill
              >
                {statusLabel(service.status)}
              </Badge>
            </div>
            <div className="flex items-center justify-between text-xs text-secondary">
              <div className="flex items-center gap-1">
                <MapPin className="h-3 w-3" aria-hidden="true" />
                <span>{service.category}</span>
              </div>
              <ArrowRight className="h-3 w-3" aria-hidden="true" />
            </div>
          </div>
        ))}
      </div>

      {/* もっと見る */}
      {toolOutput?.hasMore && (
        <Button
          color="secondary"
          variant="outline"
          onClick={handleViewMore}
          block
        >
          さらに表示
        </Button>
      )}

      {/* 空状態 */}
      {services.length === 0 && !isSearching && (
        <div className="border border-dashed border-default rounded-xl p-6 text-center">
          <Search className="h-8 w-8 text-secondary mx-auto mb-2" />
          <p className="text-sm text-secondary">
            キーワードを入力してサービスを検索してください
          </p>
        </div>
      )}
    </div>
  );
}
```

## server/src/server.ts（対応するMCPサーバー）

```typescript
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

const PORT = Number(process.env.PORT ?? 8787);
const WIDGET_URI = "ui://widget/service-search.html";
const widgetHtml = readFileSync("../web/dist/service-search.html", "utf8");

const createAppServer = (): McpServer => {
  const server = new McpServer({ name: "subsidy-app", version: "1.0.0" });

  registerAppResource(server, "service-search", WIDGET_URI, {}, async () => ({
    contents: [{
      uri: WIDGET_URI,
      mimeType: RESOURCE_MIME_TYPE,
      text: widgetHtml,
      _meta: {
        ui: {
          prefersBorder: true,
          domain: "https://myapp.example.com",
        },
      },
    }],
  }));

  // データ取得ツール（outputTemplate なし）
  registerAppTool(server, "search_services", {
    title: "サービス検索",
    description: "補助金サービスをキーワードで検索する",
    inputSchema: { query: z.string().min(1) },
    annotations: { readOnlyHint: true, destructiveHint: false, openWorldHint: false },
  }, async ({ query }) => {
    const results = await fetchServicesFromBackend(query);
    return {
      structuredContent: {
        services: results.items,
        totalCount: results.total,
        hasMore: results.hasMore,
      },
      content: [{ type: "text", text: `${results.total} 件のサービスが見つかりました。` }],
    };
  });

  // レンダーツール（outputTemplate あり）
  registerAppTool(server, "render_services", {
    title: "サービス一覧を表示",
    description: "検索結果をウィジェットで表示する。search_services の後に呼ぶ。",
    inputSchema: {
      services: z.array(z.object({
        id: z.string(),
        name: z.string(),
        description: z.string(),
        category: z.string(),
        status: z.enum(["active", "pending", "closed"]),
      })),
      totalCount: z.number(),
      hasMore: z.boolean(),
    },
    _meta: {
      ui: { resourceUri: WIDGET_URI },
      "openai/outputTemplate": WIDGET_URI,
      "openai/toolInvocation/invoking": "検索結果を表示中...",
      "openai/toolInvocation/invoked": "検索結果を表示しました",
    },
    annotations: { readOnlyHint: true, destructiveHint: false, openWorldHint: false },
  }, async ({ services, totalCount, hasMore }) => ({
    structuredContent: { services, totalCount, hasMore },
    content: [{ type: "text", text: `${totalCount} 件のサービスを表示しています。` }],
  }));

  return server;
};

// ... HTTP server setup (see node_mcp_server_template.md)
```

## ビルドと検証

```bash
# Widget をビルド
cd web && npm run build

# MCP サーバーを起動
cd ../server && npm start

# Inspector で検証
npx @modelcontextprotocol/inspector@latest --server-url http://localhost:8787/mcp --transport http
```
