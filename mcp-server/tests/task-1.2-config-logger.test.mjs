import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { existsSync, readFileSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { resolve, join } from 'node:path';
import { spawnSync } from 'node:child_process';
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

const tempDir = mkdtempSync(join(tmpdir(), 'task-1-2-'));
const runnerPath = join(tempDir, 'runner.ts');

writeFileSync(
  runnerPath,
  `import { loadConfig } from ${JSON.stringify(configPath)};\n` +
    `const cfg = loadConfig();\n` +
    `if (cfg.port !== 3001) throw new Error('default port must be 3001');\n` +
    `if (cfg.logLevel !== 'info') throw new Error('default log level must be info');\n` +
    `if (cfg.rustBackendUrl !== 'http://localhost:3000') throw new Error('default rustBackendUrl mismatch');\n` +
    `if (cfg.publicUrl !== 'http://localhost:3001') throw new Error('default publicUrl mismatch');\n` +
    `console.log('runtime config defaults ok');\n`,
  'utf8'
);

const result = spawnSync(
  process.execPath,
  ['--experimental-strip-types', runnerPath],
  {
    env: {
      ...process.env,
      PORT: '',
      RUST_BACKEND_URL: '',
      MCP_INTERNAL_API_KEY: '',
      AUTH0_DOMAIN: '',
      AUTH0_AUDIENCE: '',
      PUBLIC_URL: '',
      LOG_LEVEL: '',
    },
    encoding: 'utf8',
  }
);

if (result.status !== 0) {
  process.stderr.write(result.stdout);
  process.stderr.write(result.stderr);
}
assert.equal(result.status, 0, 'loadConfig runtime defaults test failed');

console.log('task-1.2 config/logger checks passed');
