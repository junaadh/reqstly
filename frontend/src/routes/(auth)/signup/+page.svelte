<script lang="ts">
  import { goto } from '$app/navigation';
  import {
    ArrowRight,
    Building2,
    CircleCheckBig,
    Fingerprint,
    LoaderCircle,
    Lock,
    Mail,
    User
  } from '@lucide/svelte';

  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { clearClientAuthState, setAccessTokenCookie } from '$lib/auth/session';
  import { enrollPasskeyFactor } from '$lib/auth/passkeys';
  import { getSupabaseClient } from '$lib/supabase/client';

  let displayName = $state('');
  let email = $state('');
  let password = $state('');
  let loading = $state(false);
  let errorMessage = $state('');
  let infoMessage = $state('');
  let signupMode = $state<'email' | 'passkey'>('email');
  let passkeyLoading = $state(false);
  let passkeyState = $state<'idle' | 'loading' | 'success'>('idle');
  let passkeyFullName = $state('');
  let passkeyEmail = $state('');

  function switchToPasskeyMode(): void {
    errorMessage = '';
    infoMessage = '';
    passkeyFullName = displayName.trim();
    passkeyEmail = email.trim();
    signupMode = 'passkey';
    passkeyLoading = false;
    passkeyState = 'idle';
  }

  function switchToEmailMode(): void {
    errorMessage = '';
    infoMessage = '';
    signupMode = 'email';
    passkeyState = 'idle';
  }

  function isExistingUserError(message: string): boolean {
    const value = message.toLowerCase();
    return value.includes('already') || value.includes('registered') || value.includes('exists');
  }

  function createProvisionalPassword(): string {
    return `${crypto.randomUUID()}Aa!1`;
  }

  function isMissingJwtUserError(message: string): boolean {
    const normalized = message.toLowerCase();
    return (
      normalized.includes('user from sub claim in jwt does not exist') ||
      (normalized.includes('sub claim') && normalized.includes('does not exist'))
    );
  }

  async function signUp(event: SubmitEvent): Promise<void> {
    event.preventDefault();

    const client = getSupabaseClient();
    if (!client) {
      errorMessage =
        'Supabase public config is missing. Set PUBLIC_SUPABASE_URL and PUBLIC_SUPABASE_ANON_KEY.';
      return;
    }

    loading = true;
    errorMessage = '';
    infoMessage = '';

    const { data, error } = await client.auth.signUp({
      email,
      password,
      options: {
        data: {
          display_name: displayName
        }
      }
    });

    if (error) {
      errorMessage = error.message;
      loading = false;
      return;
    }

    if (data.session?.access_token) {
      setAccessTokenCookie(data.session.access_token);
      await goto('/');
      return;
    }

    clearClientAuthState();
    infoMessage = 'Check your email for a confirmation link before signing in.';
    loading = false;
  }

  async function socialLogin(provider: 'azure'): Promise<void> {
    const client = getSupabaseClient();
    if (!client) {
      errorMessage =
        'Supabase public config is missing. Set PUBLIC_SUPABASE_URL and PUBLIC_SUPABASE_ANON_KEY.';
      return;
    }

    errorMessage = '';

    const { error } = await client.auth.signInWithOAuth({
      provider,
      options: {
        redirectTo: `${window.location.origin}/`
      }
    });

    if (error) {
      errorMessage = error.message;
    }
  }

  async function startPasskeySignup(): Promise<void> {
    const fullName = passkeyFullName.trim();
    const userEmail = passkeyEmail.trim().toLowerCase();

    if (fullName.length === 0 || userEmail.length === 0) {
      errorMessage = 'Full name and email are required for passkey signup.';
      return;
    }

    const client = getSupabaseClient();
    if (!client) {
      errorMessage =
        'Supabase public config is missing. Set PUBLIC_SUPABASE_URL and PUBLIC_SUPABASE_ANON_KEY.';
      return;
    }

    errorMessage = '';
    infoMessage = '';
    passkeyLoading = true;
    passkeyState = 'loading';

    try {
      const {
        data: { session }
      } = await client.auth.getSession();

      let activeSession = session;
      if (activeSession) {
        const { error: userError } = await client.auth.getUser();
        if (userError) {
          if (isMissingJwtUserError(userError.message)) {
            await client.auth.signOut();
            clearClientAuthState();
            activeSession = null;
          } else {
            throw userError;
          }
        }
      }

      if (activeSession) {
        const signedInEmail = activeSession.user.email?.toLowerCase() ?? '';
        if (signedInEmail.length > 0 && signedInEmail !== userEmail) {
          throw new Error(
            'You are signed in with a different email. Sign out first or use the current account email to link passkey.'
          );
        }
      } else {
        const { data, error } = await client.auth.signUp({
          email: userEmail,
          password: createProvisionalPassword(),
          options: {
            data: {
              display_name: fullName
            }
          }
        });

        if (error) {
          if (isExistingUserError(error.message)) {
            const { error: otpError } = await client.auth.signInWithOtp({
              email: userEmail,
              options: {
                shouldCreateUser: false,
                emailRedirectTo: `${window.location.origin}/login?reason=link-passkey`
              }
            });

            if (otpError) {
              infoMessage =
                'This email is already registered. Sign in with your existing method, then add passkey from Profile.';
            } else {
              infoMessage =
                'This email already exists. We sent a secure email link. Sign in, then add passkey from Profile to complete linking.';
            }

            signupMode = 'email';
            return;
          }

          throw error;
        }

        if (data.session?.access_token) {
          setAccessTokenCookie(data.session.access_token);
        } else {
          infoMessage =
            'Check your email to confirm this address. After signing in, add passkey from Profile.';
          signupMode = 'email';
          return;
        }
      }

      await enrollPasskeyFactor(client, `${fullName} Passkey`);
      infoMessage = 'Passkey created successfully. Redirecting to dashboard...';
      passkeyState = 'success';
      passkeyLoading = false;
      await new Promise((resolve) => setTimeout(resolve, 900));
      await goto('/');
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : 'Passkey setup failed.';
      passkeyState = 'idle';
    } finally {
      if (passkeyState !== 'success') {
        passkeyLoading = false;
      }
    }
  }
</script>

<section class="min-h-screen bg-[hsl(var(--background))] p-0 lg:grid lg:grid-cols-2">
  <aside class="relative hidden overflow-hidden bg-primary p-12 text-primary-foreground lg:flex lg:flex-col lg:justify-between">
    <div class="relative z-10 flex items-center gap-3">
      <div class="grid size-11 place-items-center rounded-lg bg-primary-foreground text-primary">
        <Building2 class="size-6" />
      </div>
      <h2 class="text-2xl font-heading font-black tracking-tight">Reqstly</h2>
    </div>

    <div class="relative z-10 max-w-lg">
      <h1 class="text-balance font-heading text-6xl font-black leading-[1.03] tracking-tight">
        Join the clarity movement
      </h1>
      <p class="mt-6 rounded-xl border border-primary-foreground/20 bg-primary-foreground/10 p-4 text-lg text-primary-foreground/90 backdrop-blur-sm">
        The modern standard for managing requests and tickets with precision and speed.
      </p>
    </div>

    <div class="relative z-10 flex items-center gap-4">
      <div class="flex -space-x-2">
        <div class="grid size-10 place-items-center rounded-full border-2 border-primary bg-primary-foreground text-primary text-xs font-bold">AJ</div>
        <div class="grid size-10 place-items-center rounded-full border-2 border-primary bg-primary-foreground text-primary text-xs font-bold">SK</div>
        <div class="grid size-10 place-items-center rounded-full border-2 border-primary bg-primary-foreground text-primary text-xs font-bold">RM</div>
      </div>
      <p class="text-sm font-medium text-primary-foreground/85">Join 10,000+ teams improving request clarity</p>
    </div>

    <div class="pointer-events-none absolute -right-24 -top-24 size-96 rounded-full bg-primary-foreground/10 blur-3xl"></div>
    <div class="pointer-events-none absolute -bottom-24 -left-24 size-80 rounded-full bg-black/15 blur-3xl"></div>
  </aside>

  <div class="flex items-center justify-center p-6 sm:p-12">
    <div class="w-full max-w-[520px]">
      <div class="mb-8">
        <div class="mb-8 flex items-center gap-2 text-primary lg:hidden">
          <Building2 class="size-6" />
          <h2 class="text-xl font-heading font-black">Reqstly</h2>
        </div>
        <h2 class="text-3xl font-heading font-black tracking-tight">Create your account</h2>
        <p class="mt-2 text-muted-foreground">Enter your details to get started with Reqstly.</p>
      </div>

      <div class="rounded-2xl border border-border bg-card p-8 shadow-xl shadow-primary/5">
      {#if errorMessage}
        <div
          id="signup_error_message"
          role="alert"
          aria-live="assertive"
          class="mb-3 rounded-md border border-destructive/35 bg-destructive/10 px-3 py-2 text-sm font-medium text-destructive"
        >
          {errorMessage}
        </div>
      {/if}

      {#if infoMessage}
        <div
          id="signup_info_message"
          role="status"
          aria-live="polite"
          class="mb-3 rounded-md border border-primary/30 bg-primary/10 px-3 py-2 text-sm font-medium text-primary"
        >
          {infoMessage}
        </div>
      {/if}

      {#if signupMode === 'email'}
        <form class="space-y-4" onsubmit={signUp} aria-busy={loading} aria-describedby={errorMessage ? 'signup_error_message' : undefined}>
          <div class="grid gap-2">
            <Label for="display_name">Display name</Label>
            <div class="relative">
              <User class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
              <Input id="display_name" bind:value={displayName} placeholder="Jane Support" required disabled={loading} class="pl-9" />
            </div>
          </div>

          <div class="grid gap-2">
            <Label for="email">Email</Label>
            <div class="relative">
              <Mail class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
              <Input id="email" type="email" bind:value={email} placeholder="you@company.com" required disabled={loading} class="pl-9" />
            </div>
          </div>

          <div class="grid gap-2">
            <Label for="password">Password</Label>
            <div class="relative">
              <Lock class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
              <Input id="password" type="password" bind:value={password} minlength={8} required disabled={loading} class="pl-9" />
            </div>
          </div>

          <div class="pt-1">
            <Button type="submit" class="h-11 w-full font-bold" disabled={loading}>
              {loading ? 'Creating...' : 'Sign up'}
            </Button>
          </div>
        </form>
      {:else}
        <div class="space-y-4 rounded-xl border border-border bg-muted/30 p-4" aria-busy={passkeyLoading}>
          <div class="flex items-start gap-3 rounded-lg border border-border/80 bg-background px-3 py-3">
            <div
              class={`mt-0.5 grid size-8 shrink-0 place-items-center rounded-full ${
                passkeyState === 'success'
                  ? 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-300'
                  : 'bg-primary/15 text-primary'
              }`}
            >
              {#if passkeyState === 'loading'}
                <LoaderCircle class="size-4 animate-spin" />
              {:else if passkeyState === 'success'}
                <CircleCheckBig class="size-4" />
              {:else}
                <Fingerprint class="size-4" />
              {/if}
            </div>
            <div class="min-w-0">
              <h3 class="text-lg font-heading font-semibold leading-tight">Create passkey</h3>
              <p class="mt-1 text-xs text-muted-foreground" role="status" aria-live="polite">
                {#if passkeyState === 'loading'}
                  Waiting for your device confirmation. Approve Face ID, Touch ID, or security key.
                {:else if passkeyState === 'success'}
                  Passkey created. Redirecting you to the dashboard.
                {:else}
                  Use your full name and email to create or link a passkey-secured account.
                {/if}
              </p>
            </div>
          </div>

          <div class="grid gap-2">
            <Label for="passkey_full_name">Full name</Label>
            <div class="relative">
              <User class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                id="passkey_full_name"
                bind:value={passkeyFullName}
                placeholder="Alex Thompson"
                required
                disabled={passkeyLoading || passkeyState === 'success'}
                class="pl-9"
              />
            </div>
          </div>

          <div class="grid gap-2">
            <Label for="passkey_email">Email address</Label>
            <div class="relative">
              <Mail class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                id="passkey_email"
                type="email"
                bind:value={passkeyEmail}
                placeholder="alex@reqstly.com"
                required
                disabled={passkeyLoading || passkeyState === 'success'}
                class="pl-9"
              />
            </div>
          </div>

          <Button
            class="h-11 w-full"
            onclick={startPasskeySignup}
            disabled={passkeyLoading || passkeyState === 'success'}
            aria-busy={passkeyLoading}
          >
            {#if passkeyState === 'loading'}
              <LoaderCircle class="size-4 animate-spin" />
              Creating passkey...
            {:else if passkeyState === 'success'}
              <CircleCheckBig class="size-4" />
              Passkey ready
            {:else}
              <Fingerprint class="size-4" />
              Continue with passkey
            {/if}
          </Button>

          <p class="text-center text-xs text-muted-foreground">
            Passkey uses your device biometrics and never exposes a reusable password.
          </p>
        </div>
      {/if}

      <div class="relative my-8">
        <div class="absolute inset-0 flex items-center">
          <div class="w-full border-t border-border"></div>
        </div>
        <div class="relative flex justify-center text-[11px] uppercase">
          <span class="bg-card px-4 text-muted-foreground">Or continue with</span>
        </div>
      </div>

      <div class="grid grid-cols-2 gap-3">
        <Button type="button" variant="outline" class="h-10" onclick={() => socialLogin('azure')} disabled={loading}>
          Microsoft
        </Button>
        <Button
          type="button"
          variant="outline"
          class="h-10"
          onclick={signupMode === 'email' ? switchToPasskeyMode : switchToEmailMode}
          disabled={loading || passkeyLoading}
        >
          {signupMode === 'email' ? 'Passkey' : 'Email'}
        </Button>
      </div>

      <p class="mt-4 text-sm text-muted-foreground">
        Already have an account?
        <a href="/login" class="font-semibold text-primary hover:underline">Sign in</a>
      </p>
      </div>
    </div>
  </div>
</section>
