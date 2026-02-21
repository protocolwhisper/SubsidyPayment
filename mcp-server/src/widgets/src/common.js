export function initWidget() {
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
    widgetState: app?.widgetState ?? {},
    isDark,
  };
}

export function notifyWidgetHeight(app) {
  app?.notifyIntrinsicHeight?.(document.body.scrollHeight);
}
