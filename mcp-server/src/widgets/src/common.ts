type OpenAIWidgetBridge = {
  theme?: { appearance?: 'light' | 'dark' };
  toolOutput?: unknown;
  widgetState?: Record<string, unknown>;
  maxHeight?: number;
  notifyIntrinsicHeight?: (height: number) => void;
  setWidgetState?: (state: Record<string, unknown>) => Promise<void> | void;
  callTool?: (name: string, input?: Record<string, unknown>) => Promise<unknown>;
  sendFollowUpMessage?: (message: string) => Promise<void> | void;
};

declare global {
  interface Window {
    openai?: OpenAIWidgetBridge;
  }
}

export type WidgetInitResult = {
  app: OpenAIWidgetBridge | undefined;
  toolOutput: unknown;
  widgetState: Record<string, unknown>;
  isDark: boolean;
};

export function initWidget(): WidgetInitResult {
  const app = window.openai;
  const isDark = app?.theme?.appearance === 'dark';
  document.body.classList.toggle('dark', isDark);

  if (typeof app?.maxHeight === 'number' && Number.isFinite(app.maxHeight)) {
    document.body.style.maxHeight = `${app.maxHeight}px`;
    document.body.style.overflowY = 'auto';
  }

  return {
    app,
    toolOutput: app?.toolOutput ?? null,
    widgetState: (app?.widgetState as Record<string, unknown>) ?? {},
    isDark,
  };
}

export function notifyWidgetHeight(app?: OpenAIWidgetBridge): void {
  app?.notifyIntrinsicHeight?.(document.body.scrollHeight);
}
