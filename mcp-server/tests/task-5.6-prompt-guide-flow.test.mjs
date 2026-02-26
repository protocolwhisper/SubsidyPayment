import { test } from 'vitest';
test('task assertions execute', () => {});
import { strict as assert } from 'node:assert';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { mcpRoot } from './test-paths.mjs';

const toolPath = resolve(mcpRoot, 'src/tools/get-prompt-guide-flow.ts');
assert.ok(existsSync(toolPath), 'mcp-server/src/tools/get-prompt-guide-flow.ts is required');

const src = readFileSync(toolPath, 'utf8');
assert.match(src, /registerAppTool\(\s*server\s*,\s*['"]get_prompt_guide_flow['"]/, 'must register get_prompt_guide_flow tool');
assert.match(src, /readOnlyHint:\s*true/, 'get_prompt_guide_flow must set readOnlyHint: true');
assert.match(src, /securitySchemes:\s*\[\s*\{\s*type:\s*['"]noauth['"]\s*\}\s*\]/, 'get_prompt_guide_flow must be noauth');
assert.match(src, /recommended_next_prompt/, 'must return recommended_next_prompt');
assert.match(src, /next_actions/, 'must return next_actions');
assert.match(src, /allowed_actions/, 'must return allowed_actions');

const indexPath = resolve(mcpRoot, 'src/tools/index.ts');
const indexSrc = readFileSync(indexPath, 'utf8');
assert.match(indexSrc, /registerGetPromptGuideFlowTool/, 'tools/index.ts must include get_prompt_guide_flow registration');

console.log('task-5.6 prompt guide flow checks passed');
