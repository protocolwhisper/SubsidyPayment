export interface BackendConfig {
  rustBackendUrl: string;
  mcpInternalApiKey: string;
  auth0Domain: string;
  auth0Audience: string;
  publicUrl: string;
  port: number;
  logLevel: string;
  authEnabled: boolean;
  x402WeatherUrl: string;
  x402FacilitatorUrl: string;
  x402Network: `${string}:${string}`;
  x402PrivateKey: string;
  x402RequestTimeoutMs: number;
}

function parsePort(value: string | undefined): number {
  if (!value) return 3001;
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 3001;
}

function resolveAuthEnabled(env: NodeJS.ProcessEnv): boolean {
  const explicit = env.AUTH_ENABLED;
  if (explicit !== undefined) {
    return !['false', '0', 'no'].includes(explicit.toLowerCase());
  }
  return !!(env.AUTH0_DOMAIN && env.AUTH0_AUDIENCE);
}

function parseTimeoutMs(value: string | undefined): number {
  if (!value) return 15000;
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 15000;
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
    authEnabled: resolveAuthEnabled(env),
    x402WeatherUrl: env.X402_WEATHER_URL || 'http://localhost:4021/weather',
    x402FacilitatorUrl: env.X402_FACILITATOR_URL || 'https://x402.org/facilitator',
    x402Network: (env.X402_NETWORK || 'eip155:84532') as `${string}:${string}`,
    x402PrivateKey: env.X402_PRIVATE_KEY || '',
    x402RequestTimeoutMs: parseTimeoutMs(env.X402_REQUEST_TIMEOUT_MS),
  };
}
