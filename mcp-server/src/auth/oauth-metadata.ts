import type { BackendConfig } from '../config.ts';

export const OAUTH_PROTECTED_RESOURCE_PATH = '/.well-known/oauth-protected-resource';
export const OAUTH_AUTHORIZATION_SERVER_PATH = '/.well-known/oauth-authorization-server';

const DEFAULT_SCOPES = ['user.read', 'user.write', 'tasks.read', 'tasks.write', 'services.execute'];

function normalizeAuth0Issuer(domain: string): string {
  if (!domain) return '';
  if (domain.startsWith('http://') || domain.startsWith('https://')) {
    return domain.replace(/\/$/, '');
  }
  return `https://${domain.replace(/\/$/, '')}`;
}

export function buildOAuthProtectedResourceMetadata(config: BackendConfig) {
  const issuer = normalizeAuth0Issuer(config.auth0Domain);
  return {
    resource: config.publicUrl,
    authorization_servers: issuer ? [issuer] : [],
    scopes_supported: DEFAULT_SCOPES,
  };
}

export function oauthProtectedResourceHandler(config: BackendConfig) {
  return (_req: any, res: any) => {
    res.json(buildOAuthProtectedResourceMetadata(config));
  };
}

export function oauthAuthorizationServerRedirectHandler(config: BackendConfig) {
  return (_req: any, res: any) => {
    const issuer = normalizeAuth0Issuer(config.auth0Domain);
    if (!issuer) {
      res.status(503).json({
        error: {
          code: 'auth_server_not_configured',
          message: 'AUTH0_DOMAIN is not configured',
        },
      });
      return;
    }

    res.redirect(302, `${issuer}/.well-known/oauth-authorization-server`);
  };
}
