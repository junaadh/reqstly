import type { PageServerLoad } from './$types';

import { callBackend } from '$lib/server/backend';
import type { ApiEnvelope, UserPreferences } from '$lib/types';

const fallbackPreferences: UserPreferences = {
  email_digest: true,
  browser_alerts: true,
  default_page_size: 20
};

export const load: PageServerLoad = async ({ fetch, parent }) => {
  const { token } = await parent();
  const response = await callBackend(fetch, token, '/preferences');

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
