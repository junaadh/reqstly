<script lang="ts">
  import { enhance } from '$app/forms';
  import { goto, invalidate } from '$app/navigation';
  import type { SubmitFunction } from '@sveltejs/kit';
  import { onDestroy, onMount } from 'svelte';

  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import * as Dialog from '$lib/components/ui/dialog';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Textarea } from '$lib/components/ui/textarea';
  import type { RealtimeServerEvent } from '$lib/realtime/types';
  import { subscribeRealtimeEvents, subscribeRealtimeResync } from '$lib/realtime/ws';
  import type { AssigneeSuggestion, AuditLog, ErrorDetail, SupportRequest } from '$lib/types';
  import { priorityBadgeClass, readableStatus, statusBadgeClass } from '$lib/ui/request-style';
  import { cn } from '$lib/utils';

  import type { ActionData, PageData } from './$types';

  let { data, form } = $props<{ data: PageData; form: ActionData }>();

  let requestRecord = $state<SupportRequest>({
    id: '',
    owner_user_id: '',
    title: '',
    description: null,
    category: 'IT',
    status: 'open',
    priority: 'medium',
    assignee_user_id: null,
    assignee_email: null,
    assignee_display_name: null,
    created_at: '',
    updated_at: ''
  });
  let auditEntries = $state<AuditLog[]>([]);

  let deleteOpen = $state(false);

  $effect(() => {
    requestRecord = { ...data.request };
    auditEntries = [...data.audit];
  });

  const hasValidationErrors = $derived.by(() => form?.success === false);

  const values = $derived.by(() => ({
    title: hasValidationErrors ? (form?.values?.title ?? requestRecord.title) : requestRecord.title,
    description: hasValidationErrors
      ? (form?.values?.description ?? requestRecord.description ?? '')
      : (requestRecord.description ?? ''),
    category: hasValidationErrors ? (form?.values?.category ?? requestRecord.category) : requestRecord.category,
    status: hasValidationErrors ? (form?.values?.status ?? requestRecord.status) : requestRecord.status,
    priority: hasValidationErrors ? (form?.values?.priority ?? requestRecord.priority) : requestRecord.priority,
    assignee_email: hasValidationErrors
      ? (form?.values?.assignee_email ?? requestRecord.assignee_email ?? '')
      : (requestRecord.assignee_email ?? '')
  }));

  const errorFor = (field: string): string | undefined =>
    (form?.details as ErrorDetail[] | undefined)?.find((item: ErrorDetail) => item.field === field)?.message;

  let assigneeQuery = $state('');
  let assigneeOpen = $state(false);
  let assigneeLoading = $state(false);
  let assigneeOptions = $state<AssigneeSuggestion[]>([]);
  let assigneeHighlightIndex = $state(-1);
  let assigneeBlurTimer: ReturnType<typeof setTimeout> | null = null;
  let assigneeDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  let assigneeRequestVersion = 0;

  $effect(() => {
    assigneeQuery = values.assignee_email;
    assigneeOptions = data.assigneeOptions;
  });

  const filteredAssignees = $derived.by(() => {
    const tokens = assigneeQuery
      .trim()
      .toLowerCase()
      .split(/\s+/)
      .filter((token: string) => token.length > 0);

    if (tokens.length === 0) {
      return assigneeOptions;
    }

    return assigneeOptions.filter((option) => {
      const haystack = `${option.display_name} ${option.email}`.toLowerCase();
      return tokens.every((token: string) => haystack.includes(token));
    });
  });

  const activeAssigneeOptionId = $derived.by(() => {
    if (!assigneeOpen) return undefined;
    if (assigneeHighlightIndex < 0 || assigneeHighlightIndex >= filteredAssignees.length) {
      return undefined;
    }
    return `assignee-option-detail-${assigneeHighlightIndex}`;
  });

  const assigneeDescribedBy = $derived.by(() => {
    const ids = ['assignee-help-detail'];

    if (assigneeLoading) {
      ids.push('assignee-loading-detail');
    }
    if (errorFor('assignee_email')) {
      ids.push('assignee-error-detail');
    }

    return ids.join(' ');
  });

  $effect(() => {
    if (!assigneeOpen) {
      assigneeHighlightIndex = -1;
      return;
    }

    if (filteredAssignees.length === 0) {
      assigneeHighlightIndex = -1;
      return;
    }

    if (assigneeHighlightIndex < 0 || assigneeHighlightIndex >= filteredAssignees.length) {
      assigneeHighlightIndex = 0;
    }
  });

  async function fetchAssigneeOptions(query: string): Promise<void> {
    const requestVersion = ++assigneeRequestVersion;
    assigneeLoading = true;

    const params = new URLSearchParams();
    params.set('limit', '50');
    if (query.trim().length > 0) {
      params.set('q', query.trim());
    }

    try {
      const response = await fetch(`/api/assignees/suggestions?${params.toString()}`);
      const payload = await response.json().catch(() => null);

      if (requestVersion !== assigneeRequestVersion) {
        return;
      }

      if (
        response.ok &&
        payload &&
        typeof payload === 'object' &&
        Array.isArray((payload as { data?: unknown }).data)
      ) {
        assigneeOptions = (payload as { data: AssigneeSuggestion[] }).data;
      }
    } finally {
      if (requestVersion === assigneeRequestVersion) {
        assigneeLoading = false;
      }
    }
  }

  function scheduleAssigneeFetch(query: string): void {
    if (assigneeDebounceTimer) {
      clearTimeout(assigneeDebounceTimer);
    }

    assigneeDebounceTimer = setTimeout(() => {
      void fetchAssigneeOptions(query);
    }, 120);
  }

  function openAssigneeList(): void {
    if (assigneeBlurTimer) {
      clearTimeout(assigneeBlurTimer);
      assigneeBlurTimer = null;
    }
    assigneeOpen = true;
    scheduleAssigneeFetch(assigneeQuery);
  }

  function closeAssigneeList(): void {
    assigneeBlurTimer = setTimeout(() => {
      assigneeOpen = false;
    }, 100);
  }

  function selectAssignee(email: string): void {
    assigneeQuery = email;
    assigneeHighlightIndex = -1;
    assigneeOpen = false;
  }

  function moveHighlight(direction: 1 | -1): void {
    if (filteredAssignees.length === 0) {
      assigneeHighlightIndex = -1;
      return;
    }

    if (assigneeHighlightIndex < 0) {
      assigneeHighlightIndex = direction > 0 ? 0 : filteredAssignees.length - 1;
      return;
    }

    assigneeHighlightIndex = Math.min(
      filteredAssignees.length - 1,
      Math.max(0, assigneeHighlightIndex + direction)
    );
  }

  function handleAssigneeKeydown(event: KeyboardEvent): void {
    if (event.key === 'Tab') {
      assigneeOpen = false;
      assigneeHighlightIndex = -1;
      return;
    }

    if (event.key === 'Escape') {
      if (assigneeOpen) {
        event.preventDefault();
      }
      assigneeOpen = false;
      assigneeHighlightIndex = -1;
      return;
    }

    if (event.key === 'ArrowDown') {
      event.preventDefault();
      if (!assigneeOpen) {
        openAssigneeList();
      }
      moveHighlight(1);
      return;
    }

    if (event.key === 'ArrowUp') {
      event.preventDefault();
      if (!assigneeOpen) {
        openAssigneeList();
      }
      moveHighlight(-1);
      return;
    }

    if (event.key === 'Enter' && assigneeOpen && assigneeHighlightIndex >= 0) {
      const option = filteredAssignees[assigneeHighlightIndex];
      if (!option) return;

      event.preventDefault();
      selectAssignee(option.email);
    }
  }

  function handleRealtimeEvent(event: RealtimeServerEvent): void {
    switch (event.type) {
      case 'request.patch': {
        if (event.payload.request.id !== requestRecord.id) return;
        requestRecord = { ...requestRecord, ...event.payload.request };
        form = undefined;
        return;
      }
      case 'request.deleted': {
        if (event.payload.id !== requestRecord.id) return;
        deleteOpen = false;
        form = undefined;
        void goto('/requests');
        return;
      }
      case 'audit.append': {
        const nextAudit = event.payload.audit;
        if (nextAudit.request_id !== requestRecord.id) return;
        auditEntries = [nextAudit, ...auditEntries.filter((entry) => entry.id !== nextAudit.id)].slice(0, 200);
        return;
      }
      default:
        return;
    }
  }

  type UpdateFormValues = {
    title?: string;
    description?: string | null;
    category?: string;
    status?: string;
    priority?: string;
    assignee_email?: string;
  };

  function applyLocalUpdateFromForm(valuesToApply: UpdateFormValues | undefined): void {
    if (!valuesToApply) {
      return;
    }

    const normalizedAssignee = valuesToApply.assignee_email?.trim() ?? '';
    const nextCategory = (valuesToApply.category ?? requestRecord.category) as SupportRequest['category'];
    const nextStatus = (valuesToApply.status ?? requestRecord.status) as SupportRequest['status'];
    const nextPriority = (valuesToApply.priority ?? requestRecord.priority) as SupportRequest['priority'];

    requestRecord = {
      ...requestRecord,
      title: valuesToApply.title ?? requestRecord.title,
      description: valuesToApply.description ?? null,
      category: nextCategory,
      status: nextStatus,
      priority: nextPriority,
      assignee_email: normalizedAssignee.length > 0 ? normalizedAssignee : null
    };
  }

  const enhanceUpdateForm: SubmitFunction = () => {
    return async ({ result, update }) => {
      await update();

      if (result.type === 'success') {
        const payload =
          result.data && typeof result.data === 'object'
            ? (result.data as { values?: UpdateFormValues })
            : undefined;
        applyLocalUpdateFromForm(payload?.values);
      }
    };
  };

  const enhanceDeleteForm: SubmitFunction = () => {
    return async ({ result, update }) => {
      if (result.type === 'redirect' || result.type === 'success') {
        deleteOpen = false;
        form = undefined;
        await goto('/requests');
        return;
      }

      await update();
    };
  };

  onMount(() => {
    const detailDependency = `reqstly:requests:detail:${data.request.id}`;
    const unsubscribeEvents = subscribeRealtimeEvents(handleRealtimeEvent);
    const unsubscribeResync = subscribeRealtimeResync(() => {
      void invalidate(detailDependency);
    });

    return () => {
      unsubscribeEvents();
      unsubscribeResync();
    };
  });

  onDestroy(() => {
    if (assigneeBlurTimer) {
      clearTimeout(assigneeBlurTimer);
    }
    if (assigneeDebounceTimer) {
      clearTimeout(assigneeDebounceTimer);
    }
  });

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleString();
  }

  function auditSummary(event: (typeof auditEntries)[number]): string {
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

<section class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_340px]">
  <Card class="surface-gradient border">
    <CardHeader class="gap-2">
      <div class="flex flex-wrap items-center justify-between gap-2">
        <div>
          <CardTitle>{requestRecord.title}</CardTitle>
          <CardDescription>Request ID: <span class="font-mono">{requestRecord.id}</span></CardDescription>
        </div>
        <div class="flex gap-1">
          <Badge class={cn('capitalize', priorityBadgeClass(values.priority))}>{values.priority}</Badge>
          <Badge class={cn('capitalize', statusBadgeClass(values.status))}>{readableStatus(values.status)}</Badge>
        </div>
      </div>
      <div class="grid gap-2 rounded-xl border border-border bg-background/70 p-3 text-xs sm:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-5">
        <div>
          <p class="text-muted-foreground">Category</p>
          <p class="font-semibold">{requestRecord.category}</p>
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
          <p class="font-semibold">{formatDate(requestRecord.created_at)}</p>
        </div>
        <div>
          <p class="text-muted-foreground">Assignee</p>
          <p class="font-semibold">{values.assignee_email || 'Unassigned'}</p>
        </div>
      </div>
      <p class="text-xs text-muted-foreground">Last updated: {formatDate(requestRecord.updated_at)}</p>
    </CardHeader>

    <CardContent class="grid gap-4 p-4 sm:p-5 lg:p-6">
      <form method="POST" action="?/update" class="grid gap-4" use:enhance={enhanceUpdateForm}>
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

        <div class="grid gap-2">
          <Label for="assignee_email">Assign to</Label>
          <div class="relative">
            <Input
              id="assignee_email"
              name="assignee_email"
              type="email"
              bind:value={assigneeQuery}
              placeholder="teammate@company.com"
              autocomplete="off"
              role="combobox"
              aria-haspopup="listbox"
              aria-autocomplete="list"
              aria-expanded={assigneeOpen}
              aria-controls="assignee-options-detail"
              aria-activedescendant={activeAssigneeOptionId}
              aria-describedby={assigneeDescribedBy}
              aria-invalid={errorFor('assignee_email') ? 'true' : undefined}
              onfocus={openAssigneeList}
              onclick={openAssigneeList}
              oninput={() => scheduleAssigneeFetch(assigneeQuery)}
              onkeydown={handleAssigneeKeydown}
              onblur={closeAssigneeList}
            />
            {#if assigneeOpen}
              <div
                id="assignee-options-detail"
                role="listbox"
                class="bg-popover absolute z-30 mt-1 max-h-64 w-full overflow-auto rounded-md border border-border p-1 shadow-md"
              >
                {#if filteredAssignees.length === 0}
                  <p class="px-2 py-2 text-xs text-muted-foreground">
                    No same-domain matches for that search. You can still submit a manual email.
                  </p>
                {:else}
                  {#each filteredAssignees as option, optionIndex}
                    <button
                      type="button"
                      id={`assignee-option-detail-${optionIndex}`}
                      class={cn(
                        'hover:bg-accent hover:text-accent-foreground w-full rounded-sm px-2 py-1.5 text-left',
                        optionIndex === assigneeHighlightIndex && 'bg-accent text-accent-foreground'
                      )}
                      role="option"
                      aria-selected={optionIndex === assigneeHighlightIndex}
                      onmousedown={(event) => event.preventDefault()}
                      onmouseenter={() => (assigneeHighlightIndex = optionIndex)}
                      onclick={() => selectAssignee(option.email)}
                    >
                      <p class="line-clamp-1 text-sm font-medium">{option.display_name}</p>
                      <p class="line-clamp-1 text-xs text-muted-foreground">
                        {option.email} · {option.assignment_count} assigned
                      </p>
                    </button>
                  {/each}
                {/if}
              </div>
            {/if}
          </div>
          {#if assigneeLoading}
            <p id="assignee-loading-detail" class="text-xs text-muted-foreground" aria-live="polite">
              Updating assignee suggestions...
            </p>
          {/if}
          <p id="assignee-help-detail" class="text-xs text-muted-foreground">
            Click to view popular same-domain emails, then type to narrow by words. You can still enter any email manually.
          </p>
          {#if errorFor('assignee_email')}
            <p id="assignee-error-detail" class="text-xs font-semibold text-destructive" role="alert">
              {errorFor('assignee_email')}
            </p>
          {/if}
        </div>

        <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
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

        <div class="flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
          <Button type="submit" class="w-full sm:w-auto">Save changes</Button>
          <Button type="button" variant="destructive" onclick={() => (deleteOpen = true)} class="w-full sm:w-auto">
            Delete request
          </Button>
        </div>
      </form>
    </CardContent>
  </Card>

  <Card class="surface-gradient border xl:sticky xl:top-24">
    <CardHeader>
      <CardTitle>Audit timeline</CardTitle>
      <CardDescription>Newest changes first.</CardDescription>
    </CardHeader>
    <CardContent>
      {#if auditEntries.length === 0}
        <p class="text-sm text-muted-foreground">No audit entries yet.</p>
      {:else}
        <ol class="grid gap-3">
          {#each auditEntries as event}
            <li class="rounded-lg border bg-background/80 p-3">
              <div class="flex items-center justify-between gap-2">
                <Badge variant="outline" class="capitalize">{event.action.replace('_', ' ')}</Badge>
                <span class="font-mono text-[11px] text-muted-foreground">{formatDate(event.created_at)}</span>
              </div>
              <p class="mt-2 text-sm">{auditSummary(event)}</p>
              <p class="mt-2 text-xs text-muted-foreground">actor: {event.actor_email}</p>
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
      <form method="POST" action="?/delete" use:enhance={enhanceDeleteForm}>
        <Button type="submit" variant="destructive">Delete</Button>
      </form>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
