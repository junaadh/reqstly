import { redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';
import { getAccessTokenFromCookies } from '$lib/server/auth-cookie';

export const load: LayoutServerLoad = async ({ cookies }) => {
  const token = getAccessTokenFromCookies(cookies);
  if (token) {
    throw redirect(303, '/');
  }

  return {};
};
