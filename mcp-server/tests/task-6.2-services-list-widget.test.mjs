import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const widgetPath = resolve(mcpRoot, 'src/widgets/src/services-list.html');
assert.ok(existsSync(widgetPath), 'mcp-server/src/widgets/src/services-list.html is required');

const src = readFileSync(widgetPath, 'utf8');

assert.match(src, /structuredContent\s*\?\.\s*services|structuredContent\.services/, 'must read structuredContent.services');
assert.match(src, /service\.name|name/, 'must render service name');
assert.match(src, /sponsor/, 'must render sponsor name');
assert.match(src, /subsidy_amount_cents/, 'must render subsidy amount');
assert.match(src, /category/, 'must render category tags');
assert.match(src, /relevance_score/, 'must render relevance score');

assert.match(src, /callTool\(\s*['"]get_task_details['"]\s*,\s*\{[\s\S]*campaign_id/, 'must call get_task_details with campaign_id on select');
assert.match(src, /setWidgetState\(/, 'must persist selection using setWidgetState');

assert.match(src, /:root\s*\{[\s\S]*--/, 'must define CSS custom properties');
assert.match(src, /body\.dark|\.dark/, 'must include dark mode styles');

console.log('task-6.2 services-list widget checks passed');
