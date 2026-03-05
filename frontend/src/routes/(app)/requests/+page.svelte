<script lang="ts">
  import { goto } from '$app/navigation';
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
  import { cn } from '$lib/utils';
  import { priorityBadgeClass, readableStatus, statusBadgeClass } from '$lib/ui/request-style';
  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();

  let search = $state('');

  $effect(() => {
    search = data.filters.q ?? '';
  });

  const filtered = $derived.by(() => {
    const term = search.trim().toLowerCase();
    if (term.length === 0) return data.requests;

    return data.requests.filter((request: (typeof data.requests)[number]) => {
      const composite = `${request.title} ${request.category} ${request.priority} ${request.status}`;
      return composite.toLowerCase().includes(term);
    });
  });

  const pageNumbers = $derived.by(() => {
    const totalPages = Math.max(1, data.meta.total_pages);
    const current = Math.min(totalPages, Math.max(1, data.meta.page));
    const pages = new Set<number>([1, totalPages, current, current - 1, current - 2, current + 1, current + 2]);
    return Array.from(pages)
      .filter((pageNumber) => pageNumber >= 1 && pageNumber <= totalPages)
      .sort((a, b) => a - b);
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

<section class="grid gap-3">
  <Card class="surface-gradient border">
    <CardHeader class="gap-3">
      <div class="flex flex-wrap items-end justify-between gap-2">
        <div>
          <CardTitle>Requests</CardTitle>
          <CardDescription>Filter and track all your request records.</CardDescription>
        </div>
        <Button href="/requests/new">New Request</Button>
      </div>

      <div class="grid gap-2 md:grid-cols-5">
        <div class="md:col-span-2">
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
            value={data.filters.status}
            onchange={(event) =>
              updateQuery({ status: (event.currentTarget as HTMLSelectElement).value, page: 1 })}
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
            value={data.filters.category}
            onchange={(event) =>
              updateQuery({ category: (event.currentTarget as HTMLSelectElement).value, page: 1 })}
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
            value={data.filters.priority}
            onchange={(event) =>
              updateQuery({ priority: (event.currentTarget as HTMLSelectElement).value, page: 1 })}
          >
            <option value="">All priorities</option>
            {#each data.enums.priority as option}
              <option value={option}>{option}</option>
            {/each}
          </select>
        </div>
      </div>

      <div class="flex items-center justify-between gap-2">
        <div>
          <label class="sr-only" for="request_sort">Sort requests</label>
          <select
            id="request_sort"
            class="border-input bg-background h-9 rounded-md border px-3 text-sm"
            value={data.filters.sort}
            onchange={(event) =>
              updateQuery({ sort: (event.currentTarget as HTMLSelectElement).value, page: 1 })}
          >
            <option value="-updated_at">Last updated (newest)</option>
            <option value="updated_at">Last updated (oldest)</option>
            <option value="created_at">Created (oldest first)</option>
          </select>
        </div>

        <p class="font-mono text-xs text-muted-foreground">
          total: {data.meta.total} · page {data.meta.page} / {Math.max(1, data.meta.total_pages)}
        </p>
      </div>
    </CardHeader>
  </Card>

  {#if data.backendError}
    <Card class="border-destructive/40 bg-destructive/10">
      <CardContent class="py-4 text-sm text-destructive">{data.backendError}</CardContent>
    </Card>
  {:else if filtered.length === 0}
    <Card class="surface-gradient border">
      <CardContent class="flex flex-col items-center gap-3 py-12 text-center">
        <h3 class="font-heading text-xl">No requests yet</h3>
        <p class="max-w-md text-sm text-muted-foreground">
          Create your first request to get started. You can then track updates and audit changes.
        </p>
        <Button href="/requests/new">Create Request</Button>
      </CardContent>
    </Card>
  {:else}
    <Card class="surface-gradient border">
      <CardContent class="overflow-x-auto p-0">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Title</TableHead>
              <TableHead>Category</TableHead>
              <TableHead>Priority</TableHead>
              <TableHead>Status</TableHead>
              <TableHead>Last updated</TableHead>
              <TableHead class="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {#each filtered as request}
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
                <TableCell class="text-xs text-muted-foreground">{formatDate(request.updated_at)}</TableCell>
                <TableCell class="text-right">
                  <details class="relative inline-block text-left">
                    <summary
                      class="border-border bg-background hover:bg-accent inline-flex h-9 w-9 cursor-pointer list-none items-center justify-center rounded-md border"
                      aria-label="Open actions menu"
                    >
                      <MoreHorizontal class="size-4" />
                    </summary>
                    <div class="absolute right-0 z-20 mt-1 w-44 rounded-md border border-border bg-popover p-1 shadow-sm">
                      <a
                        class="hover:bg-accent hover:text-accent-foreground flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm"
                        href={`/requests/${request.id}`}
                      >
                        <ExternalLink class="size-3.5" />
                        Open detail
                      </a>
                      <button
                        type="button"
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
      </CardContent>
    </Card>

    <div class="flex flex-wrap items-center justify-end gap-2">
      <Button
        variant="outline"
        disabled={data.meta.page <= 1}
        onclick={() => updateQuery({ page: data.meta.page - 1 })}
      >
        Previous
      </Button>
      {#each pageNumbers as pageNumber}
        <Button
          variant={pageNumber === data.meta.page ? 'default' : 'outline'}
          aria-label={`Go to page ${pageNumber}`}
          onclick={() => updateQuery({ page: pageNumber })}
        >
          {pageNumber}
        </Button>
      {/each}
      <Button
        variant="outline"
        disabled={data.meta.total_pages === 0 || data.meta.page >= data.meta.total_pages}
        onclick={() => updateQuery({ page: data.meta.page + 1 })}
      >
        Next
      </Button>
    </div>
  {/if}
</section>
