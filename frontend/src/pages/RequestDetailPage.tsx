import { useParams } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { api } from '@/lib/api'
import { StatusBadge } from '@/components/requests/StatusBadge'
import { PriorityBadge } from '@/components/requests/PriorityBadge'
import { CategoryBadge } from '@/components/requests/CategoryBadge'
import { AuditTimeline } from '@/components/requests/AuditTimeline'
import { ArrowLeft, Edit, Trash2 } from 'lucide-react'
import { Link } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { formatDistanceToNow } from 'date-fns'

export function RequestDetailPage() {
  const { id } = useParams<{ id: string }>()

  const { data: request, isLoading } = useQuery({
    queryKey: ['request', id],
    queryFn: () => api.get(id!),
    enabled: !!id,
  })

  const { data: auditLogs } = useQuery({
    queryKey: ['audit', id],
    queryFn: () => api.getAuditLog(id!),
    enabled: !!id,
  })

  if (isLoading) {
    return (
      <div className="p-8">
        <div className="animate-pulse">
          <div className="h-8 w-48 bg-muted rounded mb-4"></div>
          <div className="h-4 w-96 bg-muted rounded"></div>
        </div>
      </div>
    )
  }

  if (!request) {
    return (
      <div className="p-8">
        <p className="text-muted-foreground">Request not found</p>
      </div>
    )
  }

  return (
    <div className="p-8 animate-fade-in">
      {/* Header */}
      <div className="mb-8">
        <Link
          to="/requests"
          className="mb-4 inline-flex items-center text-sm text-muted-foreground hover:text-foreground"
        >
          <ArrowLeft className="mr-2 h-4 w-4" />
          Back to requests
        </Link>
        <div className="flex items-start justify-between">
          <div className="flex-1">
            <h1 className="font-display text-3xl font-bold tracking-tight">
              {request.title}
            </h1>
            <p className="mt-2 text-sm text-muted-foreground">
              Created {formatDistanceToNow(new Date(request.created_at), { addSuffix: true })}
              {' • '}
              Updated {formatDistanceToNow(new Date(request.updated_at), { addSuffix: true })}
            </p>
          </div>
          <div className="flex gap-2">
            <Button variant="outline" size="sm">
              <Edit className="mr-2 h-4 w-4" />
              Edit
            </Button>
            <Button variant="outline" size="sm" className="text-destructive">
              <Trash2 className="mr-2 h-4 w-4" />
              Delete
            </Button>
          </div>
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        {/* Main Content */}
        <div className="lg:col-span-2 space-y-6">
          {/* Request Details */}
          <Card className="animate-slide-up">
            <CardHeader>
              <CardTitle>Details</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <h3 className="text-sm font-medium text-muted-foreground mb-2">Description</h3>
                <p className="text-sm whitespace-pre-wrap">
                  {request.description || 'No description provided'}
                </p>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <h3 className="text-sm font-medium text-muted-foreground mb-2">Category</h3>
                  <CategoryBadge category={request.category} />
                </div>
                <div>
                  <h3 className="text-sm font-medium text-muted-foreground mb-2">Status</h3>
                  <StatusBadge status={request.status} />
                </div>
                <div>
                  <h3 className="text-sm font-medium text-muted-foreground mb-2">Priority</h3>
                  <PriorityBadge priority={request.priority} />
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Audit Timeline */}
          {auditLogs && auditLogs.length > 0 && (
            <Card className="animate-slide-up">
              <CardHeader>
                <CardTitle>Activity History</CardTitle>
              </CardHeader>
              <CardContent>
                <AuditTimeline logs={auditLogs} />
              </CardContent>
            </Card>
          )}
        </div>

        {/* Sidebar */}
        <div className="space-y-6">
          {/* Quick Actions */}
          <Card className="animate-slide-up">
            <CardHeader>
              <CardTitle>Quick Actions</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              <Button variant="outline" className="w-full justify-start">
                Change Status
              </Button>
              <Button variant="outline" className="w-full justify-start">
                Update Priority
              </Button>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}
