import { error, fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';

import { asApiError, callBackend } from '$lib/server/backend';
import { getAccessTokenFromCookies } from '$lib/server/auth-cookie';
import type {
  ApiEnvelope,
  ApiListEnvelope,
  AssigneeSuggestion,
  AuditLog,
  RequestEnums,
  SupportRequest
} from '$lib/types';

export const load: PageServerLoad = async ({ fetch, params, parent, depends }) => {
  const { token } = await parent();
  const requestId = params.id;
  depends(`reqstly:requests:detail:${requestId}`);

  const [requestResponse, auditResponse, enumResponse, assigneeResponse] = await Promise.all([
    callBackend(fetch, token, `/requests/${requestId}`),
    callBackend(fetch, token, `/requests/${requestId}/audit`),
    callBackend(fetch, token, '/meta/enums'),
    callBackend(fetch, token, '/assignees/suggestions?limit=100')
  ]);

  if (!requestResponse.ok || !requestResponse.json || typeof requestResponse.json !== 'object') {
    throw error(requestResponse.status || 404, 'Request not found');
  }

  const requestPayload = (requestResponse.json as ApiEnvelope<SupportRequest>).data;

  const auditPayload =
    auditResponse.ok && auditResponse.json && typeof auditResponse.json === 'object'
      ? (auditResponse.json as ApiListEnvelope<AuditLog>).data
      : [];

  const enumPayload =
    enumResponse.ok && enumResponse.json && typeof enumResponse.json === 'object'
      ? (enumResponse.json as ApiEnvelope<RequestEnums>).data
      : {
          status: ['open', 'in_progress', 'resolved'],
          category: ['IT', 'Ops', 'Admin', 'HR'],
          priority: ['low', 'medium', 'high']
        };

  const assigneeOptions =
    assigneeResponse.ok && assigneeResponse.json && typeof assigneeResponse.json === 'object'
      ? (assigneeResponse.json as ApiEnvelope<AssigneeSuggestion[]>).data
      : [];

  return {
    request: requestPayload,
    audit: auditPayload,
    enums: enumPayload,
    assigneeOptions
  };
};

export const actions: Actions = {
  update: async ({ cookies, fetch, params, request }) => {
    const token = getAccessTokenFromCookies(cookies);
    if (!token) {
      throw redirect(303, '/login?reason=session-expired');
    }
    const requestId = params.id;
    const form = await request.formData();

    const payload = {
      title: String(form.get('title') ?? '').trim(),
      description: String(form.get('description') ?? '').trim() || null,
      category: String(form.get('category') ?? ''),
      status: String(form.get('status') ?? ''),
      priority: String(form.get('priority') ?? ''),
      assignee_email: String(form.get('assignee_email') ?? '').trim()
    };

    const updateResponse = await callBackend(fetch, token, `/requests/${requestId}`, {
      method: 'PATCH',
      body: JSON.stringify(payload)
    });

    if (updateResponse.ok) {
      return {
        success: true,
        message: 'Request saved successfully.',
        values: payload
      };
    }

    const apiError = asApiError(updateResponse.json);

    return fail(updateResponse.status || 500, {
      success: false,
      message: apiError?.error.message ?? 'Failed to save request.',
      details: apiError?.error.details ?? [],
      values: payload
    });
  },

  delete: async ({ cookies, fetch, params }) => {
    const token = getAccessTokenFromCookies(cookies);
    if (!token) {
      throw redirect(303, '/login?reason=session-expired');
    }
    const requestId = params.id;

    const deleteResponse = await callBackend(fetch, token, `/requests/${requestId}`, {
      method: 'DELETE'
    });

    if (!deleteResponse.ok) {
      const apiError = asApiError(deleteResponse.json);
      return fail(deleteResponse.status || 500, {
        message: apiError?.error.message ?? 'Failed to delete request.'
      });
    }

    throw redirect(303, '/requests');
  }
};
