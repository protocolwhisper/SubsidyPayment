import { beforeEach, describe, expect, it, vi } from 'vitest';
const mocked = vi.hoisted(() => ({
    registrations: new Map(),
    registerAppTool: vi.fn(),
    searchServices: vi.fn(),
    authenticateUser: vi.fn(),
    getTaskDetails: vi.fn(),
    completeTask: vi.fn(),
    runService: vi.fn(),
    getUserStatus: vi.fn(),
    getPreferences: vi.fn(),
    setPreferences: vi.fn(),
    verifyToken: vi.fn(),
    BackendClientError: null,
}));
vi.mock('@modelcontextprotocol/ext-apps/server', () => ({
    registerAppTool: (...args) => mocked.registerAppTool(...args),
}));
vi.mock('../../src/backend-client.ts', () => {
    class MockBackendClientError extends Error {
        code;
        details;
        constructor(code, message, details) {
            super(message);
            this.code = code;
            this.details = details;
        }
    }
    mocked.BackendClientError = MockBackendClientError;
    class BackendClient {
        searchServices = mocked.searchServices;
        authenticateUser = mocked.authenticateUser;
        getTaskDetails = mocked.getTaskDetails;
        completeTask = mocked.completeTask;
        runService = mocked.runService;
        getUserStatus = mocked.getUserStatus;
        getPreferences = mocked.getPreferences;
        setPreferences = mocked.setPreferences;
    }
    return {
        BackendClient,
        BackendClientError: MockBackendClientError,
    };
});
vi.mock('../../src/auth/token-verifier.ts', () => {
    class TokenVerifier {
        verify = mocked.verifyToken;
    }
    return {
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
};
function registerAndCaptureTools() {
    mocked.registerAppTool.mockImplementation((_server, name, definition, handler) => {
        mocked.registrations.set(name, { definition, handler });
    });
    registerAllTools({}, config);
}
function getRegistered(name) {
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
        mocked.completeTask.mockReset();
        mocked.runService.mockReset();
        mocked.getUserStatus.mockReset();
        mocked.getPreferences.mockReset();
        mocked.setPreferences.mockReset();
        mocked.verifyToken.mockReset();
    });
    it('registers all 8 tools with expected security schemes', () => {
        registerAndCaptureTools();
        expect(mocked.registrations.size).toBe(8);
        expect(getRegistered('search_services').definition.securitySchemes).toEqual([{ type: 'noauth' }]);
        const oauthTools = [
            'authenticate_user',
            'get_task_details',
            'complete_task',
            'run_service',
            'get_user_status',
            'get_preferences',
            'set_preferences',
        ];
        for (const name of oauthTools) {
            const security = getRegistered(name).definition.securitySchemes;
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
        expect(result.content).toBeDefined();
        expect(result._meta).toBeDefined();
        expect(result.isError).toBeUndefined();
    });
    it('returns backend error for search_services failure', async () => {
        registerAndCaptureTools();
        mocked.searchServices.mockRejectedValue(new mocked.BackendClientError('backend_error', 'backend failed'));
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
        const result = await handler({ region: 'auto', roles: [], tools_used: [] }, { auth: { token: 'token' } });
        expect(mocked.verifyToken).toHaveBeenCalled();
        expect(result.structuredContent).toMatchObject({
            user_id: 'user-id',
            email: 'user@example.com',
        });
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
        const result = await handler({ service: 'design', input: 'hello', session_token: 's1' }, { auth: { token: 'token' } });
        expect(result.structuredContent.output).toBeUndefined();
        expect(result._meta.output).toBe('very-large-payload');
    });
    it('validates input schema (set_preferences invalid level)', () => {
        registerAndCaptureTools();
        const { definition } = getRegistered('set_preferences');
        expect(() => definition.inputSchema.parse({
            preferences: [{ task_type: 'survey', level: 'invalid-level' }],
        })).toThrow();
    });
});
