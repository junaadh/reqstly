import { fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';

import { asApiError, callBackend } from '$lib/server/backend';
import { ACCESS_TOKEN_COOKIE } from '$lib/auth/session';
import type { ApiEnvelope, RequestEnums, SupportRequest } from '$lib/types';

export const load: PageServerLoad = async ({ fetch, parent }) => {
  const { token } = await parent();
  const enumResponse = await callBackend(fetch, token, '/meta/enums');

  const payload =
    enumResponse.ok && enumResponse.json && typeof enumResponse.json === 'object'
      ? (enumResponse.json as ApiEnvelope<RequestEnums>).data
      : {
          status: ['open', 'in_progress', 'resolved'],
          category: ['IT', 'Ops', 'Admin', 'HR'],
          priority: ['low', 'medium', 'high']
        };

  return {
    enums: payload
  };
};

export const actions: Actions = {
  default: async ({ cookies, fetch, request }) => {
    const token = cookies.get(ACCESS_TOKEN_COOKIE);
    if (!token) {
      throw redirect(303, '/login?reason=session-expired');
    }
    const form = await request.formData();

    const payload = {
      title: String(form.get('title') ?? '').trim(),
      description: String(form.get('description') ?? '').trim() || null,
      category: String(form.get('category') ?? ''),
      priority: String(form.get('priority') ?? '')
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
