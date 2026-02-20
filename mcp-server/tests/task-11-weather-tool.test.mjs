import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot } from './test-paths.mjs';

const toolPath = resolve(mcpRoot, 'src/tools/weather.ts');
assert.ok(existsSync(toolPath), 'mcp-server/src/tools/weather.ts is required');

const src = readFileSync(toolPath, 'utf8');
assert.match(src, /registerAppTool\(\s*server\s*,\s*['"]weather['"]/, 'must register weather tool');
assert.match(src, /city:\s*z\.string\(\)\.trim\(\)\.min\(1\)/, 'weather tool must require city input');
assert.match(src, /securitySchemes:\s*\[\s*\{\s*type:\s*['"]noauth['"]\s*\}\s*\]/, 'weather tool must be noauth');

const weatherClientPath = resolve(mcpRoot, 'src/x402/weather-client.ts');
assert.ok(existsSync(weatherClientPath), 'mcp-server/src/x402/weather-client.ts is required');
const weatherClientSrc = readFileSync(weatherClientPath, 'utf8');
assert.match(weatherClientSrc, /wrapAxiosWithPayment/, 'weather client must use x402 payment wrapper');

const indexPath = resolve(mcpRoot, 'src/tools/index.ts');
const indexSrc = readFileSync(indexPath, 'utf8');
assert.match(indexSrc, /registerWeatherTool/, 'tools/index.ts must include weather registration');

console.log('task-11 weather-tool checks passed');
