import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const toolPath = resolve(mcpRoot, 'src/tools/run-service.ts');
assert.ok(existsSync(toolPath), 'mcp-server/src/tools/run-service.ts is required');

const src = readFileSync(toolPath, 'utf8');

assert.match(src, /registerAppTool\(\s*server\s*,\s*['"]run_service['"]/, 'must register run_service tool');
assert.match(src, /openWorldHint:\s*true/, 'run_service must set openWorldHint: true');
assert.match(src, /securitySchemes:[\s\S]*type:\s*['"]oauth2['"]/, 'run_service must require oauth2');
assert.match(src, /runService\(/, 'must call BackendClient.runService');

assert.match(src, /session_token/, 'run_service must use session_token');
assert.match(src, /structuredContent\s*:\s*\{[\s\S]*service[\s\S]*payment_mode[\s\S]*sponsored_by[\s\S]*tx_hash[\s\S]*\}/, 'structuredContent must only include service/payment fields');
assert.match(src, /_meta\s*:\s*\{[\s\S]*output/, 'output must be returned in _meta');

const indexPath = resolve(mcpRoot, 'src/tools/index.ts');
const indexSrc = readFileSync(indexPath, 'utf8');
assert.match(indexSrc, /registerRunServiceTool/, 'tools/index.ts must include run_service registration');

console.log('task-5.4 run-service checks passed');
