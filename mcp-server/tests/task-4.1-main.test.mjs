import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const filePath = resolve(mcpRoot, 'src/main.ts');
assert.ok(existsSync(filePath), 'mcp-server/src/main.ts is required');

const src = readFileSync(filePath, 'utf8');

assert.match(src, /from\s+['"]express['"]/, 'main.ts must import express');
assert.match(src, /from\s+['"]cors['"]/, 'main.ts must import cors');
assert.match(src, /StreamableHTTPServerTransport/, 'main.ts must use StreamableHTTPServerTransport');
assert.match(src, /createServer\(/, 'main.ts must call createServer');
assert.match(src, /loadConfig\(/, 'main.ts must load config');

for (const origin of [
  'https://chatgpt.com',
  'https://cdn.oaistatic.com',
  'https://web-sandbox.oaiusercontent.com',
]) {
  assert.match(src, new RegExp(origin.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')), `missing CORS origin: ${origin}`);
}

assert.match(src, /app\.get\(\s*['"]\/health['"]/, 'GET /health route is required');
assert.match(src, /status:\s*['"]ok['"]/, '/health must return status ok');
assert.match(src, /version:/, '/health must return version');
assert.match(src, /uptime:/, '/health must return uptime');

assert.match(src, /app\.post\(\s*['"]\/mcp['"]/, 'POST /mcp route is required');
assert.match(src, /new\s+StreamableHTTPServerTransport\(/, '/mcp must create transport per request');
assert.match(src, /await\s+server\.connect\(transport\)/, '/mcp must connect server with transport');
assert.match(src, /await\s+transport\.handleRequest\(/, '/mcp must delegate request to transport');

console.log('task-4.1 main checks passed');
