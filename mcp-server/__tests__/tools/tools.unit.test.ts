import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocked = vi.hoisted(() => ({
  registrations: new Map<string, { definition: any; handler: (input: any, context?: any) => Promise<any> }>(),
  registerAppTool: vi.fn(),
  searchServices: vi.fn(),
  authenticateUser: vi.fn(),
  getTaskDetails: vi.fn(),
  initZkpassportVerification: vi.fn(),
  completeTask: vi.fn(),
  runService: vi.fn(),
  runProxyService: vi.fn(),
  getUserStatus: vi.fn(),
  getUserRecord: vi.fn(),
  getPreferences: vi.fn(),
  setPreferences: vi.fn(),
  getWeather: vi.fn(),
  createGithubIssue: vi.fn(),
  verifyToken: vi.fn(),
}));

vi.mock('@modelcontextprotocol/ext-apps/server', () => ({
  registerAppTool: (...args: any[]) => mocked.registerAppTool(...args),
}));

vi.mock('../../src/backend-client.ts', () => {
  class BackendClientError extends Error {
    code: string;
    details?: unknown;

    constructor(code: string, message: string, details?: unknown) {
      super(message);
      this.code = code;
      this.details = details;
    }
  }

  class BackendClient {
    searchServices = mocked.searchServices;
    authenticateUser = mocked.authenticateUser;
    getTaskDetails = mocked.getTaskDetails;
    initZkpassportVerification = mocked.initZkpassportVerification;
    completeTask = mocked.completeTask;
    runService = mocked.runService;
    runProxyService = mocked.runProxyService;
    getUserStatus = mocked.getUserStatus;
    getUserRecord = mocked.getUserRecord;
    getPreferences = mocked.getPreferences;
    setPreferences = mocked.setPreferences;
  }

  return {
    BackendClient,
    BackendClientError,
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

vi.mock('../../src/x402/weather-client.ts', () => {
  class X402WeatherClient {
    getWeather = mocked.getWeather;
  }

  return { X402WeatherClient };
});

vi.mock('../../src/x402/github-issue-client.ts', () => {
  class X402GithubIssueClient {
    createGithubIssue = mocked.createGithubIssue;
  }

  return { X402GithubIssueClient };
});

vi.mock('../../src/widgets/index.ts', () => ({
  readWidgetHtml: vi.fn().mockResolvedValue('<html></html>'),
  RESOURCE_MIME_TYPE: 'text/html;profile=mcp-app',
}));

import { BackendClientError } from '../../src/backend-client.ts';
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
  x402WeatherUrl: 'http://localhost:4021/weather',
  x402GithubIssueUrl: 'http://localhost:4021/github-issue',
  x402FacilitatorUrl: 'https://x402.org/facilitator',
  x402Network: 'eip155:84532',
  x402PrivateKey: '0x1234',
  x402RequestTimeoutMs: 15000,
};

function registerAndCaptureTools() {
  mocked.registerAppTool.mockImplementation(
    (_server: any, name: string, definition: any, handler: (input: any, context?: any) => Promise<any>) => {
      mocked.registrations.set(name, { definition, handler });
    }
  );

  registerAllTools({} as any, config);
}

function getRegistered(name: string) {
  const found = mocked.registrations.get(name);
  if (!found) {
    throw new Error(`tool not registered: ${name}`);
  }
  return found;
}

describe('MCP tools unit tests (task 9.1)', () => {
  beforeEach(() => {
    mocked.registrations.clear();
    mocked.registerAppTool.mockReset();

    mocked.searchServices.mockReset();
    mocked.authenticateUser.mockReset();
    mocked.getTaskDetails.mockReset();
    mocked.initZkpassportVerification.mockReset();
    mocked.completeTask.mockReset();
    mocked.runService.mockReset();
    mocked.runProxyService.mockReset();
    mocked.getUserStatus.mockReset();
    mocked.getUserRecord.mockReset();
    mocked.getPreferences.mockReset();
    mocked.setPreferences.mockReset();
    mocked.getWeather.mockReset();
    mocked.createGithubIssue.mockReset();
    mocked.verifyToken.mockReset();
  });

  it('registers all 15 tools with expected security schemes', () => {
    registerAndCaptureTools();

    expect(mocked.registrations.size).toBe(15);
    expect(getRegistered('search_services').definition._meta.securitySchemes).toEqual([{ type: 'noauth' }]);
    expect(getRegistered('get_prompt_guide_flow').definition._meta.securitySchemes).toEqual([{ type: 'noauth' }]);
    expect(getRegistered('weather').definition._meta.securitySchemes).toEqual([{ type: 'noauth' }]);
    expect(getRegistered('create_github_issue').definition._meta.securitySchemes).toEqual([{ type: 'noauth' }]);

    const oauthTools = [
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
    ];

    for (const name of oauthTools) {
      const security = getRegistered(name).definition._meta.securitySchemes;
      expect(Array.isArray(security)).toBe(true);
      expect(security[0].type).toBe('oauth2');
    }
  });

  it('returns 3-part response for search_services success', async () => {
    registerAndCaptureTools();
    mocked.searchServices.mockResolvedValue({
      services: [],
      total_count: 0,
      message: 'ok',
      applied_filters: null,
      available_categories: [],
    });

    const { handler } = getRegistered('search_services');
    const result = await handler({ q: 'design' }, {});

    expect(result.structuredContent).toBeDefined();
    expect(result.content.find((c: any) => c.type === 'resource')).toBeUndefined();
    expect(result.contents).toBeUndefined();
    expect(result.content).toBeDefined();
    expect(result._meta).toBeDefined();
    expect(result.isError).toBeUndefined();
  });

  it('returns backend error for search_services failure', async () => {
    registerAndCaptureTools();
    mocked.searchServices.mockRejectedValue(new BackendClientError('backend_error', 'backend failed'));

    const { handler } = getRegistered('search_services');
    const result = await handler({ q: 'design' }, {});

    expect(result.isError).toBe(true);
    expect(result._meta.code).toBe('backend_error');
  });

  it('returns auth error when token verification fails', async () => {
    registerAndCaptureTools();
    mocked.verifyToken.mockResolvedValue(null);

    const { handler } = getRegistered('authenticate_user');
    const result = await handler({ region: 'auto', roles: [], tools_used: [] }, {});

    expect(result.isError).toBe(true);
    expect(result._meta['mcp/www_authenticate']).toBeDefined();
  });

  it('returns 3-part response for authenticate_user success', async () => {
    registerAndCaptureTools();
    mocked.verifyToken.mockResolvedValue({
      sub: 'auth0|1',
      email: 'user@example.com',
      scopes: ['user.write'],
      token: 'token',
    });
    mocked.authenticateUser.mockResolvedValue({
      session_token: 'session-token',
      user_id: 'user-id',
      email: 'user@example.com',
      is_new_user: false,
      message: 'authenticated',
    });

    const { handler } = getRegistered('authenticate_user');
    const result = await handler(
      { region: 'auto', roles: [], tools_used: [] },
      { auth: { token: 'token' } }
    );

    expect(mocked.verifyToken).toHaveBeenCalled();
    expect(result.structuredContent).toMatchObject({
      user_id: 'user-id',
      email: 'user@example.com',
    });
    expect(result.content.find((c: any) => c.type === 'resource')).toBeUndefined();
    expect(result.contents).toBeUndefined();
    expect(result._meta.session_token).toBe('session-token');
  });

  it('keeps run_service output in _meta and not in structuredContent', async () => {
    registerAndCaptureTools();
    mocked.verifyToken.mockResolvedValue({
      sub: 'auth0|1',
      email: 'user@example.com',
      scopes: ['services.execute'],
      token: 'token',
    });
    mocked.runService.mockResolvedValue({
      service: 'design',
      output: 'very-large-payload',
      payment_mode: 'sponsored',
      sponsored_by: 'sponsor-a',
      tx_hash: null,
      message: 'done',
    });

    const { handler } = getRegistered('run_service');
    const result = await handler(
      { service: 'design', input: 'hello', session_token: 's1' },
      { auth: { token: 'token' } }
    );

    expect(result.structuredContent.output).toBeUndefined();
    expect(result.content.find((c: any) => c.type === 'resource')).toBeUndefined();
    expect(result.contents).toBeUndefined();
    expect(result._meta.output).toBe('very-large-payload');
  });

  it('validates input schema (set_preferences invalid level)', () => {
    registerAndCaptureTools();
    const { definition } = getRegistered('set_preferences');

    expect(() =>
      definition.inputSchema.parse({
        preferences: [{ task_type: 'survey', level: 'invalid-level' }],
      })
    ).toThrow();
  });

  it('validates input schema (complete_task invalid feedback rating)', () => {
    registerAndCaptureTools();
    const { definition } = getRegistered('complete_task');

    expect(() =>
      definition.inputSchema.parse({
        campaign_id: '2f6d2c0b-3c7a-4a0a-9a2e-6f2b6b7e8d90',
        task_name: 'share_feedback',
        consent: {
          data_sharing_agreed: true,
          purpose_acknowledged: true,
          contact_permission: true,
        },
        feedback: {
          product_link: 'https://example.com/product',
          feedback_rating: 6,
          feedback_tags: 'Cost',
          feedback_reason: 'Need better onboarding copy.',
        },
      })
    ).toThrow();
  });

  it('serializes complete_task feedback into details when details is omitted', async () => {
    registerAndCaptureTools();
    mocked.verifyToken.mockResolvedValue({
      sub: 'auth0|1',
      email: 'user@example.com',
      scopes: ['tasks.write'],
      token: 'token',
    });
    mocked.completeTask.mockResolvedValue({
      task_completion_id: '7c9f8f6a-6f89-4e77-b2e1-bb8d58a5be35',
      campaign_id: '2f6d2c0b-3c7a-4a0a-9a2e-6f2b6b7e8d90',
      consent_recorded: true,
      can_use_service: true,
      message: 'ok',
    });
    mocked.getTaskDetails.mockResolvedValue({
      required_task: 'share_feedback',
      sponsor: 'Acme',
      campaign_name: 'Acme Campaign',
      subsidy_amount_cents: 120,
    });

    const { handler } = getRegistered('complete_task');
    await handler(
      {
        campaign_id: '2f6d2c0b-3c7a-4a0a-9a2e-6f2b6b7e8d90',
        task_name: 'share_feedback',
        session_token: 'session-token',
        consent: {
          data_sharing_agreed: true,
          purpose_acknowledged: true,
          contact_permission: true,
        },
        feedback: {
          product_link: 'https://example.com/product',
          feedback_rating: 4,
          feedback_tags: 'Cost, Usability',
          feedback_reason: 'Onboarding copy is ambiguous; please add setup examples.',
        },
      },
      { auth: { token: 'token' } }
    );

    expect(mocked.completeTask).toHaveBeenCalled();
    const [, payload] = mocked.completeTask.mock.calls[0];
    expect(typeof payload.details).toBe('string');
    expect(payload.details).toContain('"product_link":"https://example.com/product"');
    expect(payload.details).toContain('"feedback_rating":4');
    expect(payload.details).toContain('"feedback_tags":"Cost, Usability"');
    expect(payload.details).toContain('"feedback_reason":"Onboarding copy is ambiguous; please add setup examples."');
  });

  it('returns 3-part response for weather success', async () => {
    registerAndCaptureTools();
    mocked.getWeather.mockResolvedValue({
      city: 'San Francisco',
      weather: 'sunny',
      temperature: 70,
    });

    const { handler } = getRegistered('weather');
    const result = await handler({ city: 'San Francisco' }, {});

    expect(result.structuredContent.city).toBe('San Francisco');
    expect(result.content).toBeDefined();
    expect(result._meta.report).toBeDefined();
    expect(result.isError).toBeUndefined();
  });

  it('returns backend error for weather failure', async () => {
    registerAndCaptureTools();
    mocked.getWeather.mockRejectedValue(new BackendClientError('weather_request_failed', 'weather failed'));

    const { handler } = getRegistered('weather');
    const result = await handler({ city: 'Tokyo' }, {});

    expect(result.isError).toBe(true);
    expect(result._meta.code).toBe('weather_request_failed');
  });

  it('returns 3-part response for create_github_issue success', async () => {
    registerAndCaptureTools();
    mocked.createGithubIssue.mockResolvedValue({
      status: 'issue created',
    });

    const { handler } = getRegistered('create_github_issue');
    const result = await handler({}, {});

    expect(result.structuredContent.status).toBe('issue created');
    expect(result.content).toBeDefined();
    expect(result._meta.response).toBeDefined();
    expect(result.isError).toBeUndefined();
  });

  it('returns backend error for create_github_issue failure', async () => {
    registerAndCaptureTools();
    mocked.createGithubIssue.mockRejectedValue(new BackendClientError('github_issue_request_failed', 'issue failed'));

    const { handler } = getRegistered('create_github_issue');
    const result = await handler({}, {});

    expect(result.isError).toBe(true);
    expect(result._meta.code).toBe('github_issue_request_failed');
  });
});
