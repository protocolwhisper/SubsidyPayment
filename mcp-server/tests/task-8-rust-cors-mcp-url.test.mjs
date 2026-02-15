import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const mainRs = readFileSync(resolve(repoRoot, 'src/main.rs'), 'utf8');
assert.match(mainRs, /MCP_SERVER_URL/, 'src/main.rs must reference MCP_SERVER_URL');
assert.match(mainRs, /collect_cors_origins\(/, 'src/main.rs must collect CORS origins with helper');
assert.match(mainRs, /mcp_server_url/, 'src/main.rs must read MCP server URL for CORS');

const envExample = readFileSync(resolve(repoRoot, '.env.example'), 'utf8');
assert.match(envExample, /^MCP_SERVER_URL=.*$/m, '.env.example must document MCP_SERVER_URL');

console.log('task-8 rust cors mcp url checks passed');
