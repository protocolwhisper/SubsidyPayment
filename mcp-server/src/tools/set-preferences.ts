import { registerAppTool } from '@modelcontextprotocol/ext-apps/server';
import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';

import { TokenVerifier } from '../auth/token-verifier.ts';
import { BackendClient, BackendClientError } from '../backend-client.ts';
import type { BackendConfig } from '../config.ts';

const setPreferencesInputSchema = z.object({
  session_token: z.string().optional(),
  preferences: z.array(
    z.object({
      task_type: z.string(),
      level: z.enum(['preferred', 'neutral', 'avoided']),
    })
  ),
});

function unauthorizedSessionResponse(publicUrl: string) {
  return {
    content: [{ type: 'text' as const, text: 'このアクションを実行するにはログインが必要です。' }],
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

export function registerSetPreferencesTool(server: McpServer, config: BackendConfig): void {
  const client = new BackendClient(config);
  const verifier = new TokenVerifier({
    domain: config.auth0Domain,
    audience: config.auth0Audience,
  });

  registerAppTool(
    server,
    'set_preferences',
    {
      title: '設定変更',
      description: 'ユーザーのタスク設定を更新する。',
      inputSchema: setPreferencesInputSchema,
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        openWorldHint: false,
      },
      securitySchemes: [{ type: 'oauth2', scopes: ['user.write'] }],
      _meta: {
        'openai/toolInvocation/invoking': '設定を更新中...',
        'openai/toolInvocation/invoked': '設定を更新しました',
      },
    },
    async (input, context: any) => {
      const bearerToken = resolveBearerToken(context);
      const authInfo = bearerToken ? await verifier.verify(bearerToken) : null;
      if (!authInfo) {
        return unauthorizedSessionResponse(config.publicUrl);
      }

      const sessionToken = resolveSessionToken(input, context);
      if (!sessionToken) {
        return unauthorizedSessionResponse(config.publicUrl);
      }

      try {
        const response = await client.setPreferences({
          session_token: sessionToken,
          preferences: input.preferences,
        });
        return {
          structuredContent: {
            user_id: response.user_id,
            preferences_count: response.preferences_count,
            updated_at: response.updated_at,
          },
          content: [{ type: 'text' as const, text: response.message }],
          _meta: {
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
          content: [{ type: 'text' as const, text: '設定更新中に予期しないエラーが発生しました。' }],
          _meta: { code: 'unexpected_error' },
          isError: true,
        };
      }
    }
  );
}
