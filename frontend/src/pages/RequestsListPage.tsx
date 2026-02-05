import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { api } from '@/lib/api'
import { StatusBadge } from '@/components/requests/StatusBadge'
import { PriorityBadge } from '@/components/requests/PriorityBadge'
import { CategoryBadge } from '@/components/requests/CategoryBadge'
import { Plus, Search } from 'lucide-react'
import { Link } from 'react-router-dom'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { formatDistanceToNow } from 'date-fns'

export function RequestsListPage() {
  const [filters, setFilters] = useState({
    status: '',
    category: '',
    search: '',
  })

  const { data: requests, isLoading } = useQuery({
    queryKey: ['requests', filters.status, filters.category],
    queryFn: () => api.list({
      status: filters.status || undefined,
      category: filters.category || undefined,
    }),
  })

  const filteredRequests = requests?.filter(request =>
    request.title.toLowerCase().includes(filters.search.toLowerCase()) ||
    request.description?.toLowerCase().includes(filters.search.toLowerCase())
  ) || []

  return (
    <div className="p-8 animate-fade-in">
      {/* Header */}
      <div className="mb-8 flex items-center justify-between">
        <div>
          <h1 className="font-display text-3xl font-bold tracking-tight">
            Requests
          </h1>
          <p className="mt-2 text-muted-foreground">
            Manage and track all your requests
          </p>
        </div>
        <Link to="/requests/new">
          <Button className="gap-2">
            <Plus className="h-4 w-4" />
            New Request
          </Button>
        </Link>
      </div>

      {/* Filters */}
      <div className="mb-6 flex gap-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="Search requests..."
            value={filters.search}
            onChange={(e) => setFilters({ ...filters, search: e.target.value })}
            className="pl-9"
          />
        </div>
        <select
          value={filters.status}
          onChange={(e) => setFilters({ ...filters, status: e.target.value })}
          className="rounded-md border border-input bg-background px-3 py-2 text-sm"
        >
          <option value="">All Status</option>
          <option value="open">Open</option>
          <option value="in_progress">In Progress</option>
          <option value="resolved">Resolved</option>
        </select>
        <select
          value={filters.category}
          onChange={(e) => setFilters({ ...filters, category: e.target.value })}
          className="rounded-md border border-input bg-background px-3 py-2 text-sm"
        >
          <option value="">All Categories</option>
          <option value="IT">IT</option>
          <option value="Ops">Ops</option>
          <option value="Admin">Admin</option>
          <option value="HR">HR</option>
        </select>
      </div>

      {/* Requests List */}
      {isLoading ? (
        <div className="py-12 text-center text-muted-foreground">
          Loading requests...
        </div>
      ) : filteredRequests.length === 0 ? (
        <div className="py-12 text-center">
          <p className="text-muted-foreground mb-4">No requests found</p>
          <Link
            to="/requests/new"
            className="inline-flex items-center gap-2 text-sm font-medium text-primary hover:underline"
          >
            <Plus className="h-4 w-4" />
            Create your first request
          </Link>
        </div>
      ) : (
        <div className="space-y-3">
          {filteredRequests.map((request) => (
            <Link
              key={request.id}
              to={`/requests/${request.id}`}
              className="block rounded-lg border bg-card p-6 transition-all hover:shadow-md animate-slide-up"
            >
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <h3 className="text-lg font-semibold">{request.title}</h3>
                  <p className="mt-1 text-muted-foreground line-clamp-2">
                    {request.description || 'No description provided'}
                  </p>
                  <div className="mt-3 flex flex-wrap items-center gap-3">
                    <CategoryBadge category={request.category} />
                    <span className="text-xs text-muted-foreground">
                      {formatDistanceToNow(new Date(request.created_at), {
                        addSuffix: true,
                      })}
                    </span>
                  </div>
                </div>
                <div className="ml-6 flex gap-2">
                  <StatusBadge status={request.status} />
                  <PriorityBadge priority={request.priority} />
                </div>
              </div>
            </Link>
          ))}
        </div>
      )}
    </div>
  )
}
