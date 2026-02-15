import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

function readJson(path) {
  return JSON.parse(readFileSync(path, 'utf8'));
}

const root = mcpRoot;
const pkgPath = resolve(root, 'package.json');
const tsconfigPath = resolve(root, 'tsconfig.json');

const requiredDeps = [
  '@modelcontextprotocol/sdk',
  '@modelcontextprotocol/ext-apps',
  'express',
  'cors',
  'zod',
  'pino',
];

const requiredDevDeps = [
  'typescript',
  'tsx',
  'vitest',
  'vite',
  'vite-plugin-singlefile',
];

const requiredScripts = ['build', 'start', 'dev', 'test'];

const pkg = readJson(pkgPath);
assert.equal(typeof pkg.name, 'string');
assert.equal(typeof pkg.scripts, 'object');

for (const key of requiredScripts) {
  assert.equal(typeof pkg.scripts[key], 'string', `missing script: ${key}`);
}

for (const dep of requiredDeps) {
  assert.equal(typeof pkg.dependencies?.[dep], 'string', `missing dependency: ${dep}`);
}

for (const dep of requiredDevDeps) {
  assert.equal(typeof pkg.devDependencies?.[dep], 'string', `missing devDependency: ${dep}`);
}

const tsconfig = readJson(tsconfigPath);
assert.equal(typeof tsconfig.compilerOptions, 'object', 'compilerOptions is required');
assert.ok(Array.isArray(tsconfig.include), 'include must be an array');

console.log('task-1.1 config checks passed');
