export const ACCESS_TOKEN_COOKIE = 'reqstly_access_token';

export function setAccessTokenCookie(token: string): void {
  if (typeof document === 'undefined') return;
  document.cookie = `${ACCESS_TOKEN_COOKIE}=${encodeURIComponent(token)}; Max-Age=3600; Path=/; SameSite=Lax`;
}

export function clearAccessTokenCookie(): void {
  if (typeof document === 'undefined') return;
  document.cookie = `${ACCESS_TOKEN_COOKIE}=; Max-Age=0; Path=/; SameSite=Lax`;
  document.cookie = `${ACCESS_TOKEN_COOKIE}=; Max-Age=0; Path=/; SameSite=Lax; Secure`;
}

function clearStorageAuthKeys(storage: Storage): void {
  const keysToClear: string[] = [];
  for (let index = 0; index < storage.length; index += 1) {
    const key = storage.key(index);
    if (!key) continue;

    const isSupabaseAuthKey =
      key === 'supabase.auth.token' || (key.startsWith('sb-') && key.includes('auth-token'));
    const isReqstlyAuthKey = key === ACCESS_TOKEN_COOKIE;

    if (isSupabaseAuthKey || isReqstlyAuthKey) {
      keysToClear.push(key);
    }
  }

  for (const key of keysToClear) {
    storage.removeItem(key);
  }
}

export function clearClientAuthState(): void {
  clearAccessTokenCookie();
  if (typeof window === 'undefined') return;
  clearStorageAuthKeys(window.localStorage);
  clearStorageAuthKeys(window.sessionStorage);
}

export function readAccessTokenCookie(): string | null {
  if (typeof document === 'undefined') return null;
  const entry = document.cookie
    .split(';')
    .map((value) => value.trim())
    .find((value) => value.startsWith(`${ACCESS_TOKEN_COOKIE}=`));

  if (!entry) return null;
  const token = entry.slice(ACCESS_TOKEN_COOKIE.length + 1);
  return token.length > 0 ? decodeURIComponent(token) : null;
}
