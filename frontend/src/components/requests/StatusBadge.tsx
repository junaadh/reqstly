import { cn } from "@/lib/utils"

interface StatusBadgeProps {
  status: 'open' | 'in_progress' | 'resolved'
  className?: string
}

export function StatusBadge({ status, className }: StatusBadgeProps) {
  const styles = {
    open: 'bg-blue-50 text-blue-700 border-blue-200 dark:bg-blue-950 dark:text-blue-400 dark:border-blue-900',
    in_progress: 'bg-yellow-50 text-yellow-700 border-yellow-200 dark:bg-yellow-950 dark:text-yellow-400 dark:border-yellow-900',
    resolved: 'bg-green-50 text-green-700 border-green-200 dark:bg-green-950 dark:text-green-400 dark:border-green-900',
  }

  const labels = {
    open: 'Open',
    in_progress: 'In Progress',
    resolved: 'Resolved',
  }

  return (
    <span
      className={cn(
        'inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium border',
        styles[status],
        className
      )}
    >
      {labels[status]}
    </span>
  )
}
