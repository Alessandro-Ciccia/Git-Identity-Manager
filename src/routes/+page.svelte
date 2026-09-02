<script lang="ts">
  import { onMount } from 'svelte';

  import EnvironmentStatusPanel from '$lib/components/EnvironmentStatusPanel.svelte';
  import type { EnvironmentStatus } from '$lib/domain/environment';
  import { navigationItems } from '$lib/domain/navigation';
  import { environmentErrorMessage, getEnvironmentStatus } from '$lib/ipc/environment';

  let status = $state<EnvironmentStatus | null>(null);
  let isLoading = $state(true);
  let loadError = $state<string | null>(null);

  async function refreshEnvironment(): Promise<void> {
    isLoading = true;
    loadError = null;

    try {
      status = await getEnvironmentStatus();
    } catch (error) {
      loadError = environmentErrorMessage(error);
    } finally {
      isLoading = false;
    }
  }

  onMount(() => {
    void refreshEnvironment();
  });
</script>

<svelte:head>
  <title>Overview · Git Identity Manager</title>
</svelte:head>

<main class="min-h-screen bg-stone-950 text-stone-100">
  <div class="grid min-h-screen grid-cols-[220px_1fr]">
    <aside class="border-r border-white/10 bg-stone-950/80 p-4">
      <div class="mb-8">
        <p class="text-xs font-semibold uppercase tracking-[0.24em] text-sky-300">Local-first</p>
        <h1 class="mt-2 text-lg font-semibold">Git Identity Manager</h1>
      </div>

      <nav aria-label="Primary navigation" class="space-y-1">
        {#each navigationItems as item (item.id)}
          <a
            class={`block rounded-lg px-3 py-2 text-sm transition hover:bg-white/5 hover:text-white ${
              item.id === 'overview' ? 'bg-white/10 text-white' : 'text-stone-300'
            }`}
            href={`#${item.id}`}
            aria-current={item.id === 'overview' ? 'page' : undefined}
          >
            {item.label}
          </a>
        {/each}
      </nav>
    </aside>

    <section class="px-10 py-12">
      <div class="max-w-4xl">
        <p class="text-sm font-medium text-sky-300">Overview</p>
        <h2 class="mt-3 text-4xl font-semibold tracking-tight text-white">Environment readiness</h2>
        <p class="mt-4 max-w-2xl text-base leading-7 text-stone-300">
          Git Identity Manager checks the local tools it needs without installing software or
          exposing shell access to the interface.
        </p>

        <div class="mt-10">
          <EnvironmentStatusPanel
            {status}
            loading={isLoading}
            error={loadError}
            onRefresh={refreshEnvironment}
          />
        </div>
      </div>
    </section>
  </div>
</main>
