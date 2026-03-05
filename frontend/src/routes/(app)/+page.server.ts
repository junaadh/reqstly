import type { PageServerLoad } from './$types';

import { callBackend } from '$lib/server/backend';
import type { ApiListEnvelope, SupportRequest } from '$lib/types';

async function fetchStatusTotal(
  fetchFn: typeof fetch,
  token: string,
  status: 'open' | 'in_progress' | 'resolved'
): Promise<number> {
  const response = await callBackend(fetchFn, token, `/requests?status=${status}&page=1&limit=1`);

  if (!response.ok || !response.json || typeof response.json !== 'object') {
    return 0;
  }

  const payload = response.json as ApiListEnvelope<SupportRequest>;
  return payload.meta?.total ?? 0;
}

export const load: PageServerLoad = async ({ fetch, parent }) => {
  const { token } = await parent();

  const [openTotal, inProgressTotal, resolvedTotal, recentResponse] = await Promise.all([
    fetchStatusTotal(fetch, token, 'open'),
    fetchStatusTotal(fetch, token, 'in_progress'),
    fetchStatusTotal(fetch, token, 'resolved'),
    callBackend(fetch, token, '/requests?page=1&limit=6&sort=-updated_at')
  ]);

  const recentRequests =
    recentResponse.ok && recentResponse.json && typeof recentResponse.json === 'object'
      ? (recentResponse.json as ApiListEnvelope<SupportRequest>).data
      : [];

  return {
    stats: {
      open: openTotal,
      in_progress: inProgressTotal,
      resolved: resolvedTotal
    },
    recentRequests
  };
};
