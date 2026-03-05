<script lang="ts">
  import { onMount } from 'svelte';
  import favicon from '$lib/assets/favicon.svg';
  import { clearClientAuthState, readAccessTokenCookie, setAccessTokenCookie } from '$lib/auth/session';
  import { debugErrorDetails, logError, logInfo, logWarn } from '$lib/debug';
  import { getSupabaseClient } from '$lib/supabase/client';
  import type { Snippet } from 'svelte';
  import '../app.css';

  let { children } = $props<{ children: Snippet }>();

  function isMissingJwtUserError(message: string): boolean {
    const normalized = message.toLowerCase();
    return (
      normalized.includes('user from sub claim in jwt does not exist') ||
      (normalized.includes('sub claim') && normalized.includes('does not exist'))
    );
  }

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

    const client = getSupabaseClient();
    if (!client) {
      logWarn('app.auth', 'Supabase client unavailable in layout mount');
      return () => {
        window.removeEventListener('error', handleGlobalError);
        window.removeEventListener('unhandledrejection', handleUnhandledRejection);
        media.removeEventListener('change', syncTheme);
      };
    }

    void (async () => {
      const {
        data: { session }
      } = await client.auth.getSession();

      if (session?.access_token) {
        logInfo('app.auth', 'Session found on mount');
        const { error: userError } = await client.auth.getUser();
        if (userError) {
          logWarn('app.auth', 'Session user validation failed', { error: userError.message });
          if (isMissingJwtUserError(userError.message)) {
            await client.auth.signOut();
          }
          clearClientAuthState();
          return;
        }
        setAccessTokenCookie(session.access_token);
      } else {
        // Keep custom passkey JWT cookie session if Supabase JS session does not exist.
        const fallbackToken = readAccessTokenCookie();
        if (!fallbackToken) {
          logInfo('app.auth', 'No Supabase session or fallback token found; clearing client auth state');
          clearClientAuthState();
        }
      }
    })();

    const {
      data: { subscription }
    } = client.auth.onAuthStateChange((_event, authSession) => {
      logInfo('app.auth', 'Auth state changed', {
        event: _event,
        hasAccessToken: Boolean(authSession?.access_token)
      });
      if (authSession?.access_token) {
        setAccessTokenCookie(authSession.access_token);
      } else {
        const fallbackToken = readAccessTokenCookie();
        if (!fallbackToken) {
          clearClientAuthState();
        }
      }
    });

    return () => {
      window.removeEventListener('error', handleGlobalError);
      window.removeEventListener('unhandledrejection', handleUnhandledRejection);
      subscription.unsubscribe();
      media.removeEventListener('change', syncTheme);
    };
  });
</script>

<svelte:head>
  <link rel="icon" href={favicon} />
  <title>Reqstly</title>
</svelte:head>

{@render children()}
