---
title: Vite + React + Tailwind v4 Widget セットアップ
---

# Vite + React + Tailwind v4 Widget セットアップ

GPT Apps SDK の React Widget をビルドするための Vite 設定テンプレート。

## package.json

```json
{
  "name": "[APP_NAME]-web",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview"
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

## vite.config.mts（単一エントリ）

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
    assetsDir: ".",
    rollupOptions: {
      input: {
        "[WIDGET_NAME]": "src/[WIDGET_NAME]/index.tsx",
      },
    },
  },
});
```

## vite.config.mts（マルチエントリ — 複数ウィジェット）

```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import fg from "fast-glob";
import path from "node:path";

function buildInputs() {
  const files = fg.sync("src/**/index.{tsx,jsx}", { dot: false });
  return Object.fromEntries(
    files.map((f) => [path.basename(path.dirname(f)), path.resolve(f)])
  );
}

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
    outDir: "assets",
    assetsDir: ".",
    rollupOptions: {
      input: buildInputs(),
      preserveEntrySignatures: "strict",
    },
  },
});
```

## src/index.css

```css
@import "tailwindcss";
@import "@openai/apps-sdk-ui/css";
@source "../node_modules/@openai/apps-sdk-ui";
@source ".";
```

## tsconfig.json

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true
  },
  "include": ["src"]
}
```

## エントリポイントのパターン

### src/[widget-name]/index.tsx

```tsx
import { createRoot } from "react-dom/client";
import App from "./App";
import "../index.css";

const rootElement = document.getElementById("[widget-name]-root");
if (!rootElement) {
  throw new Error("Missing [widget-name]-root element");
}

createRoot(rootElement).render(<App />);
```

### HTML テンプレート（ビルド出力に対応）

```html
<!doctype html>
<html>
<head>
  <meta charset="utf-8" />
  <script type="module" src="./[widget-name].js"></script>
  <link rel="stylesheet" href="./[widget-name].css">
</head>
<body>
  <div id="[widget-name]-root"></div>
</body>
</html>
```

## 共有 Hooks のディレクトリ構成

```
src/
├── index.css                  # Global CSS (Tailwind + apps-sdk-ui)
├── types.ts                   # OpenAiGlobals, API, DisplayMode 等
├── use-openai-global.ts       # 基盤フック
├── use-widget-state.ts        # widgetState 読み書き
├── use-widget-props.ts        # toolOutput 読み取り
├── use-display-mode.ts        # 表示モード
├── use-max-height.ts          # 最大高さ
│
├── widget-a/                  # Widget A
│   ├── index.tsx              # createRoot エントリ
│   └── App.tsx                # メインコンポーネント
│
└── widget-b/                  # Widget B
    ├── index.tsx
    └── App.tsx
```

## ビルドコマンド

```bash
# 開発サーバー（HMR 対応）
npm run dev
# → http://localhost:4444/[widget-name].html

# プロダクションビルド
npm run build
# → dist/ に HTML + JS + CSS が出力

# MCP サーバーからビルド済み HTML を読み込む
const widgetHtml = readFileSync("../web/dist/[widget-name].html", "utf8");
```
