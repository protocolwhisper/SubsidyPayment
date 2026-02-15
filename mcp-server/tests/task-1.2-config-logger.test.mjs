import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const configPath = resolve(mcpRoot, 'src/config.ts');
const loggerPath = resolve(mcpRoot, 'src/logger.ts');
const envExamplePath = resolve(repoRoot, '.env.example');

assert.ok(existsSync(configPath), 'mcp-server/src/config.ts is required');
assert.ok(existsSync(loggerPath), 'mcp-server/src/logger.ts is required');

const loggerSrc = readFileSync(loggerPath, 'utf8');
assert.match(loggerSrc, /from\s+["']pino["']/, 'logger.ts must import pino');
assert.match(loggerSrc, /pino\s*\(/, 'logger.ts must create pino logger');

const requiredEnvVars = [
  'PORT',
  'RUST_BACKEND_URL',
  'MCP_INTERNAL_API_KEY',
  'AUTH0_DOMAIN',
  'AUTH0_AUDIENCE',
  'PUBLIC_URL',
  'LOG_LEVEL',
];

const envText = readFileSync(envExamplePath, 'utf8');
for (const key of requiredEnvVars) {
  assert.match(envText, new RegExp(`^${key}=`, 'm'), `.env.example missing ${key}`);
}

const { loadConfig } = await import('../src/config.ts');
const cfg = loadConfig({
  PORT: '',
  RUST_BACKEND_URL: '',
  MCP_INTERNAL_API_KEY: '',
  AUTH0_DOMAIN: '',
  AUTH0_AUDIENCE: '',
  PUBLIC_URL: '',
  LOG_LEVEL: '',
});
assert.equal(cfg.port, 3001, 'default port must be 3001');
assert.equal(cfg.logLevel, 'info', 'default log level must be info');
assert.equal(cfg.rustBackendUrl, 'http://localhost:3000', 'default rustBackendUrl mismatch');
assert.equal(cfg.publicUrl, 'http://localhost:3001', 'default publicUrl mismatch');

console.log('task-1.2 config/logger checks passed');
