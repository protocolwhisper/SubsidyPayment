import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const getUserStatusPath = resolve(mcpRoot, 'src/tools/get-user-status.ts');
const getPreferencesPath = resolve(mcpRoot, 'src/tools/get-preferences.ts');
const setPreferencesPath = resolve(mcpRoot, 'src/tools/set-preferences.ts');

assert.ok(existsSync(getUserStatusPath), 'mcp-server/src/tools/get-user-status.ts is required');
assert.ok(existsSync(getPreferencesPath), 'mcp-server/src/tools/get-preferences.ts is required');
assert.ok(existsSync(setPreferencesPath), 'mcp-server/src/tools/set-preferences.ts is required');

const getUserStatusSrc = readFileSync(getUserStatusPath, 'utf8');
const getPreferencesSrc = readFileSync(getPreferencesPath, 'utf8');
const setPreferencesSrc = readFileSync(setPreferencesPath, 'utf8');

assert.match(getUserStatusSrc, /registerAppTool\(\s*server\s*,\s*['"]get_user_status['"]/, 'must register get_user_status');
assert.match(getUserStatusSrc, /readOnlyHint:\s*true/, 'get_user_status must set readOnlyHint: true');
assert.match(getUserStatusSrc, /resourceUri:\s*['"]ui:\/\/widget\/user-dashboard\.html['"]/, 'get_user_status must bind user-dashboard widget');
assert.match(getUserStatusSrc, /securitySchemes:[\s\S]*type:\s*['"]oauth2['"]/, 'get_user_status must require oauth2');
assert.match(getUserStatusSrc, /getUserStatus\(/, 'must call BackendClient.getUserStatus');
assert.match(getUserStatusSrc, /session_token/, 'get_user_status must use session_token');

assert.match(getPreferencesSrc, /registerAppTool\(\s*server\s*,\s*['"]get_preferences['"]/, 'must register get_preferences');
assert.match(getPreferencesSrc, /readOnlyHint:\s*true/, 'get_preferences must set readOnlyHint: true');
assert.match(getPreferencesSrc, /securitySchemes:[\s\S]*type:\s*['"]oauth2['"]/, 'get_preferences must require oauth2');
assert.match(getPreferencesSrc, /getPreferences\(/, 'must call BackendClient.getPreferences');
assert.match(getPreferencesSrc, /session_token/, 'get_preferences must use session_token');

assert.match(setPreferencesSrc, /registerAppTool\(\s*server\s*,\s*['"]set_preferences['"]/, 'must register set_preferences');
assert.match(setPreferencesSrc, /securitySchemes:[\s\S]*type:\s*['"]oauth2['"]/, 'set_preferences must require oauth2');
assert.match(setPreferencesSrc, /preferences:\s*z\.array\(\s*z\.object\(/, 'set_preferences must validate preferences array');
assert.match(setPreferencesSrc, /setPreferences\(/, 'must call BackendClient.setPreferences');
assert.match(setPreferencesSrc, /session_token/, 'set_preferences must use session_token');

const indexPath = resolve(mcpRoot, 'src/tools/index.ts');
const indexSrc = readFileSync(indexPath, 'utf8');
assert.match(indexSrc, /registerSearchServicesTool/, 'registerAllTools must include search_services');
assert.match(indexSrc, /registerAuthenticateUserTool/, 'registerAllTools must include authenticate_user');
assert.match(indexSrc, /registerGetTaskDetailsTool/, 'registerAllTools must include get_task_details');
assert.match(indexSrc, /registerCompleteTaskTool/, 'registerAllTools must include complete_task');
assert.match(indexSrc, /registerRunServiceTool/, 'registerAllTools must include run_service');
assert.match(indexSrc, /registerGetUserStatusTool/, 'registerAllTools must include get_user_status');
assert.match(indexSrc, /registerGetPreferencesTool/, 'registerAllTools must include get_preferences');
assert.match(indexSrc, /registerSetPreferencesTool/, 'registerAllTools must include set_preferences');

console.log('task-5.5 user tools checks passed');
