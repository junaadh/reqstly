<script lang="ts">
  import { goto } from '$app/navigation';
  import { CalendarDays, Fingerprint } from '@lucide/svelte';
  import { onMount } from 'svelte';

  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import * as Dialog from '$lib/components/ui/dialog';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { clearClientAuthState } from '$lib/auth/session';
  import { enrollPasskeyFactor, listPasskeyFactors, type PasskeyFactor } from '$lib/auth/passkeys';
  import { logInfo, logWarn } from '$lib/debug';
  import { getSupabaseClient } from '$lib/supabase/client';

  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();

  let displayName = $state('');
  let signOutOpen = $state(false);
  let signingOut = $state(false);
  let savingProfile = $state(false);
  let passkeyLoading = $state(false);
  let passkeyStatusLoading = $state(true);
  let passkeyError = $state('');
  let passkeyMessage = $state('');
  let passkeyStatusError = $state('');
  let passkeyFactors = $state<PasskeyFactor[]>([]);
  let profileError = $state('');
  let profileMessage = $state('');

  const verifiedPasskeys = $derived.by(() =>
    passkeyFactors.filter((factor) => factor.status === 'verified')
  );
  const passkeyCount = $derived(verifiedPasskeys.length);
  const hasPasskey = $derived(passkeyCount > 0);
  const addedDate = $derived.by(() => {
    const sorted = [...verifiedPasskeys]
      .filter((factor) => Boolean(factor.created_at))
      .sort((left, right) => {
        const leftTime = new Date(left.created_at ?? '').getTime();
        const rightTime = new Date(right.created_at ?? '').getTime();
        return Number.isFinite(leftTime) && Number.isFinite(rightTime) ? leftTime - rightTime : 0;
      });

    const first = sorted[0];
    if (!first?.created_at) return null;
    const parsed = new Date(first.created_at);
    if (Number.isNaN(parsed.getTime())) return null;

    return new Intl.DateTimeFormat(undefined, {
      year: 'numeric',
      month: 'short',
      day: '2-digit'
    }).format(parsed);
  });

  $effect(() => {
    displayName = data.me.display_name;
  });

  async function refreshPasskeyStatus(): Promise<void> {
    const client = getSupabaseClient();
    if (!client) {
      passkeyStatusLoading = false;
      passkeyStatusError =
        'Supabase public config is missing. Set PUBLIC_SUPABASE_URL and PUBLIC_SUPABASE_ANON_KEY.';
      return;
    }

    passkeyStatusLoading = true;
    passkeyStatusError = '';

    try {
      const factors = await listPasskeyFactors(client);
      passkeyFactors = factors;
      logInfo('profile.passkeys', 'Loaded passkey factors', {
        count: factors.length
      });
    } catch (error) {
      passkeyStatusError = error instanceof Error ? error.message : 'Failed to load passkey status.';
      logWarn('profile.passkeys', 'Failed to load passkey factors', {
        error: error instanceof Error ? error.message : String(error)
      });
    } finally {
      passkeyStatusLoading = false;
    }
  }

  async function signOut(): Promise<void> {
    if (signingOut) return;
    signingOut = true;
    try {
      const client = getSupabaseClient();
      await client?.auth.signOut();
    } catch (error) {
      console.error('Sign out failed', error);
    } finally {
      clearClientAuthState();
      signOutOpen = false;
      signingOut = false;
      await goto('/login');
    }
  }

  async function addPasskey(): Promise<void> {
    if (hasPasskey) return;
    passkeyError = '';
    passkeyMessage = '';

    const client = getSupabaseClient();
    if (!client) {
      passkeyError =
        'Supabase public config is missing. Set PUBLIC_SUPABASE_URL and PUBLIC_SUPABASE_ANON_KEY.';
      return;
    }

    passkeyLoading = true;

    try {
      await enrollPasskeyFactor(client, 'Reqstly Profile Passkey');
      passkeyMessage = 'Passkey added successfully.';
      await refreshPasskeyStatus();
    } catch (error) {
      passkeyError = error instanceof Error ? error.message : 'Failed to add passkey.';
    } finally {
      passkeyLoading = false;
    }
  }

  async function saveProfile(): Promise<void> {
    profileError = '';
    profileMessage = '';

    const nextName = displayName.trim();
    if (nextName.length === 0) {
      profileError = 'Display name is required.';
      return;
    }

    if (nextName === data.me.display_name) {
      profileMessage = 'No changes to save.';
      return;
    }

    const client = getSupabaseClient();
    if (!client) {
      profileError =
        'Supabase public config is missing. Set PUBLIC_SUPABASE_URL and PUBLIC_SUPABASE_ANON_KEY.';
      return;
    }

    savingProfile = true;
    try {
      const { error } = await client.auth.updateUser({
        data: {
          display_name: nextName
        }
      });

      if (error) {
        throw error;
      }

      profileMessage = 'Display name saved. Refreshing profile context...';
      await goto('/profile', { invalidateAll: true });
    } catch (error) {
      profileError = error instanceof Error ? error.message : 'Failed to save profile.';
    } finally {
      savingProfile = false;
    }
  }

  onMount(() => {
    void refreshPasskeyStatus();
  });
</script>

<section class="mx-auto grid max-w-3xl gap-3">
  <Card class="surface-gradient border">
    <CardHeader>
      <CardTitle>Profile</CardTitle>
      <CardDescription>Manage your account identity shown across requests and audits.</CardDescription>
    </CardHeader>

    <CardContent class="grid gap-4">
      <div class="grid gap-2">
        <Label for="display_name">Display name</Label>
        <Input id="display_name" bind:value={displayName} />
        <p class="text-xs text-muted-foreground">Used across the dashboard, activity timeline, and request ownership labels.</p>
        {#if profileError}
          <p class="text-xs font-semibold text-destructive" role="alert">{profileError}</p>
        {/if}
        {#if profileMessage}
          <p class="text-xs font-semibold text-primary" role="status">{profileMessage}</p>
        {/if}
      </div>

      <div class="grid gap-2 md:grid-cols-2">
        <div class="grid gap-2">
          <Label>Email</Label>
          <Input value={data.me.email} disabled />
        </div>
        <div class="grid gap-2">
          <Label>User ID</Label>
          <Input value={data.me.id} disabled class="font-mono text-xs" />
        </div>
      </div>

      <div class="rounded-xl border border-border bg-muted/20 p-4" aria-busy={passkeyStatusLoading}>
        <div class="flex flex-wrap items-start justify-between gap-3">
          <div class="space-y-1">
            <p class="flex items-center gap-2 font-semibold text-foreground">
              <Fingerprint class="size-4 text-primary" />
              Passkeys
            </p>
            <p class="text-xs text-muted-foreground">
              Manage your device passkeys for secure sign-in with Supabase WebAuthn MFA.
            </p>
          </div>
          <Button
            type="button"
            variant="outline"
            onclick={addPasskey}
            disabled={passkeyLoading || passkeyStatusLoading || hasPasskey}
            aria-disabled={passkeyLoading || passkeyStatusLoading || hasPasskey}
          >
            {#if passkeyStatusLoading}
              Loading…
            {:else if passkeyLoading}
              Adding passkey…
            {:else if hasPasskey}
              Passkey already added
            {:else}
              Add passkey
            {/if}
          </Button>
        </div>

        {#if passkeyStatusError}
          <p class="mt-3 rounded-md border border-destructive/35 bg-destructive/10 px-3 py-2 text-xs font-medium text-destructive" role="alert">
            {passkeyStatusError}
          </p>
        {:else}
          <dl class="mt-3 grid gap-2 rounded-lg border border-border/80 bg-background/70 p-3 sm:grid-cols-2">
            <div>
              <dt class="text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">Verified passkeys</dt>
              <dd class="mt-1 text-sm font-semibold text-foreground">{passkeyCount}</dd>
            </div>
            <div>
              <dt class="flex items-center gap-1 text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">
                <CalendarDays class="size-3.5" />
                First added
              </dt>
              <dd class="mt-1 text-sm font-semibold text-foreground">{addedDate ?? 'Not added yet'}</dd>
            </div>
          </dl>
        {/if}

        {#if passkeyError}
          <p class="mt-3 rounded-md border border-destructive/35 bg-destructive/10 px-3 py-2 text-xs font-medium text-destructive" role="alert">
            {passkeyError}
          </p>
        {/if}

        {#if passkeyMessage}
          <p class="mt-3 rounded-md border border-primary/30 bg-primary/10 px-3 py-2 text-xs font-medium text-primary" role="status">
            {passkeyMessage}
          </p>
        {/if}
      </div>

      <div class="flex flex-wrap justify-end gap-2">
        <Button variant="outline" onclick={saveProfile} disabled={savingProfile}>
          {savingProfile ? 'Saving...' : 'Save changes'}
        </Button>
        <Button variant="destructive" onclick={() => (signOutOpen = true)}>Sign out</Button>
      </div>
    </CardContent>
  </Card>
</section>

<Dialog.Root bind:open={signOutOpen}>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>Sign out</Dialog.Title>
      <Dialog.Description>You will be returned to the login screen.</Dialog.Description>
    </Dialog.Header>

    <Dialog.Footer class="mt-4">
      <Button variant="outline" onclick={() => (signOutOpen = false)}>Cancel</Button>
      <Button variant="destructive" onclick={signOut} disabled={signingOut}>
        {signingOut ? 'Signing out…' : 'Sign out'}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
