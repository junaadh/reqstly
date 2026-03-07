const LEGACY_ACCESS_TOKEN_COOKIE = 'reqstly_access_token';
const CSRF_COOKIE = 'reqstly_csrf';

function clearLegacyAccessTokenCookie(): void {
  const variants = [
    'Path=/; SameSite=Lax',
    'Path=/; SameSite=Lax; Secure',
    'Path=/; SameSite=Lax; Domain=localhost',
    'Path=/; SameSite=Lax; Domain=localhost; Secure'
  ];

  for (const suffix of variants) {
    document.cookie = `${LEGACY_ACCESS_TOKEN_COOKIE}=; Max-Age=0; ${suffix}`;
  }
}

function clearCsrfCookie(): void {
  const variants = ['Path=/; SameSite=Lax', 'Path=/; SameSite=Lax; Secure'];
  for (const suffix of variants) {
    document.cookie = `${CSRF_COOKIE}=; Max-Age=0; ${suffix}`;
  }
}

function clearStorageAuthKeys(storage: Storage): void {
  const keysToClear: string[] = [];
  for (let index = 0; index < storage.length; index += 1) {
    const key = storage.key(index);
    if (!key) continue;

    const isSupabaseAuthKey =
      key === 'supabase.auth.token' ||
      (key.startsWith('sb-') && key.includes('auth-token'));
    const isLegacyReqstlyAuthKey = key === LEGACY_ACCESS_TOKEN_COOKIE;

    if (isSupabaseAuthKey || isLegacyReqstlyAuthKey) {
      keysToClear.push(key);
    }
  }

  for (const key of keysToClear) {
    storage.removeItem(key);
  }
}

export function clearClientAuthState(): void {
  if (typeof document !== 'undefined') {
    clearLegacyAccessTokenCookie();
    clearCsrfCookie();
  }

  if (typeof window === 'undefined') return;
  clearStorageAuthKeys(window.localStorage);
  clearStorageAuthKeys(window.sessionStorage);
}
