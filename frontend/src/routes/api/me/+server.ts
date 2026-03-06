import { json, type RequestHandler } from '@sveltejs/kit';

import { asApiError, callBackend } from '$lib/server/backend';
import { getAccessTokenFromCookies } from '$lib/server/auth-cookie';

export const GET: RequestHandler = async ({ cookies, fetch }) => {
  const token = getAccessTokenFromCookies(cookies);
  if (!token) {
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

  const result = await callBackend(fetch, token, '/me');
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

export const PATCH: RequestHandler = async ({ cookies, fetch, request }) => {
  const token = getAccessTokenFromCookies(cookies);
  if (!token) {
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

  const result = await callBackend(fetch, token, '/me', {
    method: 'PATCH',
    body: JSON.stringify({
      display_name: typeof displayNameValue === 'string' ? displayNameValue : displayNameValue ?? null
    })
  });

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
