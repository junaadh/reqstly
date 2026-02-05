import { useQuery } from '@tanstack/react-query'
import { api } from '@/lib/api'
import { LayoutDashboard, FileText, Clock, CheckCircle, Plus } from 'lucide-react'
import { Link } from 'react-router-dom'
import { formatDistanceToNow } from 'date-fns'
import { StatusBadge } from '@/components/requests/StatusBadge'
import { PriorityBadge } from '@/components/requests/PriorityBadge'
import { CategoryBadge } from '@/components/requests/CategoryBadge'

export function DashboardPage() {
  const { data: requests, isLoading } = useQuery({
    queryKey: ['requests'],
    queryFn: () => api.list(),
  })

  const stats = {
    total: requests?.length || 0,
    open: requests?.filter(r => r.status === 'open').length || 0,
    inProgress: requests?.filter(r => r.status === 'in_progress').length || 0,
    resolved: requests?.filter(r => r.status === 'resolved').length || 0,
  }

  const recentRequests = requests?.slice(0, 5) || []

  return (
    <div className="p-8 animate-fade-in">
      {/* Header */}
      <div className="mb-8">
        <h1 className="font-display text-3xl font-bold tracking-tight">
          Dashboard
        </h1>
        <p className="mt-2 text-muted-foreground">
          Welcome back! Here's what's happening with your requests.
        </p>
      </div>

      {isLoading ? (
        <div className="py-12 text-center text-muted-foreground">
          Loading dashboard...
        </div>
      ) : (
        <>
          {/* Stats Grid */}
          <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4 mb-12">
            <StatCard
              title="Total Requests"
              value={stats.total}
              icon={FileText}
              color="text-blue-600 dark:text-blue-400"
            />
            <StatCard
              title="Open"
              value={stats.open}
              icon={Clock}
              color="text-yellow-600 dark:text-yellow-400"
            />
            <StatCard
              title="In Progress"
              value={stats.inProgress}
              icon={LayoutDashboard}
              color="text-blue-600 dark:text-blue-400"
            />
            <StatCard
              title="Resolved"
              value={stats.resolved}
              icon={CheckCircle}
              color="text-green-600 dark:text-green-400"
            />
          </div>

          {/* Recent Activity */}
          <div className="rounded-lg border bg-card p-6">
            <div className="mb-6 flex items-center justify-between">
              <div>
                <h2 className="font-display text-xl font-semibold">Recent Requests</h2>
                <p className="text-sm text-muted-foreground">
                  Your latest activity
                </p>
              </div>
              <Link
                to="/requests"
                className="text-sm font-medium text-primary hover:underline"
              >
                View all
              </Link>
            </div>

            {recentRequests.length === 0 ? (
              <div className="py-12 text-center">
                <p className="text-muted-foreground mb-4">No requests yet</p>
                <Link
                  to="/requests/new"
                  className="inline-flex items-center gap-2 text-sm font-medium text-primary hover:underline"
                >
                  <Plus className="h-4 w-4" />
                  Create your first request
                </Link>
              </div>
            ) : (
              <div className="space-y-4">
                {recentRequests.map((request) => (
                  <Link
                    key={request.id}
                    to={`/requests/${request.id}`}
                    className="block rounded-lg border p-4 transition-colors hover:bg-accent animate-slide-up"
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <h3 className="font-medium">{request.title}</h3>
                        <p className="mt-1 text-sm text-muted-foreground line-clamp-2">
                          {request.description || 'No description'}
                        </p>
                        <div className="mt-2 flex flex-wrap items-center gap-3 text-xs text-muted-foreground">
                          <CategoryBadge category={request.category} />
                          <span>•</span>
                          <span>
                            {formatDistanceToNow(new Date(request.created_at), {
                              addSuffix: true,
                            })}
                          </span>
                        </div>
                      </div>
                      <div className="ml-4 flex gap-2">
                        <StatusBadge status={request.status} />
                        <PriorityBadge priority={request.priority} />
                      </div>
                    </div>
                  </Link>
                ))}
              </div>
            )}
          </div>
        </>
      )}
    </div>
  )
}

function StatCard({
  title,
  value,
  icon: Icon,
  color,
}: {
  title: string
  value: number
  icon: React.ElementType
  color: string
}) {
  return (
    <div className="rounded-lg border bg-card p-6">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium text-muted-foreground">{title}</p>
          <p className="mt-2 text-3xl font-bold tracking-tight">{value}</p>
        </div>
        <Icon className={`h-8 w-8 ${color}`} />
      </div>
    </div>
  )
}
