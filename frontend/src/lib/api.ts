const API_BASE = import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:3000';

// ==================== Types ====================

export interface LoginRequest {
  email: string;
  password: string;
}

export interface SignupRequest {
  email: string;
  name: string;
  password: string;
}

export interface AuthResponse {
  message: string;
  user: {
    id: string;
    email: string;
    name: string;
  };
}

export interface CreateRequestData {
  title: string;
  description?: string;
  category: 'IT' | 'Ops' | 'Admin' | 'HR';
  priority: 'low' | 'medium' | 'high';
}

export interface UpdateRequestData {
  title?: string;
  description?: string;
  status?: 'open' | 'in_progress' | 'resolved';
  priority?: 'low' | 'medium' | 'high';
}

export interface Request {
  id: string;
  user_id: string;
  title: string;
  description: string | null;
  category: 'IT' | 'Ops' | 'Admin' | 'HR';
  status: 'open' | 'in_progress' | 'resolved';
  priority: 'low' | 'medium' | 'high';
  created_at: string;
  updated_at: string;
}

export interface AuditLog {
  id: string;
  request_id: string;
  user_id: string | null;
  action: 'created' | 'updated' | 'deleted' | 'status_changed';
  old_value: unknown;
  new_value: unknown;
  created_at: string;
}

export interface RequestFilters {
  status?: string;
  category?: string;
}

// ==================== Error Handling ====================

async function handleResponse(response: Response): Promise<never> {
  const data = await response.json().catch(() => ({ error: 'Request failed' }));
  throw new Error((data as { error: string }).error || 'Request failed');
}

// ==================== Auth API ====================

export const authApi = {
  async login(data: LoginRequest): Promise<AuthResponse> {
    const response = await fetch(`${API_BASE}/auth/password/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify(data),
    });

    if (!response.ok) {
      await handleResponse(response);
    }

    return response.json();
  },

  async signup(data: SignupRequest): Promise<AuthResponse> {
    const response = await fetch(`${API_BASE}/auth/password/signup`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify(data),
    });

    if (!response.ok) {
      await handleResponse(response);
    }

    return response.json();
  },

  async logout(): Promise<void> {
    const response = await fetch(`${API_BASE}/auth/logout`, {
      method: 'POST',
      credentials: 'include',
    });

    if (!response.ok) {
      await handleResponse(response);
    }
  },

  async getMe(): Promise<AuthResponse['user'] | null> {
    const response = await fetch(`${API_BASE}/auth/me`, {
      credentials: 'include',
    });

    if (!response.ok) {
      return null;
    }

    return response.json();
  },
};

// ==================== Requests API ====================

export const requestsApi = {
  async create(data: CreateRequestData): Promise<Request> {
    const response = await fetch(`${API_BASE}/requests`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify(data),
    });

    if (!response.ok) {
      await handleResponse(response);
    }

    return response.json();
  },

  async list(filters?: RequestFilters): Promise<Request[]> {
    const params = new URLSearchParams();
    if (filters?.status) params.append('status', filters.status);
    if (filters?.category) params.append('category', filters.category);

    const response = await fetch(`${API_BASE}/requests?${params}`, {
      credentials: 'include',
    });

    if (!response.ok) {
      await handleResponse(response);
    }

    return response.json();
  },

  async get(id: string): Promise<Request> {
    const response = await fetch(`${API_BASE}/requests/${id}`, {
      credentials: 'include',
    });

    if (!response.ok) {
      await handleResponse(response);
    }

    return response.json();
  },

  async update(id: string, data: UpdateRequestData): Promise<Request> {
    const response = await fetch(`${API_BASE}/requests/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify(data),
    });

    if (!response.ok) {
      await handleResponse(response);
    }

    return response.json();
  },

  async delete(id: string): Promise<void> {
    const response = await fetch(`${API_BASE}/requests/${id}`, {
      method: 'DELETE',
      credentials: 'include',
    });

    if (!response.ok) {
      await handleResponse(response);
    }
  },

  async getAuditLog(id: string): Promise<AuditLog[]> {
    const response = await fetch(`${API_BASE}/requests/${id}/audit`, {
      credentials: 'include',
    });

    if (!response.ok) {
      await handleResponse(response);
    }

    return response.json();
  },
};

// ==================== Unified API Export ====================

export const api = {
  ...authApi,
  ...requestsApi,
};
