import {
  appendSetCookieHeaders,
  callBackend,
  withSessionCookie
} from '$lib/server/backend';

function serializePayload(payload: unknown): string {
  if (payload === null || payload === undefined) {
    return '{}';
  }

  if (typeof payload === 'string') {
    return payload;
  }

  return JSON.stringify(payload);
}

export async function proxyAuthRequest(
  fetchFn: typeof fetch,
  request: Request,
  backendPath: string
): Promise<Response> {
  const requestBody = await request.text();
  const incomingContentType = request.headers.get('content-type');
  const incomingCsrfToken = request.headers.get('x-csrf-token');

  const forwardedHeaders: Record<string, string> = {};
  if (incomingContentType) {
    forwardedHeaders['Content-Type'] = incomingContentType;
  }
  if (incomingCsrfToken) {
    forwardedHeaders['X-CSRF-Token'] = incomingCsrfToken;
  }

  const requestInit = withSessionCookie(request.headers.get('cookie'), {
    method: request.method,
    body: requestBody.length > 0 ? requestBody : undefined,
    headers: Object.keys(forwardedHeaders).length > 0 ? forwardedHeaders : undefined
  });

  const result = await callBackend(fetchFn, backendPath, requestInit);

  const headers = new Headers();
  headers.set('Content-Type', 'application/json');
  appendSetCookieHeaders(result.headers, headers);

  return new Response(serializePayload(result.json), {
    status: result.status,
    headers
  });
}
