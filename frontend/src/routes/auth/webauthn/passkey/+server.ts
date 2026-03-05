import { json } from '@sveltejs/kit';
import { generateAuthenticationOptions } from '@simplewebauthn/server';

import type { RequestHandler } from './$types';

const CHALLENGE_COOKIE = 'reqstly_webauthn_challenge';

export const POST: RequestHandler = async ({ cookies, url }) => {
  const options = await generateAuthenticationOptions({
    rpID: url.hostname,
    userVerification: 'preferred',
    allowCredentials: []
  });

  cookies.set(CHALLENGE_COOKIE, options.challenge, {
    path: '/',
    httpOnly: true,
    sameSite: 'lax',
    secure: url.protocol === 'https:',
    maxAge: 300
  });

  return json(options);
};
