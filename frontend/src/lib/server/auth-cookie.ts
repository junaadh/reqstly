import type { Cookies } from '@sveltejs/kit';

import { ACCESS_TOKEN_COOKIE } from '$lib/auth/session';

export function getAccessTokenFromCookies(cookies: Cookies): string | null {
  const candidates = cookies
    .getAll()
    .filter((cookie) => cookie.name === ACCESS_TOKEN_COOKIE)
    .map((cookie) => cookie.value.trim())
    .filter((value) => value.length > 0);

  if (candidates.length > 0) {
    return candidates[candidates.length - 1];
  }

  const fallback = cookies.get(ACCESS_TOKEN_COOKIE)?.trim();
  return fallback && fallback.length > 0 ? fallback : null;
}

