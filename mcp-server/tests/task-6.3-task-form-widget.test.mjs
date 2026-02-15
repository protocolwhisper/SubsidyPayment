import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const widgetPath = resolve(mcpRoot, 'src/widgets/src/task-form.html');
assert.ok(existsSync(widgetPath), 'mcp-server/src/widgets/src/task-form.html is required');

const src = readFileSync(widgetPath, 'utf8');

assert.match(src, /task_input_format\s*\?\.\s*required_fields|task_input_format\.required_fields/, 'must use required_fields to render dynamic inputs');
assert.match(src, /task_description/, 'must render task description text');
assert.match(src, /data_sharing_agreed/, 'must include data sharing consent checkbox');
assert.match(src, /purpose_acknowledged/, 'must include purpose acknowledgement consent checkbox');
assert.match(src, /contact_permission/, 'must include contact permission checkbox');
assert.match(src, /callTool\(\s*['"]complete_task['"]/, 'must call complete_task on submit');
assert.match(src, /already_completed/, 'must handle already_completed state');
assert.match(src, /:root\s*\{[\s\S]*--/, 'must include CSS variables for theme support');

console.log('task-6.3 task-form widget checks passed');
