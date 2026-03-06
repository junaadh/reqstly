import { fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';

import { asApiError, callBackend } from '$lib/server/backend';
import { getAccessTokenFromCookies } from '$lib/server/auth-cookie';
import type { ApiEnvelope, AssigneeSuggestion, RequestEnums, SupportRequest } from '$lib/types';

export const load: PageServerLoad = async ({ fetch, parent }) => {
  const { token } = await parent();
  const [enumResponse, assigneeResponse] = await Promise.all([
    callBackend(fetch, token, '/meta/enums'),
    callBackend(fetch, token, '/assignees/suggestions?limit=100')
  ]);

  const payload =
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
    enums: payload,
    assigneeOptions
  };
};

export const actions: Actions = {
  default: async ({ cookies, fetch, request }) => {
    const token = getAccessTokenFromCookies(cookies);
    if (!token) {
      throw redirect(303, '/login?reason=session-expired');
    }
    const form = await request.formData();

    const payload = {
      title: String(form.get('title') ?? '').trim(),
      description: String(form.get('description') ?? '').trim() || null,
      category: String(form.get('category') ?? ''),
      priority: String(form.get('priority') ?? ''),
      assignee_email: String(form.get('assignee_email') ?? '').trim() || null
    };

    const createResponse = await callBackend(fetch, token, '/requests', {
      method: 'POST',
      body: JSON.stringify(payload)
    });

    if (createResponse.ok && createResponse.json && typeof createResponse.json === 'object') {
      const created = (createResponse.json as ApiEnvelope<SupportRequest>).data;
      throw redirect(303, `/requests/${created.id}`);
    }

    const apiError = asApiError(createResponse.json);
    if (apiError && createResponse.status === 422) {
      return fail(422, {
        message: apiError.error.message,
        details: apiError.error.details ?? [],
        values: payload
      });
    }

    return fail(createResponse.status, {
      message: apiError?.error.message ?? 'Failed to create request.',
      details: apiError?.error.details ?? [],
      values: payload
    });
  }
};
