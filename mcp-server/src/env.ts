import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';

type MutableEnv = NodeJS.ProcessEnv;

function unquote(value: string): string {
  if (
    (value.startsWith('"') && value.endsWith('"')) ||
    (value.startsWith("'") && value.endsWith("'"))
  ) {
    return value.slice(1, -1);
  }
  return value;
}

function parseDotenv(content: string): Map<string, string> {
  const entries = new Map<string, string>();

  for (const rawLine of content.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line || line.startsWith('#')) continue;

    const separator = line.indexOf('=');
    if (separator <= 0) continue;

    const key = line.slice(0, separator).trim();
    const rawValue = line.slice(separator + 1).trim();
    entries.set(key, unquote(rawValue));
  }

  return entries;
}

function applyEnvFile(path: string, env: MutableEnv): void {
  if (!existsSync(path)) return;

  const parsed = parseDotenv(readFileSync(path, 'utf8'));
  for (const [key, value] of parsed) {
    if (env[key] === undefined) {
      env[key] = value;
    }
  }
}

export function loadEnvFromFiles(env: MutableEnv = process.env, cwd = process.cwd()): void {
  applyEnvFile(resolve(cwd, '.env'), env);
  applyEnvFile(resolve(cwd, '.env.local'), env);
}
