import { json, type RequestHandler } from '@sveltejs/kit';

import { asApiError, callBackend, withSessionCookie } from '$lib/server/backend';

function unauthorizedResponse() {
  return json(
    {
      error: {
        code: 'UNAUTHORIZED',
        message: 'Session expired.'
      }
    },
    { status: 401 }
  );
}

export const GET: RequestHandler = async ({ fetch, request }) => {
  const result = await callBackend(
    fetch,
    '/preferences',
    withSessionCookie(request.headers.get('cookie'))
  );
  if (result.status === 401) return unauthorizedResponse();

  if (result.json && typeof result.json === 'object') {
    return json(result.json, { status: result.status });
  }

  return json(
    {
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Unexpected preferences response.'
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
          message: 'Validation failed'
        }
      },
      { status: 422 }
    );
  }

  const result = await callBackend(
    fetch,
    '/preferences',
    withSessionCookie(cookieHeader, {
      method: 'PATCH',
      body: JSON.stringify(body)
    })
  );
  if (result.status === 401) return unauthorizedResponse();

  if (result.json && typeof result.json === 'object') {
    return json(result.json, { status: result.status });
  }

  const apiError = asApiError(result.json);
  return json(
    apiError ?? {
      error: {
        code: 'INTERNAL_ERROR',
        message: 'Unexpected preferences update response.'
      }
    },
    { status: result.status || 500 }
  );
};
