import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const guidePath = resolve(mcpRoot, 'docs/migration-guide.md');
assert.ok(existsSync(guidePath), 'mcp-server/docs/migration-guide.md is required');

const guide = readFileSync(guidePath, 'utf8');

assert.match(guide, /openapi\.yaml/i, 'guide must include openapi.yaml removal step');
assert.match(
  guide,
  /\/\.well-known\/openapi\.yaml/i,
  'guide must include /.well-known/openapi.yaml endpoint removal step'
);
assert.match(guide, /GPT Builder/i, 'guide must include GPT Builder configuration cleanup');

assert.match(guide, /移行確認チェックリスト/i, 'guide must include migration checklist section');
assert.match(guide, /ロールバック手順/i, 'guide must include rollback procedure section');

console.log('task-10.3 migration guide checks passed');
