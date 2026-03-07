import { json, type RequestHandler } from '@sveltejs/kit';

import { asApiError, callBackend, withSessionCookie } from '$lib/server/backend';

export const GET: RequestHandler = async ({ fetch, request }) => {
  const result = await callBackend(
    fetch,
    '/me',
    withSessionCookie(request.headers.get('cookie'))
  );
  if (result.json && typeof result.json === 'object') {
    return json(result.json, { status: result.status });
  }

  return json(
    {
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Unexpected profile response.'
      }
    },
    { status: 500 }
  );
};

export const PATCH: RequestHandler = async ({ fetch, request }) => {
  const cookieHeader = request.headers.get('cookie');
  const body = await request.json().catch(() => null);
  if (!body || typeof body !== 'object') {
    return json(
      {
        error: {
          code: 'VALIDATION_ERROR',
          message: 'Validation failed',
          details: [
            {
              field: 'display_name',
              message: 'display_name is required'
            }
          ]
        }
      },
      { status: 422 }
    );
  }

  const displayNameValue =
    'display_name' in body ? (body as { display_name?: unknown }).display_name : undefined;

  const result = await callBackend(
    fetch,
    '/me',
    withSessionCookie(cookieHeader, {
      method: 'PATCH',
      body: JSON.stringify({
        display_name:
          typeof displayNameValue === 'string'
            ? displayNameValue
            : displayNameValue ?? null
      })
    })
  );

  if (result.json && typeof result.json === 'object') {
    return json(result.json, { status: result.status });
  }

  const apiError = asApiError(result.json);
  return json(
    apiError ?? {
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Unexpected profile update response.'
      }
    },
    { status: result.status || 500 }
  );
};
