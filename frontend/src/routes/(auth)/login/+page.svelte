<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { AlertCircle, ArrowRight, Building2, Eye, EyeOff, Fingerprint, Lock, Mail } from '@lucide/svelte';

  import { Button } from '$lib/components/ui/button';
  import * as Dialog from '$lib/components/ui/dialog';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { clearClientAuthState, setAccessTokenCookie } from '$lib/auth/session';
  import { debugErrorDetails, logError, logInfo } from '$lib/debug';
  import { signInWithPasskey } from '$lib/auth/passkeys';
  import { getSupabaseClient } from '$lib/supabase/client';

  let email = $state('');
  let password = $state('');
  let showPassword = $state(false);
  let loading = $state(false);
  let errorMessage = $state('');
  let passkeyOpen = $state(false);

  async function signInWithPassword(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    errorMessage = '';
    logInfo('auth.login', 'Password sign-in submitted', { email });

    const client = getSupabaseClient();
    if (!client) {
      errorMessage =
        'Supabase public config is missing. Set PUBLIC_SUPABASE_URL and PUBLIC_SUPABASE_ANON_KEY.';
      return;
    }

    loading = true;

    const { data, error } = await client.auth.signInWithPassword({ email, password });

    if (error || !data.session) {
      logInfo('auth.login', 'Password sign-in failed', {
        email,
        error: error?.message ?? null
      });
      errorMessage = error?.message ?? 'Invalid email or password';
      loading = false;
      return;
    }

    setAccessTokenCookie(data.session.access_token);

    const next = $page.url.searchParams.get('next') ?? '/';
    logInfo('auth.login', 'Password sign-in succeeded', { email, next });
    await goto(next);
  }

  async function socialLogin(provider: 'azure'): Promise<void> {
    errorMessage = '';
    logInfo('auth.login', 'OAuth sign-in requested', { provider });

    const client = getSupabaseClient();
    if (!client) {
      errorMessage =
        'Supabase public config is missing. Set PUBLIC_SUPABASE_URL and PUBLIC_SUPABASE_ANON_KEY.';
      return;
    }

    const { error } = await client.auth.signInWithOAuth({
      provider,
      options: {
        redirectTo: `${window.location.origin}/`
      }
    });

    if (error) {
      logInfo('auth.login', 'OAuth sign-in failed', { provider, error: error.message });
      errorMessage = error.message;
    }
  }

  async function passkeySignIn(): Promise<void> {
    errorMessage = '';
    passkeyOpen = false;
    logInfo('auth.login', 'Passkey sign-in requested');

    const client = getSupabaseClient();
    if (!client) {
      errorMessage =
        'Supabase public config is missing. Set PUBLIC_SUPABASE_URL and PUBLIC_SUPABASE_ANON_KEY.';
      return;
    }

    loading = true;

    try {
      const accessToken = await signInWithPasskey(client);
      setAccessTokenCookie(accessToken);
      const next = $page.url.searchParams.get('next') ?? '/';
      logInfo('auth.login', 'Passkey sign-in succeeded', { next });
      await goto(next);
    } catch (error) {
      logError('auth.login', 'Passkey sign-in failed', error, {
        details: debugErrorDetails(error)
      });
      const message =
        error instanceof Error ? error.message : 'Passkey sign-in failed. Please try again.';
      errorMessage = message;
      loading = false;
    }
  }

  onMount(async () => {
    logInfo('auth.login', 'Login page mounted');
    const client = getSupabaseClient();
    if (!client) return;

    const { data } = await client.auth.getSession();
    if (!data.session) {
      logInfo('auth.login', 'No existing session found on mount');
      clearClientAuthState();
      return;
    }

    setAccessTokenCookie(data.session.access_token);
    const next = $page.url.searchParams.get('next');
    logInfo('auth.login', 'Existing session found; redirecting', { next: next ?? '/' });
    await goto(next ?? '/');
  });
</script>

<section class="min-h-screen bg-[hsl(var(--background))] p-0 lg:grid lg:grid-cols-2">
  <aside class="relative hidden overflow-hidden border-r border-border bg-primary/10 p-12 lg:flex lg:flex-col lg:justify-between">
    <div class="relative z-10">
      <div class="mb-12 flex items-center gap-3">
        <div class="grid size-10 place-items-center rounded-lg bg-primary text-primary-foreground">
          <Building2 class="size-5" />
        </div>
        <h1 class="text-2xl font-heading font-black tracking-tight text-primary">Reqstly</h1>
      </div>

      <div class="max-w-md">
        <h2 class="text-balance text-5xl font-heading font-black leading-[1.06] tracking-tight text-foreground">
          Manage requests with clarity
        </h2>
        <p class="mt-6 text-lg leading-relaxed text-muted-foreground">
          The modern SaaS platform for managing requests and tickets with a clean audit trail.
        </p>
      </div>
    </div>

    <div class="relative z-10 mt-auto rounded-xl border border-primary/25 bg-background/80 p-6 backdrop-blur-sm">
      <div class="mx-auto grid max-w-sm grid-cols-3 gap-5">
        <div class="flex h-24 flex-col items-center justify-center gap-2 rounded-lg border border-primary/30 bg-primary/20">
          <Mail class="size-5 text-primary" />
          <div class="h-1 w-10 rounded-full bg-primary/40"></div>
        </div>
        <div class="flex h-24 flex-col items-center justify-center gap-2 rounded-lg bg-primary text-primary-foreground shadow-lg shadow-primary/20">
          <ArrowRight class="size-5" />
          <div class="h-1 w-10 rounded-full bg-primary-foreground/35"></div>
        </div>
        <div class="flex h-24 flex-col items-center justify-center gap-2 rounded-lg border border-primary/30 bg-primary/20">
          <Fingerprint class="size-5 text-primary" />
          <div class="h-1 w-10 rounded-full bg-primary/40"></div>
        </div>
      </div>
    </div>

    <div class="pointer-events-none absolute -bottom-16 -left-16 size-72 rounded-full bg-primary/10 blur-3xl"></div>
    <div class="pointer-events-none absolute -right-12 top-16 size-64 rounded-full bg-primary/10 blur-3xl"></div>
  </aside>

  <div class="flex items-center justify-center p-6 sm:p-12">
    <div class="w-full max-w-[440px]">
      <div class="rounded-xl border border-border bg-card p-8 shadow-xl shadow-slate-900/5">
        <div class="mb-8">
          <h3 class="text-2xl font-heading font-black tracking-tight text-card-foreground">Sign in</h3>
          <p class="mt-2 text-sm text-muted-foreground">Welcome back. Enter your account details.</p>
        </div>

      {#if errorMessage}
        <div
          id="login_error_message"
          role="alert"
          aria-live="assertive"
          class="mb-5 flex items-start gap-2 rounded-lg border border-destructive/35 bg-destructive/10 px-3 py-2.5 text-sm font-medium text-destructive"
        >
          <AlertCircle class="mt-0.5 size-4 shrink-0" />
          {errorMessage}
        </div>
      {/if}

      <form class="grid gap-3" onsubmit={signInWithPassword} aria-busy={loading} aria-describedby={errorMessage ? 'login_error_message' : undefined}>
        <div class="grid gap-2">
          <Label for="email">Email</Label>
          <div class="relative">
            <Mail class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              id="email"
              type="email"
              bind:value={email}
              placeholder="you@company.com"
              required
              disabled={loading}
              class="pl-9"
            />
          </div>
        </div>

        <div class="grid gap-2">
          <div class="flex items-center justify-between">
            <Label for="password">Password</Label>
            <a href="/login" class="text-xs font-semibold text-primary hover:underline">Forgot password</a>
          </div>

          <div class="relative">
            <Lock class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              id="password"
              type={showPassword ? 'text' : 'password'}
              bind:value={password}
              placeholder="••••••••"
              required
              disabled={loading}
              class="pl-9 pr-16"
            />
            <Button
              type="button"
              variant="ghost"
              size="sm"
              class="absolute right-1 top-1/2 -translate-y-1/2"
              aria-label={showPassword ? 'Hide password' : 'Show password'}
              onclick={() => (showPassword = !showPassword)}
            >
              {#if showPassword}
                <EyeOff class="size-4" />
              {:else}
                <Eye class="size-4" />
              {/if}
            </Button>
          </div>
        </div>

        <Button type="submit" class="mt-1 h-11 font-bold" disabled={loading}>
          {#if loading}
            <span class="size-4 animate-spin rounded-full border-2 border-primary-foreground/30 border-t-primary-foreground"></span>
            Signing in...
          {:else}
            Sign in
            <ArrowRight class="size-4" />
          {/if}
        </Button>
      </form>

      <div class="relative my-7">
        <div class="absolute inset-0 flex items-center">
          <div class="w-full border-t border-border"></div>
        </div>
        <div class="relative flex justify-center text-[11px] uppercase">
          <span class="bg-card px-2 text-muted-foreground">Or continue with</span>
        </div>
      </div>

      <div class="grid grid-cols-2 gap-3">
        <Button type="button" variant="outline" class="h-10" onclick={() => (passkeyOpen = true)} disabled={loading}>
          Sign in with passkey
        </Button>
        <Button type="button" variant="outline" class="h-10" onclick={() => socialLogin('azure')} disabled={loading}>
          Microsoft
        </Button>
      </div>

      <p class="mt-4 text-center text-sm text-muted-foreground">
        Don’t have an account?
        <a href="/signup" class="font-semibold text-primary hover:underline">Sign up</a>
      </p>
      </div>
    </div>
  </div>
</section>

<Dialog.Root bind:open={passkeyOpen}>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>Sign in with passkey</Dialog.Title>
      <Dialog.Description>
        Use your device biometrics or security key for passwordless access through Supabase Auth.
      </Dialog.Description>
    </Dialog.Header>

    <div class="my-2 overflow-hidden rounded-xl border border-primary/20 bg-primary/10 px-4 py-5">
      <div class="mx-auto grid size-16 place-items-center rounded-full bg-background shadow-md">
        <Fingerprint class="size-7 text-primary" />
      </div>
    </div>

    <Dialog.Footer class="mt-1 grid grid-cols-1 gap-2 sm:grid-cols-2">
      <Button onclick={passkeySignIn}>Continue</Button>
      <Button variant="outline" onclick={() => (passkeyOpen = false)}>Use password instead</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
