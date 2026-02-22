import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { readFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { registerAppResource, RESOURCE_MIME_TYPE } from '@modelcontextprotocol/ext-apps/server';

export { registerAppResource, RESOURCE_MIME_TYPE };

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

function resolveWidgetUiMeta() {
  const rawPublicUrl = process.env.PUBLIC_URL?.trim() ?? '';
  let domain = '';
  try {
    if (rawPublicUrl) {
      const parsed = new URL(rawPublicUrl);
      // Avoid pinning widgets to localhost in production if PUBLIC_URL is missing/misconfigured.
      if (parsed.hostname !== 'localhost' && parsed.hostname !== '127.0.0.1') {
        domain = parsed.origin;
      }
    }
  } catch {
    // Omit domain metadata if PUBLIC_URL is invalid.
  }

  return {
    ui: {
      prefersBorder: true,
      ...(domain ? { domain } : {}),
      // Let ChatGPT use its default widget CSP; custom CSP can block runtime resources.
    },
  };
}

export async function readWidgetHtml(fileName: string): Promise<string> {
  // Support both layouts:
  // 1) `tsx src/main.ts` (module lives in `src/widgets`)
  // 2) bundled `node dist/main.js` (module is bundled into `dist/main.js`)
  const cwd = process.cwd();
  const widgetDistBases = [
    resolve(__dirname, '../../dist/widgets'),
    resolve(__dirname, 'widgets'),
    resolve(cwd, 'dist/widgets'),
    resolve(cwd, 'mcp-server/dist/widgets'),
  ];
  const distCandidates = widgetDistBases.flatMap((base) => [
    resolve(base, 'src/widgets/src', fileName),
    resolve(base, fileName),
  ]);
  const srcCandidates = [
    resolve(__dirname, 'src', fileName),
    resolve(__dirname, 'widgets/src', fileName),
    resolve(cwd, 'src/widgets/src', fileName),
    resolve(cwd, 'mcp-server/src/widgets/src', fileName),
  ];
  // Prefer source HTML even in production because Vite single-file output can still
  // emit `modulepreload-polyfill.js` references that are not registered as MCP resources.
  const candidates = [...srcCandidates, ...distCandidates];

  let lastError: unknown;
  for (const candidate of candidates) {
    try {
      return await readFile(candidate, 'utf8');
    } catch (error: any) {
      if (error?.code !== 'ENOENT') {
        throw error;
      }
      lastError = error;
    }
  }

  const pathsTried = candidates.join(', ');
  if (lastError instanceof Error) {
    lastError.message = `${lastError.message} | widget=${fileName} | tried=${pathsTried}`;
    throw lastError;
  }
  throw new Error(`Widget HTML not found: ${fileName} | tried=${pathsTried}`);
}

function registerWidgetResource(server: McpServer, name: string, uri: string, fileName: string): void {
  registerAppResource(
    server,
    name,
    uri,
    {
      description: `${name} widget resource`,
      mimeType: RESOURCE_MIME_TYPE,
    },
    async () => ({
      contents: [
        {
          uri,
          mimeType: RESOURCE_MIME_TYPE,
          text: await readWidgetHtml(fileName),
          _meta: resolveWidgetUiMeta(),
        },
      ],
    })
  );
}

export function registerAllResources(server: McpServer): void {
  registerWidgetResource(server, 'services-list', 'ui://widget/services-list.html', 'services-list.html');
  registerWidgetResource(server, 'service-tasks', 'ui://widget/service-tasks.html', 'service-tasks.html');
  registerWidgetResource(server, 'task-form', 'ui://widget/task-form.html', 'task-form.html');
  registerWidgetResource(server, 'service-access', 'ui://widget/service-access.html', 'service-access.html');
  registerWidgetResource(server, 'user-dashboard', 'ui://widget/user-dashboard.html', 'user-dashboard.html');
}
