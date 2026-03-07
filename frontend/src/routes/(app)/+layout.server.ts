import { redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';

import { callBackend, withSessionCookie } from '$lib/server/backend';
import type { ApiEnvelope, MeProfile } from '$lib/types';

export const load: LayoutServerLoad = async ({ fetch, request, url }) => {
  const meResult = await callBackend(
    fetch,
    '/me',
    withSessionCookie(request.headers.get('cookie'))
  );

  if (!meResult.ok || !meResult.json || typeof meResult.json !== 'object') {
    const next = encodeURIComponent(`${url.pathname}${url.search}`);
    throw redirect(303, `/login?next=${next}`);
  }

  const me = (meResult.json as ApiEnvelope<MeProfile>).data;

  return {
    me
  };
};
