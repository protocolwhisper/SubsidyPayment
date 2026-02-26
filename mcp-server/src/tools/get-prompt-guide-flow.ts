import { registerAppTool } from '@modelcontextprotocol/ext-apps/server';
import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';

import type { BackendConfig } from '../config.ts';

const getPromptGuideFlowInputSchema = z.object({
  context_step: z.string().optional(),
  service: z.string().optional(),
  campaign_id: z.string().optional(),
});

type FlowStep = '0' | '1' | '2' | '3' | '4' | '5';

interface FlowDefinition {
  step: FlowStep;
  goal: string;
  recommendedNextPrompt: string;
  copyPastePrompts: string[];
  nextActions: Array<{ action: string; prompt: string; tool: string }>;
}

const ALLOWED_ACTIONS = [
  'get_prompt_guide_flow',
  'search_services',
  'authenticate_user',
  'get_service_tasks',
  'get_task_details',
  'start_zkpassport_verification',
  'complete_task',
  'run_service',
  'get_user_status',
  'user_record',
  'get_preferences',
  'set_preferences',
  'weather',
  'create_github_issue',
] as const;

function normalizeStep(raw?: string): FlowStep {
  const normalized = String(raw ?? '').trim();
  if (normalized === '1' || normalized === '2' || normalized === '3' || normalized === '4' || normalized === '5') {
    return normalized;
  }
  return '0';
}

function buildFlow(step: FlowStep, service?: string, campaignId?: string): FlowDefinition {
  const resolvedService = typeof service === 'string' && service.trim().length > 0 ? service.trim() : 'github';
  const resolvedCampaign = typeof campaignId === 'string' && campaignId.trim().length > 0 ? campaignId.trim() : '<campaign_id>';

  switch (step) {
    case '1':
      return {
        step,
        goal: '候補サービスを比較して、次に進む1件を選ぶ。',
        recommendedNextPrompt: `サービス候補から ${resolvedService} 系を1件選び、次に実行する get_service_tasks の入力を1行で示してください。`,
        copyPastePrompts: [
          `search_services を使って ${resolvedService} の候補を再表示してください。`,
          `候補の中から1件選び、選定理由を1行で説明してください。`,
        ],
        nextActions: [
          {
            action: '選んだサービスのタスクを確認',
            prompt: `Please run get_service_tasks with service_key=${resolvedService}.`,
            tool: 'get_service_tasks',
          },
        ],
      };
    case '2':
      return {
        step,
        goal: '対象キャンペーンの必須タスク詳細を確定する。',
        recommendedNextPrompt: `選択した campaign_id で get_task_details を実行してください。campaign_id は ${resolvedCampaign} です。`,
        copyPastePrompts: [
          `get_task_details を campaign_id=${resolvedCampaign} で実行してください。`,
          '返ってきた required_task と task_input_format の必須項目だけを箇条書きで出してください。',
        ],
        nextActions: [
          {
            action: 'タスク入力要件を確認',
            prompt: `Please rerun get_task_details with campaign_id=${resolvedCampaign}.`,
            tool: 'get_task_details',
          },
        ],
      };
    case '3':
      return {
        step,
        goal: 'テンプレに沿って回答し、タスク完了を登録する。',
        recommendedNextPrompt:
          'complete_task を実行してください。consent 3項目を true/true/false で設定し、details には回答テンプレをそのまま入れてください。',
        copyPastePrompts: [
          'complete_task 実行前に、送信する payload を表示してください。',
          'complete_task 実行後に can_use_service と task_completion_id を1行で要約してください。',
        ],
        nextActions: [
          {
            action: 'タスク完了を登録',
            prompt: 'Please run complete_task to mark the task as completed.',
            tool: 'complete_task',
          },
        ],
      };
    case '4':
      return {
        step,
        goal: '完了後に解放されたサービス実行可否を確認する。',
        recommendedNextPrompt: `get_user_status を実行して、${resolvedService} が実行可能か確認してください。`,
        copyPastePrompts: [
          'get_user_status を実行して completed_tasks と available_services を表示してください。',
          `次に run_service へ渡す service 名を1つだけ確定してください（候補: ${resolvedService}）。`,
        ],
        nextActions: [
          {
            action: '解放状態を確認',
            prompt: 'Please run get_user_status.',
            tool: 'get_user_status',
          },
        ],
      };
    case '5':
      return {
        step,
        goal: 'サービスを実行し、結果を確認する。',
        recommendedNextPrompt: `run_service を実行してください。service=${resolvedService}、input は実行したい具体的な依頼文にしてください。`,
        copyPastePrompts: [
          `run_service を service=${resolvedService} で実行してください。`,
          '結果の payment_mode / sponsored_by / output要約 を3行で出してください。',
        ],
        nextActions: [
          {
            action: 'サービス実行',
            prompt: `Please run run_service with service=${resolvedService}.`,
            tool: 'run_service',
          },
        ],
      };
    case '0':
    default:
      return {
        step: '0',
        goal: '迷わないよう、最初に固定ガイドで開始する。',
        recommendedNextPrompt: `search_services を使って ${resolvedService} の候補を出してください。`,
        copyPastePrompts: [
          `search_services を q=${resolvedService} で実行してください。`,
          `候補を見て、次に使う service_key を1つ選んでください。`,
          '迷ったら get_prompt_guide_flow を context_step=1 で再実行してください。',
        ],
        nextActions: [
          {
            action: 'サービス検索を開始',
            prompt: `Please run search_services with q=${resolvedService}.`,
            tool: 'search_services',
          },
        ],
      };
  }
}

export function registerGetPromptGuideFlowTool(server: McpServer, _config: BackendConfig): void {
  registerAppTool(
    server,
    'get_prompt_guide_flow',
    {
      title: 'Get Prompt Guide Flow',
      description: 'Return the exact next prompt and allowed actions for the current guided flow step.',
      inputSchema: getPromptGuideFlowInputSchema.shape,
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: false,
      },
      _meta: {
        securitySchemes: [{ type: 'noauth' }],
        'openai/toolInvocation/invoking': 'Loading guided flow...',
        'openai/toolInvocation/invoked': 'Guided flow ready. Copy the next prompt.',
        'openai/widgetDescription':
          'Use this tool when the user is unsure. Return one explicit next prompt and avoid inventing alternative flows.',
      },
    },
    async (input) => {
      const flow = buildFlow(normalizeStep(input.context_step), input.service, input.campaign_id);
      return {
        structuredContent: {
          flow_step: flow.step,
          goal: flow.goal,
          recommended_next_prompt: flow.recommendedNextPrompt,
          copy_paste_prompts: flow.copyPastePrompts.slice(0, 3),
          allowed_actions: ALLOWED_ACTIONS,
          next_actions: flow.nextActions,
        },
        content: [
          {
            type: 'text' as const,
            text: `Step ${flow.step}: ${flow.goal} 次はこれを入力してください: ${flow.recommendedNextPrompt}`,
          },
        ],
      };
    }
  );
}
