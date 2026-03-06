<script lang="ts">
  import { goto, invalidate } from '$app/navigation';
  import { onMount } from 'svelte';
  import { ExternalLink, MoreHorizontal } from '@lucide/svelte';

  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow
  } from '$lib/components/ui/table';
  import type { RealtimeServerEvent } from '$lib/realtime/types';
  import { subscribeRealtimeEvents, subscribeRealtimeResync } from '$lib/realtime/ws';
  import type { SupportRequest } from '$lib/types';
  import { priorityBadgeClass, readableStatus, statusBadgeClass } from '$lib/ui/request-style';
  import { cn } from '$lib/utils';
  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();

  let requests = $state<SupportRequest[]>([]);
  let meta = $state({
    request_id: '',
    page: 1,
    limit: 20,
    total: 0,
    total_pages: 0
  });
  let search = $state('');

  const filters = $derived({ ...data.filters });

  $effect(() => {
    requests = data.requests.map((request: SupportRequest) => ({ ...request }));
    meta = { ...data.meta };
    search = data.filters.q ?? '';
  });

  const pageNumbers = $derived.by(() => {
    const totalPages = Math.max(1, meta.total_pages);
    const current = Math.min(totalPages, Math.max(1, meta.page));
    const pages = new Set<number>([1, totalPages, current, current - 1, current - 2, current + 1, current + 2]);
    return Array.from(pages)
      .filter((pageNumber) => pageNumber >= 1 && pageNumber <= totalPages)
      .sort((a, b) => a - b);
  });

  function recalcPagination(): void {
    meta = {
      ...meta,
      total_pages: meta.total === 0 ? 0 : Math.ceil(meta.total / Math.max(1, meta.limit))
    };
  }

  function compareRequests(a: SupportRequest, b: SupportRequest): number {
    const sort = filters.sort;

    if (sort === 'created_at') {
      return new Date(a.created_at).getTime() - new Date(b.created_at).getTime();
    }

    if (sort === 'updated_at') {
      return new Date(a.updated_at).getTime() - new Date(b.updated_at).getTime();
    }

    return new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime();
  }

  function matchesServerFilters(request: SupportRequest): boolean {
    if (filters.status && request.status !== filters.status) return false;
    if (filters.category && request.category !== filters.category) return false;
    if (filters.priority && request.priority !== filters.priority) return false;
    if (filters.q) {
      const searchTerms = filters.q
        .trim()
        .toLowerCase()
        .split(/\s+/)
        .filter((term: string) => term.length > 0);

      if (searchTerms.length > 0) {
        const searchableText =
          `${request.title} ${request.description ?? ''} ${request.category} ${request.priority} ${request.status} ${request.assignee_email ?? ''} ${request.assignee_display_name ?? ''}`.toLowerCase();
        const allMatched = searchTerms.every((term: string) => searchableText.includes(term));

        if (!allMatched) {
          return false;
        }
      }
    }

    return true;
  }

  function upsertFromRealtime(request: SupportRequest, mode: 'created' | 'patch'): void {
    const index = requests.findIndex((item) => item.id === request.id);
    const shouldInclude = matchesServerFilters(request);

    if (index >= 0) {
      if (!shouldInclude) {
        requests = requests.filter((item) => item.id !== request.id);
        meta = { ...meta, total: Math.max(0, meta.total - 1) };
        recalcPagination();
        return;
      }

      requests[index] = request;
      requests = [...requests].sort(compareRequests).slice(0, Math.max(1, meta.limit));
      return;
    }

    if (!shouldInclude) {
      return;
    }

    if (mode === 'created') {
      meta = { ...meta, total: meta.total + 1 };
      recalcPagination();
    }

    requests = [...requests, request].sort(compareRequests).slice(0, Math.max(1, meta.limit));
  }

  function removeFromRealtime(requestId: string, statusHint?: string): void {
    const index = requests.findIndex((item) => item.id === requestId);

    if (index >= 0) {
      requests = requests.filter((item) => item.id !== requestId);
      meta = { ...meta, total: Math.max(0, meta.total - 1) };
      recalcPagination();
      return;
    }

    if (statusHint && filters.status && filters.status === statusHint && meta.total > 0) {
      meta = { ...meta, total: Math.max(0, meta.total - 1) };
      recalcPagination();
    }
  }

  function handleRealtimeEvent(event: RealtimeServerEvent): void {
    switch (event.type) {
      case 'request.created':
        upsertFromRealtime(event.payload.request, 'created');
        return;
      case 'request.patch':
        upsertFromRealtime(event.payload.request, 'patch');
        return;
      case 'request.deleted':
        removeFromRealtime(event.payload.id, event.payload.status);
        return;
      default:
        return;
    }
  }

  onMount(() => {
    const unsubscribeEvents = subscribeRealtimeEvents(handleRealtimeEvent);
    const unsubscribeResync = subscribeRealtimeResync(() => {
      void invalidate('reqstly:requests:list');
    });

    return () => {
      unsubscribeEvents();
      unsubscribeResync();
    };
  });

  function updateQuery(query: Record<string, string | number | null>): void {
    const next = new URL(window.location.href);

    for (const [key, value] of Object.entries(query)) {
      if (value === null || value === '' || value === 0) {
        next.searchParams.delete(key);
      } else {
        next.searchParams.set(key, String(value));
      }
    }

    void goto(`${next.pathname}${next.search}`);
  }

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleString();
  }

  async function copyRequestId(requestId: string): Promise<void> {
    try {
      await navigator.clipboard.writeText(requestId);
    } catch {
      // Ignore clipboard failures in unsupported contexts.
    }
  }
</script>

<section class="grid gap-4">
  <Card class="surface-gradient border">
    <CardHeader class="gap-3">
      <div class="flex flex-wrap items-start justify-between gap-2">
        <div>
          <CardTitle>Requests</CardTitle>
          <CardDescription>Filter and track all your request records.</CardDescription>
        </div>
        <Button href="/requests/new">New Request</Button>
      </div>

      <div class="grid gap-2 sm:grid-cols-2 lg:grid-cols-5">
        <div class="sm:col-span-2 lg:col-span-2">
          <label class="sr-only" for="request_search">Search requests</label>
          <Input
            id="request_search"
            class="w-full"
            placeholder="Search title, status, category..."
            bind:value={search}
            onchange={() => updateQuery({ q: search, page: 1 })}
          />
        </div>

        <div>
          <label class="sr-only" for="request_status_filter">Filter by status</label>
          <select
            id="request_status_filter"
            class="border-input bg-background h-9 w-full rounded-md border px-3 text-sm"
            value={filters.status}
            onchange={(event) => updateQuery({ status: (event.currentTarget as HTMLSelectElement).value, page: 1 })}
          >
            <option value="">All status</option>
            {#each data.enums.status as option}
              <option value={option}>{readableStatus(option)}</option>
            {/each}
          </select>
        </div>

        <div>
          <label class="sr-only" for="request_category_filter">Filter by category</label>
          <select
            id="request_category_filter"
            class="border-input bg-background h-9 w-full rounded-md border px-3 text-sm"
            value={filters.category}
            onchange={(event) => updateQuery({ category: (event.currentTarget as HTMLSelectElement).value, page: 1 })}
          >
            <option value="">All categories</option>
            {#each data.enums.category as option}
              <option value={option}>{option}</option>
            {/each}
          </select>
        </div>

        <div>
          <label class="sr-only" for="request_priority_filter">Filter by priority</label>
          <select
            id="request_priority_filter"
            class="border-input bg-background h-9 w-full rounded-md border px-3 text-sm"
            value={filters.priority}
            onchange={(event) => updateQuery({ priority: (event.currentTarget as HTMLSelectElement).value, page: 1 })}
          >
            <option value="">All priorities</option>
            {#each data.enums.priority as option}
              <option value={option}>{option}</option>
            {/each}
          </select>
        </div>
      </div>

      <div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        <div class="w-full sm:w-auto">
          <label class="sr-only" for="request_sort">Sort requests</label>
          <select
            id="request_sort"
            class="border-input bg-background h-9 w-full rounded-md border px-3 text-sm sm:w-auto"
            value={filters.sort}
            onchange={(event) => updateQuery({ sort: (event.currentTarget as HTMLSelectElement).value, page: 1 })}
          >
            <option value="-updated_at">Last updated (newest)</option>
            <option value="updated_at">Last updated (oldest)</option>
            <option value="created_at">Created (oldest first)</option>
          </select>
        </div>

        <p class="font-mono text-xs text-muted-foreground sm:text-right">
          total: {meta.total} · page {meta.page} / {Math.max(1, meta.total_pages)}
        </p>
      </div>
    </CardHeader>
  </Card>

  {#if data.backendError}
    <Card class="border-destructive/40 bg-destructive/10">
      <CardContent class="py-4 text-sm text-destructive">{data.backendError}</CardContent>
    </Card>
  {:else if requests.length === 0}
    <Card class="surface-gradient border">
      <CardContent class="flex flex-col items-center gap-3 py-12 text-center">
        <h3 class="font-heading text-xl">
          {filters.q || filters.status || filters.category || filters.priority ? 'No matching requests' : 'No requests yet'}
        </h3>
        <p class="max-w-md text-sm text-muted-foreground">
          {filters.q || filters.status || filters.category || filters.priority
            ? 'Try adjusting your search or filters.'
            : 'Create your first request to get started. You can then track updates and audit changes.'}
        </p>
        {#if !(filters.q || filters.status || filters.category || filters.priority)}
          <Button href="/requests/new">Create Request</Button>
        {/if}
      </CardContent>
    </Card>
  {:else}
    <Card class="surface-gradient border">
      <CardContent class="p-3 sm:p-4 md:p-0">
        <div class="grid gap-2 md:hidden">
          {#each requests as request}
            <article class="rounded-xl border border-border/80 bg-background/85 p-3">
              <div class="flex flex-wrap items-start justify-between gap-2">
                <a href={`/requests/${request.id}`} class="line-clamp-2 min-w-0 text-sm font-semibold hover:underline">
                  {request.title}
                </a>
                <div class="flex shrink-0 gap-1">
                  <Badge class={cn('capitalize', priorityBadgeClass(request.priority))}>{request.priority}</Badge>
                  <Badge class={cn('capitalize', statusBadgeClass(request.status))}>{readableStatus(request.status)}</Badge>
                </div>
              </div>

              <p class="mt-1 text-xs text-muted-foreground">
                {request.category} · updated {formatDate(request.updated_at)}
              </p>

              <p class="mt-2 text-xs text-muted-foreground">
                assignee:
                <span class="font-medium text-foreground">
                  {request.assignee_display_name ?? request.assignee_email ?? 'Unassigned'}
                </span>
              </p>

              <div class="mt-3 flex items-center gap-2">
                <Button href={`/requests/${request.id}`} size="sm" class="h-9 flex-1">Open detail</Button>
                <Button type="button" variant="outline" size="sm" class="h-9" onclick={() => copyRequestId(request.id)}>
                  Copy ID
                </Button>
              </div>
            </article>
          {/each}
        </div>

        <div class="hidden overflow-x-auto md:block">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Title</TableHead>
                <TableHead>Category</TableHead>
                <TableHead>Priority</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Assignee</TableHead>
                <TableHead>Last updated</TableHead>
                <TableHead class="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {#each requests as request}
                <TableRow>
                  <TableCell class="max-w-[300px]">
                    <p class="line-clamp-1 font-medium">{request.title}</p>
                  </TableCell>
                  <TableCell>{request.category}</TableCell>
                  <TableCell>
                    <Badge class={cn('capitalize', priorityBadgeClass(request.priority))}>{request.priority}</Badge>
                  </TableCell>
                  <TableCell>
                    <Badge class={cn('capitalize', statusBadgeClass(request.status))}>{readableStatus(request.status)}</Badge>
                  </TableCell>
                  <TableCell class="max-w-[220px]">
                    {#if request.assignee_email}
                      <p class="line-clamp-1 text-sm font-medium">{request.assignee_display_name ?? request.assignee_email}</p>
                      <p class="line-clamp-1 text-xs text-muted-foreground">{request.assignee_email}</p>
                    {:else}
                      <span class="text-xs text-muted-foreground">Unassigned</span>
                    {/if}
                  </TableCell>
                  <TableCell class="text-xs text-muted-foreground">{formatDate(request.updated_at)}</TableCell>
                  <TableCell class="text-right">
                    <details class="relative inline-block text-left">
                      <summary
                        class="border-border bg-background hover:bg-accent inline-flex h-9 w-9 cursor-pointer list-none items-center justify-center rounded-md border"
                        aria-label="Open actions menu"
                        aria-haspopup="menu"
                        aria-controls={`request-actions-${request.id}`}
                      >
                        <MoreHorizontal class="size-4" />
                      </summary>
                      <div
                        id={`request-actions-${request.id}`}
                        role="menu"
                        aria-label={`Actions for ${request.title}`}
                        class="absolute right-0 z-20 mt-1 w-44 rounded-md border border-border bg-popover p-1 shadow-sm"
                      >
                        <a
                          role="menuitem"
                          class="hover:bg-accent hover:text-accent-foreground flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm"
                          href={`/requests/${request.id}`}
                        >
                          <ExternalLink class="size-3.5" />
                          Open detail
                        </a>
                        <button
                          type="button"
                          role="menuitem"
                          class="hover:bg-accent hover:text-accent-foreground block w-full rounded-sm px-2 py-1.5 text-left text-sm"
                          onclick={() => copyRequestId(request.id)}
                        >
                          Copy request ID
                        </button>
                      </div>
                    </details>
                  </TableCell>
                </TableRow>
              {/each}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>

    <div class="flex flex-wrap items-center justify-between gap-2">
      <p class="font-mono text-xs text-muted-foreground">page {meta.page} of {Math.max(1, meta.total_pages)}</p>

      <div class="flex flex-wrap items-center justify-end gap-2">
        <Button variant="outline" disabled={meta.page <= 1} onclick={() => updateQuery({ page: meta.page - 1 })}>
          Previous
        </Button>
        {#each pageNumbers as pageNumber}
          <Button
            variant={pageNumber === meta.page ? 'default' : 'outline'}
            aria-label={`Go to page ${pageNumber}`}
            onclick={() => updateQuery({ page: pageNumber })}
          >
            {pageNumber}
          </Button>
        {/each}
        <Button
          variant="outline"
          disabled={meta.total_pages === 0 || meta.page >= meta.total_pages}
          onclick={() => updateQuery({ page: meta.page + 1 })}
        >
          Next
        </Button>
      </div>
    </div>
  {/if}
</section>
