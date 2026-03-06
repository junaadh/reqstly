export function statusBadgeClass(status: string): string {
  return (
    {
      open: 'bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-200',
      in_progress: 'bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-200',
      resolved: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-200'
    }[status] ?? 'bg-slate-100 text-slate-700'
  );
}

export function priorityBadgeClass(priority: string): string {
  return (
    {
      low: 'bg-sky-100 text-sky-700 dark:bg-sky-900/40 dark:text-sky-200',
      medium: 'bg-amber-100 text-amber-700 dark:bg-amber-900/40 dark:text-amber-200',
      high: 'bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-200'
    }[priority] ?? 'bg-slate-100 text-slate-700'
  );
}

export function readableStatus(status: string): string {
  return status.replace('_', ' ');
}
