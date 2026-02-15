import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const viteConfigPath = resolve(mcpRoot, 'vite.config.ts');
assert.ok(existsSync(viteConfigPath), 'mcp-server/vite.config.ts is required');

const viteSrc = readFileSync(viteConfigPath, 'utf8');
assert.match(viteSrc, /vite-plugin-singlefile/, 'vite-plugin-singlefile must be used');
assert.match(viteSrc, /input[\s\S]*services-list/, 'services-list widget entry is required');
assert.match(viteSrc, /input[\s\S]*task-form/, 'task-form widget entry is required');
assert.match(viteSrc, /input[\s\S]*user-dashboard/, 'user-dashboard widget entry is required');
assert.match(viteSrc, /outDir:\s*['"]dist\/widgets['"]/, 'widget build output must be dist/widgets');
assert.match(viteSrc, /emptyOutDir:\s*false/, 'widget build must not wipe dist output');

const commonPath = resolve(mcpRoot, 'src/widgets/src/common.ts');
assert.ok(existsSync(commonPath), 'mcp-server/src/widgets/src/common.ts is required');
const commonSrc = readFileSync(commonPath, 'utf8');
assert.match(commonSrc, /window\.openai/, 'common module must use window.openai bridge');
assert.match(commonSrc, /theme/, 'common module must handle theme');
assert.match(commonSrc, /toolOutput/, 'common module must read tool output');
assert.match(commonSrc, /widgetState/, 'common module must restore widget state');
assert.match(commonSrc, /notifyIntrinsicHeight/, 'common module must notify intrinsic height');

const entries = [
  'src/widgets/src/services-list.html',
  'src/widgets/src/task-form.html',
  'src/widgets/src/user-dashboard.html',
];
for (const file of entries) {
  const filePath = resolve(mcpRoot, file);
  assert.ok(existsSync(filePath), `mcp-server/${file} is required as widget entry`);
}

console.log('task-6.1 widget build checks passed');
