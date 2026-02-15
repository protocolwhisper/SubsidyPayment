import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const verifierPath = resolve(mcpRoot, 'src/auth/token-verifier.ts');
assert.ok(existsSync(verifierPath), 'mcp-server/src/auth/token-verifier.ts is required');

const src = readFileSync(verifierPath, 'utf8');

assert.match(src, /from\s+['"]jwks-rsa['"]/, 'TokenVerifier must use jwks-rsa');
assert.match(src, /from\s+['"]jsonwebtoken['"]/, 'TokenVerifier must use jsonwebtoken');
assert.match(src, /export\s+interface\s+AuthInfo/, 'AuthInfo type must be defined');
assert.match(src, /sub:\s*string/, 'AuthInfo.sub must be defined');
assert.match(src, /email:\s*string/, 'AuthInfo.email must be defined');
assert.match(src, /scopes:\s*string\[\]/, 'AuthInfo.scopes must be defined');
assert.match(src, /token:\s*string/, 'AuthInfo.token must be defined');
assert.match(src, /export\s+class\s+TokenVerifier/, 'TokenVerifier class must be exported');
assert.match(src, /verify\(\s*token:\s*string\s*\)\s*:\s*Promise<\s*AuthInfo\s*\|\s*null\s*>/, 'verify must return Promise<AuthInfo | null>');
assert.match(src, /audience/, 'audience verification must be implemented');
assert.match(src, /issuer/, 'issuer verification must be implemented');
assert.match(src, /return\s+null/, 'verification failures must return null');

const packageJsonPath = resolve(mcpRoot, 'package.json');
const packageJson = readFileSync(packageJsonPath, 'utf8');
assert.match(packageJson, /"jsonwebtoken"\s*:/, 'package.json must include jsonwebtoken dependency');
assert.match(packageJson, /"jwks-rsa"\s*:/, 'package.json must include jwks-rsa dependency');

console.log('task-6.2 token-verifier checks passed');
