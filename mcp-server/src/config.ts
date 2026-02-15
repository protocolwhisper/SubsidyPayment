export interface BackendConfig {
  rustBackendUrl: string;
  mcpInternalApiKey: string;
  auth0Domain: string;
  auth0Audience: string;
  publicUrl: string;
  port: number;
  logLevel: string;
}

function parsePort(value: string | undefined): number {
  if (!value) return 3001;
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 3001;
}

export function loadConfig(env: NodeJS.ProcessEnv = process.env): BackendConfig {
  return {
    rustBackendUrl: env.RUST_BACKEND_URL || 'http://localhost:3000',
    mcpInternalApiKey: env.MCP_INTERNAL_API_KEY || '',
    auth0Domain: env.AUTH0_DOMAIN || '',
    auth0Audience: env.AUTH0_AUDIENCE || '',
    publicUrl: env.PUBLIC_URL || 'http://localhost:3001',
    port: parsePort(env.PORT),
    logLevel: env.LOG_LEVEL || 'info',
  };
}
