import { useEffect, useState } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { requestsApi, ApiError } from '../api/client';
import type { Request, AuditLog, UpdateRequestInput, RequestStatus } from '../types';

export function RequestDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [request, setRequest] = useState<Request | null>(null);
  const [auditLogs, setAuditLogs] = useState<AuditLog[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isUpdating, setIsUpdating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function loadRequest() {
      if (!id) return;

      try {
        const [requestData, logsData] = await Promise.all([
          requestsApi.get(id),
          requestsApi.getAuditLog(id),
        ]);
        setRequest(requestData);
        setAuditLogs(logsData);
      } catch (err) {
        if (err instanceof ApiError && err.status === 404) {
          setError('Request not found');
        } else {
          setError('Failed to load request');
        }
      } finally {
        setIsLoading(false);
      }
    }
    loadRequest();
  }, [id]);

  const handleUpdateStatus = async (newStatus: RequestStatus) => {
    if (!request || isUpdating) return;

    setIsUpdating(true);
    setError(null);

    try {
      const updateData: UpdateRequestInput = { status: newStatus };
      const updated = await requestsApi.update(request.id, updateData);
      setRequest(updated);

      // Reload audit logs
      const logs = await requestsApi.getAuditLog(request.id);
      setAuditLogs(logs);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update status');
    } finally {
      setIsUpdating(false);
    }
  };

  const handleDelete = async () => {
    if (!request || !confirm('Are you sure you want to delete this request?')) {
      return;
    }

    try {
      await requestsApi.delete(request.id);
      navigate('/requests');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete request');
    }
  };

  if (isLoading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (!request) {
    return (
      <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-md">
        {error || 'Request not found'}
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-start">
        <div>
          <div className="flex items-center gap-3 mb-2">
            <h1 className="text-2xl font-bold text-gray-900">{request.title}</h1>
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
          <p className="text-gray-600">
            {request.category} • {request.priority} priority • Created on {new Date(request.created_at).toLocaleDateString()}
          </p>
        </div>
        <div className="flex gap-2">
          <Link
            to={`/requests/${request.id}/edit`}
            className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
          >
            Edit
          </Link>
          <button
            onClick={handleDelete}
            className="px-4 py-2 border border-red-300 rounded-md text-sm font-medium text-red-700 hover:bg-red-50"
          >
            Delete
          </button>
        </div>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-md text-sm">
          {error}
        </div>
      )}

      {/* Request Details */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200 p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Description</h2>
        <p className="text-gray-700 whitespace-pre-wrap">
          {request.description || 'No description provided'}
        </p>
      </div>

      {/* Status Update */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200 p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Update Status</h2>
        <div className="flex gap-2">
          <button
            onClick={() => handleUpdateStatus('open')}
            disabled={isUpdating || request.status === 'open'}
            className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium hover:bg-yellow-50 disabled:opacity-50"
          >
            Open
          </button>
          <button
            onClick={() => handleUpdateStatus('in_progress')}
            disabled={isUpdating || request.status === 'in_progress'}
            className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium hover:bg-blue-50 disabled:opacity-50"
          >
            In Progress
          </button>
          <button
            onClick={() => handleUpdateStatus('resolved')}
            disabled={isUpdating || request.status === 'resolved'}
            className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium hover:bg-green-50 disabled:opacity-50"
          >
            Resolved
          </button>
        </div>
      </div>

      {/* Audit Log */}
      <div className="bg-white shadow-sm rounded-lg border border-gray-200">
        <div className="px-6 py-4 border-b border-gray-200">
          <h2 className="text-lg font-semibold text-gray-900">Activity History</h2>
        </div>
        {auditLogs.length === 0 ? (
          <div className="p-6 text-center text-gray-500">No activity yet</div>
        ) : (
          <div className="divide-y divide-gray-200">
            {auditLogs.map((log) => (
              <div key={log.id} className="px-6 py-4">
                <div className="flex justify-between items-start">
                  <div>
                    <p className="text-sm font-medium text-gray-900 capitalize">
                      {log.action.replace('_', ' ')}
                    </p>
                    <p className="text-xs text-gray-500 mt-1">
                      {new Date(log.created_at).toLocaleString()}
                    </p>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
