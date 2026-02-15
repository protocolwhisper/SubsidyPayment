import type { BackendConfig } from './config.ts';
import type {
  AuthenticateUserParams,
  BackendErrorResponse,
  CompleteTaskInput,
  GetPreferencesParams,
  GetTaskDetailsParams,
  GetUserStatusParams,
  GptAuthResponse,
  GptCompleteTaskResponse,
  GptPreferencesResponse,
  GptRunServiceResponse,
  GptSearchResponse,
  GptSetPreferencesResponse,
  GptTaskResponse,
  GptUserStatusResponse,
  RunServiceInput,
  SearchServicesParams,
  SetPreferencesInput,
} from './types.ts';

export class BackendClientError extends Error {
  code: string;
  details?: unknown;

  constructor(code: string, message: string, details?: unknown) {
    super(message);
    this.name = 'BackendClientError';
    this.code = code;
    this.details = details;
  }
}

export class BackendClient {
  private readonly baseUrl: string;
  private readonly apiKey: string;

  constructor(config: BackendConfig) {
    this.baseUrl = config.rustBackendUrl.replace(/\/$/, '');
    this.apiKey = config.mcpInternalApiKey;
  }

  async searchServices(params: SearchServicesParams): Promise<GptSearchResponse> {
    const query = new URLSearchParams();
    if (params.q) query.set('q', params.q);
    if (params.category) query.set('category', params.category);
    if (typeof params.max_budget_cents === 'number') {
      query.set('max_budget_cents', String(params.max_budget_cents));
    }
    if (params.intent) query.set('intent', params.intent);
    if (params.session_token) query.set('session_token', params.session_token);
    return this.request<GptSearchResponse>(`/gpt/services?${query.toString()}`, { method: 'GET' });
  }

  async authenticateUser(payload: AuthenticateUserParams): Promise<GptAuthResponse> {
    return this.request<GptAuthResponse>('/gpt/auth', {
      method: 'POST',
      body: JSON.stringify(payload),
    });
  }

  async getTaskDetails(campaignId: string, sessionToken: string): Promise<GptTaskResponse> {
    const params: GetTaskDetailsParams = { campaign_id: campaignId, session_token: sessionToken };
    return this.request<GptTaskResponse>(
      `/gpt/tasks/${encodeURIComponent(params.campaign_id)}?session_token=${encodeURIComponent(params.session_token)}`,
      { method: 'GET' }
    );
  }

  async completeTask(campaignId: string, payload: CompleteTaskInput): Promise<GptCompleteTaskResponse> {
    return this.request<GptCompleteTaskResponse>(`/gpt/tasks/${encodeURIComponent(campaignId)}/complete`, {
      method: 'POST',
      body: JSON.stringify({
        session_token: payload.session_token,
        task_name: payload.task_name,
        details: payload.details,
        consent: payload.consent,
      }),
    });
  }

  async runService(service: string, payload: RunServiceInput): Promise<GptRunServiceResponse> {
    return this.request<GptRunServiceResponse>(`/gpt/services/${encodeURIComponent(service)}/run`, {
      method: 'POST',
      body: JSON.stringify({
        session_token: payload.session_token,
        input: payload.input,
      }),
    });
  }

  async getUserStatus(sessionToken: string): Promise<GptUserStatusResponse> {
    const params: GetUserStatusParams = { session_token: sessionToken };
    return this.request<GptUserStatusResponse>(
      `/gpt/user/status?session_token=${encodeURIComponent(params.session_token)}`,
      { method: 'GET' }
    );
  }

  async getPreferences(sessionToken: string): Promise<GptPreferencesResponse> {
    const params: GetPreferencesParams = { session_token: sessionToken };
    return this.request<GptPreferencesResponse>(
      `/gpt/preferences?session_token=${encodeURIComponent(params.session_token)}`,
      { method: 'GET' }
    );
  }

  async setPreferences(payload: SetPreferencesInput): Promise<GptSetPreferencesResponse> {
    return this.request<GptSetPreferencesResponse>('/gpt/preferences', {
      method: 'POST',
      body: JSON.stringify(payload),
    });
  }

  private async request<T>(path: string, init: RequestInit): Promise<T> {
    const headers: Record<string, string> = {
      Authorization: `Bearer ${this.apiKey}`,
      Accept: 'application/json',
      ...(init.body ? { 'Content-Type': 'application/json' } : {}),
      ...((init.headers as Record<string, string> | undefined) ?? {}),
    };

    let response: Response;
    try {
      response = await fetch(`${this.baseUrl}${path}`, {
        ...init,
        headers,
      });
    } catch (error) {
      throw new BackendClientError('backend_unavailable', 'Rust backend is unavailable', error);
    }

    if (!response.ok) {
      const parsed = await this.safeParseError(response);
      throw new BackendClientError(parsed.code, parsed.message, parsed.details);
    }

    return (await response.json()) as T;
  }

  private async safeParseError(response: Response): Promise<{ code: string; message: string; details?: unknown }> {
    try {
      const body = (await response.json()) as BackendErrorResponse;
      if (body?.error?.code && body?.error?.message) {
        return {
          code: body.error.code,
          message: body.error.message,
          details: body.error.details,
        };
      }
    } catch {
      // ignore parse error
    }

    return {
      code: 'backend_error',
      message: `Rust backend request failed with status ${response.status}`,
    };
  }
}
