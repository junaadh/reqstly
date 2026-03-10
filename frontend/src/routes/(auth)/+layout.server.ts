import { redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';
import { callBackend, withSessionCookie } from '$lib/server/backend';

function hasSessionCookie(cookieHeader: string | null): boolean {
  if (!cookieHeader) {
    return false;
  }

  return cookieHeader
    .split(';')
    .map((part) => part.trim())
    .some((part) => part.startsWith('reqstly_session='));
}

export const load: LayoutServerLoad = async ({ fetch, request }) => {
  const cookieHeader = request.headers.get('cookie');
  if (!hasSessionCookie(cookieHeader)) {
    return {};
  }

  const response = await callBackend(
    fetch,
    '/me',
    withSessionCookie(cookieHeader)
  );

  if (response.ok) {
    throw redirect(303, '/');
  }

  return {};
};
