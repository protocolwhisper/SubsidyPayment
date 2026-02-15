import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const srcPath = resolve(mcpRoot, 'src/widgets/index.ts');
const src = readFileSync(srcPath, 'utf8');

assert.match(src, /registerAppResource/, 'must use registerAppResource');
assert.match(src, /RESOURCE_MIME_TYPE/, 'must use text\\/html;profile=mcp-app MIME type');

assert.match(src, /ui:\/\/widget\/services-list\.html/, 'must register services-list resource URI');
assert.match(src, /ui:\/\/widget\/task-form\.html/, 'must register task-form resource URI');
assert.match(src, /ui:\/\/widget\/user-dashboard\.html/, 'must register user-dashboard resource URI');

assert.match(src, /dist\/widgets/, 'must read built widget HTML files from dist/widgets');
assert.match(src, /services-list\.html/, 'must read services-list HTML file');
assert.match(src, /task-form\.html/, 'must read task-form HTML file');
assert.match(src, /user-dashboard\.html/, 'must read user-dashboard HTML file');

console.log('task-7.5 widget resources checks passed');
