<script lang="ts">
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import * as Dialog from '$lib/components/ui/dialog';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Textarea } from '$lib/components/ui/textarea';
  import { cn } from '$lib/utils';
  import { priorityBadgeClass, readableStatus, statusBadgeClass } from '$lib/ui/request-style';
  import type { ErrorDetail } from '$lib/types';

  import type { ActionData, PageData } from './$types';

  let { data, form } = $props<{ data: PageData; form: ActionData }>();

  let deleteOpen = $state(false);

  const values = $derived.by(() => ({
    title: form?.values?.title ?? data.request.title,
    description: form?.values?.description ?? data.request.description ?? '',
    category: form?.values?.category ?? data.request.category,
    status: form?.values?.status ?? data.request.status,
    priority: form?.values?.priority ?? data.request.priority
  }));

  const errorFor = (field: string): string | undefined =>
    (form?.details as ErrorDetail[] | undefined)?.find((item: ErrorDetail) => item.field === field)?.message;

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleString();
  }

  function auditSummary(event: (typeof data.audit)[number]): string {
    if (event.action === 'created') {
      return 'Request created.';
    }

    if (event.action === 'deleted') {
      return 'Request deleted.';
    }

    if (event.action === 'status_changed') {
      const previous = event.old_value?.status;
      const current = event.new_value?.status;
      if (typeof previous === 'string' && typeof current === 'string') {
        return `Status changed from ${readableStatus(previous)} to ${readableStatus(current)}.`;
      }
      return 'Status updated.';
    }

    const oldRecord =
      event.old_value && typeof event.old_value === 'object' ? (event.old_value as Record<string, unknown>) : {};
    const newRecord =
      event.new_value && typeof event.new_value === 'object' ? (event.new_value as Record<string, unknown>) : {};

    const changedKeys = Object.keys(newRecord).filter((key) => oldRecord[key] !== newRecord[key]);
    if (changedKeys.length === 0) {
      return 'Request updated.';
    }

    return `Updated: ${changedKeys
      .slice(0, 3)
      .map((key) => key.replaceAll('_', ' '))
      .join(', ')}${changedKeys.length > 3 ? '…' : ''}.`;
  }
</script>

<section class="grid gap-3 lg:grid-cols-[1fr_320px]">
  <Card class="surface-gradient border">
    <CardHeader class="gap-2">
      <div class="flex flex-wrap items-center justify-between gap-2">
        <div>
          <CardTitle>{data.request.title}</CardTitle>
          <CardDescription>Request ID: <span class="font-mono">{data.request.id}</span></CardDescription>
        </div>
        <div class="flex gap-1">
          <Badge class={cn('capitalize', priorityBadgeClass(values.priority))}>{values.priority}</Badge>
          <Badge class={cn('capitalize', statusBadgeClass(values.status))}>{readableStatus(values.status)}</Badge>
        </div>
      </div>
      <div class="grid gap-2 rounded-xl border border-border bg-background/70 p-3 text-xs sm:grid-cols-4">
        <div>
          <p class="text-muted-foreground">Category</p>
          <p class="font-semibold">{data.request.category}</p>
        </div>
        <div>
          <p class="text-muted-foreground">Priority</p>
          <p class="font-semibold capitalize">{values.priority}</p>
        </div>
        <div>
          <p class="text-muted-foreground">Status</p>
          <p class="font-semibold">{readableStatus(values.status)}</p>
        </div>
        <div>
          <p class="text-muted-foreground">Created</p>
          <p class="font-semibold">{formatDate(data.request.created_at)}</p>
        </div>
      </div>
      <p class="text-xs text-muted-foreground">Last updated: {formatDate(data.request.updated_at)}</p>
    </CardHeader>

    <CardContent>
      <form method="POST" action="?/update" class="grid gap-4">
        {#if form?.message}
          <div
            class={cn(
              'rounded-lg border px-3 py-2 text-sm',
              form.success
                ? 'border-emerald-500/40 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300'
                : 'border-destructive/40 bg-destructive/10 text-destructive'
            )}
          >
            {form.message}
          </div>
        {/if}

        <div class="grid gap-2">
          <Label for="title">Title</Label>
          <Input id="title" name="title" value={values.title} required />
          {#if errorFor('title')}
            <p class="text-xs font-semibold text-destructive">{errorFor('title')}</p>
          {/if}
        </div>

        <div class="grid gap-2">
          <Label for="description">Description</Label>
          <Textarea id="description" name="description" rows={7} value={values.description} />
          {#if errorFor('description')}
            <p class="text-xs font-semibold text-destructive">{errorFor('description')}</p>
          {/if}
        </div>

        <div class="grid gap-3 md:grid-cols-3">
          <div class="grid gap-2">
            <Label for="category">Category</Label>
            <select id="category" name="category" class="border-input bg-background h-9 rounded-md border px-3 text-sm" value={values.category}>
              {#each data.enums.category as option}
                <option value={option}>{option}</option>
              {/each}
            </select>
            {#if errorFor('category')}
              <p class="text-xs font-semibold text-destructive">{errorFor('category')}</p>
            {/if}
          </div>

          <div class="grid gap-2">
            <Label for="priority">Priority</Label>
            <select id="priority" name="priority" class="border-input bg-background h-9 rounded-md border px-3 text-sm" value={values.priority}>
              {#each data.enums.priority as option}
                <option value={option}>{option}</option>
              {/each}
            </select>
            {#if errorFor('priority')}
              <p class="text-xs font-semibold text-destructive">{errorFor('priority')}</p>
            {/if}
          </div>

          <div class="grid gap-2">
            <Label for="status">Status</Label>
            <select id="status" name="status" class="border-input bg-background h-9 rounded-md border px-3 text-sm" value={values.status}>
              {#each data.enums.status as option}
                <option value={option}>{readableStatus(option)}</option>
              {/each}
            </select>
            {#if errorFor('status')}
              <p class="text-xs font-semibold text-destructive">{errorFor('status')}</p>
            {/if}
          </div>
        </div>

        <div class="flex flex-wrap justify-end gap-2">
          <Button type="submit">Save changes</Button>
          <Button type="button" variant="destructive" onclick={() => (deleteOpen = true)}>
            Delete request
          </Button>
        </div>
      </form>
    </CardContent>
  </Card>

  <Card class="surface-gradient border">
    <CardHeader>
      <CardTitle>Audit timeline</CardTitle>
      <CardDescription>Newest changes first.</CardDescription>
    </CardHeader>
    <CardContent>
      {#if data.audit.length === 0}
        <p class="text-sm text-muted-foreground">No audit entries yet.</p>
      {:else}
        <ol class="grid gap-3">
          {#each data.audit as event}
            <li class="rounded-lg border bg-background/80 p-3">
              <div class="flex items-center justify-between gap-2">
                <Badge variant="outline" class="capitalize">{event.action.replace('_', ' ')}</Badge>
                <span class="font-mono text-[11px] text-muted-foreground">{formatDate(event.created_at)}</span>
              </div>
              <p class="mt-2 text-sm">{auditSummary(event)}</p>
              <p class="mt-2 text-xs text-muted-foreground">actor: {event.actor_user_id}</p>
            </li>
          {/each}
        </ol>
      {/if}
    </CardContent>
  </Card>
</section>

<Dialog.Root bind:open={deleteOpen}>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>Delete Request</Dialog.Title>
      <Dialog.Description>
        Are you sure you want to delete this request? This action cannot be undone.
      </Dialog.Description>
    </Dialog.Header>

    <Dialog.Footer class="mt-4">
      <Button type="button" variant="outline" onclick={() => (deleteOpen = false)}>Cancel</Button>
      <form method="POST" action="?/delete">
        <Button type="submit" variant="destructive">Delete</Button>
      </form>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
