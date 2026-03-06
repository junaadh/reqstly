import { env } from '$env/dynamic/public';

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

const PUBLIC_API_BASE_URL = env.PUBLIC_API_BASE_URL ?? '';
const PUBLIC_SUPABASE_URL = env.PUBLIC_SUPABASE_URL ?? '';
const PUBLIC_SUPABASE_ANON_KEY = env.PUBLIC_SUPABASE_ANON_KEY ?? '';

export const apiBaseUrl = normalizeApiBaseUrl(PUBLIC_API_BASE_URL);
export const supabaseUrl = PUBLIC_SUPABASE_URL.trim();
export const supabaseAnonKey = PUBLIC_SUPABASE_ANON_KEY.trim();
export const hasSupabaseConfig = supabaseUrl.length > 0 && supabaseAnonKey.length > 0;
