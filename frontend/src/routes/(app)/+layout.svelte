<script lang="ts">
  import { onMount } from 'svelte';
  import type { Snippet } from 'svelte';

  import { startRealtime, stopRealtime, subscribeRealtimeEvents } from '$lib/realtime/ws';
  import { ensureCsrfToken } from '$lib/auth/csrf';
  import type { RealtimeServerEvent } from '$lib/realtime/types';
  import { logInfo } from '$lib/debug';
  import AppShell from '$lib/ui/AppShell.svelte';
  import type { LayoutData } from './$types';

  let { children, data } = $props<{ children: Snippet; data: LayoutData }>();
  let liveDisplayName = $state<string | null>(null);
  const currentUser = $derived.by(() => ({
    ...data.me,
    display_name: liveDisplayName ?? data.me.display_name
  }));

  function handleRealtimeEvent(event: RealtimeServerEvent): void {
    if (event.type !== 'profile.patch') {
      return;
    }

    const nextUser = event.payload.user;
    if (nextUser.id !== currentUser.id) {
      return;
    }

    logInfo('app.layout', 'Applying realtime profile patch', {
      userId: nextUser.id,
      displayName: nextUser.display_name
    });
    liveDisplayName = nextUser.display_name;
  }

  onMount(() => {
    const unsubscribeEvents = subscribeRealtimeEvents(handleRealtimeEvent);
    const handlePageHide = () => {
      stopRealtime();
    };

    window.addEventListener('pagehide', handlePageHide);
    void ensureCsrfToken().catch(() => {});
    startRealtime();

    return () => {
      window.removeEventListener('pagehide', handlePageHide);
      unsubscribeEvents();
      stopRealtime();
    };
  });
</script>

<AppShell user={currentUser}>
  {@render children()}
</AppShell>
