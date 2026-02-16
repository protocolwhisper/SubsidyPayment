import cors from 'cors';
import express from 'express';
import { StreamableHTTPServerTransport } from '@modelcontextprotocol/sdk/server/streamableHttp.js';

import {
  oauthAuthorizationServerRedirectHandler,
  oauthProtectedResourceHandler,
} from './auth/oauth-metadata.ts';
import { loadConfig } from './config.ts';
import { logger } from './logger.ts';
import { createServer } from './server.ts';

const ALLOWED_ORIGINS = [
  'https://chatgpt.com',
  'https://cdn.oaistatic.com',
  'https://web-sandbox.oaiusercontent.com',
];

export function createApp() {
  const config = loadConfig();
  const app = express();

  app.use(express.json({ limit: '1mb' }));
  app.use(
    cors({
      origin: ALLOWED_ORIGINS,
      methods: ['GET', 'POST', 'OPTIONS'],
      allowedHeaders: ['Content-Type', 'Authorization'],
    })
  );

  const healthHandler = (_req: express.Request, res: express.Response) => {
    res.json({
      status: 'ok',
      version: process.env.npm_package_version ?? '0.1.0',
      uptime: process.uptime(),
    });
  };

  app.get('/', healthHandler);
  app.get('/health', healthHandler);

  app.get('/.well-known/oauth-protected-resource', oauthProtectedResourceHandler(config));
  app.get('/.well-known/oauth-authorization-server', oauthAuthorizationServerRedirectHandler(config));

  const mcpHandler: express.RequestHandler = async (req, res) => {
    try {
      const transport = new StreamableHTTPServerTransport({
        sessionIdGenerator: undefined,
      });
      const server = createServer(config);

      await server.connect(transport);
      await transport.handleRequest(req, res, req.body);
    } catch (error) {
      logger.error({ err: error }, 'failed to handle /mcp request');
      res.status(500).json({
        error: {
          code: 'mcp_internal_error',
          message: 'Failed to handle MCP request',
        },
      });
    }
  };

  app.get('/mcp', mcpHandler);
  app.post('/mcp', mcpHandler);
  app.delete('/mcp', mcpHandler);

  return { app, config };
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const { app, config } = createApp();
  app.listen(config.port, () => {
    logger.info({ port: config.port }, 'MCP server started');
    logger.info(
      { authEnabled: config.authEnabled },
      config.authEnabled
        ? 'OAuth authentication is ENABLED (Auth0)'
        : 'OAuth authentication is DISABLED (MVP mode)',
    );
  });
}
