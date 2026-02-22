import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';

import type { BackendConfig } from './config.ts';
import { registerAllTools } from './tools/index.ts';
import { registerAllResources } from './widgets/index.ts';

export function createServer(config: BackendConfig): McpServer {
  const server = new McpServer({
    name: 'subsidypayment',
    version: process.env.npm_package_version ?? '1.0.1',
  });

  registerAllTools(server, config);
  registerAllResources(server);

  return server;
}
