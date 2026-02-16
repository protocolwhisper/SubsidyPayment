import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const toolPath = resolve(mcpRoot, 'src/tools/authenticate-user.ts');
assert.ok(existsSync(toolPath), 'mcp-server/src/tools/authenticate-user.ts is required');

const src = readFileSync(toolPath, 'utf8');

assert.match(src, /registerAppTool\(\s*server\s*,\s*['"]authenticate_user['"]/, 'must register authenticate_user tool');
assert.match(src, /securitySchemes:[\s\S]*type:\s*['"]oauth2['"]/, 'oauth2 security scheme is required');
assert.match(src, /openai\/toolInvocation\/invoking/, 'invoking message is required');
assert.match(src, /openai\/toolInvocation\/invoked/, 'invoked message is required');

assert.match(src, /context\?\.?auth\?\.?email|context\?\.?_meta\?\.?auth\?\.?email/, 'must use OAuth token email from context');
assert.match(src, /authenticateUser\(\s*\{[\s\S]*email\s*:\s*oauthEmail/, 'BackendClient.authenticateUser must use oauthEmail');

assert.match(src, /session_token/, 'session_token must be referenced');
assert.match(src, /_meta\s*:\s*\{[\s\S]*session_token/, 'session_token must be returned in _meta');
assert.match(src, /mcp\/www_authenticate/, 'must return _meta.mcp/www_authenticate when auth info is missing');
assert.match(src, /isError\s*:\s*true/, 'auth missing response must be error');

const indexPath = resolve(mcpRoot, 'src/tools/index.ts');
const indexSrc = readFileSync(indexPath, 'utf8');
assert.match(indexSrc, /registerAuthenticateUserTool/, 'tools/index.ts must include authenticate_user registration');

console.log('task-5.2 authenticate-user checks passed');
