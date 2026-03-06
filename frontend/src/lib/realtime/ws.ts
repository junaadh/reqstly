import { get, writable } from 'svelte/store';

import { readAccessTokenCookie } from '$lib/auth/session';
import { apiBaseUrl } from '$lib/config';
import { logDebug, logInfo, logWarn } from '$lib/debug';
import type {
  RealtimeConnectionState,
  RealtimeEnvelope,
  RealtimeServerEvent
} from '$lib/realtime/types';

const SCOPE = 'realtime.ws';
const MAX_BACKOFF_MS = 10_000;
const BASE_BACKOFF_MS = 600;
const MAX_RETRY_EXP = 6;

const supportedTypes = new Set([
  'request.created',
  'request.patch',
  'request.deleted',
  'audit.append',
  'profile.patch',
  'sync.required'
]);

let socket: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let reconnectAttempts = 0;
let running = false;
let firstConnect = true;

const listeners = new Set<(event: RealtimeServerEvent) => void>();
const resyncListeners = new Set<
  (reason: 'reconnected' | 'server-sync-required') => void
>();

export const realtimeConnectionState = writable<RealtimeConnectionState>('offline');
export const realtimeLastEventTs = writable<string | null>(null);

function asRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return null;
  }

  return value as Record<string, unknown>;
}

function parseRealtimeEvent(raw: string): RealtimeServerEvent | null {
  let parsed: unknown;

  try {
    parsed = JSON.parse(raw);
  } catch {
    return null;
  }

  const envelope = asRecord(parsed) as RealtimeEnvelope | null;
  if (!envelope) return null;
  if (typeof envelope.type !== 'string' || !supportedTypes.has(envelope.type)) {
    return null;
  }
  if (typeof envelope.ts !== 'string' || typeof envelope.v !== 'number') {
    return null;
  }

  const payload = asRecord(envelope.payload);
  if (!payload) {
    return null;
  }

  switch (envelope.type) {
    case 'request.created': {
      const request = asRecord(payload.request);
      if (!request || typeof request.id !== 'string') return null;
      return envelope as RealtimeServerEvent;
    }
    case 'request.patch': {
      const request = asRecord(payload.request);
      if (!request || typeof request.id !== 'string') return null;
      if (!Array.isArray(payload.changed_fields)) return null;
      return envelope as RealtimeServerEvent;
    }
    case 'request.deleted': {
      if (typeof payload.id !== 'string') return null;
      if (typeof payload.status !== 'string') return null;
      return envelope as RealtimeServerEvent;
    }
    case 'audit.append': {
      const audit = asRecord(payload.audit);
      if (!audit || typeof audit.id !== 'string') return null;
      if (typeof audit.request_id !== 'string') return null;
      return envelope as RealtimeServerEvent;
    }
    case 'profile.patch': {
      const user = asRecord(payload.user);
      if (!user || typeof user.id !== 'string') return null;
      if (typeof user.email !== 'string') return null;
      if (typeof user.display_name !== 'string') return null;
      if (!Array.isArray(payload.changed_fields)) return null;
      return envelope as RealtimeServerEvent;
    }
    case 'sync.required': {
      return envelope as RealtimeServerEvent;
    }
    default:
      return null;
  }
}

function resolveWsBaseUrl(): string {
  if (typeof window !== 'undefined') {
    return window.location.origin.replace(/^http/i, 'ws');
  }

  const apiRoot = apiBaseUrl.replace(/\/api\/v1$/, '');
  return apiRoot.replace(/^http/i, 'ws');
}

function resolveConnectToken(initialToken?: string): string | null {
  const cookieToken = readAccessTokenCookie();
  if (cookieToken) return cookieToken;

  const trimmedInitial = initialToken?.trim();
  if (trimmedInitial && trimmedInitial.length > 0) {
    return trimmedInitial;
  }

  return null;
}

function clearReconnectTimer(): void {
  if (!reconnectTimer) return;
  clearTimeout(reconnectTimer);
  reconnectTimer = null;
}

function scheduleReconnect(initialToken?: string): void {
  if (!running) {
    realtimeConnectionState.set('offline');
    return;
  }

  clearReconnectTimer();

  reconnectAttempts += 1;
  const exp = Math.min(reconnectAttempts, MAX_RETRY_EXP);
  const delay = Math.min(MAX_BACKOFF_MS, BASE_BACKOFF_MS * 2 ** exp);
  const jitter = Math.floor(Math.random() * 250);
  const waitMs = delay + jitter;

  realtimeConnectionState.set('reconnecting');
  logWarn(SCOPE, 'Realtime websocket disconnected; scheduling reconnect', {
    reconnectAttempts,
    waitMs
  });

  reconnectTimer = setTimeout(() => {
    reconnectTimer = null;
    connect(initialToken);
  }, waitMs);
}

function notifyListeners(event: RealtimeServerEvent): void {
  for (const listener of listeners) {
    try {
      listener(event);
    } catch (error) {
      logWarn(SCOPE, 'Realtime listener failed', {
        error: error instanceof Error ? error.message : String(error)
      });
    }
  }
}

function notifyResyncListeners(
  reason: 'reconnected' | 'server-sync-required'
): void {
  for (const listener of resyncListeners) {
    try {
      listener(reason);
    } catch (error) {
      logWarn(SCOPE, 'Realtime resync listener failed', {
        reason,
        error: error instanceof Error ? error.message : String(error)
      });
    }
  }
}

function connect(initialToken?: string): void {
  if (!running) {
    return;
  }

  const token = resolveConnectToken(initialToken);
  if (!token) {
    logInfo(SCOPE, 'Skipping realtime connection because no access token is available');
    realtimeConnectionState.set('offline');
    return;
  }

  const wsBase = resolveWsBaseUrl();
  const socketUrl = `${wsBase}/ws?token=${encodeURIComponent(token)}`;

  if (!firstConnect) {
    realtimeConnectionState.set('reconnecting');
  } else {
    realtimeConnectionState.set('connecting');
  }

  logInfo(SCOPE, 'Connecting realtime websocket', { socketUrl: wsBase });

  socket = new WebSocket(socketUrl);

  socket.onopen = () => {
    const isReconnect = !firstConnect;
    reconnectAttempts = 0;
    firstConnect = false;
    realtimeConnectionState.set('connected');

    const lastSeenTs = get(realtimeLastEventTs);

    const helloPayload = {
      type: 'hello',
      ...(lastSeenTs ? { last_seen_ts: lastSeenTs } : {})
    };

    socket?.send(JSON.stringify(helloPayload));
    logDebug(SCOPE, 'Realtime websocket connected');

    if (isReconnect) {
      notifyResyncListeners('reconnected');
    }
  };

  socket.onmessage = (event: MessageEvent<string>) => {
    if (typeof event.data !== 'string') {
      return;
    }

    const parsed = parseRealtimeEvent(event.data);
    if (!parsed) {
      logDebug(SCOPE, 'Ignored malformed realtime event');
      return;
    }

    realtimeLastEventTs.set(parsed.ts);

  if (parsed.type === 'sync.required') {
      notifyResyncListeners('server-sync-required');
  }

    logDebug(SCOPE, 'Realtime event received', {
      type: parsed.type,
      requestId: parsed.request_id ?? null
    });

  notifyListeners(parsed);
};

  socket.onerror = () => {
    logWarn(SCOPE, 'Realtime websocket transport error');
  };

  socket.onclose = () => {
    socket = null;
    scheduleReconnect(initialToken);
  };
}

export function startRealtime(initialToken?: string): void {
  if (typeof window === 'undefined') {
    return;
  }

  if (running) {
    if (!socket || socket.readyState === WebSocket.CLOSED) {
      connect(initialToken);
    }
    return;
  }

  running = true;
  firstConnect = true;
  reconnectAttempts = 0;
  connect(initialToken);
}

export function stopRealtime(): void {
  running = false;
  clearReconnectTimer();
  reconnectAttempts = 0;
  firstConnect = true;

  if (socket) {
    socket.onclose = null;
    socket.close(1000, 'client shutdown');
    socket = null;
  }

  realtimeConnectionState.set('offline');
}

export function subscribeRealtimeEvents(
  listener: (event: RealtimeServerEvent) => void
): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function subscribeRealtimeResync(
  listener: (reason: 'reconnected' | 'server-sync-required') => void
): () => void {
  resyncListeners.add(listener);
  return () => {
    resyncListeners.delete(listener);
  };
}
