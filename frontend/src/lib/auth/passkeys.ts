import { startAuthentication, startRegistration } from '@simplewebauthn/browser';

import { ensureCsrfToken } from '$lib/auth/csrf';
import { logDebug, logInfo, logWarn } from '$lib/debug';
import type { ApiErrorEnvelope, ApiEnvelope } from '$lib/types';

interface PasskeyChallengeResponse {
  challenge_id: string;
  options: unknown;
}

export interface PasskeyCredentialSummary {
  id: string;
  nickname: string | null;
  created_at: string;
  first_used_at: string | null;
  last_used_at: string | null;
}

export interface PasskeyStatsSummary {
  passkey_count: number;
  first_registered_at: string | null;
  first_used_at: string | null;
  last_used_at: string | null;
}

export interface PasskeyListResponse {
  credentials: PasskeyCredentialSummary[];
  stats: PasskeyStatsSummary;
}

function asRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return null;
  }

  return value as Record<string, unknown>;
}

function asArray(value: unknown): unknown[] | null {
  if (!Array.isArray(value)) {
    return null;
  }

  return value;
}

function readString(
  source: Record<string, unknown>,
  keys: string[]
): string | null {
  for (const key of keys) {
    const value = source[key];
    if (typeof value === 'string' && value.trim().length > 0) {
      return value;
    }
  }

  return null;
}

function readRecord(
  source: Record<string, unknown>,
  keys: string[]
): Record<string, unknown> | null {
  for (const key of keys) {
    const value = asRecord(source[key]);
    if (value) {
      return value;
    }
  }

  return null;
}

function readArray(
  source: Record<string, unknown>,
  keys: string[]
): unknown[] | null {
  for (const key of keys) {
    const value = asArray(source[key]);
    if (value) {
      return value;
    }
  }

  return null;
}

function parseOptionsRecord(options: unknown): Record<string, unknown> {
  if (typeof options === 'string') {
    try {
      const parsed = JSON.parse(options) as unknown;
      const parsedRecord = asRecord(parsed);
      if (parsedRecord) {
        return parsedRecord;
      }
    } catch {
      // Fall through to validation error below.
    }
  }

  const record = asRecord(options);
  if (!record) {
    throw new Error('Passkey options response was malformed.');
  }

  return record;
}

function unwrapPublicKeyOptions(
  options: unknown,
  mode: 'registration' | 'authentication'
): Record<string, unknown> {
  let current = parseOptionsRecord(options);

  for (let depth = 0; depth < 3; depth += 1) {
    const challenge = readString(current, ['challenge']);
    const hasRegistrationUserId =
      mode === 'registration'
        ? readString(readRecord(current, ['user']) ?? {}, ['id']) !== null
        : true;

    if (challenge && hasRegistrationUserId) {
      return current;
    }

    const nested = readRecord(current, ['publicKey', 'public_key']);
    if (!nested) {
      return current;
    }
    current = nested;
  }

  return current;
}

function normalizeCredentialDescriptors(list: unknown[] | null): unknown[] {
  if (!list) {
    return [];
  }

  const normalized: unknown[] = [];

  for (const entry of list) {
    const record = asRecord(entry);
    if (!record) {
      continue;
    }

    const id = readString(record, ['id', 'credentialId', 'credential_id']);
    if (!id) {
      continue;
    }

    const descriptor: Record<string, unknown> = {
      id,
      type: typeof record.type === 'string' ? record.type : 'public-key'
    };

    const transports = asArray(record.transports)?.filter(
      (value): value is string => typeof value === 'string'
    );
    if (transports && transports.length > 0) {
      descriptor.transports = transports;
    }

    normalized.push(descriptor);
  }

  return normalized;
}

function normalizeRegistrationOptions(options: unknown): Record<string, unknown> {
  const normalized = { ...unwrapPublicKeyOptions(options, 'registration') };
  const challenge = readString(normalized, ['challenge']);
  if (!challenge) {
    throw new Error('Passkey registration challenge was missing from server response.');
  }

  const user = readRecord(normalized, ['user']) ?? {};
  const userId = readString(user, ['id', 'user_id', 'userId']);
  if (!userId) {
    throw new Error('Passkey registration user id was missing from server response.');
  }

  const userName = readString(user, ['name', 'email']);
  const displayName = readString(user, ['displayName', 'display_name', 'name']);

  normalized.challenge = challenge;
  normalized.user = {
    ...user,
    id: userId,
    ...(userName ? { name: userName } : {}),
    ...(displayName ? { displayName } : {})
  };

  const excludeCredentials = normalizeCredentialDescriptors(
    readArray(normalized, ['excludeCredentials', 'exclude_credentials'])
  );
  normalized.excludeCredentials = excludeCredentials;
  delete normalized.exclude_credentials;

  return normalized;
}

function normalizeAuthenticationOptions(options: unknown): Record<string, unknown> {
  const normalized = { ...unwrapPublicKeyOptions(options, 'authentication') };
  const challenge = readString(normalized, ['challenge']);
  if (!challenge) {
    throw new Error('Passkey authentication challenge was missing from server response.');
  }

  normalized.challenge = challenge;

  const allowCredentials = normalizeCredentialDescriptors(
    readArray(normalized, ['allowCredentials', 'allow_credentials'])
  );
  if (
    Array.isArray(normalized.allowCredentials) ||
    Array.isArray(normalized.allow_credentials)
  ) {
    normalized.allowCredentials = allowCredentials;
  }
  delete normalized.allow_credentials;

  return normalized;
}

function summarizeOptions(options: Record<string, unknown>): Record<string, unknown> {
  const user = asRecord(options.user);
  const allowCredentials = asArray(options.allowCredentials);
  const excludeCredentials = asArray(options.excludeCredentials);

  return {
    keys: Object.keys(options),
    hasChallenge: typeof options.challenge === 'string',
    hasUserId: typeof user?.id === 'string',
    allowCredentialsCount: allowCredentials?.length ?? 0,
    excludeCredentialsCount: excludeCredentials?.length ?? 0
  };
}

function mapWebauthnError(error: unknown): Error {
  if (!(error instanceof Error)) {
    return new Error('Passkey ceremony failed. Please try again.');
  }

  if (error.message.includes('base64URLString.replace')) {
    return new Error(
      'Passkey options from server were malformed. Refresh and try again.'
    );
  }

  return error;
}

function parseApiError(
  payload: unknown,
  fallback: string
): string {
  if (!payload || typeof payload !== 'object') {
    return fallback;
  }

  const envelope = payload as Partial<ApiErrorEnvelope>;
  if (envelope.error && typeof envelope.error.message === 'string') {
    return envelope.error.message;
  }

  return fallback;
}

async function apiRequest<T>(
  path: string,
  init: RequestInit = {}
): Promise<T> {
  const headers = new Headers(init.headers ?? {});
  if (init.body && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json');
  }

  const response = await fetch(path, {
    ...init,
    headers,
    credentials: 'include'
  });

  const payload = await response.json().catch(() => null);

  if (!response.ok) {
    throw new Error(parseApiError(payload, `Passkey request failed (${response.status})`));
  }

  return payload as T;
}

export async function signInWithPasskey(): Promise<void> {
  logInfo('auth.passkeys', 'Starting passkey sign-in ceremony');
  const challenge = await apiRequest<ApiEnvelope<PasskeyChallengeResponse>>(
    '/api/auth/passkeys/login/start',
    {
      method: 'POST',
      body: JSON.stringify({})
    }
  );
  const optionsJSON = normalizeAuthenticationOptions(challenge.data.options);
  logDebug(
    'auth.passkeys',
    'Passkey sign-in options normalized',
    summarizeOptions(optionsJSON)
  );

  let assertion;
  try {
    assertion = await startAuthentication({
      optionsJSON: optionsJSON as unknown as Parameters<
        typeof startAuthentication
      >[0]['optionsJSON']
    });
  } catch (error) {
    const mapped = mapWebauthnError(error);
    logWarn('auth.passkeys', 'Passkey sign-in ceremony failed in browser', {
      message: mapped.message,
      options: summarizeOptions(optionsJSON)
    });
    throw mapped;
  }

  await apiRequest<ApiEnvelope<unknown>>('/api/auth/passkeys/login/finish', {
    method: 'POST',
    body: JSON.stringify({
      challenge_id: challenge.data.challenge_id,
      credential: assertion
    })
  });

  logDebug('auth.passkeys', 'Passkey sign-in ceremony completed');
}

export async function listAccountPasskeys(): Promise<PasskeyListResponse> {
  const response = await apiRequest<ApiEnvelope<PasskeyCredentialSummary[] | PasskeyListResponse>>(
    '/api/auth/passkeys',
    {
      method: 'GET'
    }
  );

  if (Array.isArray(response.data)) {
    return {
      credentials: response.data,
      stats: {
        passkey_count: response.data.length,
        first_registered_at: null,
        first_used_at: null,
        last_used_at: null
      }
    };
  }

  const payload = response.data as Partial<PasskeyListResponse>;
  return {
    credentials: Array.isArray(payload.credentials) ? payload.credentials : [],
    stats: {
      passkey_count:
        typeof payload.stats?.passkey_count === 'number'
          ? payload.stats.passkey_count
          : Array.isArray(payload.credentials)
            ? payload.credentials.length
            : 0,
      first_registered_at:
        typeof payload.stats?.first_registered_at === 'string'
          ? payload.stats.first_registered_at
          : null,
      first_used_at:
        typeof payload.stats?.first_used_at === 'string'
          ? payload.stats.first_used_at
          : null,
      last_used_at:
        typeof payload.stats?.last_used_at === 'string'
          ? payload.stats.last_used_at
          : null
    }
  };
}

export async function enrollPasskey(
  nickname = 'Reqstly Passkey'
): Promise<void> {
  const csrfToken = await ensureCsrfToken();

  logInfo('auth.passkeys', 'Starting passkey registration ceremony');
  const challenge = await apiRequest<ApiEnvelope<PasskeyChallengeResponse>>(
    '/api/auth/passkeys/register/start',
    {
      method: 'POST',
      headers: {
        'X-CSRF-Token': csrfToken
      },
      body: JSON.stringify({ nickname })
    }
  );
  const optionsJSON = normalizeRegistrationOptions(challenge.data.options);
  logDebug(
    'auth.passkeys',
    'Passkey registration options normalized',
    summarizeOptions(optionsJSON)
  );

  let credential;
  try {
    credential = await startRegistration({
      optionsJSON: optionsJSON as unknown as Parameters<
        typeof startRegistration
      >[0]['optionsJSON']
    });
  } catch (error) {
    const mapped = mapWebauthnError(error);
    logWarn('auth.passkeys', 'Passkey registration ceremony failed in browser', {
      message: mapped.message,
      options: summarizeOptions(optionsJSON)
    });
    throw mapped;
  }

  await apiRequest<ApiEnvelope<unknown>>('/api/auth/passkeys/register/finish', {
    method: 'POST',
    headers: {
      'X-CSRF-Token': csrfToken
    },
    body: JSON.stringify({
      challenge_id: challenge.data.challenge_id,
      credential
    })
  });

  logDebug('auth.passkeys', 'Passkey registration ceremony completed');
}

export async function signUpWithPasskey(
  email: string,
  displayName: string,
  nickname = 'Reqstly Passkey'
): Promise<void> {
  const normalizedEmail = email.trim().toLowerCase();
  const normalizedDisplayName = displayName.trim();

  if (normalizedEmail.length === 0 || normalizedDisplayName.length === 0) {
    throw new Error('Email and display name are required for passkey signup.');
  }

  logInfo('auth.passkeys', 'Starting passkey signup ceremony');
  const challenge = await apiRequest<ApiEnvelope<PasskeyChallengeResponse>>(
    '/api/auth/passkeys/signup/start',
    {
      method: 'POST',
      body: JSON.stringify({
        email: normalizedEmail,
        display_name: normalizedDisplayName,
        nickname
      })
    }
  );
  const optionsJSON = normalizeRegistrationOptions(challenge.data.options);
  logDebug(
    'auth.passkeys',
    'Passkey signup options normalized',
    summarizeOptions(optionsJSON)
  );

  let credential;
  try {
    credential = await startRegistration({
      optionsJSON: optionsJSON as unknown as Parameters<
        typeof startRegistration
      >[0]['optionsJSON']
    });
  } catch (error) {
    const mapped = mapWebauthnError(error);
    logWarn('auth.passkeys', 'Passkey signup ceremony failed in browser', {
      message: mapped.message,
      options: summarizeOptions(optionsJSON)
    });
    throw mapped;
  }

  await apiRequest<ApiEnvelope<unknown>>('/api/auth/passkeys/signup/finish', {
    method: 'POST',
    body: JSON.stringify({
      challenge_id: challenge.data.challenge_id,
      credential
    })
  });

  logDebug('auth.passkeys', 'Passkey signup ceremony completed');
}
