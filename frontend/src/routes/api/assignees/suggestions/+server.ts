import { json, type RequestHandler } from '@sveltejs/kit';

import { callBackend, withSessionCookie } from '$lib/server/backend';

export const GET: RequestHandler = async ({ fetch, request, url }) => {
  const q = url.searchParams.get('q')?.trim() ?? '';
  const requestedLimit = Number(url.searchParams.get('limit') ?? '25');
  const limit = Number.isFinite(requestedLimit)
    ? Math.min(200, Math.max(1, requestedLimit))
    : 25;

  const params = new URLSearchParams();
  params.set('limit', String(limit));
  if (q.length > 0) {
    params.set('q', q);
  }

  const result = await callBackend(
    fetch,
    `/assignees/suggestions?${params.toString()}`,
    withSessionCookie(request.headers.get('cookie'))
  );

  if (result.json && typeof result.json === 'object') {
    return json(result.json, { status: result.status });
  }

  return json(
    {
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Unexpected assignee lookup response.'
      }
    },
    { status: 500 }
  );
};
