import type { ApiErrorEnvelope } from '$lib/types';
import { env as privateEnv } from '$env/dynamic/private';
import { env as publicEnv } from '$env/dynamic/public';

export interface BackendCallResult {
  ok: boolean;
  status: number;
  json: unknown;
  headers: Headers;
}

function readCookieValue(
  cookieHeader: string | null,
  name: string
): string | null {
  const raw = cookieHeader?.trim();
  if (!raw) {
    return null;
  }

  const parts = raw.split(';');
  for (const part of parts) {
    const [key, ...rest] = part.trim().split('=');
    if (key !== name) {
      continue;
    }

    const value = rest.join('=').trim();
    if (value.length === 0) {
      return null;
    }

    try {
      return decodeURIComponent(value);
    } catch {
      return value;
    }
  }

  return null;
}

function normalizeApiBaseUrl(input: string): string {
  const trimmed = input.trim();
  const fallback = 'https://api.localhost/api/v1';
  const base = trimmed.length > 0 ? trimmed : fallback;
  const withoutTrailing = base.replace(/\/+$/, '');

  if (withoutTrailing.endsWith('/api/v1')) {
    return withoutTrailing;
  }

  return `${withoutTrailing}/api/v1`;
}

function resolveApiBaseUrl(): string {
  const privateUrl = privateEnv.PRIVATE_API_BASE_URL ?? '';
  const publicUrl = publicEnv.PUBLIC_API_BASE_URL ?? '';

  if (privateUrl.trim().length > 0) {
    return normalizeApiBaseUrl(privateUrl);
  }

  return normalizeApiBaseUrl(publicUrl);
}

export async function callBackend(
  fetchFn: typeof fetch,
  path: string,
  init: RequestInit = {}
): Promise<BackendCallResult> {
  const apiBaseUrl = resolveApiBaseUrl();
  const url = `${apiBaseUrl}${path}`;

  const headers = new Headers(init.headers ?? {});
  if (init.body && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json');
  }

  let response: Response;
  try {
    response = await fetchFn(url, {
      ...init,
      headers
    });
  } catch (cause) {
    return {
      ok: false,
      status: 503,
      headers: new Headers(),
      json: {
        error: {
          code: 'BACKEND_UNREACHABLE',
          message: 'Reqstly backend is unreachable from frontend server runtime.',
          details: cause instanceof Error ? cause.message : String(cause)
        }
      }
    };
  }

  const text = await response.text();
  let json: unknown = null;

  if (text.length > 0) {
    try {
      json = JSON.parse(text);
    } catch {
      json = text;
    }
  }

  return {
    ok: response.ok,
    status: response.status,
    headers: response.headers,
    json
  };
}

export function asApiError(input: unknown): ApiErrorEnvelope | null {
  if (!input || typeof input !== 'object') return null;
  const maybe = input as Partial<ApiErrorEnvelope>;
  if (!maybe.error || typeof maybe.error !== 'object') return null;
  if (!maybe.meta || typeof maybe.meta !== 'object') return null;
  return maybe as ApiErrorEnvelope;
}

export function withSessionCookie(
  cookieHeader: string | null,
  init: RequestInit = {}
): RequestInit {
  const headers = new Headers(init.headers ?? {});
  const trimmed = cookieHeader?.trim();
  if (trimmed && trimmed.length > 0) {
    headers.set('Cookie', trimmed);
  }

  const method = (init.method ?? 'GET').toUpperCase();
  const isMutatingMethod =
    method === 'POST' || method === 'PUT' || method === 'PATCH' || method === 'DELETE';

  if (isMutatingMethod && !headers.has('X-CSRF-Token')) {
    const csrfToken = readCookieValue(cookieHeader, 'reqstly_csrf');
    if (csrfToken && csrfToken.length > 0) {
      headers.set('X-CSRF-Token', csrfToken);
    }
  }

  if (isMutatingMethod && !headers.has('Origin')) {
    const configuredOrigin = (privateEnv.ORIGIN ?? '').trim();
    if (configuredOrigin.length > 0) {
      headers.set('Origin', configuredOrigin);
    }
  }

  return {
    ...init,
    headers
  };
}

export function appendSetCookieHeaders(
  source: Headers,
  target: Headers
): void {
  const maybeGetSetCookie = (source as unknown as { getSetCookie?: () => string[] })
    .getSetCookie;

  if (typeof maybeGetSetCookie === 'function') {
    for (const value of maybeGetSetCookie.call(source)) {
      target.append('set-cookie', value);
    }
    return;
  }

  const raw = source.get('set-cookie');
  if (raw && raw.length > 0) {
    target.append('set-cookie', raw);
  }
}
