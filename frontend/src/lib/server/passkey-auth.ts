import { createHmac } from 'node:crypto';

import { decodeCredentialPublicKey } from '@simplewebauthn/server/helpers';
import { env as privateEnv } from '$env/dynamic/private';
import { env as publicEnv } from '$env/dynamic/public';

const USER_ID_RE =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

export interface AdminFactor {
  id: string;
  factor_type?: string;
  status?: string;
  last_webauthn_challenge_data?: Record<string, unknown> | null;
}

export interface AdminUser {
  id: string;
  email?: string | null;
  phone?: string | null;
  app_metadata?: Record<string, unknown> | null;
  user_metadata?: Record<string, unknown> | null;
  role?: string | null;
  factors?: AdminFactor[] | null;
}

interface VerifyCredential {
  id: string;
  publicKey: Buffer;
  counter: number;
  transports?: string[];
}

function requireValue(name: string, value: string): string {
  if (value.trim().length === 0) {
    throw new Error(`${name} is required for passkey login.`);
  }
  return value.trim();
}

function resolvePublicSupabaseUrl(): string {
  return requireValue(
    'PUBLIC_SUPABASE_URL',
    privateEnv.PUBLIC_SUPABASE_URL ?? publicEnv.PUBLIC_SUPABASE_URL ?? ''
  );
}

function resolveSupabaseAdminBaseUrl(): string {
  const privateUrl = (privateEnv.PRIVATE_SUPABASE_URL ?? privateEnv.SUPABASE_INTERNAL_URL ?? '').trim();
  if (privateUrl.length > 0) {
    return privateUrl;
  }
  return resolvePublicSupabaseUrl();
}

function resolveServiceRoleKey(): string {
  return requireValue(
    'SERVICE_ROLE_KEY',
    privateEnv.SERVICE_ROLE_KEY ?? privateEnv.SUPABASE_SERVICE_ROLE_KEY ?? ''
  );
}

function resolveJwtSecret(): string {
  return requireValue(
    'JWT_SECRET',
    privateEnv.JWT_SECRET ?? privateEnv.SUPABASE_AUTH_JWT_SECRET ?? ''
  );
}

function resolveJwtIssuer(supabaseUrl: string): string {
  const configured = (privateEnv.SUPABASE_AUTH_JWT_ISSUER ?? '').trim();
  if (configured.length > 0) return configured;
  return `${supabaseUrl.replace(/\/+$/, '')}/auth/v1`;
}

function decodeBase64Url(value: string): Buffer {
  const normalized = value.replace(/-/g, '+').replace(/_/g, '/');
  const padding = '='.repeat((4 - (normalized.length % 4)) % 4);
  return Buffer.from(`${normalized}${padding}`, 'base64');
}

function normalizeCredentialId(value: string): string | null {
  const input = value.trim();
  if (input.length === 0) return null;

  try {
    return decodeBase64Url(input).toString('base64url');
  } catch {
    try {
      return Buffer.from(input, 'base64').toString('base64url');
    } catch {
      return null;
    }
  }
}

function decodeBase64Any(value: string): Buffer | null {
  const trimmed = value.trim();
  if (trimmed.length === 0) return null;

  try {
    return decodeBase64Url(trimmed);
  } catch {
    try {
      return Buffer.from(trimmed, 'base64');
    } catch {
      return null;
    }
  }
}

function isLikelyCOSEPublicKey(bytes: Buffer): boolean {
  if (bytes.length === 0) return false;
  // COSE keys are CBOR maps (major type 5 -> 0b101xxxxx => 0xA0..0xBF).
  return (bytes[0] & 0xe0) === 0xa0;
}

function canDecodeCOSEPublicKey(bytes: Buffer): boolean {
  try {
    const decoded = decodeCredentialPublicKey(new Uint8Array(bytes));
    return typeof decoded?.get === 'function';
  } catch {
    return false;
  }
}

function asObject(value: unknown): Record<string, unknown> | null {
  return value && typeof value === 'object' ? (value as Record<string, unknown>) : null;
}

function asString(value: unknown): string | null {
  return typeof value === 'string' && value.trim().length > 0 ? value : null;
}

function asStringArray(value: unknown): string[] | undefined {
  if (!Array.isArray(value)) return undefined;
  const items = value.filter((item): item is string => typeof item === 'string' && item.length > 0);
  return items.length > 0 ? items : undefined;
}

function pickFirstString(values: unknown[]): string | null {
  for (const value of values) {
    const parsed = asString(value);
    if (parsed) return parsed;
  }
  return null;
}

async function authAdminRequest<T>(path: string): Promise<T> {
  const supabaseUrl = resolveSupabaseAdminBaseUrl();
  const serviceRoleKey = resolveServiceRoleKey();
  const endpoint = `${supabaseUrl.replace(/\/+$/, '')}/auth/v1${path}`;

  const response = await fetch(endpoint, {
    headers: {
      apikey: serviceRoleKey,
      Authorization: `Bearer ${serviceRoleKey}`
    }
  });

  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    const message =
      asString(asObject(payload)?.msg) ??
      asString(asObject(payload)?.message) ??
      `Auth admin request failed (${response.status})`;
    throw new Error(message);
  }

  return payload as T;
}

export function decodeUserHandleToUserId(userHandle: string | null | undefined): string | null {
  if (!userHandle) return null;

  try {
    const decoded = decodeBase64Url(userHandle).toString('utf8').trim();
    return USER_ID_RE.test(decoded) ? decoded : null;
  } catch {
    return null;
  }
}

export async function getAdminUser(userId: string): Promise<AdminUser | null> {
  if (!USER_ID_RE.test(userId)) return null;
  try {
    const user = await authAdminRequest<AdminUser>(`/admin/users/${userId}`);
    if (!Array.isArray(user.factors) || user.factors.length === 0) {
      const factors = await listAdminUserFactors(userId);
      if (factors.length > 0) {
        return { ...user, factors };
      }
    }
    return user;
  } catch {
    return null;
  }
}

async function listAdminUsers(page: number, perPage = 100): Promise<AdminUser[]> {
  const payload = await authAdminRequest<{ users?: AdminUser[] }>(
    `/admin/users?page=${page}&per_page=${perPage}`
  );
  return payload.users ?? [];
}

async function listAdminUserFactors(userId: string): Promise<AdminFactor[]> {
  try {
    const factors = await authAdminRequest<AdminFactor[]>(`/admin/users/${userId}/factors`);
    return Array.isArray(factors) ? factors : [];
  } catch {
    return [];
  }
}

export async function resolveUserForCredential(
  credentialId: string,
  userHandle?: string | null
): Promise<AdminUser | null> {
  const decodedUserId = decodeUserHandleToUserId(userHandle);
  if (decodedUserId) {
    const directUser = await getAdminUser(decodedUserId);
    if (directUser) return directUser;
  }

  for (let page = 1; page <= 5; page += 1) {
    const users = await listAdminUsers(page);
    if (users.length === 0) break;

    for (const user of users) {
      const fullUser = await getAdminUser(user.id);
      if (!fullUser) continue;
      if (findMatchingFactor(fullUser.factors ?? [], credentialId)) {
        return fullUser;
      }
    }
  }

  return null;
}

export function extractCredentialFromFactor(factor: AdminFactor): VerifyCredential | null {
  const challengeData = asObject(factor.last_webauthn_challenge_data);
  const credentialResponse = asObject(challengeData?.credential_response);
  const rawPayload = asObject(credentialResponse?.Raw);
  const responsePayload = asObject(credentialResponse?.Response);
  const responseAttestationObjectPayload = asObject(responsePayload?.AttestationObject);
  const responseAttestationAuthDataPayload = asObject(responseAttestationObjectPayload?.AuthData);
  const authDataPayload = asObject(responsePayload?.AuthData) ?? responseAttestationAuthDataPayload;
  const attDataPayload = asObject(authDataPayload?.att_data);

  const credentialIdRaw = pickFirstString([
    credentialResponse?.ID,
    credentialResponse?.rawId,
    rawPayload?.id,
    rawPayload?.rawId,
    responsePayload?.rawId,
    attDataPayload?.credential_id,
    attDataPayload?.credentialId
  ]);
  const publicKeyRaw = pickFirstString([attDataPayload?.public_key, attDataPayload?.publicKey]);

  if (!credentialIdRaw || !publicKeyRaw) return null;

  const normalizedId = normalizeCredentialId(credentialIdRaw);
  if (!normalizedId) return null;

  const publicKey = decodeBase64Any(publicKeyRaw);
  if (!publicKey) {
    return null;
  }

  if (!isLikelyCOSEPublicKey(publicKey)) {
    return null;
  }

  if (!canDecodeCOSEPublicKey(publicKey)) {
    return null;
  }

  const signCountRaw = authDataPayload?.sign_count;
  const counter = typeof signCountRaw === 'number' && Number.isFinite(signCountRaw) ? signCountRaw : 0;
  const transports =
    asStringArray(responsePayload?.Transports) ??
    asStringArray(asObject(rawPayload?.response)?.transports);

  return {
    id: normalizedId,
    publicKey,
    counter,
    transports
  };
}

export function findMatchingFactor(
  factors: AdminFactor[] | null | undefined,
  assertionCredentialId: string
): AdminFactor | null {
  const normalizedAssertionId = normalizeCredentialId(assertionCredentialId);
  if (!normalizedAssertionId) return null;
  if (!Array.isArray(factors)) return null;

  for (const factor of factors) {
    if (factor.factor_type !== 'webauthn' || factor.status !== 'verified') continue;
    const credential = extractCredentialFromFactor(factor);
    if (!credential) continue;
    if (credential.id === normalizedAssertionId) {
      return factor;
    }
  }

  return null;
}

function signJwt(payload: Record<string, unknown>, secret: string): string {
  const header = { alg: 'HS256', typ: 'JWT' };
  const encodedHeader = Buffer.from(JSON.stringify(header)).toString('base64url');
  const encodedPayload = Buffer.from(JSON.stringify(payload)).toString('base64url');
  const unsignedToken = `${encodedHeader}.${encodedPayload}`;
  const signature = createHmac('sha256', secret).update(unsignedToken).digest('base64url');
  return `${unsignedToken}.${signature}`;
}

export function mintPasskeyTokens(user: AdminUser): { access_token: string; refresh_token: string } {
  const publicSupabaseUrl = resolvePublicSupabaseUrl();
  const jwtSecret = resolveJwtSecret();
  const issuer = resolveJwtIssuer(publicSupabaseUrl);

  const issuedAt = Math.floor(Date.now() / 1000);
  const expiresAt = issuedAt + 60 * 60;

  const payload: Record<string, unknown> = {
    iss: issuer,
    sub: user.id,
    aud: 'authenticated',
    exp: expiresAt,
    iat: issuedAt,
    email: user.email ?? null,
    phone: user.phone ?? '',
    app_metadata: user.app_metadata ?? {},
    user_metadata: user.user_metadata ?? {},
    role: user.role ?? 'authenticated',
    aal: 'aal2',
    amr: [{ method: 'mfa/webauthn', timestamp: issuedAt }],
    is_anonymous: false
  };

  const accessToken = signJwt(payload, jwtSecret);

  // Supabase JS requires a refresh token shape for setSession, but this flow is passkey-first.
  return {
    access_token: accessToken,
    refresh_token: accessToken
  };
}
