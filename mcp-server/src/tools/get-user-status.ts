import { registerAppTool } from '@modelcontextprotocol/ext-apps/server';
import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';

import { TokenVerifier } from '../auth/token-verifier.ts';
import { BackendClient, BackendClientError } from '../backend-client.ts';
import type { BackendConfig } from '../config.ts';
import { resolveOrCreateNoAuthSessionToken } from './session-manager.ts';

const getUserStatusInputSchema = z.object({
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

export function registerGetUserStatusTool(server: McpServer, config: BackendConfig): void {
  const client = new BackendClient(config);
  const verifier = new TokenVerifier({
    domain: config.auth0Domain,
    audience: config.auth0Audience,
  });

  registerAppTool(
    server,
    'get_user_status',
    {
      title: 'Get User Status',
      description: 'Check registration status, completed tasks, and available services for the user.',
      inputSchema: getUserStatusInputSchema.shape,
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: false,
      },
      _meta: {
        securitySchemes: config.authEnabled
          ? [{ type: 'oauth2', scopes: ['user.read'] }]
          : [{ type: 'noauth' }],
        ui: { resourceUri: 'ui://widget/user-dashboard.html' },
        'openai/resultCanProduceWidget': true,
        'openai/widgetAccessible': true,
        'openai/toolInvocation/invoking': 'Fetching user status...',
        'openai/toolInvocation/invoked': 'User status fetched',
        'openai/outputTemplate': 'ui://widget/user-dashboard.html',
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

      try {
        const sessionToken = await resolveOrCreateNoAuthSessionToken(client, config, input, context);
        if (!sessionToken) {
          return unauthorizedSessionResponse(config.publicUrl);
        }

        const response = await client.getUserStatus(sessionToken);

        return {
          structuredContent: {
            user_id: response.user_id,
            email: response.email,
            completed_tasks: response.completed_tasks,
            available_services: response.available_services,
          },
          content: [
            { type: 'text' as const, text: response.message },
          ],
          _meta: {
            'openai/outputTemplate': 'ui://widget/user-dashboard.html',
            full_response: response,
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
          content: [{ type: 'text' as const, text: 'An unexpected error occurred while fetching user status.' }],
          _meta: { code: 'unexpected_error' },
          isError: true,
        };
      }
    }
  );
}
