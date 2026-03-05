import { redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';
import { ACCESS_TOKEN_COOKIE } from '$lib/auth/session';

export const load: LayoutServerLoad = async ({ cookies }) => {
  const token = cookies.get(ACCESS_TOKEN_COOKIE);
  if (token) {
    throw redirect(303, '/');
  }

  return {};
};
