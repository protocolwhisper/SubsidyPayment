import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';

import type { BackendConfig } from './config.ts';
import { registerAllTools } from './tools/index.ts';
import { registerAllResources } from './widgets/index.ts';

export function createServer(config: BackendConfig): McpServer {
  const server = new McpServer({
    name: 'subsidypayment',
    version: '1.0.0',
  });

  registerAllTools(server, config);
  registerAllResources(server);

  return server;
}
