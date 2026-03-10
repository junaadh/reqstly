type DebugMeta = Record<string, unknown> | undefined;

const DEBUG_PREFIX = '[reqstly]';

function isClient(): boolean {
  return typeof window !== 'undefined';
}

function hasDebugOverride(): boolean {
  if (!isClient()) return false;
  try {
    return window.localStorage.getItem('REQSTLY_DEBUG') === '1';
  } catch {
    return false;
  }
}

function isEnabled(): boolean {
  return import.meta.env.DEV || hasDebugOverride();
}

function format(scope: string, message: string): string {
  return `${DEBUG_PREFIX}[${scope}] ${message}`;
}

export function logDebug(scope: string, message: string, meta?: DebugMeta): void {
  if (!isEnabled()) return;
  if (meta) {
    console.debug(format(scope, message), meta);
    return;
  }
  console.debug(format(scope, message));
}

export function logInfo(scope: string, message: string, meta?: DebugMeta): void {
  if (!isEnabled()) return;
  if (meta) {
    console.info(format(scope, message), meta);
    return;
  }
  console.info(format(scope, message));
}

export function logWarn(scope: string, message: string, meta?: DebugMeta): void {
  if (!isEnabled()) return;
  if (meta) {
    console.warn(format(scope, message), meta);
    return;
  }
  console.warn(format(scope, message));
}

export function logError(scope: string, message: string, error?: unknown, meta?: DebugMeta): void {
  if (!isEnabled()) return;
  if (meta) {
    console.error(format(scope, message), { error, ...meta });
    return;
  }
  console.error(format(scope, message), error);
}

export function debugErrorDetails(error: unknown): Record<string, unknown> {
  if (error instanceof Error) {
    return {
      name: error.name,
      message: error.message,
      stack: error.stack
    };
  }
  return { error };
}
