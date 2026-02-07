import { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { requestsApi, ApiError } from '../api/client';
import type { Request, AuditLog, UpdateRequestInput, RequestStatus } from '../types';
import { toast } from 'sonner';

export function RequestDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [request, setRequest] = useState<Request | null>(null);
  const [auditLogs, setAuditLogs] = useState<AuditLog[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isUpdating, setIsUpdating] = useState(false);

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
          toast.error('Request not found');
        } else {
          toast.error('Failed to load request');
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

    try {
      const updateData: UpdateRequestInput = { status: newStatus };
      const updated = await requestsApi.update(request.id, updateData);
      setRequest(updated);

      // Reload audit logs
      const logs = await requestsApi.getAuditLog(request.id);
      setAuditLogs(logs);

      toast.success(`Status updated to ${newStatus.replace('_', ' ')}`);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to update status';
      toast.error(errorMessage);
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
      toast.success('Request deleted successfully');
      navigate('/requests');
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to delete request';
      toast.error(errorMessage);
    }
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

  if (!request) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="text-center">
          <p className="text-slate-500">Request not found</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-start">
        <div className="flex-1">
          <div className="flex items-center gap-3 mb-3">
            <h1 className="text-3xl font-display font-semibold text-slate-900">
              {request.title}
            </h1>
            <span
              className={`inline-flex items-center px-4 py-1.5 rounded-full text-sm font-semibold ${
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
          <div className="flex items-center gap-4 text-sm text-slate-500">
            <span className="tag-pill">{request.category}</span>
            <span className="capitalize">{request.priority} priority</span>
            <span>{new Date(request.created_at).toLocaleDateString()}</span>
          </div>
        </div>
        <div className="flex gap-2">
          <button
            onClick={handleDelete}
            className="px-4 py-2 border border-rose-200/70 rounded-xl text-sm font-semibold text-rose-700 hover:bg-rose-50/70 hover:border-rose-300/70 transition-all"
          >
            Delete
          </button>
        </div>
      </div>

      {/* Request Details */}
      <div className="surface-card p-8">
        <h2 className="text-lg font-semibold text-slate-900 mb-4">
          Description
        </h2>
        <p className="text-slate-700 whitespace-pre-wrap leading-relaxed">
          {request.description || 'No description provided'}
        </p>
      </div>

      {/* Status Update */}
      <div className="surface-card p-8">
        <h2 className="text-lg font-semibold text-slate-900 mb-4">
          Update Status
        </h2>
        <div className="flex gap-3">
          <button
            onClick={() => handleUpdateStatus('open')}
            disabled={isUpdating || request.status === 'open'}
            className={`flex-1 px-6 py-3 rounded-xl text-sm font-semibold transition-all ${
              request.status === 'open'
                ? 'bg-amber-500 text-white shadow-lg shadow-amber-500/20'
                : 'border border-amber-200/70 text-amber-700 hover:bg-amber-50/70 hover:border-amber-300/70 disabled:opacity-40'
            }`}
          >
            Open
          </button>
          <button
            onClick={() => handleUpdateStatus('in_progress')}
            disabled={isUpdating || request.status === 'in_progress'}
            className={`flex-1 px-6 py-3 rounded-xl text-sm font-semibold transition-all ${
              request.status === 'in_progress'
                ? 'bg-sky-500 text-white shadow-lg shadow-sky-500/20'
                : 'border border-sky-200/70 text-sky-700 hover:bg-sky-50/70 hover:border-sky-300/70 disabled:opacity-40'
            }`}
          >
            In Progress
          </button>
          <button
            onClick={() => handleUpdateStatus('resolved')}
            disabled={isUpdating || request.status === 'resolved'}
            className={`flex-1 px-6 py-3 rounded-xl text-sm font-semibold transition-all ${
              request.status === 'resolved'
                ? 'bg-emerald-500 text-white shadow-lg shadow-emerald-500/20'
                : 'border border-emerald-200/70 text-emerald-700 hover:bg-emerald-50/70 hover:border-emerald-300/70 disabled:opacity-40'
            }`}
          >
            Resolved
          </button>
        </div>
      </div>

      {/* Audit Log */}
      <div className="surface-card">
        <div className="px-8 py-6 border-b soft-divider">
          <h2 className="text-lg font-semibold text-slate-900">
            Activity History
          </h2>
        </div>
        {auditLogs.length === 0 ? (
          <div className="p-12 text-center">
            <p className="text-slate-500 text-sm">No activity yet</p>
          </div>
        ) : (
          <div className="divide-y soft-divider">
            {auditLogs.map((log) => (
              <div key={log.id} className="px-8 py-5 hover:bg-white/70 transition-all">
                <div className="flex justify-between items-start">
                  <div>
                    <p className="text-sm font-semibold text-slate-900 capitalize">
                      {log.action.replace('_', ' ')}
                    </p>
                    <p className="text-xs text-slate-500 mt-1">
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
