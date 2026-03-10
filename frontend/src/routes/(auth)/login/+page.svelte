<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import {
    AlertCircle,
    ArrowRight,
    Building2,
    Eye,
    EyeOff,
    Fingerprint,
    LoaderCircle,
    Lock,
    Mail
  } from '@lucide/svelte';

  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { clearClientAuthState } from '$lib/auth/session';
  import { ensureCsrfToken } from '$lib/auth/csrf';
  import { debugErrorDetails, logError, logInfo } from '$lib/debug';
  import { signInWithPasskey } from '$lib/auth/passkeys';
  import type { ApiErrorEnvelope } from '$lib/types';

  let email = $state('');
  let password = $state('');
  let showPassword = $state(false);
  let loading = $state(false);
  let errorMessage = $state('');
  let signinMode = $state<'password' | 'passkey'>('password');

  function readQueryParam(key: string): string | null {
    if (typeof window === 'undefined') {
      return null;
    }
    return new URL(window.location.href).searchParams.get(key);
  }

  function getSafeNextPath(): string {
    const next = readQueryParam('next');
    if (!next || !next.startsWith('/') || next.startsWith('//')) {
      return '/';
    }
    return next;
  }

  function switchToPasskeyMode(): void {
    errorMessage = '';
    signinMode = 'passkey';
  }

  function switchToPasswordMode(): void {
    errorMessage = '';
    signinMode = 'password';
  }

  function parseApiError(
    payload: unknown,
    fallback: string
  ): string {
    if (!payload || typeof payload !== 'object') {
      return fallback;
    }

    const envelope = payload as Partial<ApiErrorEnvelope>;
    if (envelope.error && typeof envelope.error.message === 'string') {
      return envelope.error.message;
    }

    return fallback;
  }

  async function postAuthJson(
    path: string,
    body: Record<string, unknown>
  ): Promise<void> {
    const response = await fetch(path, {
      method: 'POST',
      credentials: 'include',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(body)
    });

    const payload = await response.json().catch(() => null);
    if (!response.ok) {
      throw new Error(parseApiError(payload, `Auth request failed (${response.status})`));
    }
  }

  async function signInWithPassword(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    errorMessage = '';
    logInfo('auth.login', 'Password sign-in submitted', { email });

    loading = true;
    try {
      await postAuthJson('/api/auth/login/password', {
        email: email.trim().toLowerCase(),
        password
      });

      await ensureCsrfToken();
      const next = getSafeNextPath();
      logInfo('auth.login', 'Password sign-in succeeded', { email, next });
      await goto(next);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Invalid email or password';
      logInfo('auth.login', 'Password sign-in failed', {
        email,
        error: message
      });
      errorMessage = message;
      loading = false;
    }
  }

  async function socialLogin(provider: 'azure'): Promise<void> {
    errorMessage = '';
    logInfo('auth.login', 'OAuth sign-in requested', { provider });
    errorMessage = 'Microsoft sign-in is disabled in the current Phase 5 baseline.';
  }

  async function passkeySignIn(): Promise<void> {
    errorMessage = '';
    logInfo('auth.login', 'Passkey sign-in requested');

    loading = true;

    try {
      await signInWithPasskey();
      await ensureCsrfToken();
      const next = getSafeNextPath();
      logInfo('auth.login', 'Passkey sign-in succeeded', { next });
      await goto(next);
    } catch (error) {
      logError('auth.login', 'Passkey sign-in failed', error, {
        details: debugErrorDetails(error)
      });
      const message = error instanceof Error ? error.message : 'Passkey sign-in failed. Please try again.';
      errorMessage = message;
      loading = false;
    }
  }

  onMount(async () => {
    logInfo('auth.login', 'Login page mounted');

    const reason = readQueryParam('reason');
    if (reason === 'session-expired') {
      clearClientAuthState();
    }
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

  <div class="flex items-center justify-center p-5 sm:p-8 md:p-10 lg:p-12">
    <div class="w-full md:max-w-[640px] lg:max-w-[440px]">
      <div class="rounded-xl border border-border bg-card p-5 shadow-xl shadow-slate-900/5 sm:p-8">
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

      {#if signinMode === 'password'}
        <form
          class="grid gap-3"
          onsubmit={signInWithPassword}
          aria-busy={loading}
          aria-describedby={errorMessage ? 'login_error_message' : undefined}
        >
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
      {:else}
        <div class="space-y-4 rounded-xl border border-border bg-muted/30 p-4" aria-busy={loading}>
          <div class="flex items-start gap-3 rounded-lg border border-border/80 bg-background px-3 py-3">
            <div class="mt-0.5 grid size-8 shrink-0 place-items-center rounded-full bg-primary/15 text-primary">
              {#if loading}
                <LoaderCircle class="size-4 animate-spin" />
              {:else}
                <Fingerprint class="size-4" />
              {/if}
            </div>
            <div class="min-w-0">
              <h3 class="text-lg font-heading font-semibold leading-tight">Sign in with passkey</h3>
              <p class="mt-1 text-xs text-muted-foreground" role="status" aria-live="polite">
                {#if loading}
                  Waiting for device confirmation. Approve Face ID, Touch ID, Windows Hello, or security key.
                {:else}
                  Use your existing passkey for passwordless sign-in.
                {/if}
              </p>
            </div>
          </div>

          <Button class="h-11 w-full" onclick={passkeySignIn} disabled={loading} aria-busy={loading}>
            {#if loading}
              <LoaderCircle class="size-4 animate-spin" />
              Authenticating...
            {:else}
              <Fingerprint class="size-4" />
              Continue with passkey
            {/if}
          </Button>

          <Button type="button" variant="outline" class="h-10 w-full" onclick={switchToPasswordMode} disabled={loading}>
            Use email and password
          </Button>
        </div>
      {/if}

      <div class="relative my-7">
        <div class="absolute inset-0 flex items-center">
          <div class="w-full border-t border-border"></div>
        </div>
        <div class="relative flex justify-center text-[11px] uppercase">
          <span class="bg-card px-2 text-muted-foreground">Or continue with</span>
        </div>
      </div>

      <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <Button type="button" variant="outline" class="h-10" onclick={() => socialLogin('azure')} disabled={loading}>
          Microsoft
        </Button>
        <Button
          type="button"
          variant="outline"
          class="h-10"
          onclick={signinMode === 'password' ? switchToPasskeyMode : switchToPasswordMode}
          disabled={loading}
        >
          {signinMode === 'password' ? 'Passkey' : 'Email'}
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
