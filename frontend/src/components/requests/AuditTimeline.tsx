import { formatDistanceToNow } from 'date-fns'
import type { AuditLog } from '@/lib/api'
import { Clock, Edit, Trash2, GitCompare } from 'lucide-react'

interface AuditTimelineProps {
  logs: AuditLog[]
}

export function AuditTimeline({ logs }: AuditTimelineProps) {
  const getActionIcon = (action: AuditLog['action']) => {
    switch (action) {
      case 'created':
        return Clock
      case 'updated':
        return Edit
      case 'deleted':
        return Trash2
      case 'status_changed':
        return GitCompare
    }
  }

  const getActionLabel = (action: AuditLog['action']) => {
    switch (action) {
      case 'created':
        return 'Created'
      case 'updated':
        return 'Updated'
      case 'deleted':
        return 'Deleted'
      case 'status_changed':
        return 'Status Changed'
    }
  }

  if (logs.length === 0) {
    return (
      <div className="text-center text-muted-foreground py-8">
        No activity yet
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {logs.map((log, index) => {
        const Icon = getActionIcon(log.action)
        const isLast = index === logs.length - 1

        const valueDisplay = (log.old_value || log.new_value) ? (
          <div className="text-sm" key={`value-${log.id}`}>
            {log.old_value != null && (
              <span className="text-muted-foreground line-clamp-1">
                {String(log.old_value)}
              </span>
            )}
            {log.old_value != null && log.new_value != null && <span> → </span>}
            {log.new_value != null && (
              <span className="text-foreground">
                {String(log.new_value)}
              </span>
            )}
          </div>
        ) : null

        return (
          <div key={log.id} className="relative">
            {/* Timeline line */}
            {!isLast && (
              <div className="absolute left-[11px] top-6 h-[calc(100%+12px)] w-0.5 bg-border" />
            )}

            <div className="flex gap-4">
              {/* Icon */}
              <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-primary">
                <Icon className="h-3 w-3 text-primary-foreground" />
              </div>

              {/* Content */}
              <div className="flex-1 space-y-1">
                <div className="flex items-center justify-between">
                  <p className="text-sm font-medium">{getActionLabel(log.action)}</p>
                  <span className="text-xs text-muted-foreground">
                    {formatDistanceToNow(new Date(log.created_at), { addSuffix: true })}
                  </span>
                </div>

                {valueDisplay}
              </div>
            </div>
          </div>
        )
      })}
    </div>
  )
}
