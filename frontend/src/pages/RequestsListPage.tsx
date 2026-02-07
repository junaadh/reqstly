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
          <div className="animate-spin rounded-full h-12 w-12 border-4 border-indigo-500 border-t-transparent absolute top-0"></div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold bg-gradient-to-r from-slate-900 to-slate-700 bg-clip-text text-transparent">
            My Requests
          </h1>
          <p className="text-slate-500 mt-1">Manage and track all your requests</p>
        </div>
        <Link
          to="/requests/new"
          className="bg-gradient-to-r from-indigo-500 to-purple-600 hover:from-indigo-600 hover:to-purple-700 text-white px-6 py-3 rounded-xl text-sm font-semibold shadow-lg shadow-indigo-500/20 hover:shadow-xl hover:shadow-indigo-500/30 transition-all flex items-center gap-2"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
          </svg>
          New Request
        </Link>
      </div>

      {/* Filters */}
      <div className="bg-white/70 backdrop-blur-sm rounded-2xl border border-slate-200/50 p-6 shadow-lg shadow-slate-200/50">
        <div className="flex gap-4">
          <div className="flex-1">
            <label className="block text-sm font-semibold text-slate-700 mb-2">
              Status
            </label>
            <select
              value={statusFilter}
              onChange={(e) => setStatusFilter(e.target.value as any)}
              className="w-full border border-slate-300 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500 transition-all bg-white/50 hover:bg-white/80 appearance-none cursor-pointer"
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
              className="w-full border border-slate-300 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500 transition-all bg-white/50 hover:bg-white/80 appearance-none cursor-pointer"
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
        <div className="bg-white/70 backdrop-blur-sm rounded-2xl border border-slate-200/50 shadow-xl shadow-slate-200/50 p-12 text-center">
          <div className="w-16 h-16 bg-slate-100 rounded-2xl flex items-center justify-center mx-auto mb-4">
            <svg className="w-8 h-8 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
            </svg>
          </div>
          <p className="text-slate-500 mb-4">No requests found</p>
          <Link
            to="/requests/new"
            className="inline-flex items-center gap-2 text-indigo-600 hover:text-indigo-700 font-semibold text-sm"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
            </svg>
            Create your first request
          </Link>
        </div>
      ) : (
        <div className="bg-white/70 backdrop-blur-sm rounded-2xl border border-slate-200/50 shadow-xl shadow-slate-200/50 divide-y divide-slate-200/50">
          {requests.map((request) => (
            <Link
              key={request.id}
              to={`/requests/${request.id}`}
              className="block px-8 py-6 hover:bg-slate-50/50 transition-all"
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
                    <span className="px-2 py-1 bg-slate-100 rounded-lg font-medium">{request.category}</span>
                    <span className="flex items-center gap-1">
                      <span className="capitalize">{request.priority}</span> priority
                    </span>
                    <span className="flex items-center gap-1">
                      <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                      </svg>
                      {new Date(request.created_at).toLocaleDateString()}
                    </span>
                  </div>
                </div>
                <span
                  className={`ml-6 inline-flex items-center px-4 py-2 rounded-full text-sm font-semibold ${
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
    </div>
  );
}
