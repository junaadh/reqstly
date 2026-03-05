<script lang="ts">
  import type { Snippet } from 'svelte';
  import AppShell from '$lib/ui/AppShell.svelte';
  import type { LayoutData } from './$types';

  let { children, data } = $props<{ children: Snippet; data: LayoutData }>();
</script>

<AppShell user={data.me}>
  {@render children()}
</AppShell>
