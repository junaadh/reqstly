import type { PageServerLoad } from './$types';

import { callBackend } from '$lib/server/backend';
import type { ApiEnvelope, ApiListEnvelope, RequestEnums, SupportRequest } from '$lib/types';

const allowedSort = new Set(['created_at', 'updated_at', '-updated_at']);
const allowedStatus = new Set(['open', 'in_progress', 'resolved']);
const allowedCategory = new Set(['IT', 'Ops', 'Admin', 'HR']);
const allowedPriority = new Set(['low', 'medium', 'high']);

export const load: PageServerLoad = async ({ fetch, parent, url, depends }) => {
  depends('reqstly:requests:list');

  const { token } = await parent();

  const status = url.searchParams.get('status') ?? '';
  const category = url.searchParams.get('category') ?? '';
  const priority = url.searchParams.get('priority') ?? '';
  const sort = url.searchParams.get('sort') ?? '-updated_at';
  const page = Number(url.searchParams.get('page') ?? '1');
  const limit = Number(url.searchParams.get('limit') ?? '20');
  const q = url.searchParams.get('q') ?? '';

  const params = new URLSearchParams();
  params.set('page', Number.isFinite(page) ? String(Math.max(1, page)) : '1');
  params.set('limit', Number.isFinite(limit) ? String(Math.min(100, Math.max(1, limit))) : '20');

  if (allowedSort.has(sort)) params.set('sort', sort);
  if (allowedStatus.has(status)) params.set('status', status);
  if (allowedCategory.has(category)) params.set('category', category);
  if (allowedPriority.has(priority)) params.set('priority', priority);
  if (q.trim().length > 0) params.set('q', q.trim());

  const [listResponse, enumResponse] = await Promise.all([
    callBackend(fetch, token, `/requests?${params.toString()}`),
    callBackend(fetch, token, '/meta/enums')
  ]);

  const listPayload =
    listResponse.ok && listResponse.json && typeof listResponse.json === 'object'
      ? (listResponse.json as ApiListEnvelope<SupportRequest>)
      : null;

  const enumPayload =
    enumResponse.ok && enumResponse.json && typeof enumResponse.json === 'object'
      ? (enumResponse.json as ApiEnvelope<RequestEnums>)
      : null;

  return {
    requests: listPayload?.data ?? [],
    meta: listPayload?.meta ?? {
      request_id: '',
      page: 1,
      limit: 20,
      total: 0,
      total_pages: 0
    },
    enums: enumPayload?.data ?? {
      status: ['open', 'in_progress', 'resolved'],
      category: ['IT', 'Ops', 'Admin', 'HR'],
      priority: ['low', 'medium', 'high']
    },
    filters: {
      status,
      category,
      priority,
      sort: allowedSort.has(sort) ? sort : '-updated_at',
      q
    },
    backendError: listResponse.ok ? null : `Failed to load requests (${listResponse.status})`
  };
};
