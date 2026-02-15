import { readFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { registerAppResource, RESOURCE_MIME_TYPE } from '@modelcontextprotocol/ext-apps/server';
import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const widgetsDistDir = resolve(__dirname, '../../dist/widgets');

async function readWidgetHtml(fileName: string): Promise<string> {
  return readFile(resolve(widgetsDistDir, fileName), 'utf8');
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
  registerWidgetResource(server, 'task-form', 'ui://widget/task-form.html', 'task-form.html');
  registerWidgetResource(server, 'user-dashboard', 'ui://widget/user-dashboard.html', 'user-dashboard.html');
}
