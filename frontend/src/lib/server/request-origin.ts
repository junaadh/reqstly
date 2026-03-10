import { env as privateEnv } from '$env/dynamic/private';

function firstHeaderValue(value: string | null): string | null {
  if (!value) return null;
  const first = value.split(',')[0]?.trim();
  return first && first.length > 0 ? first : null;
}

export function resolveRequestOrigin(request: Request, fallbackOrigin: string): string {
  const originHeader = firstHeaderValue(request.headers.get('origin'));
  if (originHeader) return originHeader;

  const forwardedProto = firstHeaderValue(request.headers.get('x-forwarded-proto'));
  const forwardedHost = firstHeaderValue(request.headers.get('x-forwarded-host'));
  if (forwardedProto && forwardedHost) {
    return `${forwardedProto}://${forwardedHost}`;
  }

  const configuredOrigin = (privateEnv.ORIGIN ?? '').trim();
  if (configuredOrigin.length > 0) {
    return configuredOrigin;
  }

  const host = firstHeaderValue(request.headers.get('host'));
  if (forwardedProto && host) {
    return `${forwardedProto}://${host}`;
  }

  return fallbackOrigin;
}

export function resolveRequestHostname(request: Request, fallbackOrigin: string): string {
  const origin = resolveRequestOrigin(request, fallbackOrigin);
  try {
    return new URL(origin).hostname;
  } catch {
    return new URL(fallbackOrigin).hostname;
  }
}

export function isSecureOrigin(origin: string): boolean {
  try {
    return new URL(origin).protocol === 'https:';
  } catch {
    return origin.startsWith('https://');
  }
}
