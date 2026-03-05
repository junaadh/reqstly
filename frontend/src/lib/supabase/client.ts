import { createClient, type SupabaseClient } from '@supabase/supabase-js';

import { hasSupabaseConfig, supabaseAnonKey, supabaseUrl } from '$lib/config';

let singleton: SupabaseClient | null = null;

export function getSupabaseClient(): SupabaseClient | null {
  if (!hasSupabaseConfig) {
    return null;
  }

  if (singleton) {
    return singleton;
  }

  singleton = createClient(supabaseUrl, supabaseAnonKey, {
    auth: {
      persistSession: true,
      autoRefreshToken: true,
      detectSessionInUrl: true
    }
  });

  return singleton;
}
