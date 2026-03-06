import { json } from '@sveltejs/kit';
import { verifyAuthenticationResponse } from '@simplewebauthn/server';

import { logInfo, logWarn } from '$lib/debug';
import {
  extractCredentialFromFactor,
  mintPasskeyTokens,
  resolveUserForCredential
} from '$lib/server/passkey-auth';
import { resolveRequestOrigin } from '$lib/server/request-origin';

import type { RequestHandler } from './$types';

const CHALLENGE_COOKIE = 'reqstly_webauthn_challenge';

export const POST: RequestHandler = async ({ request, cookies, url }) => {
  try {
    logInfo('auth.passkeys.verify', 'Passkey verify request received');
    const expectedChallenge = cookies.get(CHALLENGE_COOKIE);
    cookies.delete(CHALLENGE_COOKIE, { path: '/' });

    if (!expectedChallenge) {
      logWarn('auth.passkeys.verify', 'Missing passkey challenge cookie');
      return json(
        { message: 'Passkey challenge is missing or expired. Please try again.' },
        { status: 400 }
      );
    }

    const payload = await request.json().catch(() => null);
    const credentialId = payload && typeof payload.id === 'string' ? payload.id : '';

    if (credentialId.length === 0) {
      logWarn('auth.passkeys.verify', 'Invalid assertion payload (missing id)');
      return json({ message: 'Invalid passkey assertion payload.' }, { status: 400 });
    }

    const userHandle =
      payload &&
      typeof payload === 'object' &&
      payload !== null &&
      typeof (payload as { response?: { userHandle?: unknown } }).response?.userHandle === 'string'
        ? (payload as { response: { userHandle: string } }).response.userHandle
        : undefined;

    const user = await resolveUserForCredential(credentialId, userHandle);
    if (!user) {
      logWarn('auth.passkeys.verify', 'No user resolved for passkey credential', {
        hasUserHandle: Boolean(userHandle)
      });
      return json({ message: 'No matching passkey account found.' }, { status: 404 });
    }

    const credentials = (user.factors ?? [])
      .filter((factor) => factor.factor_type === 'webauthn' && factor.status === 'verified')
      .map((factor) => extractCredentialFromFactor(factor))
      .filter((credential): credential is NonNullable<typeof credential> => credential !== null);

    if (credentials.length === 0) {
      logWarn('auth.passkeys.verify', 'Resolved user has no verified passkey factors', {
        userId: user.id
      });
      return json({ message: 'No verified passkey factor found for this account.' }, { status: 404 });
    }

    const expectedOrigin = resolveRequestOrigin(request, url.origin);
    const expectedRPID = new URL(expectedOrigin).hostname;
    for (const credential of credentials) {
      try {
        const verification = await verifyAuthenticationResponse({
          response: payload,
          expectedChallenge,
          expectedOrigin,
          expectedRPID,
          credential: {
            id: credential.id,
            publicKey: new Uint8Array(credential.publicKey),
            counter: credential.counter,
            transports: credential.transports as
              | Parameters<typeof verifyAuthenticationResponse>[0]['credential']['transports']
              | undefined
          },
          requireUserVerification: false
        });

        if (verification.verified) {
          logInfo('auth.passkeys.verify', 'Passkey verification succeeded', {
            userId: user.id
          });
          return json(mintPasskeyTokens(user));
        }
      } catch {
        // Try the next verified factor.
      }
    }

    logWarn('auth.passkeys.verify', 'Passkey verification failed for all candidate factors', {
      userId: user.id,
      candidateCount: credentials.length
    });
    return json(
      {
        message:
          'Passkey verification failed. The selected passkey may not be enrolled for this account on this deployment.'
      },
      { status: 401 }
    );
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Passkey verification failed.';
    return json({ message }, { status: 400 });
  }
};
