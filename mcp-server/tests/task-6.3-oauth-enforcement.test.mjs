import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot, repoRoot } from './test-paths.mjs';

const mustEnforce = [
  'authenticate-user.ts',
  'get-task-details.ts',
  'complete-task.ts',
  'run-service.ts',
  'get-user-status.ts',
  'get-preferences.ts',
  'set-preferences.ts',
];

for (const file of mustEnforce) {
  const src = readFileSync(resolve(mcpRoot, `src/tools/${file}`), 'utf8');
  assert.match(src, /TokenVerifier/, `${file}: must import/use TokenVerifier`);
  assert.match(src, /verify\(/, `${file}: must call TokenVerifier.verify(...)`);
  assert.match(src, /mcp\/www_authenticate/, `${file}: must return mcp/www_authenticate on auth failure`);
}

const searchSrc = readFileSync(resolve(mcpRoot, 'src/tools/search-services.ts'), 'utf8');
assert.match(searchSrc, /securitySchemes:\s*\[\s*\{\s*type:\s*['"]noauth['"]\s*\}\s*\]/, 'search_services must remain noauth');

console.log('task-6.3 oauth enforcement checks passed');
