import { registerAppTool } from '@modelcontextprotocol/ext-apps/server';
import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';

import { TokenVerifier } from '../auth/token-verifier.ts';
import { BackendClient, BackendClientError } from '../backend-client.ts';
import type { BackendConfig } from '../config.ts';

const runServiceInputSchema = z.object({
  service: z.string(),
  input: z.string(),
  session_token: z.string().optional(),
});

function unauthorizedSessionResponse(publicUrl: string) {
  return {
    content: [{ type: 'text' as const, text: 'Login is required to perform this action.' }],
    _meta: {
      'mcp/www_authenticate': [
        `Bearer resource_metadata="${publicUrl}/.well-known/oauth-protected-resource"`,
      ],
    },
    isError: true,
  };
}

function resolveSessionToken(input: { session_token?: string }, context: any): string | null {
  const contextToken = context?._meta?.session_token ?? context?.session_token ?? null;
  if (typeof contextToken === 'string' && contextToken.length > 0) {
    return contextToken;
  }

  if (typeof input.session_token === 'string' && input.session_token.length > 0) {
    return input.session_token;
  }

  return null;
}

function resolveBearerToken(context: any): string | null {
  const authToken = context?.auth?.token ?? context?._meta?.auth?.token ?? null;
  if (typeof authToken === 'string' && authToken.length > 0) {
    return authToken;
  }
  const authorization = context?.headers?.authorization ?? context?._meta?.headers?.authorization ?? null;
  if (typeof authorization === 'string' && authorization.startsWith('Bearer ')) {
    return authorization.slice('Bearer '.length);
  }
  return null;
}

export function registerRunServiceTool(server: McpServer, config: BackendConfig): void {
  const client = new BackendClient(config);
  const verifier = new TokenVerifier({
    domain: config.auth0Domain,
    audience: config.auth0Audience,
  });

  registerAppTool(
    server,
    'run_service',
    {
      title: 'Run Service',
      description: 'Execute a service with sponsor-backed payment.',
      inputSchema: runServiceInputSchema.shape,
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        openWorldHint: true,
      },
      _meta: {
        securitySchemes: config.authEnabled
          ? [{ type: 'oauth2', scopes: ['services.execute'] }]
          : [{ type: 'noauth' }],
        'openai/toolInvocation/invoking': 'Running service...',
        'openai/toolInvocation/invoked': 'Service run completed',
      },
    },
    async (input, context: any) => {
      if (config.authEnabled) {
        const bearerToken = resolveBearerToken(context);
        const authInfo = bearerToken ? await verifier.verify(bearerToken) : null;
        if (!authInfo) {
          return unauthorizedSessionResponse(config.publicUrl);
        }
      }

      const sessionToken = resolveSessionToken(input, context);
      if (!sessionToken) {
        return unauthorizedSessionResponse(config.publicUrl);
      }

      try {
        const response = await client.runService(input.service, {
          service: input.service,
          session_token: sessionToken,
          input: input.input,
        });

        return {
          structuredContent: {
            service: response.service,
            payment_mode: response.payment_mode,
            sponsored_by: response.sponsored_by,
            tx_hash: response.tx_hash,
          },
          content: [{ type: 'text' as const, text: response.message }],
          _meta: {
            output: response.output,
          },
        };
      } catch (error) {
        if (error instanceof BackendClientError) {
          return {
            content: [{ type: 'text' as const, text: error.message }],
            _meta: { code: error.code, details: error.details },
            isError: true,
          };
        }

        return {
          content: [{ type: 'text' as const, text: 'An unexpected error occurred while running the service.' }],
          _meta: { code: 'unexpected_error' },
          isError: true,
        };
      }
    }
  );
}
