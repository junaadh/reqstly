<script lang="ts">
  import { onDestroy } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Textarea } from '$lib/components/ui/textarea';
  import type { AssigneeSuggestion, ErrorDetail } from '$lib/types';
  import { cn } from '$lib/utils';

  import type { ActionData, PageData } from './$types';

  let { data, form } = $props<{ data: PageData; form: ActionData }>();

  const values = $derived.by(() => ({
    title: form?.values?.title ?? '',
    description: form?.values?.description ?? '',
    category: form?.values?.category ?? data.enums.category[0],
    priority: form?.values?.priority ?? data.enums.priority[1] ?? data.enums.priority[0],
    assignee_email: form?.values?.assignee_email ?? ''
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
    return `assignee-option-new-${assigneeHighlightIndex}`;
  });

  const assigneeDescribedBy = $derived.by(() => {
    const ids = ['assignee-help-new'];

    if (assigneeLoading) {
      ids.push('assignee-loading-new');
    }
    if (errorFor('assignee_email')) {
      ids.push('assignee-error-new');
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

  onDestroy(() => {
    if (assigneeBlurTimer) {
      clearTimeout(assigneeBlurTimer);
    }
    if (assigneeDebounceTimer) {
      clearTimeout(assigneeDebounceTimer);
    }
  });
</script>

<section class="grid gap-4">
  <Card class="surface-gradient border">
    <CardHeader>
      <CardTitle>Create Request</CardTitle>
      <CardDescription>
        Provide clear details so triage and resolution can happen without back-and-forth.
      </CardDescription>
    </CardHeader>

    <CardContent class="grid gap-4 p-4 sm:p-5 lg:p-6">
      <form method="POST" class="grid gap-4">
        {#if form?.message}
          <div class="rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive">
            {form.message}
          </div>
        {/if}

        <div class="grid gap-2">
          <Label for="title">Title</Label>
          <Input id="title" name="title" required value={values.title} placeholder="VPN access for on-call" />
          {#if errorFor('title')}
            <p class="text-xs font-semibold text-destructive">{errorFor('title')}</p>
          {/if}
        </div>

        <div class="grid gap-2">
          <Label for="description">Description</Label>
          <Textarea
            id="description"
            name="description"
            rows={6}
            value={values.description}
            placeholder="Include context, urgency, and acceptance criteria."
          />
          <p class="text-xs text-muted-foreground">Description can be up to 5000 characters.</p>
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
              aria-controls="assignee-options-new"
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
                id="assignee-options-new"
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
                      id={`assignee-option-new-${optionIndex}`}
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
            <p id="assignee-loading-new" class="text-xs text-muted-foreground" aria-live="polite">
              Updating assignee suggestions...
            </p>
          {/if}
          {#if assigneeOptions.length > 0}
            <p id="assignee-help-new" class="text-xs text-muted-foreground">
              Click to view popular same-domain emails, then type to narrow by words. You can still enter any email manually.
            </p>
          {:else}
            <p id="assignee-help-new" class="text-xs text-muted-foreground">
              No same-domain suggestions found. Enter any valid user email to assign this ticket.
            </p>
          {/if}
          {#if errorFor('assignee_email')}
            <p id="assignee-error-new" class="text-xs font-semibold text-destructive" role="alert">
              {errorFor('assignee_email')}
            </p>
          {/if}
        </div>

        <div class="grid gap-4 sm:grid-cols-2">
          <div class="grid gap-2">
            <Label for="category">Category</Label>
            <select class="border-input bg-background h-9 rounded-md border px-3 text-sm" id="category" name="category" value={values.category}>
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
            <select class="border-input bg-background h-9 rounded-md border px-3 text-sm" id="priority" name="priority" value={values.priority}>
              {#each data.enums.priority as option}
                <option value={option}>{option}</option>
              {/each}
            </select>
            {#if errorFor('priority')}
              <p class="text-xs font-semibold text-destructive">{errorFor('priority')}</p>
            {/if}
          </div>
        </div>

        <p class="text-xs text-muted-foreground">
          New requests are created with status <span class="font-semibold">open</span> and become
          visible in your request list immediately.
        </p>

        <div class="flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
          <Button href="/requests" variant="outline" class="w-full sm:w-auto">Cancel</Button>
          <Button type="submit" class="w-full sm:w-auto">Submit Request</Button>
        </div>
      </form>
    </CardContent>
  </Card>
</section>
