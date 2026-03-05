<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Textarea } from '$lib/components/ui/textarea';
  import type { ErrorDetail } from '$lib/types';

  import type { ActionData, PageData } from './$types';

  let { data, form } = $props<{ data: PageData; form: ActionData }>();

  const values = $derived.by(() => ({
    title: form?.values?.title ?? '',
    description: form?.values?.description ?? '',
    category: form?.values?.category ?? data.enums.category[0],
    priority: form?.values?.priority ?? data.enums.priority[1] ?? data.enums.priority[0]
  }));

  const errorFor = (field: string): string | undefined =>
    (form?.details as ErrorDetail[] | undefined)?.find((item: ErrorDetail) => item.field === field)?.message;
</script>

<section class="mx-auto grid max-w-3xl gap-3">
  <Card class="surface-gradient border">
    <CardHeader>
      <CardTitle>Create Request</CardTitle>
      <CardDescription>
        Provide clear details so triage and resolution can happen without back-and-forth.
      </CardDescription>
    </CardHeader>

    <CardContent>
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

        <div class="grid gap-4 md:grid-cols-2">
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

        <div class="flex flex-wrap justify-end gap-2">
          <Button href="/requests" variant="outline">Cancel</Button>
          <Button type="submit">Submit Request</Button>
        </div>
      </form>
    </CardContent>
  </Card>
</section>
