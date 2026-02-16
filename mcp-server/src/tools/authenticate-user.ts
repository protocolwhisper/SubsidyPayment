import { registerAppTool } from '@modelcontextprotocol/ext-apps/server';
import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';

import { BackendClient, BackendClientError } from '../backend-client.ts';
import { TokenVerifier } from '../auth/token-verifier.ts';
import type { BackendConfig } from '../config.ts';


const authenticateUserInputSchema = z.object({
  email: z.string().email().optional(),
  region: z.string().default('auto'),
  roles: z.array(z.string()).optional().default([]),
  tools_used: z.array(z.string()).optional().default([]),
});

function unauthorizedAuthResponse(publicUrl: string) {
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

function resolveOAuthEmail(input: { email?: string }, context: any): string | null {
  const authEmail = context?.auth?.email ?? context?._meta?.auth?.email ?? null;
  if (typeof authEmail === 'string' && authEmail.length > 0) {
    return authEmail;
  }

  if (typeof input.email === 'string' && input.email.length > 0) {
    return input.email;
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

export function registerAuthenticateUserTool(server: McpServer, config: BackendConfig): void {
  const client = new BackendClient(config);
  const verifier = new TokenVerifier({
    domain: config.auth0Domain,
    audience: config.auth0Audience,
  });

  registerAppTool(
    server,
    'authenticate_user',
    {
      title: 'Authenticate User',
      description: 'Authenticate and register a user using OAuth token details.',
      inputSchema: authenticateUserInputSchema.shape,
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        openWorldHint: false,
      },
      _meta: {
        securitySchemes: config.authEnabled
          ? [{ type: 'oauth2', scopes: ['user.write'] }]
          : [{ type: 'noauth' }],
        'openai/toolInvocation/invoking': 'Authenticating user...',
        'openai/toolInvocation/invoked': 'Authentication complete',
      },
    },
    async (input, context) => {
      let oauthEmail: string | null = null;

      if (config.authEnabled) {
        const bearerToken = resolveBearerToken(context);
        const authInfo = bearerToken ? await verifier.verify(bearerToken) : null;
        if (!authInfo) {
          return unauthorizedAuthResponse(config.publicUrl);
        }
        oauthEmail = authInfo.email ?? resolveOAuthEmail(input, context);
      } else {
        oauthEmail = resolveOAuthEmail(input, context);
      }

      if (!oauthEmail) {
        return config.authEnabled
          ? unauthorizedAuthResponse(config.publicUrl)
          : {
              content: [{ type: 'text' as const, text: 'Auth is disabled. Please provide the email field.' }],
              isError: true,
            };
      }

      try {
        const response = await client.authenticateUser({
          email: oauthEmail,
          region: input.region ?? 'auto',
          roles: input.roles ?? [],
          tools_used: input.tools_used ?? [],
        });

        return {
          structuredContent: {
            user_id: response.user_id,
            email: response.email,
            is_new_user: response.is_new_user,
          },
          content: [{ type: 'text' as const, text: response.message }],
          _meta: {
            session_token: response.session_token,
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
          content: [{ type: 'text' as const, text: 'An unexpected error occurred during authentication.' }],
          _meta: { code: 'unexpected_error' },
          isError: true,
        };
      }
    }
  );
}
