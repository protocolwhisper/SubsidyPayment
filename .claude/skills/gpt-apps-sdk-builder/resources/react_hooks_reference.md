---
title: React Hooks for window.openai
---

# React Hooks for window.openai

OpenAI Apps SDK Examples リポジトリの公式フック実装。
Widget 内で `window.openai` のプロパティをリアクティブに購読するための React Hooks。

## types.ts

```typescript
export type OpenAiGlobals<
  ToolInput = UnknownObject,
  ToolOutput = UnknownObject,
  ToolResponseMetadata = UnknownObject,
  WidgetState = UnknownObject
> = {
  theme: Theme;
  userAgent: UserAgent;
  locale: string;
  maxHeight: number;
  displayMode: DisplayMode;
  safeArea: SafeArea;
  toolInput: ToolInput;
  toolOutput: ToolOutput | null;
  toolResponseMetadata: ToolResponseMetadata | null;
  widgetState: WidgetState | null;
  setWidgetState: (state: WidgetState) => Promise<void>;
};

type API = {
  callTool: CallTool;
  sendFollowUpMessage: (args: { prompt: string }) => Promise<void>;
  openExternal(payload: { href: string }): void;
  requestDisplayMode: RequestDisplayMode;
  requestModal: (args: { title?: string; params?: UnknownObject }) => Promise<unknown>;
  requestClose: () => Promise<void>;
};

export type UnknownObject = Record<string, unknown>;
export type Theme = "light" | "dark";
export type SafeAreaInsets = { top: number; bottom: number; left: number; right: number };
export type SafeArea = { insets: SafeAreaInsets };
export type DeviceType = "mobile" | "tablet" | "desktop" | "unknown";
export type UserAgent = {
  device: { type: DeviceType };
  capabilities: { hover: boolean; touch: boolean };
};
export type DisplayMode = "pip" | "inline" | "fullscreen";
export type RequestDisplayMode = (args: { mode: DisplayMode }) => Promise<{ mode: DisplayMode }>;
export type CallToolResponse = { result: string };
export type CallTool = (name: string, args: Record<string, unknown>) => Promise<CallToolResponse>;

export const SET_GLOBALS_EVENT_TYPE = "openai:set_globals";
export class SetGlobalsEvent extends CustomEvent<{ globals: Partial<OpenAiGlobals> }> {
  readonly type = SET_GLOBALS_EVENT_TYPE;
}

declare global {
  interface Window {
    openai: API & OpenAiGlobals;
  }
  interface WindowEventMap {
    [SET_GLOBALS_EVENT_TYPE]: SetGlobalsEvent;
  }
}
```

## useOpenAiGlobal

`window.openai` の任意のプロパティをリアクティブに購読する基盤フック。
`useSyncExternalStore` を使用し、`openai:set_globals` カスタムイベントで変更を検知する。

```typescript
import { useSyncExternalStore } from "react";
import {
  SET_GLOBALS_EVENT_TYPE,
  SetGlobalsEvent,
  type OpenAiGlobals,
} from "./types";

export function useOpenAiGlobal<K extends keyof OpenAiGlobals>(
  key: K
): OpenAiGlobals[K] | null {
  return useSyncExternalStore(
    (onChange) => {
      if (typeof window === "undefined") {
        return () => {};
      }

      const handleSetGlobal = (event: SetGlobalsEvent) => {
        const value = event.detail.globals[key];
        if (value === undefined) {
          return;
        }
        onChange();
      };

      window.addEventListener(SET_GLOBALS_EVENT_TYPE, handleSetGlobal, {
        passive: true,
      });

      return () => {
        window.removeEventListener(SET_GLOBALS_EVENT_TYPE, handleSetGlobal);
      };
    },
    () => window.openai?.[key] ?? null,
    () => window.openai?.[key] ?? null
  );
}
```

### 使用例

```tsx
const theme = useOpenAiGlobal("theme");       // "light" | "dark" | null
const locale = useOpenAiGlobal("locale");     // "en-US" | null
const toolInput = useOpenAiGlobal("toolInput");
const toolOutput = useOpenAiGlobal("toolOutput");
const userAgent = useOpenAiGlobal("userAgent");
```

## useWidgetState

ChatGPT ホストに永続化される Widget State の読み書きフック。
ツール呼び出し間で状態を保持する（選択状態、ソート順、展開パネル等）。

```typescript
import { useCallback, useEffect, useState, type SetStateAction } from "react";
import { useOpenAiGlobal } from "./use-openai-global";
import type { UnknownObject } from "./types";

export function useWidgetState<T extends UnknownObject>(
  defaultState: T | (() => T)
): readonly [T, (state: SetStateAction<T>) => void];
export function useWidgetState<T extends UnknownObject>(
  defaultState?: T | (() => T | null) | null
): readonly [T | null, (state: SetStateAction<T | null>) => void];
export function useWidgetState<T extends UnknownObject>(
  defaultState?: T | (() => T | null) | null
): readonly [T | null, (state: SetStateAction<T | null>) => void] {
  const widgetStateFromWindow = useOpenAiGlobal("widgetState") as T;

  const [widgetState, _setWidgetState] = useState<T | null>(() => {
    if (widgetStateFromWindow != null) {
      return widgetStateFromWindow;
    }
    return typeof defaultState === "function"
      ? defaultState()
      : defaultState ?? null;
  });

  useEffect(() => {
    _setWidgetState(widgetStateFromWindow);
  }, [widgetStateFromWindow]);

  const setWidgetState = useCallback(
    (state: SetStateAction<T | null>) => {
      _setWidgetState((prevState) => {
        const newState = typeof state === "function" ? state(prevState) : state;
        if (newState != null && typeof window !== "undefined") {
          void window.openai?.setWidgetState?.(newState);
        }
        return newState;
      });
    },
    []
  );

  return [widgetState, setWidgetState] as const;
}
```

### 使用例

```tsx
type MyState = { selectedTab: string; sortOrder: "asc" | "desc" };

const [state, setState] = useWidgetState<MyState>({
  selectedTab: "all",
  sortOrder: "asc",
});

// 更新（自動的に window.openai.setWidgetState も呼ばれる）
setState((prev) => ({ ...prev, selectedTab: "active" }));
```

## useWidgetProps

`toolOutput`（structuredContent）をリアクティブに読み取るフック。

```typescript
import { useOpenAiGlobal } from "./use-openai-global";

export function useWidgetProps<T extends Record<string, unknown>>(
  defaultState?: T | (() => T)
): T {
  const props = useOpenAiGlobal("toolOutput") as T;
  const fallback =
    typeof defaultState === "function"
      ? (defaultState as () => T | null)()
      : defaultState ?? null;
  return props ?? fallback;
}
```

### 使用例

```tsx
type ServiceResult = { services: Array<{ id: string; name: string; status: string }> };

const { services } = useWidgetProps<ServiceResult>({ services: [] });

return (
  <ul>
    {services.map((s) => (
      <li key={s.id}>{s.name} — {s.status}</li>
    ))}
  </ul>
);
```

## useDisplayMode

現在の表示モードを購読する。

```typescript
import { useOpenAiGlobal } from "./use-openai-global";
import { type DisplayMode } from "./types";

export const useDisplayMode = (): DisplayMode | null => {
  return useOpenAiGlobal("displayMode");
};
```

### 使用例

```tsx
const displayMode = useDisplayMode();

return (
  <div className={displayMode === "fullscreen" ? "h-screen" : "max-h-[400px]"}>
    {/* ... */}
  </div>
);
```

## useMaxHeight

Widget の最大高さを購読する。

```typescript
import { useOpenAiGlobal } from "./use-openai-global";

export const useMaxHeight = (): number | null => {
  return useOpenAiGlobal("maxHeight");
};
```

### 使用例

```tsx
const maxHeight = useMaxHeight();

return (
  <div style={{ maxHeight: maxHeight ? `${maxHeight}px` : undefined }}>
    {/* コンテンツ */}
  </div>
);
```

## toolOutput 差分マージパターン

ショッピングカートのような状態同期パターン：

```tsx
const toolOutput = useOpenAiGlobal("toolOutput");
const toolResponseMetadata = useOpenAiGlobal("toolResponseMetadata");
const [cartState, setCartState] = useWidgetState<CartState>(createDefaultState);

const lastToolOutputRef = useRef<string>("__unset__");

useEffect(() => {
  if (toolOutput == null) return;

  // toolOutput の変更を検知（UI起因の widgetState 更新をスキップ）
  const serialized = JSON.stringify({ toolOutput, toolResponseMetadata });
  if (serialized === lastToolOutputRef.current) return;
  lastToolOutputRef.current = serialized;

  // 既存の widgetState に新しい toolOutput のデータを差分マージ
  const incoming = (toolOutput as { items?: Item[] }).items ?? [];
  setCartState((prev) => {
    const merged = new Map(prev.items.map((i) => [i.name, i]));
    for (const item of incoming) {
      merged.set(item.name, { ...merged.get(item.name), ...item });
    }
    return { ...prev, items: Array.from(merged.values()) };
  });
}, [toolOutput, toolResponseMetadata]);
```
