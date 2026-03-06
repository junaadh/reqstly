import type { ApiErrorEnvelope } from '$lib/types';
import { env as privateEnv } from '$env/dynamic/private';
import { env as publicEnv } from '$env/dynamic/public';

export interface BackendCallResult {
  ok: boolean;
  status: number;
  json: unknown;
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
  token: string,
  path: string,
  init: RequestInit = {}
): Promise<BackendCallResult> {
  const apiBaseUrl = resolveApiBaseUrl();
  const url = `${apiBaseUrl}${path}`;

  let response: Response;
  try {
    response = await fetchFn(url, {
      ...init,
      headers: {
        Authorization: `Bearer ${token}`,
        'Content-Type': 'application/json',
        ...(init.headers ?? {})
      }
    });
  } catch (cause) {
    return {
      ok: false,
      status: 503,
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
