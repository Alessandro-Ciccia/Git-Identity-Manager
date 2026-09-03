<script lang="ts">
  import RepositoryProfilePreviewDialog from '$lib/components/RepositoryProfilePreviewDialog.svelte';
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

  function statusClass(status: string): string {
    if (status === 'correct') {
      return 'border-emerald-400/20 bg-emerald-400/10 text-emerald-200';
    }
    if (status === 'unassigned') {
      return 'border-white/10 bg-white/5 text-stone-400';
    }
    return 'border-amber-400/20 bg-amber-400/10 text-amber-200';
  }
</script>

<section
  class="overflow-hidden rounded-2xl border border-white/10 bg-stone-900/50"
  aria-labelledby="repositories-heading"
  aria-busy={operationPending}
>
  <header
    class="flex flex-wrap items-start justify-between gap-5 border-b border-white/10 px-6 py-5"
  >
    <div>
      <p class="text-xs font-semibold uppercase tracking-[0.2em] text-stone-500">
        Working repositories
      </p>
      <h3 id="repositories-heading" class="mt-2 text-lg font-semibold text-white">
        Registered repositories
      </h3>
      <p class="mt-1 max-w-2xl text-sm text-stone-400">
        Assign expected profiles, detect mismatches, and apply only repository-local Git identity.
      </p>
    </div>

    <div class="flex items-center gap-2">
      <button
        type="button"
        class="rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm font-medium text-stone-200 transition hover:border-white/20 hover:bg-white/10 disabled:cursor-wait disabled:opacity-60"
        onclick={onRefreshAll}
        disabled={operationPending}
      >
        {loading ? 'Refreshing…' : 'Refresh all'}
      </button>
      <button
        type="button"
        class="rounded-lg bg-sky-300 px-3 py-2 text-sm font-semibold text-stone-950 transition hover:bg-sky-200 disabled:cursor-wait disabled:opacity-60"
        onclick={onAdd}
        disabled={operationPending}
      >
        {adding ? 'Selecting…' : 'Add repository'}
      </button>
    </div>
  </header>

  <div aria-live="polite">
    {#if error}
      <div
        class="border-b border-rose-400/20 bg-rose-400/5 px-6 py-4 text-sm text-rose-200"
        role="alert"
      >
        {error}
      </div>
    {/if}
    {#if success}
      <div
        class="border-b border-emerald-400/20 bg-emerald-400/5 px-6 py-4 text-sm text-emerald-200"
        role="status"
      >
        {success}
      </div>
    {/if}

    {#if repositories.length > 0}
      <ul class="space-y-4 p-6">
        {#each repositories as repository (repository.id)}
          {@const inspection = repository.inspection}
          {@const evaluation = evaluateRepositoryAssignment(repository, profiles, githubStatus)}
          {@const expectedAccount = evaluation.profile?.githubAccount}
          <li class="rounded-xl border border-white/10 bg-black/10 p-5">
            <div class="flex flex-wrap items-start justify-between gap-4">
              <div class="min-w-0">
                <div class="flex flex-wrap items-center gap-2">
                  <h4 class="font-semibold text-white">{repositoryName(repository.path)}</h4>
                  <span
                    class={`rounded-full border px-2 py-0.5 text-[11px] font-semibold uppercase tracking-wide ${statusClass(evaluation.status)}`}
                  >
                    {repositoryAssignmentStatusLabels[evaluation.status]}
                  </span>
                  {#if repository.state !== 'available'}
                    <span
                      class="rounded-full border border-amber-400/20 bg-amber-400/10 px-2 py-0.5 text-[11px] font-semibold uppercase tracking-wide text-amber-200"
                    >
                      {repository.state}
                    </span>
                  {/if}
                </div>
                <p class="mt-1 break-all font-mono text-xs text-stone-500">{repository.path}</p>
              </div>

              <div class="flex items-center gap-3">
                <button
                  type="button"
                  class="text-xs font-medium text-stone-400 transition hover:text-white disabled:opacity-50"
                  onclick={() => onRefresh(repository.id)}
                  disabled={operationPending}
                  aria-label={`Refresh ${repositoryName(repository.path)}`}
                >
                  {refreshingId === repository.id ? 'Refreshing…' : 'Refresh'}
                </button>
                <button
                  type="button"
                  class="text-xs font-medium text-stone-400 transition hover:text-sky-200 disabled:opacity-50"
                  onclick={() => onReveal(repository.id)}
                  disabled={operationPending || repository.state === 'missing'}
                  aria-label={`Reveal ${repositoryName(repository.path)}`}
                >
                  {revealingId === repository.id ? 'Opening…' : 'Reveal'}
                </button>
              </div>
            </div>

            <div class="mt-5 rounded-lg border border-white/8 bg-stone-950/40 p-4">
              <label
                class="text-xs font-semibold uppercase tracking-[0.16em] text-stone-500"
                for={`profile-${repository.id}`}>Expected profile</label
              >
              <div class="mt-2 flex flex-wrap items-center gap-3">
                <select
                  id={`profile-${repository.id}`}
                  class="min-w-56 rounded-lg border border-white/10 bg-stone-900 px-3 py-2 text-sm text-stone-100 outline-none transition focus:border-sky-300 disabled:opacity-60"
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
                  <span class="text-xs text-stone-400">Saving assignment…</span>
                {/if}
                {#if evaluation.gitIdentityMatches === false && evaluation.profile}
                  <button
                    type="button"
                    class="rounded-lg bg-amber-200 px-3 py-2 text-xs font-semibold text-stone-950 transition hover:bg-amber-100 disabled:cursor-wait disabled:opacity-60"
                    onclick={() => onPreviewProfile(repository.id)}
                    disabled={operationPending || repository.state !== 'available'}
                  >
                    {previewingId === repository.id ? 'Preparing preview…' : 'Fix Git identity'}
                  </button>
                {/if}
              </div>
              {#if repository.profileId && !evaluation.profile}
                <p class="mt-2 text-sm text-amber-200">
                  The assigned profile no longer exists. Select another profile or unassign it.
                </p>
              {:else if evaluation.profile}
                <p class="mt-2 text-sm text-stone-300">
                  {evaluation.profile.gitName} · {evaluation.profile.gitEmail}
                </p>
              {/if}
            </div>

            {#if repository.state === 'available' && inspection}
              <div class="mt-4 grid gap-4 md:grid-cols-2">
                <div class="rounded-lg border border-white/8 bg-stone-950/40 p-4">
                  <p class="text-xs font-semibold uppercase tracking-[0.16em] text-stone-500">
                    Current Git name
                  </p>
                  <p class="mt-2 text-sm text-stone-100">
                    {identityValue(inspection.identity.name)}
                  </p>
                  <p class="mt-1 text-xs text-stone-400">
                    {configSourceLabel(inspection.identity.name.source)}
                  </p>
                  {#if configOriginLabel(inspection.identity.name.source)}
                    <p class="mt-1 break-all font-mono text-[11px] text-stone-600">
                      {configOriginLabel(inspection.identity.name.source)}
                    </p>
                  {/if}
                </div>

                <div class="rounded-lg border border-white/8 bg-stone-950/40 p-4">
                  <p class="text-xs font-semibold uppercase tracking-[0.16em] text-stone-500">
                    Current Git email
                  </p>
                  <p class="mt-2 break-all font-mono text-sm text-stone-100">
                    {identityValue(inspection.identity.email)}
                  </p>
                  <p class="mt-1 text-xs text-stone-400">
                    {configSourceLabel(inspection.identity.email.source)}
                  </p>
                  {#if configOriginLabel(inspection.identity.email.source)}
                    <p class="mt-1 break-all font-mono text-[11px] text-stone-600">
                      {configOriginLabel(inspection.identity.email.source)}
                    </p>
                  {/if}
                </div>
              </div>

              <div class="mt-4 grid gap-4 md:grid-cols-2">
                <div class="rounded-lg border border-white/8 bg-stone-950/40 p-4">
                  <p class="text-xs font-semibold uppercase tracking-[0.16em] text-stone-500">
                    GitHub account
                  </p>
                  {#if expectedAccount}
                    <p class="mt-2 text-sm text-stone-100">
                      Expected @{expectedAccount.username} on {expectedAccount.hostname}
                    </p>
                    <p class="mt-1 text-xs text-stone-400">
                      {evaluation.activeGithubAccount
                        ? `Active @${evaluation.activeGithubAccount.username}`
                        : evaluation.githubAccountMatch === 'unknown'
                          ? 'Account status unavailable'
                          : 'No healthy active account on this host'}
                    </p>
                    {#if evaluation.githubAccountMatch === 'mismatch'}
                      <p class="mt-2 text-xs text-amber-200">
                        Switch the active account from Overview.
                      </p>
                    {/if}
                  {:else}
                    <p class="mt-2 text-sm text-stone-400">Not associated with this profile.</p>
                  {/if}
                </div>

                <div class="rounded-lg border border-white/8 bg-stone-950/40 p-4">
                  <div class="flex items-center justify-between gap-3">
                    <p class="text-xs font-semibold uppercase tracking-[0.16em] text-stone-500">
                      Remotes
                    </p>
                    {#if originRemotes(inspection.remotes).length === 0}
                      <span class="text-xs text-stone-500">No origin remote</span>
                    {/if}
                  </div>
                  {#if inspection.remotes.length > 0}
                    <ul class="mt-3 space-y-2">
                      {#each inspection.remotes as remote (`${remote.name}-${remote.direction}-${remote.url}`)}
                        <li class="grid gap-1 text-xs sm:grid-cols-[70px_45px_1fr]">
                          <span class="font-medium text-stone-300">{remote.name}</span>
                          <span class="text-stone-500">{remote.direction}</span>
                          <span class="break-all font-mono text-stone-400">{remote.url}</span>
                        </li>
                      {/each}
                    </ul>
                  {:else}
                    <p class="mt-2 text-sm text-stone-400">No remotes configured.</p>
                  {/if}
                </div>
              </div>
            {:else}
              <div class="mt-5 rounded-lg border border-amber-400/15 bg-amber-400/5 p-4">
                <p class="text-sm text-amber-100">
                  {repository.message ?? 'This repository is currently unavailable.'}
                </p>
              </div>
            {/if}

            <div class="mt-4 border-t border-white/10 pt-4">
              {#if confirmingRemoveId === repository.id}
                <div class="rounded-lg border border-rose-400/20 bg-rose-400/5 p-3">
                  <p class="text-sm text-rose-100">
                    Remove this registration? Repository files and Git configuration will not be
                    changed.
                  </p>
                  <div class="mt-3 flex justify-end gap-2">
                    <button
                      type="button"
                      class="rounded-md px-2.5 py-1.5 text-xs font-semibold text-stone-300 hover:bg-white/5 disabled:opacity-60"
                      onclick={() => (confirmingRemoveId = null)}
                      disabled={removingId !== null}>Cancel</button
                    >
                    <button
                      type="button"
                      class="rounded-md bg-rose-300 px-2.5 py-1.5 text-xs font-semibold text-stone-950 hover:bg-rose-200 disabled:cursor-wait disabled:opacity-60"
                      onclick={() => confirmRemove(repository.id)}
                      disabled={removingId !== null}
                    >
                      {removingId === repository.id ? 'Removing…' : 'Remove registration'}
                    </button>
                  </div>
                </div>
              {:else}
                <button
                  type="button"
                  class="text-xs font-medium text-stone-500 transition hover:text-rose-300 disabled:opacity-50"
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
        <div class="h-72 animate-pulse rounded-xl bg-white/5"></div>
      </div>
    {:else}
      <div class="px-6 py-10">
        <p class="font-medium text-white">No repositories registered</p>
        <p class="mt-2 max-w-xl text-sm leading-6 text-stone-400">
          Select a Git repository to inspect its effective identity and assign an expected profile.
        </p>
      </div>
    {/if}
  </div>
</section>

{#if preview}
  <RepositoryProfilePreviewDialog
    {preview}
    applying={applyingId === preview.repositoryId}
    onCancel={onCancelPreview}
    onApply={onApplyProfile}
  />
{/if}
