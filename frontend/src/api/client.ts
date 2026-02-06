import {
  Request,
  CreateRequestInput,
  UpdateRequestInput,
  RequestFilters,
  AuditLog,
  AuthResponse,
} from '../types';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'https://api.localhost';

class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'ApiError';
  }
}

async function request<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const url = `${API_BASE_URL}${endpoint}`;

  const config: RequestInit = {
    ...options,
    credentials: 'include', // Include cookies for session auth
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
  };

  const response = await fetch(url, config);

  if (!response.ok) {
    const error = await response.text();
    throw new ApiError(response.status, error || 'Request failed');
  }

  // Handle 204 No Content
  if (response.status === 204) {
    return undefined as T;
  }

  return response.json();
}

// Auth API
export const authApi = {
  getMe: () => request<AuthResponse>('/auth/me'),

  passwordLogin: (email: string, password: string) =>
    request<AuthResponse>('/auth/password/login', {
      method: 'POST',
      credentials: 'include',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password }),
    }),

  passwordSignup: (email: string, name: string, password: string) =>
    request<AuthResponse>('/auth/password/signup', {
      method: 'POST',
      body: JSON.stringify({ email, name, password }),
    }),

  azureLogin: () => {
    window.location.href = `${API_BASE_URL}/auth/azure/login`;
  },

  logout: () => request<{ message: string }>('/auth/logout', { method: 'POST' }),
};

// Requests API
export const requestsApi = {
  list: (filters?: RequestFilters) => {
    const params = new URLSearchParams();
    if (filters?.status) params.append('status', filters.status);
    if (filters?.category) params.append('category', filters.category);

    const query = params.toString();
    return request<Request[]>(`/requests${query ? `?${query}` : ''}`);
  },

  get: (id: string) => request<Request>(`/requests/${id}`),

  create: (data: CreateRequestInput) =>
    request<Request>('/requests', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: string, data: UpdateRequestInput) =>
    request<Request>(`/requests/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string) =>
    request<void>(`/requests/${id}`, { method: 'DELETE' }),

  getAuditLog: (id: string) => request<AuditLog[]>(`/requests/${id}/audit`),
};

export { ApiError };
