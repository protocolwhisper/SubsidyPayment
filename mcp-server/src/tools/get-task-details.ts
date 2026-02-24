import { registerAppTool } from '@modelcontextprotocol/ext-apps/server';
import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';

import { TokenVerifier } from '../auth/token-verifier.ts';
import { BackendClient, BackendClientError } from '../backend-client.ts';
import type { BackendConfig } from '../config.ts';
import { resolveOrCreateNoAuthSessionToken } from './session-manager.ts';

const getTaskDetailsInputSchema = z.object({
  campaign_id: z.string().min(1, 'campaign_id is required'),
  session_token: z.string().optional(),
});

const UUID_REGEX = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
function isUuid(s: string): boolean {
  return UUID_REGEX.test(s);
}

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

export function registerGetTaskDetailsTool(server: McpServer, config: BackendConfig): void {
  const client = new BackendClient(config);
  const verifier = new TokenVerifier({
    domain: config.auth0Domain,
    audience: config.auth0Audience,
  });

  registerAppTool(
    server,
    'get_task_details',
    {
      title: 'Get Task Details',
      description: 'Get required task details for a campaign.',
      inputSchema: getTaskDetailsInputSchema.shape,
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: false,
      },
      _meta: {
        securitySchemes: config.authEnabled
          ? [{ type: 'oauth2', scopes: ['tasks.read'] }]
          : [{ type: 'noauth' }],
        ui: { resourceUri: 'ui://widget/task-form.html' },
        'openai/resultCanProduceWidget': true,
        'openai/widgetAccessible': true,
        'openai/toolInvocation/invoking': 'Fetching task details...',
        'openai/toolInvocation/invoked': 'Task details fetched',
        'openai/outputTemplate': 'ui://widget/task-form.html',
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
        if (!isUuid(input.campaign_id)) {
          return {
            content: [{ type: 'text' as const, text: 'Invalid campaign ID format. Please select a task from the list.' }],
            _meta: { code: 'invalid_campaign_id' },
            isError: true,
          };
        }

        const sessionToken = await resolveOrCreateNoAuthSessionToken(client, config, input, context);
        if (!sessionToken) {
          return unauthorizedSessionResponse(config.publicUrl);
        }

        const response = await client.getTaskDetails(input.campaign_id, sessionToken);

        return {
          structuredContent: {
            campaign_id: response.campaign_id,
            campaign_name: response.campaign_name,
            sponsor: response.sponsor,
            required_task: response.required_task,
            task_description: response.task_description,
            task_input_format: response.task_input_format,
            already_completed: response.already_completed,
            subsidy_amount_cents: response.subsidy_amount_cents,
          },
          content: [
            { type: 'text' as const, text: response.message },
          ],
          _meta: {
            'openai/outputTemplate': 'ui://widget/task-form.html',
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
          content: [{ type: 'text' as const, text: 'An unexpected error occurred while fetching task details.' }],
          _meta: { code: 'unexpected_error' },
          isError: true,
        };
      }
    }
  );
}
