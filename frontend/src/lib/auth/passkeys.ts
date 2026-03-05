import { startAuthentication, startRegistration } from '@simplewebauthn/browser';
import type { Session, SupabaseClient } from '@supabase/supabase-js';

import { debugErrorDetails, logDebug, logError, logInfo, logWarn } from '$lib/debug';
import { setAccessTokenCookie } from '$lib/auth/session';
import { supabaseAnonKey, supabaseUrl } from '$lib/config';

interface ErrorPayload {
  msg?: string;
  message?: string;
  error?: string;
  error_description?: string;
  code?: string;
}

interface ChallengePayload {
  id?: string;
  type?: string;
  webauthn?: {
    type?: 'create' | 'request';
    credential_options?: {
      publicKey?: Record<string, unknown>;
    };
  };
}

interface TokenPayload {
  access_token?: string;
  refresh_token?: string;
  session?: {
    access_token?: string;
    refresh_token?: string;
  };
}

export interface PasskeyFactor {
  id: string;
  factor_type: string;
  status: string;
  friendly_name?: string | null;
  created_at?: string;
  updated_at?: string;
}

interface MfaListFactorsPayload {
  all?: PasskeyFactor[];
  webauthn?: PasskeyFactor[];
}

function authBaseUrl(): string {
  return `${supabaseUrl.replace(/\/+$/, '')}/auth/v1`;
}

function ensureSupabasePublicConfig(): void {
  if (supabaseUrl.trim().length === 0 || supabaseAnonKey.trim().length === 0) {
    throw new Error('Supabase public config is missing.');
  }
}

function toErrorMessage(payload: unknown, fallback: string): string {
  if (!payload || typeof payload !== 'object') return fallback;
  const data = payload as ErrorPayload;
  return data.msg ?? data.message ?? data.error_description ?? data.error ?? fallback;
}

async function authRequest<T>(
  path: string,
  init: RequestInit = {},
  accessToken?: string
): Promise<T> {
  logDebug('auth.passkeys', 'Calling Supabase auth endpoint', {
    path,
    method: init.method ?? 'GET',
    hasAccessToken: Boolean(accessToken)
  });
  ensureSupabasePublicConfig();

  const headers = new Headers(init.headers ?? {});
  headers.set('Content-Type', 'application/json');
  headers.set('apikey', supabaseAnonKey);

  if (accessToken) {
    headers.set('Authorization', `Bearer ${accessToken}`);
  }

  const response = await fetch(`${authBaseUrl()}${path}`, {
    ...init,
    headers
  });

  let payload: unknown = null;
  try {
    payload = await response.json();
  } catch {
    payload = null;
  }

  if (!response.ok) {
    logWarn('auth.passkeys', 'Supabase auth endpoint failed', {
      path,
      status: response.status,
      payload
    });
    throw new Error(toErrorMessage(payload, `Auth request failed (${response.status})`));
  }

  return payload as T;
}

async function appRequest<T>(path: string, init: RequestInit = {}): Promise<T> {
  logDebug('auth.passkeys', 'Calling app passkey endpoint', {
    path,
    method: init.method ?? 'GET'
  });
  const headers = new Headers(init.headers ?? {});
  headers.set('Content-Type', 'application/json');

  const response = await fetch(path, {
    ...init,
    headers
  });

  let payload: unknown = null;
  try {
    payload = await response.json();
  } catch {
    payload = null;
  }

  if (!response.ok) {
    logInfo('auth.passkeys', 'App passkey endpoint failed', {
      path,
      status: response.status,
      payload
    });
    throw new Error(toErrorMessage(payload, `Passkey request failed (${response.status})`));
  }

  return payload as T;
}

function extractTokens(payload: unknown): { accessToken: string; refreshToken: string } | null {
  if (!payload || typeof payload !== 'object') return null;
  const tokenPayload = payload as TokenPayload;

  const accessToken = tokenPayload.access_token ?? tokenPayload.session?.access_token;
  const refreshToken = tokenPayload.refresh_token ?? tokenPayload.session?.refresh_token;

  if (!accessToken || !refreshToken) return null;

  return { accessToken, refreshToken };
}

async function applySupabaseSession(
  client: SupabaseClient,
  tokens: { accessToken: string; refreshToken: string }
): Promise<Session> {
  const { data, error } = await client.auth.setSession({
    access_token: tokens.accessToken,
    refresh_token: tokens.refreshToken
  });

  if (error || !data.session) {
    throw new Error(error?.message ?? 'Unable to store Supabase session.');
  }

  setAccessTokenCookie(data.session.access_token);
  return data.session;
}

function getRpContext(): { rpId: string; rpOrigins: string[] } {
  return {
    rpId: window.location.hostname,
    rpOrigins: [window.location.origin]
  };
}

export async function signInWithPasskey(client: SupabaseClient): Promise<string> {
  logInfo('auth.passkeys', 'Starting passkey sign-in');
  const options = await appRequest<Record<string, unknown>>('/auth/webauthn/passkey', {
    method: 'POST',
    body: JSON.stringify({})
  });
  logDebug('auth.passkeys', 'Received passkey challenge', {
    hasChallenge: typeof options.challenge === 'string',
    rpId: typeof options.rpId === 'string' ? options.rpId : null
  });

  const authenticationResponse = await startAuthentication({
    optionsJSON: options as unknown as Parameters<typeof startAuthentication>[0]['optionsJSON']
  });
  logDebug('auth.passkeys', 'Passkey assertion produced', {
    hasId: typeof authenticationResponse.id === 'string',
    responseKeys:
      typeof authenticationResponse.response === 'object' && authenticationResponse.response !== null
        ? Object.keys(authenticationResponse.response)
        : []
  });

  const verification = await appRequest<Record<string, unknown>>('/auth/webauthn/passkey/verify', {
    method: 'POST',
    body: JSON.stringify(authenticationResponse)
  });

  const tokens = extractTokens(verification);
  if (!tokens) {
    logError('auth.passkeys', 'Passkey verify response did not include tokens', verification);
    throw new Error('Passkey verification succeeded but no session tokens were returned.');
  }

  try {
    const session = await applySupabaseSession(client, tokens);
    logInfo('auth.passkeys', 'Passkey sign-in completed with Supabase session');
    return session.access_token;
  } catch (error) {
    // Fallback for deployments that don't accept custom refresh tokens.
    logInfo('auth.passkeys', 'Supabase setSession failed; falling back to cookie token', {
      error: debugErrorDetails(error)
    });
    setAccessTokenCookie(tokens.accessToken);
    return tokens.accessToken;
  }
}

export async function enrollPasskeyFactor(
  client: SupabaseClient,
  friendlyName = 'Reqstly Passkey'
): Promise<Session> {
  logInfo('auth.passkeys', 'Starting passkey enrollment', { friendlyName });
  const {
    data: { session }
  } = await client.auth.getSession();

  const accessToken = session?.access_token;
  if (!accessToken) {
    throw new Error('You must be signed in before adding a passkey.');
  }

  const enroll = await authRequest<{ id?: string }>(
    '/factors',
    {
      method: 'POST',
      body: JSON.stringify({
        factor_type: 'webauthn',
        friendly_name: friendlyName
      })
    },
    accessToken
  );

  if (!enroll.id) {
    logError('auth.passkeys', 'Passkey enrollment did not return factor id', enroll);
    throw new Error('Passkey enrollment failed to start.');
  }

  const rp = getRpContext();

  const challenge = await authRequest<ChallengePayload>(
    `/factors/${enroll.id}/challenge`,
    {
      method: 'POST',
      body: JSON.stringify({ webauthn: rp })
    },
    accessToken
  );

  const challengeId = challenge.id;
  const ceremonyType = challenge.webauthn?.type;
  const registrationOptions = challenge.webauthn?.credential_options?.publicKey;

  if (!challengeId || ceremonyType !== 'create' || !registrationOptions) {
    logError('auth.passkeys', 'Supabase challenge payload invalid', challenge);
    throw new Error('Supabase returned invalid WebAuthn registration options.');
  }

  const credentialResponse = await startRegistration({
    optionsJSON:
      registrationOptions as unknown as Parameters<typeof startRegistration>[0]['optionsJSON']
  });

  const verification = await authRequest<Record<string, unknown>>(
    `/factors/${enroll.id}/verify`,
    {
      method: 'POST',
      body: JSON.stringify({
        challenge_id: challengeId,
        webauthn: {
          ...rp,
          type: ceremonyType,
          credential_response: credentialResponse
        }
      })
    },
    accessToken
  );

  const tokens = extractTokens(verification);
  if (!tokens) {
    logError('auth.passkeys', 'Passkey enrollment verify did not return tokens', verification);
    throw new Error('Passkey was created but updated session tokens were not returned.');
  }

  logInfo('auth.passkeys', 'Passkey enrollment verified; applying session');
  return applySupabaseSession(client, tokens);
}

export async function listPasskeyFactors(client: SupabaseClient): Promise<PasskeyFactor[]> {
  logDebug('auth.passkeys', 'Listing passkey factors');
  const { data, error } = (await client.auth.mfa.listFactors()) as {
    data: MfaListFactorsPayload | null;
    error: Error | null;
  };

  if (error) {
    logWarn('auth.passkeys', 'Failed to list passkey factors', {
      message: error.message
    });
    throw error;
  }

  const all = Array.isArray(data?.all) ? data.all : [];
  return all.filter((factor) => factor.factor_type === 'webauthn');
}
