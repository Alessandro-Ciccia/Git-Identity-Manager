<script lang="ts">
  import Badge from '$lib/components/ui/Badge.svelte';
  import Banner from '$lib/components/ui/Banner.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Panel from '$lib/components/ui/Panel.svelte';
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
  const summary = $derived(
    status ? environmentSummary(status) : 'Checking the tools used by Git Identity Manager.',
  );

  const badgeTone = {
    available: 'positive',
    missing: 'warning',
    error: 'danger',
  } as const satisfies Record<DependencyState, 'positive' | 'warning' | 'danger'>;

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

<Panel
  eyebrow="System health"
  heading="Dependencies"
  headingId="environment-heading"
  description={summary}
  busy={loading}
>
  {#snippet actions()}
    <Button onclick={onRefresh} disabled={loading} pending={loading} pendingLabel="Checking…">
      Refresh
    </Button>
  {/snippet}

  {#if error}<Banner tone="danger">{error}</Banner>{/if}

  {#if status}
    <ul class="divide-y divide-edge">
      {#each dependencies as dependency (dependency.dependency)}
        <li class="grid gap-4 px-6 py-5 sm:grid-cols-[1fr_auto] sm:items-center">
          <div>
            <div class="flex flex-wrap items-center gap-3">
              <h4 class="font-medium text-fg-strong">{dependencyLabels[dependency.dependency]}</h4>
              <Badge tone={badgeTone[dependency.state]}>{stateLabel(dependency.state)}</Badge>
            </div>
            <p class="mt-2 max-w-2xl text-sm leading-6 text-fg-muted">{dependency.message}</p>
          </div>

          <p class="font-mono text-sm text-fg">
            {dependency.version ? `v${dependency.version}` : '—'}
          </p>
        </li>
      {/each}
    </ul>
  {:else if loading}
    <div class="space-y-4 px-6 py-6" aria-label="Checking dependencies">
      <div class="h-16 animate-pulse rounded-xl bg-hover"></div>
      <div class="h-16 animate-pulse rounded-xl bg-hover"></div>
    </div>
  {:else if !error}
    <p class="px-6 py-8 text-sm text-fg-muted">Dependency status is not available yet.</p>
  {/if}
</Panel>
