import type { PageServerLoad } from './$types';

import { callBackend, withSessionCookie } from '$lib/server/backend';
import type { ApiListEnvelope, SupportRequest } from '$lib/types';

async function fetchStatusTotal(
  fetchFn: typeof fetch,
  cookieHeader: string | null,
  status: 'open' | 'in_progress' | 'resolved'
): Promise<number> {
  const response = await callBackend(
    fetchFn,
    `/requests?status=${status}&page=1&limit=1`,
    withSessionCookie(cookieHeader)
  );

  if (!response.ok || !response.json || typeof response.json !== 'object') {
    return 0;
  }

  const payload = response.json as ApiListEnvelope<SupportRequest>;
  return payload.meta?.total ?? 0;
}

export const load: PageServerLoad = async ({ fetch, parent, depends, request }) => {
  depends('reqstly:dashboard');

  await parent();
  const cookieHeader = request.headers.get('cookie');

  const [openTotal, inProgressTotal, resolvedTotal, recentResponse] = await Promise.all([
    fetchStatusTotal(fetch, cookieHeader, 'open'),
    fetchStatusTotal(fetch, cookieHeader, 'in_progress'),
    fetchStatusTotal(fetch, cookieHeader, 'resolved'),
    callBackend(
      fetch,
      '/requests?page=1&limit=6&sort=-updated_at',
      withSessionCookie(cookieHeader)
    )
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
