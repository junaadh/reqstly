// Domain types matching backend models

export type RequestStatus = 'open' | 'in_progress' | 'resolved';
export type RequestCategory = 'IT' | 'Ops' | 'Admin' | 'HR';
export type RequestPriority = 'low' | 'medium' | 'high';
export type AuditAction = 'created' | 'updated' | 'deleted' | 'status_changed';

export interface User {
  id: string;
  email: string;
  name: string;
  provider: 'azure' | 'passkey';
  federated: boolean;
}

export interface Request {
  id: string;
  user_id: string | null;
  title: string;
  description: string | null;
  category: RequestCategory;
  status: RequestStatus;
  priority: RequestPriority;
  created_at: string;
  updated_at: string;
}

export interface CreateRequestInput {
  title: string;
  description?: string;
  category: RequestCategory;
  priority: RequestPriority;
}

export interface UpdateRequestInput {
  title?: string;
  description?: string;
  status?: RequestStatus;
  priority?: RequestPriority;
}

export interface AuditLog {
  id: string;
  request_id: string;
  user_id: string | null;
  action: AuditAction;
  old_value: unknown;
  new_value: unknown;
  created_at: string;
}

export interface RequestFilters {
  status?: RequestStatus;
  category?: RequestCategory;
  user_id?: string;
}

// API Response types
export interface ApiResponse<T> {
  data?: T;
  error?: string;
  message?: string;
}

export interface AuthResponse {
  id: string;
  email: string;
  name: string;
  provider: 'azure' | 'passkey';
  federated: boolean;
}
