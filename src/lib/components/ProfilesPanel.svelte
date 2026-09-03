<script lang="ts">
  import Badge from '$lib/components/ui/Badge.svelte';
  import Banner from '$lib/components/ui/Banner.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import EmptyState from '$lib/components/ui/EmptyState.svelte';
  import Panel from '$lib/components/ui/Panel.svelte';
  import { groupGithubAccounts, type GithubAccount } from '$lib/domain/github';
  import {
    emptyProfileDraft,
    findAssociatedAccount,
    githubReferenceKey,
    hasProfileValidationErrors,
    profileDraft,
    profileInputFromDraft,
    validateProfileDraft,
    type GitProfile,
    type ProfileDraft,
    type ProfileInput,
    type ProfileValidationErrors,
  } from '$lib/domain/profiles';

  type Props = {
    profiles: GitProfile[];
    accounts: GithubAccount[];
    loading: boolean;
    error: string | null;
    savingProfileId: string | null;
    deletingProfileId: string | null;
    onRefresh: () => void | Promise<void>;
    onCreate: (profile: ProfileInput) => Promise<boolean>;
    onUpdate: (id: string, profile: ProfileInput) => Promise<boolean>;
    onDelete: (id: string) => Promise<boolean>;
  };

  let {
    profiles,
    accounts,
    loading,
    error,
    savingProfileId,
    deletingProfileId,
    onRefresh,
    onCreate,
    onUpdate,
    onDelete,
  }: Props = $props();

  let formMode = $state<'create' | 'edit' | null>(null);
  let editingProfileId = $state<string | null>(null);
  let draft = $state<ProfileDraft>({ ...emptyProfileDraft });
  let validationErrors = $state<ProfileValidationErrors>({});
  let confirmingDeleteId = $state<string | null>(null);

  const accountGroups = $derived(groupGithubAccounts(accounts));
  const operationPending = $derived(
    loading || savingProfileId !== null || deletingProfileId !== null,
  );
  const editingProfile = $derived(
    editingProfileId ? (profiles.find((profile) => profile.id === editingProfileId) ?? null) : null,
  );
  const staleEditingReference = $derived(
    editingProfile?.githubAccount && !findAssociatedAccount(editingProfile.githubAccount, accounts)
      ? editingProfile.githubAccount
      : null,
  );

  function startCreate(): void {
    formMode = 'create';
    editingProfileId = null;
    draft = { ...emptyProfileDraft };
    validationErrors = {};
    confirmingDeleteId = null;
  }

  function startEdit(profile: GitProfile): void {
    formMode = 'edit';
    editingProfileId = profile.id;
    draft = profileDraft(profile);
    validationErrors = {};
    confirmingDeleteId = null;
  }

  function closeForm(): void {
    formMode = null;
    editingProfileId = null;
    validationErrors = {};
  }

  async function submitProfile(event: globalThis.SubmitEvent): Promise<void> {
    event.preventDefault();
    validationErrors = validateProfileDraft(draft);
    if (hasProfileValidationErrors(validationErrors)) {
      return;
    }

    const input = profileInputFromDraft(draft);
    if (!input) {
      return;
    }

    const saved =
      formMode === 'edit' && editingProfileId
        ? await onUpdate(editingProfileId, input)
        : await onCreate(input);
    if (saved) {
      closeForm();
    }
  }

  async function confirmDelete(id: string): Promise<void> {
    if (await onDelete(id)) {
      confirmingDeleteId = null;
      if (editingProfileId === id) {
        closeForm();
      }
    }
  }

  function associationLabel(profile: GitProfile): string {
    if (!profile.githubAccount) {
      return 'No GitHub account associated';
    }
    return `@${profile.githubAccount.username} · ${profile.githubAccount.hostname}`;
  }
</script>

<Panel
  eyebrow="Local identities"
  heading="Git profiles"
  headingId="profiles-heading"
  description="Reusable names and emails stored locally without authentication credentials."
  busy={operationPending}
>
  {#snippet actions()}
    <Button
      onclick={onRefresh}
      disabled={operationPending}
      pending={loading}
      pendingLabel="Refreshing…"
    >
      Refresh
    </Button>
    <Button
      variant="primary"
      onclick={startCreate}
      disabled={operationPending || formMode === 'create'}
    >
      New profile
    </Button>
  {/snippet}

  <div>
    {#if error}<Banner tone="danger">{error}</Banner>{/if}

    {#if formMode}
      <form
        class="border-b border-accent-edge bg-accent-softer px-6 py-6"
        onsubmit={submitProfile}
        novalidate
      >
        <div class="flex items-start justify-between gap-4">
          <div>
            <p class="text-xs font-semibold uppercase tracking-[0.18em] text-accent-text">
              {formMode === 'create' ? 'New profile' : 'Edit profile'}
            </p>
            <h4 class="mt-2 font-semibold text-fg-strong">
              {formMode === 'create'
                ? 'Define a Git identity'
                : `Update ${editingProfile?.label ?? 'profile'}`}
            </h4>
          </div>
          <button
            type="button"
            class="text-sm font-medium text-fg-muted transition hover:text-fg-strong disabled:opacity-60"
            onclick={closeForm}
            disabled={savingProfileId !== null}
          >
            Cancel
          </button>
        </div>

        <div class="mt-5 grid gap-5 sm:grid-cols-2">
          <div class="block sm:col-span-2">
            <label for="profile-label" class="text-sm font-medium text-fg">Profile label</label>
            <input
              id="profile-label"
              class="mt-2 w-full rounded-lg border border-edge bg-surface-input px-3 py-2.5 text-sm text-fg-strong placeholder:text-fg-faint focus:border-accent disabled:opacity-60"
              bind:value={draft.label}
              aria-invalid={validationErrors.label ? 'true' : undefined}
              aria-describedby={validationErrors.label ? 'profile-label-error' : undefined}
              placeholder="Personal"
              disabled={savingProfileId !== null}
            />
            {#if validationErrors.label}
              <span id="profile-label-error" class="mt-1.5 block text-xs text-danger"
                >{validationErrors.label}</span
              >
            {/if}
          </div>

          <div class="block">
            <label for="profile-name" class="text-sm font-medium text-fg">Git name</label>
            <input
              id="profile-name"
              class="mt-2 w-full rounded-lg border border-edge bg-surface-input px-3 py-2.5 text-sm text-fg-strong placeholder:text-fg-faint focus:border-accent disabled:opacity-60"
              bind:value={draft.gitName}
              aria-invalid={validationErrors.gitName ? 'true' : undefined}
              aria-describedby={validationErrors.gitName ? 'profile-name-error' : undefined}
              placeholder="Octo Cat"
              disabled={savingProfileId !== null}
            />
            {#if validationErrors.gitName}
              <span id="profile-name-error" class="mt-1.5 block text-xs text-danger"
                >{validationErrors.gitName}</span
              >
            {/if}
          </div>

          <div class="block">
            <label for="profile-email" class="text-sm font-medium text-fg">Git email</label>
            <input
              id="profile-email"
              type="email"
              class="mt-2 w-full rounded-lg border border-edge bg-surface-input px-3 py-2.5 text-sm text-fg-strong placeholder:text-fg-faint focus:border-accent disabled:opacity-60"
              bind:value={draft.gitEmail}
              aria-invalid={validationErrors.gitEmail ? 'true' : undefined}
              aria-describedby={validationErrors.gitEmail ? 'profile-email-error' : undefined}
              placeholder="octo@example.com"
              disabled={savingProfileId !== null}
            />
            {#if validationErrors.gitEmail}
              <span id="profile-email-error" class="mt-1.5 block text-xs text-danger"
                >{validationErrors.gitEmail}</span
              >
            {/if}
          </div>

          <div class="block sm:col-span-2">
            <label for="profile-account" class="text-sm font-medium text-fg">GitHub account</label>
            <select
              id="profile-account"
              class="mt-2 w-full rounded-lg border border-edge bg-surface-input px-3 py-2.5 text-sm text-fg-strong focus:border-accent disabled:opacity-60"
              bind:value={draft.githubAccountKey}
              aria-invalid={validationErrors.githubAccount ? 'true' : undefined}
              aria-describedby={validationErrors.githubAccount
                ? 'profile-account-error'
                : 'profile-account-help'}
              disabled={savingProfileId !== null}
            >
              <option value="">No association</option>
              {#if staleEditingReference}
                <option value={githubReferenceKey(staleEditingReference)}>
                  @{staleEditingReference.username} · {staleEditingReference.hostname} (not currently
                  discovered)
                </option>
              {/if}
              {#each accountGroups as group (group.hostname)}
                <optgroup label={group.hostname}>
                  {#each group.accounts as account (`${account.hostname}::${account.username}`)}
                    <option value={githubReferenceKey(account)}>
                      @{account.username}{account.active ? ' · active' : ''}
                    </option>
                  {/each}
                </optgroup>
              {/each}
            </select>
            {#if validationErrors.githubAccount}
              <span id="profile-account-error" class="mt-1.5 block text-xs text-danger"
                >{validationErrors.githubAccount}</span
              >
            {:else}
              <span id="profile-account-help" class="mt-1.5 block text-xs text-fg-subtle">
                Only the account hostname and username are saved.
              </span>
            {/if}
          </div>
        </div>

        <div class="mt-6 flex justify-end">
          <Button type="submit" variant="primary" size="md" disabled={savingProfileId !== null}>
            {savingProfileId !== null
              ? 'Saving…'
              : formMode === 'create'
                ? 'Create profile'
                : 'Save changes'}
          </Button>
        </div>
      </form>
    {/if}

    {#if profiles.length > 0}
      <ul class="grid gap-4 p-6 sm:grid-cols-2">
        {#each profiles as profile (profile.id)}
          {@const associatedAccount = profile.githubAccount
            ? findAssociatedAccount(profile.githubAccount, accounts)
            : undefined}
          <li class="rounded-xl border border-edge bg-surface-raised p-5">
            <div class="flex items-start justify-between gap-4">
              <div class="min-w-0">
                <h4 class="truncate font-semibold text-fg-strong">{profile.label}</h4>
                <p class="mt-3 text-sm text-fg">{profile.gitName}</p>
                <p class="mt-1 break-all font-mono text-xs text-fg-muted">{profile.gitEmail}</p>
              </div>
              <button
                type="button"
                class="shrink-0 text-sm font-medium text-fg-muted transition hover:text-accent-text disabled:opacity-60"
                onclick={() => startEdit(profile)}
                disabled={operationPending}
                aria-label={`Edit ${profile.label}`}>Edit</button
              >
            </div>

            <div class="mt-5 border-t border-edge pt-4">
              <div class="flex flex-wrap items-center gap-2">
                <p class="text-xs text-fg-muted">{associationLabel(profile)}</p>
                {#if profile.githubAccount && !associatedAccount}
                  <Badge tone="warning">Not discovered</Badge>
                {:else if associatedAccount?.active}
                  <Badge tone="accent">Active</Badge>
                {/if}
              </div>

              {#if confirmingDeleteId === profile.id}
                <div class="mt-4 rounded-lg border border-danger-edge bg-danger-softer p-3">
                  <p class="text-sm text-danger">
                    Delete this local profile? This cannot be undone.
                  </p>
                  <div class="mt-3 flex justify-end gap-2">
                    <Button
                      variant="ghost"
                      size="xs"
                      onclick={() => (confirmingDeleteId = null)}
                      disabled={deletingProfileId !== null}>Cancel</Button
                    >
                    <Button
                      variant="danger"
                      size="xs"
                      onclick={() => confirmDelete(profile.id)}
                      disabled={deletingProfileId !== null}
                      pending={deletingProfileId === profile.id}
                      pendingLabel="Deleting…">Delete profile</Button
                    >
                  </div>
                </div>
              {:else}
                <button
                  type="button"
                  class="mt-3 text-xs font-medium text-fg-subtle transition hover:text-danger disabled:opacity-60"
                  onclick={() => (confirmingDeleteId = profile.id)}
                  disabled={operationPending}
                  aria-label={`Delete ${profile.label}`}>Delete</button
                >
              {/if}
            </div>
          </li>
        {/each}
      </ul>
    {:else if loading}
      <div class="grid gap-4 p-6 sm:grid-cols-2" aria-label="Loading profiles">
        <div class="h-44 animate-pulse rounded-xl bg-hover"></div>
        <div class="h-44 animate-pulse rounded-xl bg-hover"></div>
      </div>
    {:else if !formMode}
      <EmptyState
        title="No profiles yet"
        description="Create a reusable Git name and email, then assign it to repositories and directories to keep your identity consistent."
      >
        {#snippet action()}
          <Button variant="primary" onclick={startCreate} disabled={operationPending}>
            Create your first profile
          </Button>
        {/snippet}
      </EmptyState>
    {/if}
  </div>
</Panel>
