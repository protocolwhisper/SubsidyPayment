import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const widgetPath = resolve(mcpRoot, 'src/widgets/src/user-dashboard.html');
assert.ok(existsSync(widgetPath), 'mcp-server/src/widgets/src/user-dashboard.html is required');

const src = readFileSync(widgetPath, 'utf8');

assert.match(src, /structuredContent/, 'must consume structuredContent');
assert.match(src, /email/, 'must render user email');
assert.match(src, /completed_tasks/, 'must render completed tasks');
assert.match(src, /campaign_name|task_name|completed_at/, 'must render task table columns');
assert.match(src, /available_services/, 'must render available services');
assert.match(src, /ready/, 'must handle ready state');
assert.match(src, /callTool\(\s*['"]run_service['"]\s*,\s*\{[\s\S]*service[\s\S]*input/, 'must call run_service with service and input');
assert.match(src, /sendFollowUpMessage\(/, 'should call sendFollowUpMessage for follow-up');
assert.match(src, /:root\s*\{[\s\S]*--/, 'must include CSS variables for theme support');

console.log('task-7.4 user-dashboard widget checks passed');
