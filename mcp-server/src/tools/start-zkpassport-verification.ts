import { registerAppTool } from '@modelcontextprotocol/ext-apps/server';
import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';

import { TokenVerifier } from '../auth/token-verifier.ts';
import { BackendClient, BackendClientError } from '../backend-client.ts';
import type { BackendConfig } from '../config.ts';
import { resolveOrCreateNoAuthSessionToken } from './session-manager.ts';

const startZkpassportVerificationInputSchema = z.object({
  campaign_id: z.string().uuid(),
  consent: z.object({
    data_sharing_agreed: z.boolean(),
    purpose_acknowledged: z.boolean(),
    contact_permission: z.boolean(),
  }),
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

export function registerStartZkpassportVerificationTool(server: McpServer, config: BackendConfig): void {
  const client = new BackendClient(config);
  const verifier = new TokenVerifier({
    domain: config.auth0Domain,
    audience: config.auth0Audience,
  });

  registerAppTool(
    server,
    'start_zkpassport_verification',
    {
      title: 'Start zkPassport Verification',
      description: 'Create a verification session for age and country proof.',
      inputSchema: startZkpassportVerificationInputSchema.shape,
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        openWorldHint: true,
      },
      _meta: {
        securitySchemes: config.authEnabled
          ? [{ type: 'oauth2', scopes: ['tasks.write'] }]
          : [{ type: 'noauth' }],
        'openai/toolInvocation/invoking': 'Creating verification session...',
        'openai/toolInvocation/invoked': 'Verification session ready',
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

        const response = await client.initZkpassportVerification(input.campaign_id, {
          campaign_id: input.campaign_id,
          session_token: sessionToken,
          consent: input.consent,
        });

        return {
          structuredContent: {
            verification_id: response.verification_id,
            verification_token: response.verification_token,
            campaign_id: response.campaign_id,
            verification_url: response.verification_url,
            expires_at: response.expires_at,
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
          content: [{ type: 'text' as const, text: 'An unexpected error occurred while starting verification.' }],
          _meta: { code: 'unexpected_error' },
          isError: true,
        };
      }
    }
  );
}
