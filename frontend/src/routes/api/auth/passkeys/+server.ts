import type { RequestHandler } from './$types';

import { proxyAuthRequest } from '$lib/server/auth-proxy';

export const GET: RequestHandler = async ({ fetch, request }) =>
  proxyAuthRequest(fetch, request, '/auth/passkeys');
