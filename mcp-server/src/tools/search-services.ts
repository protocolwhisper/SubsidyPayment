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

function buildNextActions(query?: string) {
  const keyword = typeof query === 'string' && query.trim().length > 0 ? query.trim() : 'github';
  return [
    {
      action: '候補サービスのタスクを見る',
      prompt: `Please run get_service_tasks with service_key=${keyword}.`,
      tool: 'get_service_tasks',
    },
  ];
}

function toSearchServicesResult(response: GptSearchResponse) {
  const services = response.services.filter((service) => service.service_type === 'campaign');
  const candidateServices = response.candidate_services ?? [];
  const serviceCatalog = response.service_catalog ?? [];
  const sponsorCatalog = response.sponsor_catalog ?? [];
  const totalCount = services.length;
  const message =
    totalCount === 0
      ? 'No campaign-backed sponsored services found. Please create or activate a sponsor campaign first.'
      : 'Interactive sponsored services list ready in the widget.';

  return {
    structuredContent: {
      services,
      total_count: totalCount,
      candidate_services: candidateServices,
      service_catalog: serviceCatalog,
      sponsor_catalog: sponsorCatalog,
      applied_filters: response.applied_filters,
      available_categories: response.available_categories,
      next_actions: buildNextActions(response.applied_filters?.keyword ?? response.applied_filters?.intent ?? ''),
    },
    content: [
      { type: 'text' as const, text: message },
    ],
    _meta: {
      'openai/outputTemplate': 'ui://widget/services-list.html',
      'openai/widgetDescription':
        'Use the widget as the primary UI. After selection, ask the user to continue in chat with: "get_service_tasks を実行してください。service_key を指定します。"',
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
      description:
        'Search available sponsored services and return a guided next step to continue the flow without confusion.',
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
        'openai/widgetDescription':
          'Interactive sponsored services list. After selecting a card, continue in chat using get_service_tasks or get_task_details.',
        'openai/toolInvocation/invoking': 'Searching services...',
        'openai/toolInvocation/invoked': 'Services found. Next prompt is ready.',
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
            content: [{ type: 'text' as const, text: `${error.message} 次は「get_prompt_guide_flow を実行してください」と入力してください。` }],
            _meta: { code: error.code, details: error.details },
            isError: true,
          };
        }

        return {
          content: [{ type: 'text' as const, text: 'An unexpected error occurred while searching services. 次は「get_prompt_guide_flow を実行してください」と入力してください。' }],
          _meta: { code: 'unexpected_error' },
          isError: true,
        };
      }
    }
  );
}
