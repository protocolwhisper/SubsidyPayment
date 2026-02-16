import { registerAppTool } from '@modelcontextprotocol/ext-apps/server';
import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';

import { TokenVerifier } from '../auth/token-verifier.ts';
import { BackendClient, BackendClientError } from '../backend-client.ts';
import type { BackendConfig } from '../config.ts';
import type { CompleteTaskInput } from '../types.ts';

const completeTaskInputSchema = z.object({
  campaign_id: z.string().uuid(),
  task_name: z.string(),
  details: z.string().optional(),
  session_token: z.string().optional(),
  consent: z.object({
    data_sharing_agreed: z.boolean(),
    purpose_acknowledged: z.boolean(),
    contact_permission: z.boolean(),
  }),
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

export function registerCompleteTaskTool(server: McpServer, config: BackendConfig): void {
  const client = new BackendClient(config);
  const verifier = new TokenVerifier({
    domain: config.auth0Domain,
    audience: config.auth0Audience,
  });

  registerAppTool(
    server,
    'complete_task',
    {
      title: 'Complete Task',
      description: 'Record task completion details and consent data.',
      inputSchema: completeTaskInputSchema.shape,
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        openWorldHint: false,
      },
      _meta: {
        securitySchemes: config.authEnabled
          ? [{ type: 'oauth2', scopes: ['tasks.write'] }]
          : [{ type: 'noauth' }],
        'openai/toolInvocation/invoking': 'Recording task completion...',
        'openai/toolInvocation/invoked': 'Task completed',
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

      const payload: CompleteTaskInput = {
        campaign_id: input.campaign_id,
        session_token: sessionToken,
        task_name: input.task_name,
        details: input.details,
        consent: input.consent,
      };

      try {
        const response = await client.completeTask(input.campaign_id, payload);
        return {
          structuredContent: {
            task_completion_id: response.task_completion_id,
            campaign_id: response.campaign_id,
            consent_recorded: response.consent_recorded,
            can_use_service: response.can_use_service,
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
          content: [{ type: 'text' as const, text: 'An unexpected error occurred while recording task completion.' }],
          _meta: { code: 'unexpected_error' },
          isError: true,
        };
      }
    }
  );
}
