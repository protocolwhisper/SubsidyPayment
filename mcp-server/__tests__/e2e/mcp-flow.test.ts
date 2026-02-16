import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocked = vi.hoisted(() => ({
  registrations: new Map<string, { handler: (input: any, context?: any) => Promise<any> }>(),
  registerAppTool: vi.fn(),
  searchServices: vi.fn(),
  authenticateUser: vi.fn(),
  getTaskDetails: vi.fn(),
  completeTask: vi.fn(),
  runService: vi.fn(),
  verifyToken: vi.fn(),
}));

vi.mock('@modelcontextprotocol/ext-apps/server', () => ({
  registerAppTool: (...args: any[]) => mocked.registerAppTool(...args),
}));

vi.mock('../../src/backend-client.ts', async () => {
  const actual = await vi.importActual<typeof import('../../src/backend-client.ts')>(
    '../../src/backend-client.ts'
  );

  class BackendClient {
    searchServices = mocked.searchServices;
    authenticateUser = mocked.authenticateUser;
    getTaskDetails = mocked.getTaskDetails;
    completeTask = mocked.completeTask;
    runService = mocked.runService;
    getUserStatus = vi.fn();
    getPreferences = vi.fn();
    setPreferences = vi.fn();
  }

  return {
    ...actual,
    BackendClient,
  };
});

vi.mock('../../src/auth/token-verifier.ts', async () => {
  const actual = await vi.importActual<typeof import('../../src/auth/token-verifier.ts')>(
    '../../src/auth/token-verifier.ts'
  );

  class TokenVerifier {
    verify = mocked.verifyToken;
  }

  return {
    ...actual,
    TokenVerifier,
  };
});

import { registerAllTools } from '../../src/tools/index.ts';

const config = {
  rustBackendUrl: 'http://localhost:3000',
  mcpInternalApiKey: 'internal',
  auth0Domain: 'tenant.example.com',
  auth0Audience: 'https://example.com/api',
  publicUrl: 'http://localhost:3001',
  port: 3001,
  logLevel: 'info',
  authEnabled: true,
};

function registerTools() {
  mocked.registerAppTool.mockImplementation(
    (_server: any, name: string, _def: any, handler: (input: any, context?: any) => Promise<any>) => {
      mocked.registrations.set(name, { handler });
    }
  );
  registerAllTools({} as any, config);
}

describe('MCP E2E flow (task 9.3)', () => {
  beforeEach(() => {
    mocked.registrations.clear();
    mocked.registerAppTool.mockReset();
    mocked.searchServices.mockReset();
    mocked.authenticateUser.mockReset();
    mocked.getTaskDetails.mockReset();
    mocked.completeTask.mockReset();
    mocked.runService.mockReset();
    mocked.verifyToken.mockReset();
  });

  it('runs search -> auth -> task details -> complete task -> run service through tool handlers', async () => {
    registerTools();

    mocked.searchServices.mockResolvedValue({
      services: [
        {
          service_type: 'campaign',
          service_id: 'campaign-1',
          name: 'Design Campaign',
          sponsor: 'Sponsor A',
          required_task: 'survey',
          subsidy_amount_cents: 1000,
          category: ['design'],
          active: true,
          tags: [],
          relevance_score: 0.95,
        },
      ],
      total_count: 1,
      message: 'ok',
      applied_filters: null,
      available_categories: ['design'],
    });

    mocked.verifyToken.mockResolvedValue({
      sub: 'auth0|123',
      email: 'user@example.com',
      scopes: ['user.write', 'tasks.read', 'tasks.write', 'services.execute'],
      token: 'oauth-token',
    });

    mocked.authenticateUser.mockResolvedValue({
      session_token: 'session-1',
      user_id: 'user-1',
      email: 'user@example.com',
      is_new_user: false,
      message: 'authenticated',
    });

    mocked.getTaskDetails.mockResolvedValue({
      campaign_id: 'campaign-1',
      campaign_name: 'Design Campaign',
      sponsor: 'Sponsor A',
      required_task: 'survey',
      task_description: 'Please answer survey',
      task_input_format: { task_type: 'survey', required_fields: ['age'], instructions: 'fill in' },
      already_completed: false,
      subsidy_amount_cents: 1000,
      message: 'task loaded',
    });

    mocked.completeTask.mockResolvedValue({
      task_completion_id: 'tc-1',
      campaign_id: 'campaign-1',
      consent_recorded: true,
      can_use_service: true,
      message: 'task completed',
    });

    mocked.runService.mockResolvedValue({
      service: 'design',
      output: 'service output',
      payment_mode: 'sponsored',
      sponsored_by: 'Sponsor A',
      tx_hash: null,
      message: 'service executed',
    });

    const search = await mocked.registrations.get('search_services')!.handler({ q: 'design' }, {});
    expect(search.structuredContent.services).toHaveLength(1);

    const auth = await mocked.registrations
      .get('authenticate_user')!
      .handler({ region: 'auto', roles: [], tools_used: [] }, { auth: { token: 'oauth-token' } });
    expect(auth._meta.session_token).toBe('session-1');

    const context = { auth: { token: 'oauth-token' } };
    const taskDetails = await mocked.registrations
      .get('get_task_details')!
      .handler({ campaign_id: 'campaign-1', session_token: auth._meta.session_token }, context);
    expect(taskDetails.structuredContent.campaign_id).toBe('campaign-1');

    const complete = await mocked.registrations.get('complete_task')!.handler(
      {
        campaign_id: 'campaign-1',
        session_token: auth._meta.session_token,
        task_name: 'survey',
        details: 'age=20',
        consent: {
          data_sharing_agreed: true,
          purpose_acknowledged: true,
          contact_permission: false,
        },
      },
      context
    );
    expect(complete.structuredContent.can_use_service).toBe(true);

    const run = await mocked.registrations.get('run_service')!.handler(
      { service: 'design', input: 'generate logo', session_token: auth._meta.session_token },
      context
    );
    expect(run.structuredContent.service).toBe('design');
    expect(run._meta.output).toBe('service output');
  });
});
