import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { readFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { registerAppResource, RESOURCE_MIME_TYPE } from '@modelcontextprotocol/ext-apps/server';

export { registerAppResource, RESOURCE_MIME_TYPE };

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const widgetsDistDir = resolve(__dirname, '../../dist/widgets');

export async function readWidgetHtml(fileName: string): Promise<string> {
  const distCandidates = [
    resolve(widgetsDistDir, 'src/widgets/src', fileName),
    resolve(widgetsDistDir, fileName),
  ];
  const srcCandidate = resolve(__dirname, 'src', fileName);
  const isProduction = process.env.NODE_ENV === 'production';

  const candidates = isProduction
    ? [...distCandidates, srcCandidate]
    : [srcCandidate, ...distCandidates];

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

  throw lastError ?? new Error(`Widget HTML not found: ${fileName}`);
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
