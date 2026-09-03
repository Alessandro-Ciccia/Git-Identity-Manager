<script lang="ts">
  import Badge from '$lib/components/ui/Badge.svelte';
  import Banner from '$lib/components/ui/Banner.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import { dependencyLabels, type EnvironmentStatus } from '$lib/domain/environment';

  type Props = {
    status: EnvironmentStatus | null;
    envLoading: boolean;
    hasProfiles: boolean;
    hasRepositories: boolean;
    skipping: boolean;
    skipError: string | null;
    onRecheck: () => void | Promise<void>;
    onCreateProfile: () => void;
    onAddRepository: () => void;
    onSkip: () => void | Promise<void>;
  };

  let {
    status,
    envLoading,
    hasProfiles,
    hasRepositories,
    skipping,
    skipError,
    onRecheck,
    onCreateProfile,
    onAddRepository,
    onSkip,
  }: Props = $props();

  const dependencies = $derived(status ? [status.git, status.githubCli] : []);
  const environmentReady = $derived(status?.isReady ?? false);
</script>

<section aria-labelledby="welcome-heading" class="mx-auto max-w-2xl">
  <p class="text-sm font-medium text-accent-text">Welcome</p>
  <h2 id="welcome-heading" class="mt-3 text-4xl font-semibold tracking-tight text-fg-strong">
    Let’s set up Git Identity Manager
  </h2>
  <p class="mt-4 text-base leading-7 text-fg-muted">
    Three quick steps and you’ll be managing Git and GitHub identities safely. You can revisit any
    of this later from the sidebar.
  </p>

  {#if skipError}
    <div class="mt-6"><Banner tone="danger" variant="inset">{skipError}</Banner></div>
  {/if}

  <ol class="mt-10 space-y-4">
    <li class="rounded-2xl border border-edge bg-surface p-6">
      <div class="flex items-start justify-between gap-4">
        <div>
          <div class="flex items-center gap-2">
            <h3 class="font-semibold text-fg-strong">1. Check required tools</h3>
            <Badge tone={environmentReady ? 'positive' : 'warning'}>
              {environmentReady ? 'Ready' : 'Action needed'}
            </Badge>
          </div>
          <p class="mt-1 text-sm text-fg-muted">
            Git Identity Manager drives the <code>git</code> and <code>gh</code> command-line tools.
          </p>
        </div>
        <Button
          onclick={onRecheck}
          disabled={envLoading}
          pending={envLoading}
          pendingLabel="Checking…"
        >
          Recheck
        </Button>
      </div>
      <ul class="mt-4 space-y-2">
        {#each dependencies as dependency (dependency.dependency)}
          <li class="flex flex-wrap items-baseline gap-x-2 text-sm">
            <span class="font-medium text-fg-strong">{dependencyLabels[dependency.dependency]}</span
            >
            <Badge tone={dependency.state === 'available' ? 'positive' : 'warning'}>
              {dependency.state === 'available' ? `v${dependency.version}` : 'Not detected'}
            </Badge>
            <span class="w-full text-xs text-fg-muted sm:w-auto">{dependency.message}</span>
          </li>
        {/each}
      </ul>
    </li>

    <li class="rounded-2xl border border-edge bg-surface p-6">
      <div class="flex items-start justify-between gap-4">
        <div>
          <div class="flex items-center gap-2">
            <h3 class="font-semibold text-fg-strong">2. Create your first profile</h3>
            {#if hasProfiles}<Badge tone="positive">Done</Badge>{/if}
          </div>
          <p class="mt-1 text-sm text-fg-muted">
            A profile is a reusable Git name and email, optionally linked to a GitHub account.
          </p>
        </div>
        <Button variant="primary" onclick={onCreateProfile}>
          {hasProfiles ? 'Manage profiles' : 'Create profile'}
        </Button>
      </div>
    </li>

    <li class="rounded-2xl border border-edge bg-surface p-6">
      <div class="flex items-start justify-between gap-4">
        <div>
          <div class="flex items-center gap-2">
            <h3 class="font-semibold text-fg-strong">3. Register a repository</h3>
            {#if hasRepositories}<Badge tone="positive">Done</Badge>{/if}
          </div>
          <p class="mt-1 text-sm text-fg-muted">
            Point Git Identity Manager at a repository to inspect and correct its identity.
          </p>
        </div>
        <Button variant="primary" onclick={onAddRepository}>
          {hasRepositories ? 'Manage repositories' : 'Add repository'}
        </Button>
      </div>
    </li>
  </ol>

  <div class="mt-8 flex justify-end">
    <Button
      variant="ghost"
      onclick={onSkip}
      disabled={skipping}
      pending={skipping}
      pendingLabel="Skipping…"
    >
      Skip for now
    </Button>
  </div>
</section>
