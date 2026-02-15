import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const filePath = resolve(mcpRoot, 'src/server.ts');
assert.ok(existsSync(filePath), 'mcp-server/src/server.ts is required');

const src = readFileSync(filePath, 'utf8');

assert.match(src, /from\s+['"]@modelcontextprotocol\/sdk\/server\/mcp\.js['"]/, 'must import McpServer');
assert.match(src, /from\s+['"]\.\/tools\/index\.ts['"]/, 'must import registerAllTools');
assert.match(src, /from\s+['"]\.\/widgets\/index\.ts['"]/, 'must import registerAllResources');
assert.match(src, /export\s+function\s+createServer\s*\(\s*config\s*:\s*BackendConfig\s*\)\s*:\s*McpServer/, 'createServer signature mismatch');
assert.match(src, /new\s+McpServer\(\s*\{[\s\S]*name:\s*['"]subsidypayment['"][\s\S]*version:\s*['"]1\.0\.0['"][\s\S]*\}\s*\)/, 'McpServer must be initialized with name/version');
assert.match(src, /registerAllTools\(\s*server\s*,\s*config\s*\)/, 'must call registerAllTools(server, config)');
assert.match(src, /registerAllResources\(\s*server\s*\)/, 'must call registerAllResources(server)');
assert.match(src, /return\s+server\s*;/, 'must return server instance');

console.log('task-4.2 server checks passed');
