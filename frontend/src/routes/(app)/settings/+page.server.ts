import type { PageServerLoad } from './$types';

import { callBackend, withSessionCookie } from '$lib/server/backend';
import type { ApiEnvelope, UserPreferences } from '$lib/types';

const fallbackPreferences: UserPreferences = {
  email_digest: true,
  browser_alerts: true,
  default_page_size: 20
};

export const load: PageServerLoad = async ({ fetch, parent, request }) => {
  await parent();
  const response = await callBackend(
    fetch,
    '/preferences',
    withSessionCookie(request.headers.get('cookie'))
  );

  if (!response.ok || !response.json || typeof response.json !== 'object') {
    return {
      preferences: fallbackPreferences,
      backendError: `Failed to load preferences (${response.status})`
    };
  }

  const payload = response.json as ApiEnvelope<UserPreferences>;
  return {
    preferences: payload.data,
    backendError: null
  };
};
