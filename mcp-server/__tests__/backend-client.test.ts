import { beforeEach, describe, expect, it, vi } from 'vitest';

import { BackendClient, BackendClientError } from '../src/backend-client.ts';

const config = {
  rustBackendUrl: 'http://rust-backend.local',
  mcpInternalApiKey: 'secret-internal-key',
  auth0Domain: 'tenant.example.com',
  auth0Audience: 'https://example.com/api',
  publicUrl: 'http://localhost:3001',
  port: 3001,
  logLevel: 'info',
  authEnabled: true,
  x402WeatherUrl: 'http://localhost:4021/weather',
  x402FacilitatorUrl: 'https://x402.org/facilitator',
  x402Network: 'eip155:84532',
  x402PrivateKey: '0x1234',
  x402RequestTimeoutMs: 15000,
};

function mockJsonResponse(status: number, body: unknown): Response {
  return {
    ok: status >= 200 && status < 300,
    status,
    json: vi.fn().mockResolvedValue(body),
  } as unknown as Response;
}

describe('BackendClient integration tests (task 9.2)', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it('adds Authorization header and query parameters for searchServices', async () => {
    const fetchMock = vi.spyOn(globalThis, 'fetch').mockResolvedValue(
      mockJsonResponse(200, {
        services: [],
        total_count: 0,
        message: 'ok',
      })
    );
    const client = new BackendClient(config);

    await client.searchServices({
      q: 'design',
      category: 'creative',
      max_budget_cents: 1500,
      intent: 'logo',
      session_token: 'sess-1',
    });

    expect(fetchMock).toHaveBeenCalledTimes(1);
    const [url, init] = fetchMock.mock.calls[0];
    expect(String(url)).toContain('/gpt/services?');
    expect(String(url)).toContain('q=design');
    expect(String(url)).toContain('category=creative');
    expect(String(url)).toContain('max_budget_cents=1500');
    expect(String(url)).toContain('intent=logo');
    expect(String(url)).toContain('session_token=sess-1');

    const headers = (init?.headers ?? {}) as Record<string, string>;
    expect(headers.Authorization).toBe('Bearer secret-internal-key');
    expect(headers.Accept).toBe('application/json');
  });

  it('uses encoded path params for getTaskDetails', async () => {
    const fetchMock = vi.spyOn(globalThis, 'fetch').mockResolvedValue(
      mockJsonResponse(200, {
        campaign_id: 'cmp',
        campaign_name: 'name',
        sponsor: 's',
        required_task: 'survey',
        task_description: 'desc',
        task_input_format: { task_type: 'survey', required_fields: [], instructions: '' },
        already_completed: false,
        subsidy_amount_cents: 100,
        message: 'ok',
      })
    );
    const client = new BackendClient(config);

    await client.getTaskDetails('campaign id/with space', 'tok/en');
    const [url] = fetchMock.mock.calls[0];
    const calledUrl = String(url);

    expect(calledUrl).toContain('/gpt/tasks/campaign%20id%2Fwith%20space?session_token=tok%2Fen');
  });

  it('sends JSON body with Content-Type for completeTask', async () => {
    const fetchMock = vi.spyOn(globalThis, 'fetch').mockResolvedValue(
      mockJsonResponse(200, {
        task_completion_id: 't1',
        campaign_id: 'c1',
        consent_recorded: true,
        can_use_service: true,
        message: 'done',
      })
    );
    const client = new BackendClient(config);

    await client.completeTask('c1', {
      campaign_id: 'c1',
      session_token: 's1',
      task_name: 'survey',
      details: 'answer',
      consent: {
        data_sharing_agreed: true,
        purpose_acknowledged: true,
        contact_permission: false,
      },
    });

    const [, init] = fetchMock.mock.calls[0];
    const headers = (init?.headers ?? {}) as Record<string, string>;
    expect(headers['Content-Type']).toBe('application/json');
    expect(init?.method).toBe('POST');
    expect(typeof init?.body).toBe('string');
  });

  it('maps 4xx/5xx backend error response to BackendClientError', async () => {
    vi.spyOn(globalThis, 'fetch').mockResolvedValue(
      mockJsonResponse(500, {
        error: {
          code: 'internal_failure',
          message: 'backend exploded',
          details: { trace_id: 'abc' },
        },
      })
    );
    const client = new BackendClient(config);

    await expect(client.getPreferences('session')).rejects.toMatchObject<Partial<BackendClientError>>({
      name: 'BackendClientError',
      code: 'internal_failure',
      message: 'backend exploded',
      details: { trace_id: 'abc' },
    });
  });

  it('uses fallback backend_error when 4xx/5xx response is not json', async () => {
    vi.spyOn(globalThis, 'fetch').mockResolvedValue({
      ok: false,
      status: 502,
      json: vi.fn().mockRejectedValue(new Error('bad json')),
    } as unknown as Response);
    const client = new BackendClient(config);

    await expect(client.getUserStatus('session')).rejects.toMatchObject<Partial<BackendClientError>>({
      name: 'BackendClientError',
      code: 'backend_error',
    });
  });

  it('maps network failures to backend_unavailable', async () => {
    vi.spyOn(globalThis, 'fetch').mockRejectedValue(new Error('ECONNREFUSED'));
    const client = new BackendClient(config);

    await expect(client.runService('design', { service: 'design', session_token: 's', input: 'x' })).rejects.toMatchObject<
      Partial<BackendClientError>
    >({
      name: 'BackendClientError',
      code: 'backend_unavailable',
    });
  });
});
