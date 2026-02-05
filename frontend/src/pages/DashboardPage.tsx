import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { requestsApi } from '../api/client';
import type { Request } from '../types';

export function DashboardPage() {
  const [requests, setRequests] = useState<Request[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    async function loadRequests() {
      try {
        const data = await requestsApi.list();
        setRequests(data.slice(0, 5)); // Show recent 5
      } catch (error) {
        console.error('Failed to load requests:', error);
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
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Dashboard</h1>
          <p className="text-gray-600">Welcome to Reqstly</p>
        </div>
        <Link
          to="/requests/new"
          className="bg-blue-600 text-white hover:bg-blue-700 px-4 py-2 rounded-md text-sm font-medium"
        >
          New Request
        </Link>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <div className="text-3xl font-bold text-gray-900">{stats.total}</div>
          <div className="text-sm text-gray-600 mt-1">Total Requests</div>
        </div>
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <div className="text-3xl font-bold text-yellow-600">{stats.open}</div>
          <div className="text-sm text-gray-600 mt-1">Open</div>
        </div>
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <div className="text-3xl font-bold text-blue-600">{stats.inProgress}</div>
          <div className="text-sm text-gray-600 mt-1">In Progress</div>
        </div>
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <div className="text-3xl font-bold text-green-600">{stats.resolved}</div>
          <div className="text-sm text-gray-600 mt-1">Resolved</div>
        </div>
      </div>

      {/* Recent Requests */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200">
        <div className="px-6 py-4 border-b border-gray-200">
          <h2 className="text-lg font-semibold text-gray-900">Recent Requests</h2>
        </div>
        {requests.length === 0 ? (
          <div className="p-6 text-center text-gray-500">
            No requests yet. Create your first request!
          </div>
        ) : (
          <div className="divide-y divide-gray-200">
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
                      {request.category} • {request.priority} priority
                    </p>
                  </div>
                  <span
                    className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
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
        <div className="px-6 py-4 border-t border-gray-200">
          <Link
            to="/requests"
            className="text-sm text-blue-600 hover:text-blue-700 font-medium"
          >
            View all requests →
          </Link>
        </div>
      </div>
    </div>
  );
}
