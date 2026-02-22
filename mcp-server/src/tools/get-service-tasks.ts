import { registerAppTool } from '@modelcontextprotocol/ext-apps/server';
import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';

import { TokenVerifier } from '../auth/token-verifier.ts';
import { BackendClient, BackendClientError } from '../backend-client.ts';
import type { BackendConfig } from '../config.ts';
import type { GptCandidateServiceOffer, GptSearchResponse } from '../types.ts';
import { resolveOrCreateNoAuthSessionToken } from './session-manager.ts';

const getServiceTasksInputSchema = z.object({
  service_key: z.string(),
  session_token: z.string().optional(),
});

interface ServiceTaskItem {
  campaign_id: string;
  campaign_name: string;
  sponsor: string;
  required_task: string | null;
  subsidy_amount_cents: number;
  category: string[];
  tags: string[];
  active: boolean;
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

function buildTasksFromSearchResponse(
  serviceKey: string,
  response: GptSearchResponse
): {
  display_name: string;
  tasks: ServiceTaskItem[];
  sponsor_names: string[];
  total_subsidy_cents: number;
} | null {
  const normalizedKey = serviceKey.toLowerCase();

  // Try candidate_services first (pre-aggregated by service)
  const candidate = response.candidate_services?.find(
    (cs) => cs.service_key.toLowerCase() === normalizedKey
  );

  if (candidate && candidate.offers.length > 0) {
    return buildFromCandidateOffers(candidate.display_name, candidate.offers, response);
  }

  // Fallback: filter raw services array by name match
  const matchedServices = response.services.filter(
    (svc) =>
      svc.service_type === 'campaign' &&
      svc.name.toLowerCase().includes(normalizedKey)
  );

  if (matchedServices.length > 0) {
    const tasks: ServiceTaskItem[] = matchedServices.map((svc) => ({
      campaign_id: svc.service_id,
      campaign_name: svc.name,
      sponsor: svc.sponsor,
      required_task: svc.required_task,
      subsidy_amount_cents: svc.subsidy_amount_cents,
      category: svc.category,
      tags: svc.tags,
      active: svc.active,
    }));

    const sponsorNames = [...new Set(tasks.map((t) => t.sponsor))];
    const totalSubsidyCents = tasks.reduce((sum, t) => sum + t.subsidy_amount_cents, 0);

    return {
      display_name: serviceKey,
      tasks,
      sponsor_names: sponsorNames,
      total_subsidy_cents: totalSubsidyCents,
    };
  }

  return null;
}

function buildFromCandidateOffers(
  displayName: string,
  offers: GptCandidateServiceOffer[],
  response: GptSearchResponse
): {
  display_name: string;
  tasks: ServiceTaskItem[];
  sponsor_names: string[];
  total_subsidy_cents: number;
} {
  const tasks: ServiceTaskItem[] = offers.map((offer) => {
    // Enrich with category/tags from the raw services array if available
    const rawService = response.services.find(
      (svc) => svc.service_id === offer.campaign_id
    );

    return {
      campaign_id: offer.campaign_id,
      campaign_name: offer.campaign_name,
      sponsor: offer.sponsor,
      required_task: offer.required_task,
      subsidy_amount_cents: offer.subsidy_amount_cents,
      category: rawService?.category ?? [],
      tags: rawService?.tags ?? [],
      active: rawService?.active ?? true,
    };
  });

  const sponsorNames = [...new Set(tasks.map((t) => t.sponsor))];
  const totalSubsidyCents = tasks.reduce((sum, t) => sum + t.subsidy_amount_cents, 0);

  return {
    display_name: displayName,
    tasks,
    sponsor_names: sponsorNames,
    total_subsidy_cents: totalSubsidyCents,
  };
}

export function registerGetServiceTasksTool(server: McpServer, config: BackendConfig): void {
  const client = new BackendClient(config);
  const verifier = new TokenVerifier({
    domain: config.auth0Domain,
    audience: config.auth0Audience,
  });

  registerAppTool(
    server,
    'get_service_tasks',
    {
      title: 'Get Service Tasks',
      description: 'List all available subsidized tasks for a specific service.',
      inputSchema: getServiceTasksInputSchema.shape,
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: false,
      },
      _meta: {
        securitySchemes: config.authEnabled
          ? [{ type: 'oauth2', scopes: ['tasks.read'] }]
          : [{ type: 'noauth' }],
        ui: { resourceUri: 'ui://widget/service-tasks.html' },
        'openai/resultCanProduceWidget': true,
        'openai/widgetAccessible': true,
        'openai/toolInvocation/invoking': 'Loading service tasks...',
        'openai/toolInvocation/invoked': 'Service tasks loaded',
        'openai/outputTemplate': 'ui://widget/service-tasks.html',
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

        const searchResponse = await client.searchServices({
          q: input.service_key,
          session_token: sessionToken ?? undefined,
        });

        const result = buildTasksFromSearchResponse(input.service_key, searchResponse);

        if (!result || result.tasks.length === 0) {
          return {
            structuredContent: {
              service_key: input.service_key,
              display_name: input.service_key,
              tasks: [],
              task_count: 0,
              sponsor_names: [],
              total_subsidy_cents: 0,
            },
            content: [
              {
                type: 'text' as const,
                text: `No subsidized tasks found for service "${input.service_key}".`,
              },
            ],
            _meta: {
              'openai/outputTemplate': 'ui://widget/service-tasks.html',
            },
          };
        }

        const message =
          `Found ${result.tasks.length} subsidized task(s) for ${result.display_name}` +
          ` from ${result.sponsor_names.length} sponsor(s).` +
          ` Total available subsidy: $${(result.total_subsidy_cents / 100).toFixed(2)}.`;


        return {
          structuredContent: {
            service_key: input.service_key,
            display_name: result.display_name,
            tasks: result.tasks,
            task_count: result.tasks.length,
            sponsor_names: result.sponsor_names,
            total_subsidy_cents: result.total_subsidy_cents,
          },
          content: [
            { type: 'text' as const, text: message },
          ],
          _meta: {
            'openai/outputTemplate': 'ui://widget/service-tasks.html',
            full_response: searchResponse,
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
          content: [
            { type: 'text' as const, text: 'An unexpected error occurred while fetching service tasks.' },
          ],
          _meta: { code: 'unexpected_error' },
          isError: true,
        };
      }
    }
  );
}
