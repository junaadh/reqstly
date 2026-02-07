import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { requestsApi } from '../api/client';
import type { CreateRequestInput, RequestCategory, RequestPriority } from '../types';
import { toast } from 'sonner';

export function CreateRequestPage() {
  const navigate = useNavigate();
  const [isSubmitting, setIsSubmitting] = useState(false);

  const [formData, setFormData] = useState<CreateRequestInput>({
    title: '',
    description: '',
    category: 'IT',
    priority: 'medium',
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSubmitting(true);

    try {
      const request = await requestsApi.create(formData);
      toast.success('Request created successfully');
      navigate(`/requests/${request.id}`);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to create request';
      toast.error(errorMessage);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="max-w-2xl">
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <div className="w-10 h-10 bg-gradient-to-br from-indigo-500 to-purple-600 rounded-xl flex items-center justify-center shadow-lg shadow-indigo-500/20">
            <svg className="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
            </svg>
          </div>
          <div>
            <h1 className="text-2xl font-bold bg-gradient-to-r from-slate-900 to-slate-700 bg-clip-text text-transparent">
              Create New Request
            </h1>
            <p className="text-slate-500 text-sm">Fill in the details to submit a new request</p>
          </div>
        </div>
      </div>

      <form onSubmit={handleSubmit} className="bg-white/70 backdrop-blur-sm rounded-2xl border border-slate-200/50 shadow-xl shadow-slate-200/50 p-8 space-y-6">
        {/* Title */}
        <div className="space-y-2">
          <label className="block text-sm font-semibold text-slate-700">
            Title <span className="text-rose-500">*</span>
          </label>
          <input
            type="text"
            required
            value={formData.title}
            onChange={(e) => setFormData({ ...formData, title: e.target.value })}
            className="w-full border border-slate-300 rounded-xl px-4 py-3 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500 transition-all bg-white/50 hover:bg-white/80"
            placeholder="Brief title for your request"
          />
        </div>

        {/* Description */}
        <div className="space-y-2">
          <label className="block text-sm font-semibold text-slate-700">
            Description
          </label>
          <textarea
            value={formData.description}
            onChange={(e) => setFormData({ ...formData, description: e.target.value })}
            rows={4}
            className="w-full border border-slate-300 rounded-xl px-4 py-3 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500 transition-all bg-white/50 hover:bg-white/80 resize-none"
            placeholder="Detailed description of your request"
          />
        </div>

        {/* Category */}
        <div className="space-y-2">
          <label className="block text-sm font-semibold text-slate-700">
            Category <span className="text-rose-500">*</span>
          </label>
          <select
            required
            value={formData.category}
            onChange={(e) => setFormData({ ...formData, category: e.target.value as RequestCategory })}
            className="w-full border border-slate-300 rounded-xl px-4 py-3 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500 transition-all bg-white/50 hover:bg-white/80 appearance-none cursor-pointer"
          >
            <option value="IT">IT</option>
            <option value="Ops">Ops</option>
            <option value="Admin">Admin</option>
            <option value="HR">HR</option>
          </select>
        </div>

        {/* Priority */}
        <div className="space-y-2">
          <label className="block text-sm font-semibold text-slate-700">
            Priority <span className="text-rose-500">*</span>
          </label>
          <div className="grid grid-cols-3 gap-3">
            {(['low', 'medium', 'high'] as RequestPriority[]).map((priority) => (
              <button
                key={priority}
                type="button"
                onClick={() => setFormData({ ...formData, priority })}
                className={`relative px-4 py-3 rounded-xl border-2 font-medium text-sm transition-all capitalize ${
                  formData.priority === priority
                    ? priority === 'high'
                      ? 'border-rose-500 bg-rose-50 text-rose-700 shadow-lg shadow-rose-500/20'
                      : priority === 'medium'
                      ? 'border-amber-500 bg-amber-50 text-amber-700 shadow-lg shadow-amber-500/20'
                      : 'border-emerald-500 bg-emerald-50 text-emerald-700 shadow-lg shadow-emerald-500/20'
                    : 'border-slate-200 hover:border-slate-300 text-slate-600 hover:bg-slate-50'
                }`}
              >
                {priority}
                {formData.priority === priority && (
                  <span className="absolute -top-1.5 -right-1.5 w-4 h-4 bg-white rounded-full flex items-center justify-center shadow-md">
                    <svg className="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                      <path
                        fillRule="evenodd"
                        d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                        clipRule="evenodd"
                      />
                    </svg>
                  </span>
                )}
              </button>
            ))}
          </div>
        </div>

        {/* Actions */}
        <div className="flex justify-end gap-3 pt-6 border-t border-slate-200/50">
          <button
            type="button"
            onClick={() => navigate(-1)}
            className="px-6 py-3 border border-slate-300 rounded-xl text-sm font-semibold text-slate-700 hover:bg-slate-50 hover:border-slate-400 transition-all"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={isSubmitting}
            className="px-6 py-3 bg-gradient-to-r from-indigo-500 to-purple-600 hover:from-indigo-600 hover:to-purple-700 text-white rounded-xl text-sm font-semibold shadow-lg shadow-indigo-500/20 hover:shadow-xl hover:shadow-indigo-500/30 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center gap-2"
          >
            {isSubmitting ? (
              <>
                <svg className="animate-spin h-4 w-4" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                Creating...
              </>
            ) : (
              <>
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                </svg>
                Create Request
              </>
            )}
          </button>
        </div>
      </form>
    </div>
  );
}
