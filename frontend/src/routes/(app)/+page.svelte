<script lang="ts">
  import { invalidate } from '$app/navigation';
  import { onMount } from 'svelte';
  import { ArrowRight, Clock3, Plus, Search } from '@lucide/svelte';

  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import type { RealtimeServerEvent } from '$lib/realtime/types';
  import { subscribeRealtimeEvents, subscribeRealtimeResync } from '$lib/realtime/ws';
  import type { SupportRequest } from '$lib/types';
  import { cn } from '$lib/utils';
  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();

  let stats = $state({ open: 0, in_progress: 0, resolved: 0 });
  let recentRequests = $state<SupportRequest[]>([]);

  $effect(() => {
    stats = { ...data.stats };
    recentRequests = data.recentRequests.map((request: SupportRequest) => ({ ...request }));
  });

  function formatRelative(iso: string): string {
    const date = new Date(iso);
    const deltaMinutes = Math.floor((Date.now() - date.getTime()) / 60000);

    if (deltaMinutes < 1) return 'just now';
    if (deltaMinutes < 60) return `${deltaMinutes}m ago`;
    const hours = Math.floor(deltaMinutes / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    return `${days}d ago`;
  }

  const statusClass = (status: string): string =>
    ({
      open: 'border border-[#c8a57a]/40 bg-[#f8efe2] text-[#7b4e2b]',
      in_progress: 'border border-[#8fb6b5]/50 bg-[#e4f1f0] text-[#1d6161]',
      resolved: 'border border-[#95c2a5]/45 bg-[#ebf6ee] text-[#285d3e]'
    }[status] ?? 'border border-border bg-muted text-foreground');

  const priorityClass = (priority: string): string =>
    ({
      low: 'border border-[#9bb9d4]/50 bg-[#eaf2f9] text-[#345f89]',
      medium: 'border border-[#d3b170]/50 bg-[#fdf3de] text-[#8b5d12]',
      high: 'border border-[#d69580]/55 bg-[#fce9e4] text-[#8a3c2b]'
    }[priority] ?? 'border border-border bg-muted text-foreground');

  function updateStatusCount(status: string, delta: number): void {
    if (!(status in stats)) {
      return;
    }

    const key = status as keyof typeof stats;
    stats = {
      ...stats,
      [key]: Math.max(0, stats[key] + delta)
    };
  }

  function upsertRecentRequest(request: SupportRequest): void {
    recentRequests = [request, ...recentRequests.filter((item) => item.id !== request.id)]
      .sort((left, right) => new Date(right.updated_at).getTime() - new Date(left.updated_at).getTime())
      .slice(0, 6);
  }

  function handleRealtimeEvent(event: RealtimeServerEvent): void {
    switch (event.type) {
      case 'request.created': {
        const request = event.payload.request;
        updateStatusCount(request.status, 1);
        upsertRecentRequest(request);
        return;
      }
      case 'request.patch': {
        const request = event.payload.request;
        const previousStatus = event.payload.previous_status;

        if (previousStatus !== request.status) {
          updateStatusCount(previousStatus, -1);
          updateStatusCount(request.status, 1);
        }

        upsertRecentRequest(request);
        return;
      }
      case 'request.deleted': {
        updateStatusCount(event.payload.status, -1);
        recentRequests = recentRequests.filter((item) => item.id !== event.payload.id);
        return;
      }
      default:
        return;
    }
  }

  onMount(() => {
    const unsubscribeEvents = subscribeRealtimeEvents(handleRealtimeEvent);
    const unsubscribeResync = subscribeRealtimeResync(() => {
      void invalidate('reqstly:dashboard');
    });

    return () => {
      unsubscribeEvents();
      unsubscribeResync();
    };
  });
</script>

<section class="grid gap-6">
  <Card class="border-border/80 bg-card">
    <CardHeader class="gap-4 md:flex-row md:items-center md:justify-between">
      <div class="space-y-1">
        <CardTitle class="text-3xl">Welcome back</CardTitle>
        <CardDescription class="max-w-[62ch] text-sm">
          Manage open work, keep responders aligned, and close requests with complete context.
        </CardDescription>
      </div>
      <div class="flex flex-wrap gap-2">
        <Button href="/requests/new" class="h-11 px-4">
          <Plus class="size-4" />
          Create Request
        </Button>
        <Button href="/requests" variant="outline" class="h-11 px-4">
          <Search class="size-4" />
          View Requests
        </Button>
      </div>
    </CardHeader>
  </Card>

  <div class="grid gap-4 md:grid-cols-3">
    <Card class="border-border/80 bg-card">
      <CardHeader class="pb-2">
        <CardDescription class="uppercase tracking-[0.14em]">Open Requests</CardDescription>
      </CardHeader>
      <CardContent>
        <p class="font-heading text-4xl font-semibold leading-none">{stats.open}</p>
      </CardContent>
    </Card>

    <Card class="border-border/80 bg-card">
      <CardHeader class="pb-2">
        <CardDescription class="uppercase tracking-[0.14em]">In Progress</CardDescription>
      </CardHeader>
      <CardContent>
        <p class="font-heading text-4xl font-semibold leading-none">{stats.in_progress}</p>
      </CardContent>
    </Card>

    <Card class="border-border/80 bg-card">
      <CardHeader class="pb-2">
        <CardDescription class="uppercase tracking-[0.14em]">Closed Requests</CardDescription>
      </CardHeader>
      <CardContent>
        <p class="font-heading text-4xl font-semibold leading-none">{stats.resolved}</p>
      </CardContent>
    </Card>
  </div>

  <Card class="border-border/80 bg-card">
    <CardHeader class="flex-row flex-wrap items-center justify-between gap-2">
      <div class="space-y-1">
        <CardTitle class="text-2xl">Recent activity</CardTitle>
        <CardDescription>Latest requests created or updated.</CardDescription>
      </div>
      <Button href="/requests" variant="ghost" class="h-11">
        Full queue
        <ArrowRight class="size-4" />
      </Button>
    </CardHeader>

    <CardContent>
      {#if recentRequests.length === 0}
        <div class="rounded-xl border border-dashed border-border/80 bg-muted/20 p-8 text-center">
          <p class="text-sm text-muted-foreground">No requests yet. Create your first request to get started.</p>
          <Button class="mt-4 h-11" href="/requests/new">
            <Plus class="size-4" />
            Create Request
          </Button>
        </div>
      {:else}
        <ul class="grid gap-2" aria-live="polite">
          {#each recentRequests as item, index}
            <li
              class="rounded-xl border border-border/80 bg-background px-4 py-3"
              style={`animation-delay:${index * 35}ms`}
            >
              <div class="flex flex-col gap-2 md:flex-row md:items-center md:justify-between">
                <div class="min-w-0">
                  <a class="line-clamp-1 text-sm font-semibold hover:underline" href={`/requests/${item.id}`}>
                    {item.title}
                  </a>
                  <p class="mt-1 inline-flex items-center gap-1 text-xs text-muted-foreground">
                    <Clock3 class="size-3.5" />
                    {formatRelative(item.updated_at)} · {item.category}
                  </p>
                </div>
                <div class="flex gap-1">
                  <Badge class={cn('capitalize', priorityClass(item.priority))}>{item.priority}</Badge>
                  <Badge class={cn('capitalize', statusClass(item.status))}>{item.status.replace('_', ' ')}</Badge>
                </div>
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    </CardContent>
  </Card>
</section>
