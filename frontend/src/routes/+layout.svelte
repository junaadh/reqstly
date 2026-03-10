<script lang="ts">
  import { onMount } from 'svelte';
  import favicon from '$lib/assets/favicon.svg';
  import { debugErrorDetails, logError } from '$lib/debug';
  import type { Snippet } from 'svelte';
  import '../app.css';

  let { children } = $props<{ children: Snippet }>();

  onMount(() => {
    const handleGlobalError = (event: ErrorEvent): void => {
      logError('app.global', 'Unhandled window error', event.error ?? event.message, {
        filename: event.filename,
        lineno: event.lineno,
        colno: event.colno
      });
    };

    const handleUnhandledRejection = (event: PromiseRejectionEvent): void => {
      logError('app.global', 'Unhandled promise rejection', event.reason, {
        details: debugErrorDetails(event.reason)
      });
    };

    window.addEventListener('error', handleGlobalError);
    window.addEventListener('unhandledrejection', handleUnhandledRejection);

    const media = window.matchMedia('(prefers-color-scheme: dark)');
    const syncTheme = (): void => {
      document.documentElement.classList.toggle('dark', media.matches);
    };
    syncTheme();
    media.addEventListener('change', syncTheme);

    return () => {
      window.removeEventListener('error', handleGlobalError);
      window.removeEventListener('unhandledrejection', handleUnhandledRejection);
      media.removeEventListener('change', syncTheme);
    };
  });
</script>

<svelte:head>
  <link rel="icon" href={favicon} />
  <title>Reqstly</title>
</svelte:head>

{@render children()}
