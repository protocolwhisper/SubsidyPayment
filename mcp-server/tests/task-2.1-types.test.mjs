import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const filePath = resolve(mcpRoot, 'src/types.ts');
assert.ok(existsSync(filePath), 'mcp-server/src/types.ts is required');

const src = readFileSync(filePath, 'utf8');

const requiredInterfaces = [
  'GptSearchResponse',
  'GptAuthResponse',
  'GptTaskResponse',
  'GptCompleteTaskResponse',
  'GptRunServiceResponse',
  'GptUserStatusResponse',
  'GptPreferencesResponse',
  'GptSetPreferencesResponse',
  'BackendErrorResponse',
  'SearchServicesParams',
  'AuthenticateUserParams',
  'GetTaskDetailsParams',
  'CompleteTaskInput',
  'GptConsentInput',
  'RunServiceInput',
  'GetUserStatusParams',
  'GetPreferencesParams',
  'SetPreferencesInput',
  'TaskPreference',
];

for (const name of requiredInterfaces) {
  assert.match(src, new RegExp(`export\\s+interface\\s+${name}\\b`), `missing interface: ${name}`);
}

assert.match(src, /service_type:\s*"campaign"\s*\|\s*"sponsored_api"/, 'GptServiceItem.service_type union mismatch');
assert.match(src, /payment_mode:\s*"sponsored"\s*\|\s*"user_direct"/, 'GptRunServiceResponse.payment_mode union mismatch');
assert.match(src, /level:\s*"preferred"\s*\|\s*"neutral"\s*\|\s*"avoided"/, 'TaskPreference.level union mismatch');
assert.match(src, /details\?:\s*unknown/, 'BackendErrorResponse.error.details?: unknown is required');

console.log('task-2.1 types checks passed');
