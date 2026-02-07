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
          <div className="animate-spin rounded-full h-12 w-12 border-4 border-indigo-500 border-t-transparent absolute top-0"></div>
        </div>
      </div>
    );
  }

  if (!request) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="text-center">
          <div className="w-16 h-16 bg-slate-100 rounded-2xl flex items-center justify-center mx-auto mb-4">
            <svg className="w-8 h-8 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M12 12h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </div>
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
            <h1 className="text-3xl font-bold bg-gradient-to-r from-slate-900 to-slate-700 bg-clip-text text-transparent">
              {request.title}
            </h1>
            <span
              className={`inline-flex items-center px-4 py-1.5 rounded-full text-sm font-semibold ${
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
          <div className="flex items-center gap-4 text-sm text-slate-500">
            <span className="px-3 py-1 bg-slate-100 rounded-lg font-medium">{request.category}</span>
            <span className="capitalize">{request.priority} priority</span>
            <span className="flex items-center gap-1">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
              </svg>
              {new Date(request.created_at).toLocaleDateString()}
            </span>
          </div>
        </div>
        <div className="flex gap-2">
          <button
            onClick={handleDelete}
            className="px-4 py-2 border-2 border-rose-200 rounded-xl text-sm font-semibold text-rose-700 hover:bg-rose-50 hover:border-rose-300 transition-all"
          >
            Delete
          </button>
        </div>
      </div>

      {/* Request Details */}
      <div className="bg-white/70 backdrop-blur-sm rounded-2xl border border-slate-200/50 shadow-xl shadow-slate-200/50 p-8">
        <h2 className="text-lg font-semibold text-slate-900 mb-4 flex items-center gap-2">
          <svg className="w-5 h-5 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          Description
        </h2>
        <p className="text-slate-700 whitespace-pre-wrap leading-relaxed">
          {request.description || 'No description provided'}
        </p>
      </div>

      {/* Status Update */}
      <div className="bg-white/70 backdrop-blur-sm rounded-2xl border border-slate-200/50 shadow-xl shadow-slate-200/50 p-8">
        <h2 className="text-lg font-semibold text-slate-900 mb-4 flex items-center gap-2">
          <svg className="w-5 h-5 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          Update Status
        </h2>
        <div className="flex gap-3">
          <button
            onClick={() => handleUpdateStatus('open')}
            disabled={isUpdating || request.status === 'open'}
            className={`flex-1 px-6 py-3 rounded-xl text-sm font-semibold transition-all ${
              request.status === 'open'
                ? 'bg-amber-500 text-white shadow-lg shadow-amber-500/20'
                : 'border-2 border-amber-200 text-amber-700 hover:bg-amber-50 hover:border-amber-300 disabled:opacity-40'
            }`}
          >
            Open
          </button>
          <button
            onClick={() => handleUpdateStatus('in_progress')}
            disabled={isUpdating || request.status === 'in_progress'}
            className={`flex-1 px-6 py-3 rounded-xl text-sm font-semibold transition-all ${
              request.status === 'in_progress'
                ? 'bg-blue-500 text-white shadow-lg shadow-blue-500/20'
                : 'border-2 border-blue-200 text-blue-700 hover:bg-blue-50 hover:border-blue-300 disabled:opacity-40'
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
                : 'border-2 border-emerald-200 text-emerald-700 hover:bg-emerald-50 hover:border-emerald-300 disabled:opacity-40'
            }`}
          >
            Resolved
          </button>
        </div>
      </div>

      {/* Audit Log */}
      <div className="bg-white/70 backdrop-blur-sm rounded-2xl border border-slate-200/50 shadow-xl shadow-slate-200/50">
        <div className="px-8 py-6 border-b border-slate-200/50">
          <h2 className="text-lg font-semibold text-slate-900 flex items-center gap-2">
            <svg className="w-5 h-5 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            Activity History
          </h2>
        </div>
        {auditLogs.length === 0 ? (
          <div className="p-12 text-center">
            <div className="w-12 h-12 bg-slate-100 rounded-xl flex items-center justify-center mx-auto mb-3">
              <svg className="w-6 h-6 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <p className="text-slate-500 text-sm">No activity yet</p>
          </div>
        ) : (
          <div className="divide-y divide-slate-200/50">
            {auditLogs.map((log) => (
              <div key={log.id} className="px-8 py-5 hover:bg-slate-50/30 transition-all">
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
