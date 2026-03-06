import { json, type RequestHandler } from '@sveltejs/kit';

import type { ApiEnvelope, MeProfile } from '$lib/types';
import { callBackend } from '$lib/server/backend';
import { getAccessTokenFromCookies } from '$lib/server/auth-cookie';
import { getAdminUser } from '$lib/server/passkey-auth';

interface PasskeyFactorResponse {
  id: string;
  factor_type: string;
  status: string;
  friendly_name?: string | null;
  created_at?: string;
  updated_at?: string;
}

export const GET: RequestHandler = async ({ cookies, fetch }) => {
  const token = getAccessTokenFromCookies(cookies);
  if (!token) {
    return json({ message: 'Session expired.' }, { status: 401 });
  }

  const meResult = await callBackend(fetch, token, '/me');
  if (!meResult.ok || !meResult.json || typeof meResult.json !== 'object') {
    return json({ message: 'Unable to resolve active user.' }, { status: 401 });
  }

  const me = (meResult.json as ApiEnvelope<MeProfile>).data;
  if (!me || typeof me.id !== 'string' || me.id.trim().length === 0) {
    return json({ message: 'Invalid user context.' }, { status: 500 });
  }

  const user = await getAdminUser(me.id);
  const factors = (user?.factors ?? [])
    .filter((factor) => factor.factor_type === 'webauthn')
    .map<PasskeyFactorResponse>((factor) => ({
      id: factor.id,
      factor_type: factor.factor_type ?? 'webauthn',
      status: factor.status ?? 'unverified',
      friendly_name: factor.friendly_name ?? null,
      created_at: factor.created_at,
      updated_at: factor.updated_at
    }));

  return json({ factors }, { status: 200 });
};
