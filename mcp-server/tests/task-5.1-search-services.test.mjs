import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const toolPath = resolve(mcpRoot, 'src/tools/search-services.ts');
assert.ok(existsSync(toolPath), 'mcp-server/src/tools/search-services.ts is required');

const src = readFileSync(toolPath, 'utf8');

assert.match(src, /from\s+['"]@modelcontextprotocol\/ext-apps\/server['"]/, 'must import registerAppTool');
assert.match(src, /from\s+['"]zod['"]/, 'must import zod');
assert.match(src, /BackendClient/, 'must use BackendClient');
assert.match(src, /registerAppTool\(\s*server\s*,\s*['"]search_services['"]/, 'must register search_services tool');

assert.match(src, /annotations:\s*\{[\s\S]*readOnlyHint:\s*true[\s\S]*\}/, 'readOnlyHint: true is required');
assert.match(src, /securitySchemes:\s*\[\s*\{\s*type:\s*['"]noauth['"]\s*\}\s*\]/, 'securitySchemes noauth is required');

assert.match(src, /resourceUri:\s*['"]ui:\/\/widget\/services-list\.html['"]/, 'widget resourceUri is required');
assert.match(src, /openai\/toolInvocation\/invoking/, 'invoking message is required');
assert.match(src, /openai\/toolInvocation\/invoked/, 'invoked message is required');

assert.match(src, /searchServices\(/, 'must call BackendClient.searchServices');
assert.match(src, /structuredContent\s*:/, 'must return structuredContent');
assert.match(src, /content\s*:/, 'must return content');
assert.match(src, /_meta\s*:/, 'must return _meta');

const indexPath = resolve(mcpRoot, 'src/tools/index.ts');
const indexSrc = readFileSync(indexPath, 'utf8');
assert.match(indexSrc, /registerSearchServicesTool/, 'tools/index.ts must include search_services registration');

console.log('task-5.1 search-services checks passed');
