import { registerAppTool } from '@modelcontextprotocol/ext-apps/server';
import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';

import { BackendClient, BackendClientError } from '../backend-client.ts';
import type { BackendConfig } from '../config.ts';
import type { GptSearchResponse, SearchServicesParams } from '../types.ts';

const searchServicesInputSchema = z.object({
  q: z.string().optional(),
  category: z.string().optional(),
  max_budget_cents: z.number().int().nonnegative().optional(),
  intent: z.string().optional(),
});

function toSearchServicesResult(response: GptSearchResponse) {
  const services = response.services.filter((service) => service.service_type === 'campaign');
  const candidateServices = response.candidate_services ?? [];
  const serviceCatalog = response.service_catalog ?? [];
  const sponsorCatalog = response.sponsor_catalog ?? [];
  const totalCount = services.length;
  const message =
    totalCount === 0
      ? 'No campaign-backed sponsored services found. Please create or activate a sponsor campaign first.'
      : `Found ${totalCount} campaign-backed sponsored service(s) across ${serviceCatalog.length} service(s) and ${sponsorCatalog.length} sponsor(s).`;

  return {
    structuredContent: {
      services,
      total_count: totalCount,
      candidate_services: candidateServices,
      service_catalog: serviceCatalog,
      sponsor_catalog: sponsorCatalog,
      applied_filters: response.applied_filters,
      available_categories: response.available_categories,
    },
    content: [
      { type: 'text' as const, text: message },
    ],
    _meta: {
      'openai/outputTemplate': 'ui://widget/services-list.html',
      full_response: response,
    },
  };
}

export function registerSearchServicesTool(server: McpServer, config: BackendConfig): void {
  const client = new BackendClient(config);

  registerAppTool(
    server,
    'search_services',
    {
      title: 'Search Services',
      description: 'Search available sponsored services.',
      inputSchema: searchServicesInputSchema.shape,
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: false,
      },
      _meta: {
        securitySchemes: [{ type: 'noauth' }],
        ui: { resourceUri: 'ui://widget/services-list.html' },
        'openai/resultCanProduceWidget': true,
        'openai/widgetAccessible': true,
        'openai/widgetDescription': 'Interactive list of sponsored services with tap-to-select cards.',
        'openai/toolInvocation/invoking': 'Searching services...',
        'openai/toolInvocation/invoked': 'Services found',
        'openai/outputTemplate': 'ui://widget/services-list.html',
      },
    },
    async (input: SearchServicesParams) => {
      try {
        const response = await client.searchServices(input);
        return toSearchServicesResult(response);
      } catch (error) {
        if (error instanceof BackendClientError) {
          return {
            content: [{ type: 'text' as const, text: error.message }],
            _meta: { code: error.code, details: error.details },
            isError: true,
          };
        }

        return {
          content: [{ type: 'text' as const, text: 'An unexpected error occurred while searching services.' }],
          _meta: { code: 'unexpected_error' },
          isError: true,
        };
      }
    }
  );
}
