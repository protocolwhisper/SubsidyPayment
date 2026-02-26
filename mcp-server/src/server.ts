import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';

import type { BackendConfig } from './config.ts';
import { registerAllTools } from './tools/index.ts';
import { registerAllResources } from './widgets/index.ts';

export function createServer(config: BackendConfig): McpServer {
  const server = new McpServer({
    name: 'snapfuel',
    version: '1.0.0',
    description:
      'Guided 6-step MCP flow. Always show one explicit next prompt and prefer get_prompt_guide_flow when the user is unsure.',
  });

  registerAllTools(server, config);
  registerAllResources(server);

  return server;
}
