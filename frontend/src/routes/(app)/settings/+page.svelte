<script lang="ts">
  import { goto } from '$app/navigation';

  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Label } from '$lib/components/ui/label';
  import type { ApiEnvelope, ApiErrorEnvelope, UserPreferences } from '$lib/types';

  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();

  let emailDigest = $state(true);
  let browserAlerts = $state(true);
  let defaultPageSize = $state('20');
  let saveMessage = $state('');
  let saveError = $state('');
  let saving = $state(false);

  $effect(() => {
    emailDigest = data.preferences.email_digest;
    browserAlerts = data.preferences.browser_alerts;
    defaultPageSize = String(data.preferences.default_page_size);
  });

  async function savePreferences(): Promise<void> {
    saveMessage = '';
    saveError = '';
    saving = true;

    try {
      const response = await fetch('/api/preferences', {
        method: 'PATCH',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          email_digest: emailDigest,
          browser_alerts: browserAlerts,
          default_page_size: Number(defaultPageSize)
        })
      });

      const payload = (await response.json().catch(() => null)) as
        | ApiEnvelope<UserPreferences>
        | ApiErrorEnvelope
        | null;

      if (!response.ok) {
        if (response.status === 401) {
          await goto('/login?reason=session-expired');
          return;
        }

        saveError =
          payload && 'error' in payload && typeof payload.error.message === 'string'
            ? payload.error.message
            : 'Failed to save preferences.';
        return;
      }

      if (!payload || !('data' in payload)) {
        saveError = 'Unexpected preferences response.';
        return;
      }

      emailDigest = payload.data.email_digest;
      browserAlerts = payload.data.browser_alerts;
      defaultPageSize = String(payload.data.default_page_size);
      saveMessage = 'Preferences saved.';
    } catch (error) {
      saveError = error instanceof Error ? error.message : 'Failed to save preferences.';
    } finally {
      saving = false;
    }
  }
</script>

<section class="grid gap-4">
  <Card class="surface-gradient border">
    <CardHeader>
      <CardTitle>Settings & Preferences</CardTitle>
      <CardDescription>
        Configure default list behavior and notification preferences for your request workspace.
      </CardDescription>
    </CardHeader>

    <CardContent class="grid gap-5 p-4 sm:p-5 lg:p-6">
      {#if data.backendError}
        <p class="rounded-md border border-amber-500/35 bg-amber-500/10 px-3 py-2 text-xs font-semibold text-amber-700 dark:text-amber-300" role="alert">
          {data.backendError}
        </p>
      {/if}

      <div class="grid gap-3 rounded-xl border p-4 sm:p-5">
        <h3 class="font-heading text-lg font-semibold">Notifications</h3>

        <label class="flex items-start justify-between gap-3 text-sm sm:items-center">
          <span class="pr-2">Email digest for request updates</span>
          <input type="checkbox" bind:checked={emailDigest} class="size-4 accent-primary" />
        </label>

        <label class="flex items-start justify-between gap-3 text-sm sm:items-center">
          <span class="pr-2">Browser alerts for status changes</span>
          <input type="checkbox" bind:checked={browserAlerts} class="size-4 accent-primary" />
        </label>
      </div>

      <div class="grid gap-3 rounded-xl border p-4 sm:p-5">
        <h3 class="font-heading text-lg font-semibold">List Defaults</h3>
        <div class="grid gap-2 sm:max-w-xs">
          <Label for="page_size">Default page size</Label>
          <select id="page_size" class="border-input bg-background h-9 rounded-md border px-3 text-sm" bind:value={defaultPageSize}>
            <option value="10">10</option>
            <option value="20">20</option>
            <option value="50">50</option>
            <option value="100">100</option>
          </select>
        </div>
        <p class="text-xs text-muted-foreground">Preferences are synced to your account and loaded on every signed-in device.</p>
      </div>

      <div class="flex flex-col-reverse gap-2 sm:flex-row sm:items-center sm:justify-end">
        {#if saveError}
          <p class="text-xs font-semibold text-destructive" role="alert">{saveError}</p>
        {/if}
        {#if saveMessage}
          <p class="text-xs font-semibold text-primary" role="status">{saveMessage}</p>
        {/if}
        <Button variant="outline" onclick={savePreferences} class="w-full sm:w-auto" disabled={saving}>
          {saving ? 'Saving...' : 'Save preferences'}
        </Button>
      </div>
    </CardContent>
  </Card>
</section>
