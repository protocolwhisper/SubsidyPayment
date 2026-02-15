import { registerAppTool } from '@modelcontextprotocol/ext-apps/server';
import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';

import { TokenVerifier } from '../auth/token-verifier.ts';
import { BackendClient, BackendClientError } from '../backend-client.ts';
import type { BackendConfig } from '../config.ts';

const getUserStatusInputSchema = z.object({
  session_token: z.string().optional(),
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
      title: 'ユーザー状態確認',
      description: 'ユーザーの登録状態、完了済みタスク、利用可能サービスを確認する。',
      inputSchema: getUserStatusInputSchema,
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: false,
      },
      securitySchemes: [{ type: 'oauth2', scopes: ['user.read'] }],
      _meta: {
        ui: { resourceUri: 'ui://widget/user-dashboard.html' },
        'openai/toolInvocation/invoking': 'ステータスを確認中...',
        'openai/toolInvocation/invoked': 'ステータスを取得しました',
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
        const response = await client.getUserStatus(sessionToken);
        return {
          structuredContent: {
            user_id: response.user_id,
            email: response.email,
            completed_tasks: response.completed_tasks,
            available_services: response.available_services,
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
          content: [{ type: 'text' as const, text: 'ユーザー状態取得中に予期しないエラーが発生しました。' }],
          _meta: { code: 'unexpected_error' },
          isError: true,
        };
      }
    }
  );
}
