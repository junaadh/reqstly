import type { ApiEnvelope } from '$lib/types';

const CSRF_COOKIE_NAME = 'reqstly_csrf';

interface CsrfTokenPayload {
  token: string;
  expires_at: string;
}

function readCookie(name: string): string | null {
  if (typeof document === 'undefined') {
    return null;
  }

  const segments = document.cookie.split(';');
  for (const segment of segments) {
    const [key, ...rest] = segment.trim().split('=');
    if (key !== name) {
      continue;
    }

    return decodeURIComponent(rest.join('='));
  }

  return null;
}

function writeCookie(name: string, value: string): void {
  if (typeof document === 'undefined') {
    return;
  }

  const secure = window.location.protocol === 'https:' ? '; Secure' : '';
  document.cookie = `${name}=${encodeURIComponent(value)}; Path=/; SameSite=Lax${secure}`;
}

function parseCsrfToken(payload: unknown): string | null {
  if (!payload || typeof payload !== 'object') {
    return null;
  }

  const envelope = payload as Partial<ApiEnvelope<CsrfTokenPayload>>;
  const token = envelope.data?.token;
  if (typeof token !== 'string' || token.trim().length === 0) {
    return null;
  }

  return token;
}

export async function ensureCsrfToken(): Promise<string> {
  const existing = readCookie(CSRF_COOKIE_NAME);
  if (existing && existing.length > 0) {
    return existing;
  }

  const response = await fetch('/api/auth/csrf', {
    method: 'GET',
    credentials: 'include'
  });

  const payload = await response.json().catch(() => null);
  if (!response.ok) {
    throw new Error(`Unable to establish CSRF token (${response.status})`);
  }

  const token = parseCsrfToken(payload);
  if (!token) {
    throw new Error('Invalid CSRF response from server');
  }

  writeCookie(CSRF_COOKIE_NAME, token);
  return token;
}

export function clearCsrfToken(): void {
  if (typeof document === 'undefined') {
    return;
  }

  const secure = window.location.protocol === 'https:' ? '; Secure' : '';
  document.cookie = `${CSRF_COOKIE_NAME}=; Max-Age=0; Path=/; SameSite=Lax${secure}`;
}
