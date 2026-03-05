<script lang="ts">
  import { onMount } from 'svelte';

  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Label } from '$lib/components/ui/label';

  let emailDigest = $state(true);
  let browserAlerts = $state(true);
  let defaultPageSize = $state('20');
  let saveMessage = $state('');

  const PREFERENCES_KEY = 'reqstly_preferences';

  onMount(() => {
    const raw = window.localStorage.getItem(PREFERENCES_KEY);
    if (!raw) return;

    try {
      const saved = JSON.parse(raw) as {
        emailDigest?: boolean;
        browserAlerts?: boolean;
        defaultPageSize?: string;
      };
      emailDigest = saved.emailDigest ?? emailDigest;
      browserAlerts = saved.browserAlerts ?? browserAlerts;
      defaultPageSize = saved.defaultPageSize ?? defaultPageSize;
    } catch {
      // Ignore malformed local preference values.
    }
  });

  function savePreferences(): void {
    window.localStorage.setItem(
      PREFERENCES_KEY,
      JSON.stringify({
        emailDigest,
        browserAlerts,
        defaultPageSize
      })
    );
    saveMessage = 'Preferences saved locally for this browser.';
  }
</script>

<section class="mx-auto grid max-w-4xl gap-3">
  <Card class="surface-gradient border">
    <CardHeader>
      <CardTitle>Settings & Preferences</CardTitle>
      <CardDescription>
        Configure default list behavior and notification preferences for your request workspace.
      </CardDescription>
    </CardHeader>

    <CardContent class="grid gap-5">
      <div class="grid gap-3 rounded-xl border p-4">
        <h3 class="font-heading text-lg font-semibold">Notifications</h3>

        <label class="flex items-center justify-between gap-3 text-sm">
          <span>Email digest for request updates</span>
          <input type="checkbox" bind:checked={emailDigest} class="size-4 accent-primary" />
        </label>

        <label class="flex items-center justify-between gap-3 text-sm">
          <span>Browser alerts for status changes</span>
          <input type="checkbox" bind:checked={browserAlerts} class="size-4 accent-primary" />
        </label>
      </div>

      <div class="grid gap-3 rounded-xl border p-4">
        <h3 class="font-heading text-lg font-semibold">List Defaults</h3>
        <div class="grid gap-2 max-w-xs">
          <Label for="page_size">Default page size</Label>
          <select id="page_size" class="border-input bg-background h-9 rounded-md border px-3 text-sm" bind:value={defaultPageSize}>
            <option value="10">10</option>
            <option value="20">20</option>
            <option value="50">50</option>
            <option value="100">100</option>
          </select>
        </div>
        <p class="text-xs text-muted-foreground">Preferences are currently stored in browser local storage.</p>
      </div>

      <div class="flex flex-wrap items-center justify-end gap-2">
        {#if saveMessage}
          <p class="text-xs font-semibold text-primary" role="status">{saveMessage}</p>
        {/if}
        <Button variant="outline" onclick={savePreferences}>Save preferences</Button>
      </div>
    </CardContent>
  </Card>
</section>
