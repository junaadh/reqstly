export type RequestStatus = 'open' | 'in_progress' | 'resolved';
export type RequestCategory = 'IT' | 'Ops' | 'Admin' | 'HR';
export type RequestPriority = 'low' | 'medium' | 'high';

export interface ApiMeta {
  request_id: string;
}

export interface ApiListMeta extends ApiMeta {
  total: number;
  page: number;
  limit: number;
  total_pages: number;
}

export interface ApiEnvelope<T> {
  data: T;
  meta: ApiMeta;
}

export interface ApiListEnvelope<T> {
  data: T[];
  meta: ApiListMeta;
}

export interface ErrorDetail {
  field: string;
  message: string;
}

export interface ApiErrorPayload {
  code: string;
  message: string;
  details?: ErrorDetail[];
}

export interface ApiErrorEnvelope {
  error: ApiErrorPayload;
  meta: ApiMeta;
}

export interface MeProfile {
  id: string;
  email: string;
  display_name: string;
}

export interface SupportRequest {
  id: string;
  owner_user_id: string;
  title: string;
  description: string | null;
  category: RequestCategory;
  status: RequestStatus;
  priority: RequestPriority;
  created_at: string;
  updated_at: string;
}

export interface AuditLog {
  id: string;
  request_id: string;
  actor_user_id: string;
  action: 'created' | 'updated' | 'deleted' | 'status_changed';
  old_value: Record<string, unknown>;
  new_value: Record<string, unknown>;
  created_at: string;
}

export interface RequestEnums {
  status: RequestStatus[];
  category: RequestCategory[];
  priority: RequestPriority[];
}
