import { redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';

import { ACCESS_TOKEN_COOKIE } from '$lib/auth/session';
import { callBackend } from '$lib/server/backend';
import type { ApiEnvelope, MeProfile } from '$lib/types';

export const load: LayoutServerLoad = async ({ cookies, fetch, url }) => {
  const token = cookies.get(ACCESS_TOKEN_COOKIE);

  if (!token) {
    const next = encodeURIComponent(`${url.pathname}${url.search}`);
    throw redirect(303, `/login?next=${next}`);
  }

  const meResult = await callBackend(fetch, token, '/me');

  if (!meResult.ok || !meResult.json || typeof meResult.json !== 'object') {
    cookies.delete(ACCESS_TOKEN_COOKIE, { path: '/' });
    throw redirect(303, '/login?reason=session-expired');
  }

  const me = (meResult.json as ApiEnvelope<MeProfile>).data;

  return {
    me,
    token
  };
};
