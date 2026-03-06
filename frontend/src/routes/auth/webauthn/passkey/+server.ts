import { json } from '@sveltejs/kit';
import { generateAuthenticationOptions } from '@simplewebauthn/server';

import { isSecureOrigin, resolveRequestHostname, resolveRequestOrigin } from '$lib/server/request-origin';

import type { RequestHandler } from './$types';

const CHALLENGE_COOKIE = 'reqstly_webauthn_challenge';

export const POST: RequestHandler = async ({ request, cookies, url }) => {
  const requestOrigin = resolveRequestOrigin(request, url.origin);
  const rpID = resolveRequestHostname(request, url.origin);

  const options = await generateAuthenticationOptions({
    rpID,
    userVerification: 'preferred',
    allowCredentials: []
  });

  cookies.set(CHALLENGE_COOKIE, options.challenge, {
    path: '/',
    httpOnly: true,
    sameSite: 'lax',
    secure: isSecureOrigin(requestOrigin),
    maxAge: 300
  });

  return json(options);
};
