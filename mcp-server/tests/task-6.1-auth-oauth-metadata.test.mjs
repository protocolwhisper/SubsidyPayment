import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const modulePath = resolve(mcpRoot, 'src/auth/oauth-metadata.ts');
assert.ok(existsSync(modulePath), 'mcp-server/src/auth/oauth-metadata.ts is required');

const moduleSrc = readFileSync(modulePath, 'utf8');
assert.match(moduleSrc, /oauth-protected-resource/, 'must handle oauth-protected-resource metadata');
assert.match(moduleSrc, /authorization_servers/, 'metadata must include authorization_servers');
assert.match(moduleSrc, /scopes_supported/, 'metadata must include scopes_supported');
assert.match(moduleSrc, /resource/, 'metadata must include resource');
assert.match(moduleSrc, /auth0Domain/, 'metadata must use auth0Domain');
assert.match(moduleSrc, /publicUrl/, 'metadata must use publicUrl');

const mainPath = resolve(mcpRoot, 'src/main.ts');
const mainSrc = readFileSync(mainPath, 'utf8');
assert.match(mainSrc, /\/\.well-known\/oauth-protected-resource/, 'main.ts must expose oauth protected resource endpoint');

console.log('task-6.1 auth oauth-metadata checks passed');
