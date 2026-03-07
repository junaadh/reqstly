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
  import { ensureCsrfToken } from '$lib/auth/csrf';
  import {
    enrollPasskey,
    listAccountPasskeys,
    type PasskeyCredentialSummary,
    type PasskeyStatsSummary
  } from '$lib/auth/passkeys';
  import { logInfo } from '$lib/debug';
  import { subscribeRealtimeEvents } from '$lib/realtime/ws';
  import type { RealtimeServerEvent } from '$lib/realtime/types';
  import type { ApiEnvelope, ApiErrorEnvelope, MeProfile } from '$lib/types';

  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();

  let liveDisplayName = $state<string | null>(null);
  const me = $derived.by<MeProfile>(() => ({
    ...data.me,
    display_name: liveDisplayName ?? data.me.display_name
  }));
  let displayName = $state('');
  let signOutOpen = $state(false);
  let signingOut = $state(false);
  let savingProfile = $state(false);
  let passkeyLoading = $state(false);
  let passkeyListLoading = $state(false);
  let passkeys = $state<PasskeyCredentialSummary[]>([]);
  let passkeyStats = $state<PasskeyStatsSummary>({
    passkey_count: 0,
    first_registered_at: null,
    first_used_at: null,
    last_used_at: null
  });
  let passkeyError = $state('');
  let passkeyMessage = $state('');
  let profileError = $state('');
  let profileMessage = $state('');
  const passkeyCount = $derived(passkeyStats.passkey_count || passkeys.length);
  const firstUsedDate = $derived.by(() => {
    const fromStats = parseTimestamp(passkeyStats.first_used_at);
    if (fromStats) {
      return new Intl.DateTimeFormat(undefined, {
        year: 'numeric',
        month: 'short',
        day: '2-digit'
      }).format(fromStats);
    }

    const earliest = passkeys
      .map((passkey) => parseTimestamp(passkey.first_used_at ?? passkey.last_used_at))
      .filter((value): value is Date => value !== null)
      .sort((left, right) => left.getTime() - right.getTime())[0];

    if (!earliest) {
      return null;
    }

    return new Intl.DateTimeFormat(undefined, {
      year: 'numeric',
      month: 'short',
      day: '2-digit'
    }).format(earliest);
  });

  $effect(() => {
    if (!savingProfile) {
      displayName = me.display_name;
    }
  });

  function handleRealtimeEvent(event: RealtimeServerEvent): void {
    if (event.type !== 'profile.patch') {
      return;
    }

    const nextUser = event.payload.user;
    if (nextUser.id !== me.id) {
      return;
    }

    liveDisplayName = nextUser.display_name;
  }

  async function signOut(): Promise<void> {
    if (signingOut) return;
    signingOut = true;
    try {
      const csrfToken = await ensureCsrfToken();
      await fetch('/api/auth/logout', {
        method: 'POST',
        credentials: 'include',
        headers: {
          'X-CSRF-Token': csrfToken
        }
      });
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
    passkeyError = '';
    passkeyMessage = '';

    passkeyLoading = true;

    try {
      await enrollPasskey('Reqstly Profile Passkey');
      passkeyMessage = 'Passkey added successfully.';
      await loadPasskeys();
    } catch (error) {
      passkeyError = error instanceof Error ? error.message : 'Failed to add passkey.';
    } finally {
      passkeyLoading = false;
    }
  }

  async function loadPasskeys(): Promise<void> {
    passkeyListLoading = true;
    passkeyError = '';

    try {
      const payload = await listAccountPasskeys();
      passkeys = payload.credentials;
      passkeyStats = payload.stats;
    } catch (error) {
      passkeyError = error instanceof Error ? error.message : 'Failed to load passkeys.';
    } finally {
      passkeyListLoading = false;
    }
  }

  function parseTimestamp(raw: string | null): Date | null {
    if (!raw) {
      return null;
    }

    const candidates = [raw];
    if (!raw.includes('T') && raw.includes(' ')) {
      candidates.push(raw.replace(' ', 'T'));
    }
    if (/([+-]\d{2})$/.test(raw)) {
      candidates.push(raw.replace(/([+-]\d{2})$/, '$1:00'));
    }

    for (const candidate of candidates) {
      const parsed = new Date(candidate);
      if (!Number.isNaN(parsed.getTime())) {
        return parsed;
      }
    }

    return null;
  }

  async function saveProfile(): Promise<void> {
    profileError = '';
    profileMessage = '';

    const nextName = displayName.trim();
    if (nextName.length === 0) {
      profileError = 'Display name is required.';
      return;
    }

    if (nextName === me.display_name) {
      profileMessage = 'No changes to save.';
      return;
    }

    savingProfile = true;
    try {
      await ensureCsrfToken();
      const response = await fetch('/api/me', {
        method: 'PATCH',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          display_name: nextName
        })
      });

      const payload = (await response.json().catch(() => null)) as
        | ApiEnvelope<MeProfile>
        | ApiErrorEnvelope
        | null;

      if (!response.ok) {
        const message =
          payload && 'error' in payload && typeof payload.error?.message === 'string'
            ? payload.error.message
            : 'Failed to save profile.';

        if (response.status === 401) {
          await goto('/login?reason=session-expired');
          return;
        }

        profileError = message;
        return;
      }

      if (!payload || !('data' in payload) || typeof payload.data.display_name !== 'string') {
        profileError = 'Unexpected profile update response.';
        return;
      }

      liveDisplayName = payload.data.display_name;
      displayName = payload.data.display_name;
      profileMessage = 'Display name saved. Refreshing profile context...';
      await goto('/profile', { invalidateAll: true });
    } catch (error) {
      profileError = error instanceof Error ? error.message : 'Failed to save profile.';
    } finally {
      savingProfile = false;
    }
  }

  onMount(() => {
    const unsubscribeEvents = subscribeRealtimeEvents(handleRealtimeEvent);
    void loadPasskeys();

    return () => {
      unsubscribeEvents();
    };
  });
</script>

<section class="grid gap-4">
  <Card class="surface-gradient border">
    <CardHeader>
      <CardTitle>Profile</CardTitle>
      <CardDescription>Manage your account identity shown across requests and audits.</CardDescription>
    </CardHeader>

    <CardContent class="grid gap-4 p-4 sm:p-5 lg:p-6">
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

      <div class="grid gap-2 lg:grid-cols-2">
        <div class="grid gap-2">
          <Label>Email</Label>
          <Input value={me.email} disabled />
        </div>
        <div class="grid gap-2">
          <Label>User ID</Label>
          <Input value={me.id} disabled class="font-mono text-xs" />
        </div>
      </div>

      <div class="rounded-xl border border-border bg-muted/20 p-4" aria-busy={passkeyLoading}>
        <div class="flex flex-wrap items-start justify-between gap-3">
          <div class="space-y-1">
            <p class="flex items-center gap-2 font-semibold text-foreground">
              <Fingerprint class="size-4 text-primary" />
              Passkeys
            </p>
            <p class="text-xs text-muted-foreground">
              Register a passkey on this account for passwordless sign-in.
            </p>
          </div>
          <Button type="button" variant="outline" onclick={addPasskey} disabled={passkeyLoading} aria-disabled={passkeyLoading}>
            {passkeyLoading ? 'Adding passkey…' : 'Add passkey'}
          </Button>
        </div>

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

        {#if passkeyListLoading}
          <p class="mt-3 text-xs text-muted-foreground">Loading passkeys...</p>
        {:else}
          <dl class="mt-3 grid gap-2 rounded-lg border border-border/80 bg-background/70 p-3 sm:grid-cols-2">
            <div>
              <dt class="text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">Passkeys</dt>
              <dd class="mt-1 text-sm font-semibold text-foreground">{passkeyCount}</dd>
            </div>
            <div>
              <dt class="flex items-center gap-1 text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">
                <CalendarDays class="size-3.5" />
                First used
              </dt>
              <dd class="mt-1 text-sm font-semibold text-foreground">
                {firstUsedDate ?? 'Not used yet'}
              </dd>
            </div>
          </dl>
        {/if}
      </div>

      <div class="flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
        <Button variant="outline" onclick={saveProfile} disabled={savingProfile} class="w-full sm:w-auto">
          {savingProfile ? 'Saving...' : 'Save changes'}
        </Button>
        <Button variant="destructive" onclick={() => (signOutOpen = true)} class="w-full sm:w-auto">Sign out</Button>
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
