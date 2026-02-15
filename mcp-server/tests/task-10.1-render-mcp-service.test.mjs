import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const renderYaml = readFileSync(resolve(repoRoot, 'render.yaml'), 'utf8');

assert.match(renderYaml, /name:\s*subsidypayment-mcp/, 'render.yaml must define subsidypayment-mcp service');
assert.match(
  renderYaml,
  /buildCommand:\s*cd mcp-server && npm ci && npm run build/,
  'mcp service build command must be configured'
);
assert.match(
  renderYaml,
  /startCommand:\s*cd mcp-server && npm start/,
  'mcp service start command must be configured'
);

for (const key of [
  'RUST_BACKEND_URL',
  'MCP_INTERNAL_API_KEY',
  'AUTH0_DOMAIN',
  'AUTH0_AUDIENCE',
  'PUBLIC_URL',
  'PORT',
]) {
  assert.match(renderYaml, new RegExp(`key:\\s*${key}`), `mcp service must include env var ${key}`);
}

console.log('task-10.1 render mcp service checks passed');
