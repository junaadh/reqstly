import type { AuditLog, MeProfile, SupportRequest } from '$lib/types';

export type RealtimeEventType =
  | 'request.created'
  | 'request.patch'
  | 'request.deleted'
  | 'audit.append'
  | 'profile.patch'
  | 'sync.required';

export interface RealtimeEnvelope {
  v: number;
  type: RealtimeEventType;
  ts: string;
  request_id?: string;
  trace_id?: string;
  payload: unknown;
}

export interface RequestCreatedPayload {
  request: SupportRequest;
}

export interface RequestPatchPayload {
  request: SupportRequest;
  changed_fields: string[];
  previous_status: string;
}

export interface RequestDeletedPayload {
  id: string;
  status: string;
}

export interface AuditAppendPayload {
  audit: AuditLog;
}

export interface ProfilePatchPayload {
  user: MeProfile;
  changed_fields: string[];
}

export type RealtimeServerEvent =
  | ({ type: 'request.created' } & Omit<RealtimeEnvelope, 'type' | 'payload'> & {
      payload: RequestCreatedPayload;
    })
  | ({ type: 'request.patch' } & Omit<RealtimeEnvelope, 'type' | 'payload'> & {
      payload: RequestPatchPayload;
    })
  | ({ type: 'request.deleted' } & Omit<RealtimeEnvelope, 'type' | 'payload'> & {
      payload: RequestDeletedPayload;
    })
  | ({ type: 'audit.append' } & Omit<RealtimeEnvelope, 'type' | 'payload'> & {
      payload: AuditAppendPayload;
    })
  | ({ type: 'profile.patch' } & Omit<RealtimeEnvelope, 'type' | 'payload'> & {
      payload: ProfilePatchPayload;
    })
  | ({ type: 'sync.required' } & Omit<RealtimeEnvelope, 'type' | 'payload'> & {
      payload: Record<string, never>;
    });

export type RealtimeConnectionState =
  | 'offline'
  | 'connecting'
  | 'connected'
  | 'reconnecting';
