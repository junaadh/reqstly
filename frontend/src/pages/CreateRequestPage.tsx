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
        <div>
          <h1 className="text-2xl font-display font-semibold text-slate-900">
            Create New Request
          </h1>
          <p className="text-slate-500 text-sm">Fill in the details to submit a new request</p>
        </div>
      </div>

      <form onSubmit={handleSubmit} className="surface-card p-8 space-y-6">
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
            className="input-field w-full"
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
            className="input-field w-full resize-none"
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
            className="input-field w-full appearance-none cursor-pointer"
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
                      ? 'border-rose-500/80 bg-rose-50/70 text-rose-700 shadow-lg shadow-rose-500/10'
                      : priority === 'medium'
                      ? 'border-amber-500/80 bg-amber-50/70 text-amber-700 shadow-lg shadow-amber-500/10'
                      : 'border-emerald-500/80 bg-emerald-50/70 text-emerald-700 shadow-lg shadow-emerald-500/10'
                    : 'border-slate-200/80 hover:border-slate-300 text-slate-600 hover:bg-white/70'
                }`}
              >
                {priority}
              </button>
            ))}
          </div>
        </div>

        {/* Actions */}
        <div className="flex justify-end gap-3 pt-6 border-t soft-divider">
          <button
            type="button"
            onClick={() => navigate(-1)}
            className="btn-secondary text-sm"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={isSubmitting}
            className="btn-primary text-sm disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
          >
            {isSubmitting ? 'Creating...' : 'Create Request'}
          </button>
        </div>
      </form>
    </div>
  );
}
