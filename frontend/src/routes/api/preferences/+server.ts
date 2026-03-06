import { json, type RequestHandler } from '@sveltejs/kit';

import { asApiError, callBackend } from '$lib/server/backend';
import { getAccessTokenFromCookies } from '$lib/server/auth-cookie';

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

export const GET: RequestHandler = async ({ cookies, fetch }) => {
  const token = getAccessTokenFromCookies(cookies);
  if (!token) return unauthorizedResponse();

  const result = await callBackend(fetch, token, '/preferences');
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

export const PATCH: RequestHandler = async ({ cookies, fetch, request }) => {
  const token = getAccessTokenFromCookies(cookies);
  if (!token) return unauthorizedResponse();

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

  const result = await callBackend(fetch, token, '/preferences', {
    method: 'PATCH',
    body: JSON.stringify(body)
  });

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
