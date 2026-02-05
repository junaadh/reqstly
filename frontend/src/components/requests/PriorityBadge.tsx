import { cn } from "@/lib/utils"
import { AlertCircle } from "lucide-react"

interface PriorityBadgeProps {
  priority: 'low' | 'medium' | 'high'
  className?: string
}

export function PriorityBadge({ priority, className }: PriorityBadgeProps) {
  const styles = {
    low: 'bg-gray-50 text-gray-700 border-gray-200 dark:bg-gray-950 dark:text-gray-400 dark:border-gray-900',
    medium: 'bg-orange-50 text-orange-700 border-orange-200 dark:bg-orange-950 dark:text-orange-400 dark:border-orange-900',
    high: 'bg-red-50 text-red-700 border-red-200 dark:bg-red-950 dark:text-red-400 dark:border-red-900',
  }

  const labels = {
    low: 'Low',
    medium: 'Medium',
    high: 'High',
  }

  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium border',
        styles[priority],
        className
      )}
    >
      <AlertCircle className="h-3 w-3" />
      {labels[priority]}
    </span>
  )
}
