import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { requestsApi } from '../api/client';
import type { Request } from '../types';
import { toast } from 'sonner';

export function DashboardPage() {
  const [requests, setRequests] = useState<Request[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    async function loadRequests() {
      try {
        const data = await requestsApi.list();
        setRequests(data.slice(0, 5)); // Show recent 5
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Failed to load requests';
        toast.error(errorMessage);
      } finally {
        setIsLoading(false);
      }
    }
    loadRequests();
  }, []);

  const stats = {
    total: requests.length,
    open: requests.filter((r) => r.status === 'open').length,
    inProgress: requests.filter((r) => r.status === 'in_progress').length,
    resolved: requests.filter((r) => r.status === 'resolved').length,
  };

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
    <div className="space-y-8">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-display font-semibold text-slate-900">
            Dashboard
          </h1>
          <p className="text-slate-500 mt-1">Welcome back! Here's what's happening with your requests.</p>
        </div>
        <Link
          to="/requests/new"
          className="btn-primary text-sm flex items-center gap-2"
        >
          New Request
        </Link>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="surface-card hover-lift p-6">
          <div>
            <p className="text-sm font-medium text-slate-500 mb-1">Total Requests</p>
            <p className="text-3xl font-display font-semibold text-slate-900">{stats.total}</p>
          </div>
        </div>
        <div className="surface-card hover-lift p-6">
          <div>
            <p className="text-sm font-medium text-slate-500 mb-1">Open</p>
            <p className="text-3xl font-bold text-amber-600">{stats.open}</p>
          </div>
        </div>
        <div className="surface-card hover-lift p-6">
          <div>
            <p className="text-sm font-medium text-slate-500 mb-1">In Progress</p>
            <p className="text-3xl font-bold text-blue-600">{stats.inProgress}</p>
          </div>
        </div>
        <div className="surface-card hover-lift p-6">
          <div>
            <p className="text-sm font-medium text-slate-500 mb-1">Resolved</p>
            <p className="text-3xl font-bold text-emerald-600">{stats.resolved}</p>
          </div>
        </div>
      </div>

      {/* Recent Requests */}
      <div className="surface-card">
        <div className="px-8 py-6 border-b soft-divider">
          <h2 className="text-lg font-semibold text-slate-900">Recent Requests</h2>
        </div>
        {requests.length === 0 ? (
          <div className="p-12 text-center">
            <p className="text-slate-500 mb-4">No requests yet. Create your first request!</p>
            <Link
              to="/requests/new"
              className="inline-flex items-center gap-2 link-strong text-sm"
            >
              Create your first request
            </Link>
          </div>
        ) : (
          <div className="divide-y soft-divider">
            {requests.map((request) => (
              <Link
                key={request.id}
                to={`/requests/${request.id}`}
                className="block px-8 py-5 hover:bg-white/70 transition-all"
              >
                <div className="flex justify-between items-start">
                  <div className="flex-1">
                    <h3 className="text-sm font-semibold text-slate-900 mb-1">
                      {request.title}
                    </h3>
                    <p className="text-sm text-slate-500">
                      {request.category} • <span className="capitalize">{request.priority}</span> priority
                    </p>
                  </div>
                  <span
                    className={`inline-flex items-center px-3 py-1 rounded-full text-xs font-semibold ${
                      request.status === 'open'
                        ? 'bg-amber-100 text-amber-700'
                        : request.status === 'in_progress'
                        ? 'bg-blue-100 text-blue-700'
                        : 'bg-emerald-100 text-emerald-700'
                    }`}
                  >
                    {request.status.replace('_', ' ')}
                  </span>
                </div>
              </Link>
            ))}
          </div>
        )}
        <div className="px-8 py-4 border-t soft-divider">
          <Link
            to="/requests"
            className="text-sm link-strong flex items-center gap-1"
          >
            View all requests
          </Link>
        </div>
      </div>
    </div>
  );
}
