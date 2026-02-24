import { registerAppTool } from '@modelcontextprotocol/ext-apps/server';
import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';

import { BackendClient, BackendClientError } from '../backend-client.ts';
import type { BackendConfig } from '../config.ts';
import type {
  CreateCampaignRequest,
  CreateCampaignResponse,
  GptCandidateService,
  GptCandidateServiceOffer,
  GptSearchResponse,
  GptServiceItem,
  SearchServicesParams,
} from '../types.ts';

// ツール入力スキーマ定義
const createCampaignFromGoalInputSchema = z.object({
  purpose: z.string().trim().min(1),
  sponsor: z.string().trim().min(1),
  target_roles: z.array(z.string().trim().min(1)).min(1),
  target_tools: z.array(z.string().trim().min(1)).optional(),
  budget_cents: z.number().int().positive(),
  query_urls: z.array(z.string().url()).optional(),
  region: z.string().optional(),
  intent: z.string().optional(),
  max_budget_cents: z.number().int().positive().optional(),
});

// 候補選定結果の最小情報
type CandidateSelection = {
  service_key: string;
  offer: GptCandidateServiceOffer;
  source: 'candidate' | 'service';
};

/**
 * 検索結果から最適な候補を選定する
 *
 * @param response 候補検索のレスポンス
 * @returns 選定候補。候補なしの場合は null
 */
function selectCandidate(response: GptSearchResponse): CandidateSelection | null {
  const candidates = response.candidate_services ?? [];
  const candidate = candidates.find((item: GptCandidateService) => item.offers.length > 0);
  if (candidate) {
    const offer = candidate.offers.reduce((best, current) =>
      current.subsidy_amount_cents > best.subsidy_amount_cents ? current : best
    );
    return {
      service_key: candidate.service_key,
      offer,
      source: 'candidate',
    };
  }

  const services = response.services.filter(
    (service: GptServiceItem) => service.service_type === 'campaign' && service.required_task
  );
  if (services.length === 0) {
    return null;
  }

  const topService = services.reduce((best, current) =>
    current.subsidy_amount_cents > best.subsidy_amount_cents ? current : best
  );
  const fallbackOffer: GptCandidateServiceOffer = {
    campaign_id: topService.service_id,
    campaign_name: topService.name,
    sponsor: topService.sponsor,
    required_task: topService.required_task,
    subsidy_amount_cents: topService.subsidy_amount_cents,
  };
  const serviceKey = topService.category[0] ?? '';

  return {
    service_key: serviceKey,
    offer: fallbackOffer,
    source: 'service',
  };
}

/**
 * キャンペーン名を目的とスポンサーから生成する
 *
 * @param purpose 目的テキスト
 * @param sponsor スポンサー名
 * @returns 生成したキャンペーン名
 */
function buildCampaignName(purpose: string, sponsor: string): string {
  const trimmedPurpose = purpose.trim();
  const trimmedSponsor = sponsor.trim();
  return `${trimmedSponsor} ${trimmedPurpose}`.trim();
}

/**
 * 検索APIに渡す条件を組み立てる
 *
 * @param input ツール入力
 * @returns 検索パラメータ
 */
function buildSearchParams(input: z.infer<typeof createCampaignFromGoalInputSchema>): SearchServicesParams {
  const params: SearchServicesParams = {
    q: input.purpose,
    intent: input.intent,
  };
  if (typeof input.max_budget_cents === 'number') {
    params.max_budget_cents = input.max_budget_cents;
  } else {
    params.max_budget_cents = input.budget_cents;
  }
  return params;
}

function buildCreateCampaignRequest(
  input: z.infer<typeof createCampaignFromGoalInputSchema>,
  selection: CandidateSelection,
  subsidyPerCallCents: number,
  requiredTask: string
): CreateCampaignRequest {
  // target_tools が未指定の場合は選定した service_key を補完する
  const targetTools = input.target_tools?.length ? input.target_tools : [selection.service_key].filter(Boolean);
  return {
    name: buildCampaignName(input.purpose, input.sponsor),
    sponsor: input.sponsor,
    target_roles: input.target_roles,
    target_tools: targetTools,
    required_task: requiredTask,
    subsidy_per_call_cents: subsidyPerCallCents,
    budget_cents: input.budget_cents,
    query_urls: input.query_urls ?? [],
  };
}

function buildStructuredContent(
  config: BackendConfig,
  response: CreateCampaignResponse,
  searchResponse: GptSearchResponse,
  selection: CandidateSelection,
  subsidyPerCallCents: number,
  requiredTask: string
) {
  const frontendDashboardUrl = buildFrontendDashboardUrl(config.frontendUrl, response.campaign.id);
  const frontendCampaignUrl = buildFrontendCampaignUrl(config.frontendUrl, response.campaign.id);
  // モデル向けの構造化レスポンスを組み立てる
  return {
    campaign_id: response.campaign.id,
    campaign: response.campaign,
    frontend_dashboard_url: frontendDashboardUrl,
    frontend_campaign_url: frontendCampaignUrl,
    backend_campaign_url: response.campaign_url,
    backend_dashboard_api_url: response.dashboard_url,
    selected_service_key: selection.service_key,
    selected_offer: selection.offer,
    selected_services: searchResponse.candidate_services ?? searchResponse.services,
    selected_task: {
      required_task: requiredTask,
      subsidy_per_call_cents: subsidyPerCallCents,
    },
    rationale:
      selection.source === 'candidate'
        ? 'Selected the highest-subsidy offer from candidate services.'
        : 'Selected the highest-subsidy campaign from search results.',
  };
}

function normalizeBaseUrl(value: string): string {
  return value.trim().replace(/\/+$/, '');
}

function buildFrontendDashboardUrl(frontendUrl: string, campaignId: string): string {
  const base = normalizeBaseUrl(frontendUrl);
  if (!base) return '';
  try {
    const url = new URL(base);
    url.searchParams.set('view', 'dashboard');
    url.searchParams.set('campaign_id', campaignId);
    return url.toString();
  } catch {
    return '';
  }
}

function buildFrontendCampaignUrl(frontendUrl: string, campaignId: string): string {
  const base = normalizeBaseUrl(frontendUrl);
  if (!base) return '';
  try {
    const url = new URL(base);
    url.searchParams.set('view', 'dashboard');
    url.searchParams.set('campaign_id', campaignId);
    return url.toString();
  } catch {
    return '';
  }
}

export function registerCreateCampaignFromGoalTool(server: McpServer, config: BackendConfig): void {
  const client = new BackendClient(config);

  registerAppTool(
    server,
    'create_campaign_from_goal',
    {
      title: 'Create Campaign From Goal',
      description: 'Create a sponsor campaign from a purpose and target audience.',
      inputSchema: createCampaignFromGoalInputSchema.shape,
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        openWorldHint: false,
      },
      _meta: {
        securitySchemes: [{ type: 'noauth' }],
        'openai/toolInvocation/invoking': 'Creating campaign...',
        'openai/toolInvocation/invoked': 'Campaign created',
      },
    },
    /**
     * 目的/ターゲットからキャンペーンを自動作成する
     *
     * @param input ツール入力
     * @returns MCP ツール応答
     */
    async (input) => {
      try {
        const searchParams = buildSearchParams(input);
        const searchResponse = await client.searchServices(searchParams);
        const selection = selectCandidate(searchResponse);
        if (!selection) {
          return {
            content: [
              {
                type: 'text' as const,
                text: 'No suitable sponsored services found. Try a more specific purpose or adjust the budget.',
              },
            ],
            _meta: { code: 'no_candidate_service', details: searchResponse },
            isError: true,
          };
        }

        const requiredTask = selection.offer.required_task;
        if (!requiredTask) {
          return {
            content: [
              {
                type: 'text' as const,
                text: 'The selected service does not include a required task. Please refine the purpose.',
              },
            ],
            _meta: { code: 'missing_required_task', details: selection },
            isError: true,
          };
        }

        const subsidyPerCallCents = selection.offer.subsidy_amount_cents;
        if (input.budget_cents < subsidyPerCallCents) {
          return {
            content: [
              {
                type: 'text' as const,
                text: 'Budget is below the selected subsidy amount. Increase budget or adjust purpose.',
              },
            ],
            _meta: {
              code: 'budget_too_low',
              details: { budget_cents: input.budget_cents, subsidy_per_call_cents: subsidyPerCallCents },
            },
            isError: true,
          };
        }

        const request = buildCreateCampaignRequest(input, selection, subsidyPerCallCents, requiredTask);
        if (request.target_tools.length === 0) {
          return {
            content: [
              {
                type: 'text' as const,
                text: 'Target tools could not be determined. Provide target_tools explicitly.',
              },
            ],
            _meta: { code: 'missing_target_tools', details: selection },
            isError: true,
          };
        }

        const response = await client.createCampaign(request);
        const frontendDashboardUrl = buildFrontendDashboardUrl(config.frontendUrl, response.campaign.id);
        return {
          structuredContent: buildStructuredContent(
            config,
            response,
            searchResponse,
            selection,
            subsidyPerCallCents,
            requiredTask
          ),
          content: [
            {
              type: 'text' as const,
              text: frontendDashboardUrl
                ? `Campaign created: ${response.campaign.name}. Open in frontend dashboard: ${frontendDashboardUrl}`
                : `Campaign created: ${response.campaign.name}`,
            },
          ],
          _meta: {
            full_response: response,
            search_response: searchResponse,
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
            {
              type: 'text' as const,
              text: 'An unexpected error occurred while creating the campaign.',
            },
          ],
          _meta: { code: 'unexpected_error' },
          isError: true,
        };
      }
    }
  );
}
