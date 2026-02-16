import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const getTaskPath = resolve(mcpRoot, 'src/tools/get-task-details.ts');
const completeTaskPath = resolve(mcpRoot, 'src/tools/complete-task.ts');

assert.ok(existsSync(getTaskPath), 'mcp-server/src/tools/get-task-details.ts is required');
assert.ok(existsSync(completeTaskPath), 'mcp-server/src/tools/complete-task.ts is required');

const getTaskSrc = readFileSync(getTaskPath, 'utf8');
const completeTaskSrc = readFileSync(completeTaskPath, 'utf8');

assert.match(getTaskSrc, /registerAppTool\(\s*server\s*,\s*['"]get_task_details['"]/, 'must register get_task_details');
assert.match(getTaskSrc, /readOnlyHint:\s*true/, 'get_task_details must be readOnly');
assert.match(getTaskSrc, /resourceUri:\s*['"]ui:\/\/widget\/task-form\.html['"]/, 'get_task_details must bind task-form widget');
assert.match(getTaskSrc, /securitySchemes:[\s\S]*type:\s*['"]oauth2['"]/, 'get_task_details must require oauth2');
assert.match(getTaskSrc, /getTaskDetails\(/, 'must call BackendClient.getTaskDetails');
assert.match(getTaskSrc, /session_token/, 'get_task_details must use session_token');

assert.match(completeTaskSrc, /registerAppTool\(\s*server\s*,\s*['"]complete_task['"]/, 'must register complete_task');
assert.match(completeTaskSrc, /securitySchemes:[\s\S]*type:\s*['"]oauth2['"]/, 'complete_task must require oauth2');
assert.match(completeTaskSrc, /consent:\s*z\.object\(/, 'complete_task must validate consent object');
assert.match(completeTaskSrc, /completeTask\(/, 'must call BackendClient.completeTask');
assert.match(completeTaskSrc, /consent_recorded|can_use_service/, 'complete_task must return completion fields');
assert.match(completeTaskSrc, /session_token/, 'complete_task must use session_token');

const indexPath = resolve(mcpRoot, 'src/tools/index.ts');
const indexSrc = readFileSync(indexPath, 'utf8');
assert.match(indexSrc, /registerGetTaskDetailsTool/, 'tools/index.ts must include get_task_details registration');
assert.match(indexSrc, /registerCompleteTaskTool/, 'tools/index.ts must include complete_task registration');

console.log('task-5.3 task tools checks passed');
