<script lang="ts">
  import RepositoryProfilePreviewDialog from '$lib/components/RepositoryProfilePreviewDialog.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Banner from '$lib/components/ui/Banner.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import EmptyState from '$lib/components/ui/EmptyState.svelte';
  import Panel from '$lib/components/ui/Panel.svelte';
  import type { GithubAccountsStatus } from '$lib/domain/github';
  import type { GitProfile } from '$lib/domain/profiles';
  import {
    configOriginLabel,
    configSourceLabel,
    evaluateRepositoryAssignment,
    originRemotes,
    repositoryAssignmentStatusLabels,
    repositoryName,
    type GitConfigValue,
    type RegisteredRepository,
    type RepositoryProfilePreview,
  } from '$lib/domain/repositories';

  type Props = {
    repositories: RegisteredRepository[];
    profiles: GitProfile[];
    githubStatus: GithubAccountsStatus | null;
    loading: boolean;
    error: string | null;
    success: string | null;
    adding: boolean;
    refreshingId: string | null;
    removingId: string | null;
    revealingId: string | null;
    assigningId: string | null;
    previewingId: string | null;
    applyingId: string | null;
    preview: RepositoryProfilePreview | null;
    onAdd: () => void | Promise<void>;
    onRefreshAll: () => void | Promise<void>;
    onRefresh: (id: string) => void | Promise<void>;
    onRemove: (id: string) => Promise<boolean>;
    onReveal: (id: string) => void | Promise<void>;
    onAssignProfile: (id: string, profileId: string | null) => void | Promise<void>;
    onPreviewProfile: (id: string) => void | Promise<void>;
    onCancelPreview: () => void;
    onApplyProfile: () => void | Promise<void>;
  };

  let {
    repositories,
    profiles,
    githubStatus,
    loading,
    error,
    success,
    adding,
    refreshingId,
    removingId,
    revealingId,
    assigningId,
    previewingId,
    applyingId,
    preview,
    onAdd,
    onRefreshAll,
    onRefresh,
    onRemove,
    onReveal,
    onAssignProfile,
    onPreviewProfile,
    onCancelPreview,
    onApplyProfile,
  }: Props = $props();

  let confirmingRemoveId = $state<string | null>(null);
  const operationPending = $derived(
    loading ||
      adding ||
      refreshingId !== null ||
      removingId !== null ||
      revealingId !== null ||
      assigningId !== null ||
      previewingId !== null ||
      applyingId !== null,
  );

  async function confirmRemove(id: string): Promise<void> {
    if (await onRemove(id)) {
      confirmingRemoveId = null;
    }
  }

  function identityValue(value: GitConfigValue): string {
    return value.value ?? 'Not configured';
  }

  function assignProfile(id: string, event: globalThis.Event): void {
    const profileId = (event.currentTarget as globalThis.HTMLSelectElement).value;
    void onAssignProfile(id, profileId || null);
  }

  function statusTone(status: string): 'positive' | 'neutral' | 'warning' {
    if (status === 'correct') return 'positive';
    if (status === 'unassigned') return 'neutral';
    return 'warning';
  }
</script>

<Panel
  eyebrow="Working repositories"
  heading="Registered repositories"
  headingId="repositories-heading"
  description="Assign expected profiles, detect mismatches, and apply only repository-local Git identity."
  busy={operationPending}
>
  {#snippet actions()}
    <Button
      onclick={onRefreshAll}
      disabled={operationPending}
      pending={loading}
      pendingLabel="Refreshing…"
    >
      Refresh all
    </Button>
    <Button
      variant="primary"
      onclick={onAdd}
      disabled={operationPending}
      pending={adding}
      pendingLabel="Selecting…"
    >
      Add repository
    </Button>
  {/snippet}

  <div>
    {#if error}<Banner tone="danger">{error}</Banner>{/if}
    {#if success}<Banner tone="positive">{success}</Banner>{/if}

    {#if repositories.length > 0}
      <ul class="space-y-4 p-6">
        {#each repositories as repository (repository.id)}
          {@const inspection = repository.inspection}
          {@const evaluation = evaluateRepositoryAssignment(repository, profiles, githubStatus)}
          {@const expectedAccount = evaluation.profile?.githubAccount}
          <li class="rounded-xl border border-edge bg-surface-raised p-5">
            <div class="flex flex-wrap items-start justify-between gap-4">
              <div class="min-w-0">
                <div class="flex flex-wrap items-center gap-2">
                  <h4 class="font-semibold text-fg-strong">{repositoryName(repository.path)}</h4>
                  <Badge tone={statusTone(evaluation.status)} uppercase>
                    {repositoryAssignmentStatusLabels[evaluation.status]}
                  </Badge>
                  {#if repository.state !== 'available'}
                    <Badge tone="warning" uppercase>{repository.state}</Badge>
                  {/if}
                </div>
                <p class="mt-1 break-all font-mono text-xs text-fg-subtle">{repository.path}</p>
              </div>

              <div class="flex items-center gap-3">
                <button
                  type="button"
                  class="text-xs font-medium text-fg-muted transition hover:text-fg-strong disabled:opacity-50"
                  onclick={() => onRefresh(repository.id)}
                  disabled={operationPending}
                  aria-label={`Refresh ${repositoryName(repository.path)}`}
                >
                  {refreshingId === repository.id ? 'Refreshing…' : 'Refresh'}
                </button>
                <button
                  type="button"
                  class="text-xs font-medium text-fg-muted transition hover:text-accent-text disabled:opacity-50"
                  onclick={() => onReveal(repository.id)}
                  disabled={operationPending || repository.state === 'missing'}
                  aria-label={`Reveal ${repositoryName(repository.path)}`}
                >
                  {revealingId === repository.id ? 'Opening…' : 'Reveal'}
                </button>
              </div>
            </div>

            <div class="mt-5 rounded-lg border border-edge bg-surface-sunken p-4">
              <label
                class="text-xs font-semibold uppercase tracking-[0.16em] text-fg-subtle"
                for={`profile-${repository.id}`}>Expected profile</label
              >
              <div class="mt-2 flex flex-wrap items-center gap-3">
                <select
                  id={`profile-${repository.id}`}
                  class="min-w-56 rounded-lg border border-edge bg-surface-input px-3 py-2 text-sm text-fg-strong outline-none transition focus:border-accent disabled:opacity-60"
                  value={repository.profileId ?? ''}
                  onchange={(event) => assignProfile(repository.id, event)}
                  disabled={operationPending}
                  aria-label={`Expected profile for ${repositoryName(repository.path)}`}
                >
                  <option value="">No profile assigned</option>
                  {#each profiles as profile (profile.id)}
                    <option value={profile.id}>{profile.label}</option>
                  {/each}
                  {#if repository.profileId && !evaluation.profile}
                    <option value={repository.profileId}>Missing profile</option>
                  {/if}
                </select>
                {#if assigningId === repository.id}
                  <span class="text-xs text-fg-muted">Saving assignment…</span>
                {/if}
                {#if evaluation.gitIdentityMatches === false && evaluation.profile}
                  <Button
                    variant="warning"
                    size="xs"
                    onclick={() => onPreviewProfile(repository.id)}
                    disabled={operationPending || repository.state !== 'available'}
                    pending={previewingId === repository.id}
                    pendingLabel="Preparing preview…"
                  >
                    Fix Git identity
                  </Button>
                {/if}
              </div>
              {#if repository.profileId && !evaluation.profile}
                <p class="mt-2 text-sm text-warning">
                  The assigned profile no longer exists. Select another profile or unassign it.
                </p>
              {:else if evaluation.profile}
                <p class="mt-2 text-sm text-fg">
                  {evaluation.profile.gitName} · {evaluation.profile.gitEmail}
                </p>
              {/if}
            </div>

            {#if repository.state === 'available' && inspection}
              <div class="mt-4 grid gap-4 md:grid-cols-2">
                <div class="rounded-lg border border-edge bg-surface-sunken p-4">
                  <p class="text-xs font-semibold uppercase tracking-[0.16em] text-fg-subtle">
                    Current Git name
                  </p>
                  <p class="mt-2 text-sm text-fg-strong">
                    {identityValue(inspection.identity.name)}
                  </p>
                  <p class="mt-1 text-xs text-fg-muted">
                    {configSourceLabel(inspection.identity.name.source)}
                  </p>
                  {#if configOriginLabel(inspection.identity.name.source)}
                    <p class="mt-1 break-all font-mono text-[11px] text-fg-faint">
                      {configOriginLabel(inspection.identity.name.source)}
                    </p>
                  {/if}
                </div>

                <div class="rounded-lg border border-edge bg-surface-sunken p-4">
                  <p class="text-xs font-semibold uppercase tracking-[0.16em] text-fg-subtle">
                    Current Git email
                  </p>
                  <p class="mt-2 break-all font-mono text-sm text-fg-strong">
                    {identityValue(inspection.identity.email)}
                  </p>
                  <p class="mt-1 text-xs text-fg-muted">
                    {configSourceLabel(inspection.identity.email.source)}
                  </p>
                  {#if configOriginLabel(inspection.identity.email.source)}
                    <p class="mt-1 break-all font-mono text-[11px] text-fg-faint">
                      {configOriginLabel(inspection.identity.email.source)}
                    </p>
                  {/if}
                </div>
              </div>

              <div class="mt-4 grid gap-4 md:grid-cols-2">
                <div class="rounded-lg border border-edge bg-surface-sunken p-4">
                  <p class="text-xs font-semibold uppercase tracking-[0.16em] text-fg-subtle">
                    GitHub account
                  </p>
                  {#if expectedAccount}
                    <p class="mt-2 text-sm text-fg-strong">
                      Expected @{expectedAccount.username} on {expectedAccount.hostname}
                    </p>
                    <p class="mt-1 text-xs text-fg-muted">
                      {evaluation.activeGithubAccount
                        ? `Active @${evaluation.activeGithubAccount.username}`
                        : evaluation.githubAccountMatch === 'unknown'
                          ? 'Account status unavailable'
                          : 'No healthy active account on this host'}
                    </p>
                    {#if evaluation.githubAccountMatch === 'mismatch'}
                      <p class="mt-2 text-xs text-warning">
                        Switch the active account from Overview.
                      </p>
                    {/if}
                  {:else}
                    <p class="mt-2 text-sm text-fg-muted">Not associated with this profile.</p>
                  {/if}
                </div>

                <div class="rounded-lg border border-edge bg-surface-sunken p-4">
                  <div class="flex items-center justify-between gap-3">
                    <p class="text-xs font-semibold uppercase tracking-[0.16em] text-fg-subtle">
                      Remotes
                    </p>
                    {#if originRemotes(inspection.remotes).length === 0}
                      <span class="text-xs text-fg-subtle">No origin remote</span>
                    {/if}
                  </div>
                  {#if inspection.remotes.length > 0}
                    <ul class="mt-3 space-y-2">
                      {#each inspection.remotes as remote (`${remote.name}-${remote.direction}-${remote.url}`)}
                        <li class="grid gap-1 text-xs sm:grid-cols-[70px_45px_1fr]">
                          <span class="font-medium text-fg">{remote.name}</span>
                          <span class="text-fg-subtle">{remote.direction}</span>
                          <span class="break-all font-mono text-fg-muted">{remote.url}</span>
                        </li>
                      {/each}
                    </ul>
                  {:else}
                    <p class="mt-2 text-sm text-fg-muted">No remotes configured.</p>
                  {/if}
                </div>
              </div>
            {:else}
              <div class="mt-5 rounded-lg border border-warning-edge bg-warning-softer p-4">
                <p class="text-sm text-warning">
                  {repository.message ?? 'This repository is currently unavailable.'}
                </p>
              </div>
            {/if}

            <div class="mt-4 border-t border-edge pt-4">
              {#if confirmingRemoveId === repository.id}
                <div class="rounded-lg border border-danger-edge bg-danger-softer p-3">
                  <p class="text-sm text-danger">
                    Remove this registration? Repository files and Git configuration will not be
                    changed.
                  </p>
                  <div class="mt-3 flex justify-end gap-2">
                    <Button
                      variant="ghost"
                      size="xs"
                      onclick={() => (confirmingRemoveId = null)}
                      disabled={removingId !== null}>Cancel</Button
                    >
                    <Button
                      variant="danger"
                      size="xs"
                      onclick={() => confirmRemove(repository.id)}
                      disabled={removingId !== null}
                      pending={removingId === repository.id}
                      pendingLabel="Removing…"
                    >
                      Remove registration
                    </Button>
                  </div>
                </div>
              {:else}
                <button
                  type="button"
                  class="text-xs font-medium text-fg-subtle transition hover:text-danger disabled:opacity-50"
                  onclick={() => (confirmingRemoveId = repository.id)}
                  disabled={operationPending}
                  aria-label={`Remove ${repositoryName(repository.path)}`}>Remove</button
                >
              {/if}
            </div>
          </li>
        {/each}
      </ul>
    {:else if loading}
      <div class="space-y-4 p-6" aria-label="Loading repositories">
        <div class="h-72 animate-pulse rounded-xl bg-hover"></div>
      </div>
    {:else}
      <EmptyState
        title="No repositories registered"
        description="Select a Git repository to inspect its effective identity and assign an expected profile."
      >
        {#snippet action()}
          <Button
            variant="primary"
            onclick={onAdd}
            disabled={operationPending}
            pending={adding}
            pendingLabel="Selecting…"
          >
            Add your first repository
          </Button>
        {/snippet}
      </EmptyState>
    {/if}
  </div>
</Panel>

{#if preview}
  <RepositoryProfilePreviewDialog
    {preview}
    applying={applyingId === preview.repositoryId}
    onCancel={onCancelPreview}
    onApply={onApplyProfile}
  />
{/if}
