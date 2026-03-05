<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { Bell, LogOut, Menu, Plus } from '@lucide/svelte';
  import type { Snippet } from 'svelte';

  import { clearClientAuthState } from '$lib/auth/session';
  import { Button } from '$lib/components/ui/button';
  import { getSupabaseClient } from '$lib/supabase/client';
  import type { MeProfile } from '$lib/types';
  import { cn } from '$lib/utils';

  let { user, children } = $props<{ user: MeProfile; children: Snippet }>();
  let mobileNavOpen = $state(false);
  let signingOut = $state(false);

  const nav = [
    { href: '/', label: 'Dashboard' },
    { href: '/requests', label: 'Requests' },
    { href: '/requests/new', label: 'New Request' },
    { href: '/profile', label: 'Profile' },
    { href: '/settings', label: 'Settings' }
  ];

  const isActive = (href: string, pathname: string): boolean => {
    if (href === '/') return pathname === '/';
    return pathname === href || pathname.startsWith(`${href}/`);
  };

  const pageTitle = $derived.by(() => {
    const path = $page.url.pathname;
    if (path === '/') return 'Dashboard';
    return path
      .split('/')
      .filter(Boolean)
      .map((segment) => segment.replace('-', ' '))
      .join(' / ');
  });

  const initials = $derived.by(() => {
    const parts = user.display_name.trim().split(/\s+/).filter(Boolean);
    if (parts.length === 0) return 'U';
    if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
    return `${parts[0][0]}${parts[parts.length - 1][0]}`.toUpperCase();
  });

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
      signingOut = false;
      mobileNavOpen = false;
      await goto('/login');
    }
  }
</script>

<div class="min-h-dvh bg-background">
  <div class="mx-auto grid min-h-dvh w-full max-w-[1480px] lg:grid-cols-[264px_1fr]">
    <aside class="hidden border-r border-sidebar-border/75 bg-sidebar lg:flex lg:flex-col lg:px-4 lg:py-5">
      <a class="flex items-center gap-3 rounded-xl border border-sidebar-border/80 bg-background px-3 py-3" href="/">
        <div class="grid size-9 place-items-center rounded-xl bg-primary text-primary-foreground font-heading text-lg font-bold">
          R
        </div>
        <div>
          <p class="font-heading text-base font-semibold leading-none text-sidebar-foreground">Reqstly</p>
          <p class="mt-1 text-xs tracking-wide text-muted-foreground">Manage requests with clarity</p>
        </div>
      </a>

      <nav class="mt-5 grid gap-1" aria-label="Primary">
        {#each nav as item}
          <a
            class={cn(
              'rounded-xl px-3 py-3 text-sm font-semibold transition-colors',
              isActive(item.href, $page.url.pathname)
                ? 'bg-sidebar-primary text-sidebar-primary-foreground'
                : 'text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground'
            )}
            aria-current={isActive(item.href, $page.url.pathname) ? 'page' : undefined}
            href={item.href}
          >
            {item.label}
          </a>
        {/each}
      </nav>

      <div class="mt-auto space-y-2 rounded-xl border border-sidebar-border/85 bg-background p-3">
        <div class="flex items-center gap-3">
          <div class="grid size-10 place-items-center rounded-full bg-primary/15 text-xs font-bold text-primary">{initials}</div>
          <div class="min-w-0">
            <p class="truncate text-sm font-semibold text-foreground">{user.display_name}</p>
            <p class="truncate text-xs text-muted-foreground">{user.email}</p>
          </div>
        </div>
        <div class="grid grid-cols-1 gap-2">
          <Button
            type="button"
            size="sm"
            variant="ghost"
            class="h-11 text-destructive hover:bg-destructive/10 hover:text-destructive"
            onclick={signOut}
            disabled={signingOut}
          >
            <LogOut class="size-4" />
            {signingOut ? 'Signing out' : 'Logout'}
          </Button>
        </div>
      </div>
    </aside>

    <section class="flex min-w-0 flex-col">
      <header class="sticky top-0 z-40 border-b border-border/80 bg-background/95 backdrop-blur-md">
        <div class="flex h-16 items-center justify-between gap-3 px-4 sm:px-6 lg:px-8">
          <div class="flex min-w-0 items-center gap-3">
            <Button
              type="button"
              variant="outline"
              size="icon"
              class="size-11 lg:hidden"
              aria-label="Toggle navigation"
              onclick={() => (mobileNavOpen = !mobileNavOpen)}
            >
              <Menu class="size-5" />
            </Button>

            <div class="min-w-0">
              <h1 class="truncate font-heading text-xl font-semibold tracking-tight text-foreground">{pageTitle}</h1>
              <p class="truncate text-xs text-muted-foreground">Welcome back, {user.display_name}</p>
            </div>
          </div>

          <div class="flex items-center gap-2">
            <Button href="/requests/new" size="sm" class="h-11">
              <Plus class="size-4" />
              New Request
            </Button>
            <Button type="button" variant="outline" size="icon" class="size-11" aria-label="Notifications">
              <Bell class="size-4" />
            </Button>
            <a
              href="/profile"
              class="grid size-11 place-items-center rounded-full border border-border bg-card text-xs font-bold text-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
              aria-label="Open profile"
            >
              {initials}
            </a>
          </div>
        </div>

        {#if mobileNavOpen}
          <div class="border-t border-border/70 bg-background px-4 py-3 lg:hidden">
            <nav class="grid gap-1" aria-label="Mobile navigation">
              {#each nav as item}
                <a
                  class={cn(
                    'rounded-lg px-3 py-3 text-sm font-semibold',
                    isActive(item.href, $page.url.pathname)
                      ? 'bg-primary text-primary-foreground'
                      : 'text-foreground hover:bg-accent hover:text-accent-foreground'
                  )}
                  href={item.href}
                  aria-current={isActive(item.href, $page.url.pathname) ? 'page' : undefined}
                  onclick={() => (mobileNavOpen = false)}
                >
                  {item.label}
                </a>
              {/each}
            </nav>

            <div class="mt-3 flex items-center justify-between rounded-xl border border-border/70 bg-card p-3">
              <div class="min-w-0">
                <p class="truncate text-sm font-semibold">{user.display_name}</p>
                <p class="truncate text-xs text-muted-foreground">{user.email}</p>
              </div>
              <Button type="button" size="sm" variant="ghost" class="h-11" onclick={signOut} disabled={signingOut}>
                <LogOut class="size-4" />
                {signingOut ? 'Signing out' : 'Logout'}
              </Button>
            </div>
          </div>
        {/if}
      </header>

      <main class="flex-1 px-4 py-6 sm:px-6 lg:px-8">
        {@render children()}
      </main>
    </section>
  </div>
</div>
