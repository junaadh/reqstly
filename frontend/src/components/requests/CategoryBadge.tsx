import { cn } from "@/lib/utils"
import { Tag } from "lucide-react"

interface CategoryBadgeProps {
  category: 'IT' | 'Ops' | 'Admin' | 'HR'
  className?: string
}

export function CategoryBadge({ category, className }: CategoryBadgeProps) {
  const styles = {
    IT: 'bg-blue-50 text-blue-700 border-blue-200 dark:bg-blue-950 dark:text-blue-400 dark:border-blue-900',
    Ops: 'bg-green-50 text-green-700 border-green-200 dark:bg-green-950 dark:text-green-400 dark:border-green-900',
    Admin: 'bg-purple-50 text-purple-700 border-purple-200 dark:bg-purple-950 dark:text-purple-400 dark:border-purple-900',
    HR: 'bg-orange-50 text-orange-700 border-orange-200 dark:bg-orange-950 dark:text-orange-400 dark:border-orange-900',
  }

  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 rounded-md px-2.5 py-0.5 text-xs font-medium border',
        styles[category],
        className
      )}
    >
      <Tag className="h-3 w-3" />
      {category}
    </span>
  )
}
