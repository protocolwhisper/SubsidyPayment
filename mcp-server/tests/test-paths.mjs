import { existsSync } from 'node:fs';
import { resolve } from 'node:path';

export const repoRoot = existsSync(resolve(process.cwd(), 'mcp-server'))
  ? process.cwd()
  : resolve(process.cwd(), '..');

export const mcpRoot = resolve(repoRoot, 'mcp-server');
