import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const filePath = resolve(mcpRoot, 'src/backend-client.ts');
assert.ok(existsSync(filePath), 'mcp-server/src/backend-client.ts is required');

const src = readFileSync(filePath, 'utf8');
for (const method of [
  'searchServices',
  'authenticateUser',
  'getTaskDetails',
  'completeTask',
  'runService',
  'getUserStatus',
  'getPreferences',
  'setPreferences',
]) {
  assert.match(src, new RegExp(`\\b${method}\\s*\\(`), `missing method: ${method}`);
}

const { BackendClient, BackendClientError } = await import('../src/backend-client.ts');

const calls = [];
globalThis.fetch = async (input, init) => {
  calls.push({ input: String(input), init });
  return {
    ok: true,
    status: 200,
    json: async () => ({ message: 'ok' }),
  };
};

const client = new BackendClient({
  rustBackendUrl: 'http://localhost:3000',
  mcpInternalApiKey: 'secret-key',
  auth0Domain: '',
  auth0Audience: '',
  publicUrl: 'http://localhost:3001',
  port: 3001,
  logLevel: 'info',
});

await client.searchServices({ q: 'design', max_budget_cents: 100 });
await client.authenticateUser({ email: 'a@example.com', region: 'JP' });
await client.getTaskDetails('camp-1', 'sess-1');
await client.completeTask('camp-1', {
  session_token: 'sess-1',
  task_name: 'fill_form',
  consent: {
    data_sharing_agreed: true,
    purpose_acknowledged: true,
    contact_permission: false,
  },
});
await client.runService('scraping', { session_token: 'sess-1', input: 'hello' });
await client.getUserStatus('sess-1');
await client.getPreferences('sess-1');
await client.setPreferences({
  session_token: 'sess-1',
  preferences: [{ task_type: 'survey', level: 'preferred' }],
});

assert.equal(calls.length, 8, 'must call fetch 8 times');
assert.match(calls[0].input, /\/gpt\/services\?q=design&max_budget_cents=100/, 'searchServices query mismatch');
assert.equal(calls[0].init.headers.Authorization, 'Bearer secret-key', 'auth header missing');
assert.match(calls[2].input, /\/gpt\/tasks\/camp-1\?session_token=sess-1/, 'getTaskDetails path mismatch');
assert.match(calls[3].input, /\/gpt\/tasks\/camp-1\/complete$/, 'completeTask path mismatch');
assert.match(calls[4].input, /\/gpt\/services\/scraping\/run$/, 'runService path mismatch');
assert.match(calls[5].input, /\/gpt\/user\/status\?session_token=sess-1/, 'getUserStatus path mismatch');
assert.match(calls[6].input, /\/gpt\/preferences\?session_token=sess-1/, 'getPreferences path mismatch');
assert.match(calls[7].input, /\/gpt\/preferences$/, 'setPreferences path mismatch');

// Backend error mapping (4xx/5xx)
globalThis.fetch = async () => ({
  ok: false,
  status: 400,
  json: async () => ({ error: { code: 'invalid_request', message: 'bad input' } }),
});
await assert.rejects(
  () => client.searchServices({}),
  (err) => err instanceof BackendClientError && err.code === 'invalid_request' && err.message === 'bad input'
);

// Network error mapping
globalThis.fetch = async () => {
  throw new Error('network down');
};
await assert.rejects(
  () => client.getUserStatus('sess-1'),
  (err) => err instanceof BackendClientError && err.code === 'backend_unavailable'
);

console.log('task-3 backend-client checks passed');
