import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { requestsApi } from '../api/client';
import type { Request, RequestStatus, RequestCategory } from '../types';
import { toast } from 'sonner';

export function RequestsListPage() {
  const [requests, setRequests] = useState<Request[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [statusFilter, setStatusFilter] = useState<RequestStatus | 'all'>('all');
  const [categoryFilter, setCategoryFilter] = useState<RequestCategory | 'all'>('all');

  useEffect(() => {
    async function loadRequests() {
      try {
        const filters: any = {};
        if (statusFilter !== 'all') filters.status = statusFilter;
        if (categoryFilter !== 'all') filters.category = categoryFilter;

        const data = await requestsApi.list(filters);
        setRequests(data);
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Failed to load requests';
        toast.error(errorMessage);
      } finally {
        setIsLoading(false);
      }
    }
    loadRequests();
  }, [statusFilter, categoryFilter]);

  if (isLoading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="relative">
          <div className="animate-spin rounded-full h-12 w-12 border-4 border-slate-200"></div>
          <div className="animate-spin rounded-full h-12 w-12 border-4 border-teal-500 border-t-transparent absolute top-0"></div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-display font-semibold text-slate-900">
            My Requests
          </h1>
          <p className="text-slate-500 mt-1">Manage and track all your requests</p>
        </div>
        <Link
          to="/requests/new"
          className="btn-primary text-sm flex items-center gap-2"
        >
          New Request
        </Link>
      </div>

      {/* Filters */}
      <div className="surface-card p-6">
        <div className="flex gap-4">
          <div className="flex-1">
            <label className="block text-sm font-semibold text-slate-700 mb-2">
              Status
            </label>
            <select
              value={statusFilter}
              onChange={(e) => setStatusFilter(e.target.value as any)}
              className="input-field w-full text-sm appearance-none cursor-pointer"
            >
              <option value="all">All Status</option>
              <option value="open">Open</option>
              <option value="in_progress">In Progress</option>
              <option value="resolved">Resolved</option>
            </select>
          </div>
          <div className="flex-1">
            <label className="block text-sm font-semibold text-slate-700 mb-2">
              Category
            </label>
            <select
              value={categoryFilter}
              onChange={(e) => setCategoryFilter(e.target.value as any)}
              className="input-field w-full text-sm appearance-none cursor-pointer"
            >
              <option value="all">All Categories</option>
              <option value="IT">IT</option>
              <option value="Ops">Ops</option>
              <option value="Admin">Admin</option>
              <option value="HR">HR</option>
            </select>
          </div>
        </div>
      </div>

      {/* Requests List */}
      {requests.length === 0 ? (
        <div className="surface-card p-12 text-center">
          <p className="text-slate-500 mb-4">No requests found</p>
          <Link
            to="/requests/new"
            className="inline-flex items-center gap-2 link-strong text-sm"
          >
            Create your first request
          </Link>
        </div>
      ) : (
        <div className="surface-card divide-y soft-divider">
          {requests.map((request) => (
            <Link
              key={request.id}
              to={`/requests/${request.id}`}
              className="block px-8 py-6 hover:bg-white/70 transition-all"
            >
              <div className="flex justify-between items-start">
                <div className="flex-1">
                  <h3 className="text-base font-semibold text-slate-900 mb-2">
                    {request.title}
                  </h3>
                  <p className="text-sm text-slate-500 mb-3 line-clamp-2">
                    {request.description || 'No description'}
                  </p>
                  <div className="flex gap-4 text-xs text-slate-500">
                    <span className="tag-pill">{request.category}</span>
                    <span className="flex items-center gap-1">
                      <span className="capitalize">{request.priority}</span> priority
                    </span>
                    <span className="flex items-center gap-1">
                      {new Date(request.created_at).toLocaleDateString()}
                    </span>
                  </div>
                </div>
                <span
                  className={`ml-6 inline-flex items-center px-4 py-2 rounded-full text-sm font-semibold ${
                    request.status === 'open'
                      ? 'bg-amber-100/70 text-amber-700'
                      : request.status === 'in_progress'
                      ? 'bg-sky-100/70 text-sky-700'
                      : 'bg-emerald-100/70 text-emerald-700'
                  }`}
                >
                  {request.status.replace('_', ' ')}
                </span>
              </div>
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}
