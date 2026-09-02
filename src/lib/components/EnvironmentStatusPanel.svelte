<script lang="ts">
  import {
    dependencyLabels,
    environmentSummary,
    type DependencyState,
    type EnvironmentStatus,
  } from '$lib/domain/environment';

  type Props = {
    status: EnvironmentStatus | null;
    loading: boolean;
    error: string | null;
    onRefresh: () => void | Promise<void>;
  };

  let { status, loading, error, onRefresh }: Props = $props();

  const dependencies = $derived(status ? [status.git, status.githubCli] : []);

  function badgeClasses(state: DependencyState): string {
    switch (state) {
      case 'available':
        return 'border-emerald-400/20 bg-emerald-400/10 text-emerald-300';
      case 'missing':
        return 'border-amber-400/20 bg-amber-400/10 text-amber-200';
      case 'error':
        return 'border-rose-400/20 bg-rose-400/10 text-rose-200';
    }
  }

  function stateLabel(state: DependencyState): string {
    switch (state) {
      case 'available':
        return 'Available';
      case 'missing':
        return 'Missing';
      case 'error':
        return 'Check failed';
    }
  }
</script>

<section
  class="overflow-hidden rounded-2xl border border-white/10 bg-stone-900/50"
  aria-labelledby="environment-heading"
  aria-busy={loading}
>
  <header class="flex items-start justify-between gap-6 border-b border-white/10 px-6 py-5">
    <div>
      <p class="text-xs font-semibold uppercase tracking-[0.2em] text-stone-500">System health</p>
      <h3 id="environment-heading" class="mt-2 text-lg font-semibold text-white">Dependencies</h3>
      {#if status}
        <p class="mt-1 text-sm text-stone-400">{environmentSummary(status)}</p>
      {:else}
        <p class="mt-1 text-sm text-stone-400">Checking the tools used by Git Identity Manager.</p>
      {/if}
    </div>

    <button
      type="button"
      class="rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm font-medium text-stone-200 transition hover:border-white/20 hover:bg-white/10 disabled:cursor-wait disabled:opacity-60"
      onclick={onRefresh}
      disabled={loading}
    >
      {loading ? 'Checking…' : 'Refresh'}
    </button>
  </header>

  <div aria-live="polite">
    {#if error}
      <div class="border-b border-rose-400/20 bg-rose-400/5 px-6 py-4 text-sm text-rose-200" role="alert">
        {error}
      </div>
    {/if}

    {#if status}
      <ul class="divide-y divide-white/10">
        {#each dependencies as dependency (dependency.dependency)}
          <li class="grid gap-4 px-6 py-5 sm:grid-cols-[1fr_auto] sm:items-center">
            <div>
              <div class="flex flex-wrap items-center gap-3">
                <h4 class="font-medium text-white">{dependencyLabels[dependency.dependency]}</h4>
                <span
                  class={`rounded-full border px-2 py-0.5 text-xs font-semibold ${badgeClasses(dependency.state)}`}
                >
                  {stateLabel(dependency.state)}
                </span>
              </div>
              <p class="mt-2 max-w-2xl text-sm leading-6 text-stone-400">{dependency.message}</p>
            </div>

            <p class="font-mono text-sm text-stone-300">
              {dependency.version ? `v${dependency.version}` : '—'}
            </p>
          </li>
        {/each}
      </ul>
    {:else if loading}
      <div class="space-y-4 px-6 py-6" aria-label="Checking dependencies">
        <div class="h-16 animate-pulse rounded-xl bg-white/5"></div>
        <div class="h-16 animate-pulse rounded-xl bg-white/5"></div>
      </div>
    {:else if !error}
      <p class="px-6 py-8 text-sm text-stone-400">Dependency status is not available yet.</p>
    {/if}
  </div>
</section>
