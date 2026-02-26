import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot } from './test-paths.mjs';

const widgetFiles = [
  'src/widgets/src/services-list.html',
  'src/widgets/src/service-tasks.html',
  'src/widgets/src/task-form.html',
  'src/widgets/src/service-access.html',
  'src/widgets/src/user-dashboard.html',
];

for (const file of widgetFiles) {
  const src = readFileSync(resolve(mcpRoot, file), 'utf8');
  assert.match(src, /チャットに入力してください（ask in chat to do tasks）/, `${file} must show chat guide banner text`);
  assert.match(src, /sendFollowUpMessage/, `${file} must support follow-up message guidance`);
}

console.log('task-6.4 chat guide banner checks passed');
