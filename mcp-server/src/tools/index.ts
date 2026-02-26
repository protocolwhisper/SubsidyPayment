import type { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';

import type { BackendConfig } from '../config.ts';
import { registerAuthenticateUserTool } from './authenticate-user.ts';
import { registerCompleteTaskTool } from './complete-task.ts';
import { registerCreateCampaignFromGoalTool } from './create-campaign-from-goal.ts';
import { registerGetPreferencesTool } from './get-preferences.ts';
import { registerGetPromptGuideFlowTool } from './get-prompt-guide-flow.ts';
import { registerGetServiceTasksTool } from './get-service-tasks.ts';
import { registerGetTaskDetailsTool } from './get-task-details.ts';
import { registerGetUserStatusTool } from './get-user-status.ts';
import { registerCreateGithubIssueTool } from './github-issue.ts';
import { registerRunServiceTool } from './run-service.ts';
import { registerSearchServicesTool } from './search-services.ts';
import { registerSetPreferencesTool } from './set-preferences.ts';
import { registerStartZkpassportVerificationTool } from './start-zkpassport-verification.ts';
import { registerUserRecordTool } from './user-record.ts';
import { registerWeatherTool } from './weather.ts';

/**
 * MCP ツール群をまとめて登録する
 *
 * @param server MCP サーバー
 * @param config バックエンド設定
 */
export function registerAllTools(server: McpServer, config: BackendConfig): void {
  registerSearchServicesTool(server, config);
  registerGetPromptGuideFlowTool(server, config);
  registerAuthenticateUserTool(server, config);
  registerCreateCampaignFromGoalTool(server, config);
  registerGetServiceTasksTool(server, config);
  registerGetTaskDetailsTool(server, config);
  registerStartZkpassportVerificationTool(server, config);
  registerCompleteTaskTool(server, config);
  registerRunServiceTool(server, config);
  registerGetUserStatusTool(server, config);
  registerUserRecordTool(server, config);
  registerGetPreferencesTool(server, config);
  registerSetPreferencesTool(server, config);
  registerWeatherTool(server, config);
  registerCreateGithubIssueTool(server, config);
}
