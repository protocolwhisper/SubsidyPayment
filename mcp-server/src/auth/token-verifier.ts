import jwt, { type JwtPayload } from 'jsonwebtoken';
import jwksClient, { type SigningKey } from 'jwks-rsa';

export interface AuthInfo {
  sub: string;
  email: string;
  scopes: string[];
  token: string;
}

type TokenVerifierConfig = {
  domain: string;
  audience: string;
};

function normalizeIssuer(domain: string): string {
  if (!domain) return '';
  const withProtocol = domain.startsWith('http://') || domain.startsWith('https://') ? domain : `https://${domain}`;
  return `${withProtocol.replace(/\/$/, '')}/`;
}

function parseScopes(payload: JwtPayload): string[] {
  const scope = payload.scope;
  if (typeof scope === 'string') {
    return scope.split(/\s+/).filter(Boolean);
  }

  const scopes = payload.scopes;
  if (Array.isArray(scopes)) {
    return scopes.filter((v): v is string => typeof v === 'string' && v.length > 0);
  }

  return [];
}

function resolveEmail(payload: JwtPayload): string | null {
  if (typeof payload.email === 'string' && payload.email.length > 0) {
    return payload.email;
  }

  const customEmail = payload['https://subsidypayment/email'];
  if (typeof customEmail === 'string' && customEmail.length > 0) {
    return customEmail;
  }

  return null;
}

export class TokenVerifier {
  private readonly audience: string;
  private readonly issuer: string;
  private readonly jwks;

  constructor(config: TokenVerifierConfig) {
    this.audience = config.audience;
    this.issuer = normalizeIssuer(config.domain);
    this.jwks = jwksClient({
      jwksUri: `${this.issuer}.well-known/jwks.json`,
      cache: true,
      rateLimit: true,
    });
  }

  async verify(token: string): Promise<AuthInfo | null> {
    if (!token || !this.audience || !this.issuer) {
      return null;
    }

    try {
      const decoded = jwt.decode(token, { complete: true });
      const kid = decoded && typeof decoded === 'object' ? decoded.header?.kid : undefined;
      if (!kid || typeof kid !== 'string') {
        return null;
      }

      const signingKey = await this.getSigningKey(kid);
      if (!signingKey) {
        return null;
      }

      const verified = jwt.verify(token, signingKey, {
        algorithms: ['RS256'],
        audience: this.audience,
        issuer: this.issuer,
      });

      if (!verified || typeof verified !== 'object') {
        return null;
      }

      const payload = verified as JwtPayload;
      const sub = typeof payload.sub === 'string' ? payload.sub : null;
      const email = resolveEmail(payload);
      if (!sub || !email) {
        return null;
      }

      return {
        sub,
        email,
        scopes: parseScopes(payload),
        token,
      };
    } catch {
      return null;
    }
  }

  private async getSigningKey(kid: string): Promise<string | null> {
    try {
      const key = await new Promise<SigningKey>((resolve, reject) => {
        this.jwks.getSigningKey(kid, (error, signingKey) => {
          if (error || !signingKey) {
            reject(error ?? new Error('missing signing key'));
            return;
          }
          resolve(signingKey);
        });
      });

      if ('getPublicKey' in key && typeof key.getPublicKey === 'function') {
        return key.getPublicKey();
      }

      return null;
    } catch {
      return null;
    }
  }
}
