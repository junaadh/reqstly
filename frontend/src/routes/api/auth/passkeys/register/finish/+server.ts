import type { RequestHandler } from './$types';

import { proxyAuthRequest } from '$lib/server/auth-proxy';

export const POST: RequestHandler = async ({ fetch, request }) =>
  proxyAuthRequest(fetch, request, '/auth/passkeys/register/finish');
