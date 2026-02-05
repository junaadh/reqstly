import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { requestsApi } from '../api/client';
import type { Request, RequestStatus, RequestCategory } from '../types';

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
        console.error('Failed to load requests:', error);
      } finally {
        setIsLoading(false);
      }
    }
    loadRequests();
  }, [statusFilter, categoryFilter]);

  if (isLoading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">My Requests</h1>
          <p className="text-gray-600">Manage your requests</p>
        </div>
        <Link
          to="/requests/new"
          className="bg-blue-600 text-white hover:bg-blue-700 px-4 py-2 rounded-md text-sm font-medium"
        >
          New Request
        </Link>
      </div>

      {/* Filters */}
      <div className="bg-white p-4 rounded-lg shadow-sm border border-gray-200">
        <div className="flex gap-4">
          <div className="flex-1">
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Status
            </label>
            <select
              value={statusFilter}
              onChange={(e) => setStatusFilter(e.target.value as any)}
              className="w-full border border-gray-300 rounded-md px-3 py-2 text-sm"
            >
              <option value="all">All Status</option>
              <option value="open">Open</option>
              <option value="in_progress">In Progress</option>
              <option value="resolved">Resolved</option>
            </select>
          </div>
          <div className="flex-1">
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Category
            </label>
            <select
              value={categoryFilter}
              onChange={(e) => setCategoryFilter(e.target.value as any)}
              className="w-full border border-gray-300 rounded-md px-3 py-2 text-sm"
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
        <div className="bg-white p-12 rounded-lg shadow-sm border border-gray-200 text-center">
          <p className="text-gray-500">No requests found</p>
          <Link
            to="/requests/new"
            className="inline-block mt-4 text-blue-600 hover:text-blue-700"
          >
            Create your first request →
          </Link>
        </div>
      ) : (
        <div className="bg-white shadow-sm rounded-lg border border-gray-200 divide-y divide-gray-200">
          {requests.map((request) => (
            <Link
              key={request.id}
              to={`/requests/${request.id}`}
              className="block px-6 py-4 hover:bg-gray-50"
            >
              <div className="flex justify-between items-start">
                <div className="flex-1">
                  <h3 className="text-sm font-medium text-gray-900">
                    {request.title}
                  </h3>
                  <p className="text-sm text-gray-600 mt-1">
                    {request.description || 'No description'}
                  </p>
                  <div className="flex gap-4 mt-2 text-xs text-gray-500">
                    <span>{request.category}</span>
                    <span>•</span>
                    <span className="capitalize">{request.priority} priority</span>
                    <span>•</span>
                    <span>{new Date(request.created_at).toLocaleDateString()}</span>
                  </div>
                </div>
                <span
                  className={`ml-4 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                    request.status === 'open'
                      ? 'bg-yellow-100 text-yellow-800'
                      : request.status === 'in_progress'
                      ? 'bg-blue-100 text-blue-800'
                      : 'bg-green-100 text-green-800'
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
