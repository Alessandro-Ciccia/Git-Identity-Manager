<script lang="ts">
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

<section
  class="overflow-hidden rounded-2xl border border-white/10 bg-stone-900/50"
  aria-labelledby="profiles-heading"
  aria-busy={operationPending}
>
  <header
    class="flex flex-wrap items-start justify-between gap-5 border-b border-white/10 px-6 py-5"
  >
    <div>
      <p class="text-xs font-semibold uppercase tracking-[0.2em] text-stone-500">
        Local identities
      </p>
      <h3 id="profiles-heading" class="mt-2 text-lg font-semibold text-white">Git profiles</h3>
      <p class="mt-1 max-w-2xl text-sm text-stone-400">
        Reusable names and emails stored locally without authentication credentials.
      </p>
    </div>

    <div class="flex items-center gap-2">
      <button
        type="button"
        class="rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm font-medium text-stone-200 transition hover:border-white/20 hover:bg-white/10 disabled:cursor-wait disabled:opacity-60"
        onclick={onRefresh}
        disabled={operationPending}
      >
        {loading ? 'Refreshing…' : 'Refresh'}
      </button>
      <button
        type="button"
        class="rounded-lg bg-sky-300 px-3 py-2 text-sm font-semibold text-stone-950 transition hover:bg-sky-200 disabled:cursor-wait disabled:opacity-60"
        onclick={startCreate}
        disabled={operationPending || formMode === 'create'}
      >
        New profile
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

    {#if formMode}
      <form
        class="border-b border-sky-300/15 bg-sky-300/5 px-6 py-6"
        onsubmit={submitProfile}
        novalidate
      >
        <div class="flex items-start justify-between gap-4">
          <div>
            <p class="text-xs font-semibold uppercase tracking-[0.18em] text-sky-300">
              {formMode === 'create' ? 'New profile' : 'Edit profile'}
            </p>
            <h4 class="mt-2 font-semibold text-white">
              {formMode === 'create'
                ? 'Define a Git identity'
                : `Update ${editingProfile?.label ?? 'profile'}`}
            </h4>
          </div>
          <button
            type="button"
            class="text-sm font-medium text-stone-400 transition hover:text-white disabled:opacity-60"
            onclick={closeForm}
            disabled={savingProfileId !== null}
          >
            Cancel
          </button>
        </div>

        <div class="mt-5 grid gap-5 sm:grid-cols-2">
          <div class="block sm:col-span-2">
            <label for="profile-label" class="text-sm font-medium text-stone-200"
              >Profile label</label
            >
            <input
              id="profile-label"
              class="mt-2 w-full rounded-lg border border-white/10 bg-stone-950/70 px-3 py-2.5 text-sm text-white placeholder:text-stone-600 focus:border-sky-300/50 disabled:opacity-60"
              bind:value={draft.label}
              aria-invalid={validationErrors.label ? 'true' : undefined}
              aria-describedby={validationErrors.label ? 'profile-label-error' : undefined}
              placeholder="Personal"
              disabled={savingProfileId !== null}
            />
            {#if validationErrors.label}
              <span id="profile-label-error" class="mt-1.5 block text-xs text-rose-300"
                >{validationErrors.label}</span
              >
            {/if}
          </div>

          <div class="block">
            <label for="profile-name" class="text-sm font-medium text-stone-200">Git name</label>
            <input
              id="profile-name"
              class="mt-2 w-full rounded-lg border border-white/10 bg-stone-950/70 px-3 py-2.5 text-sm text-white placeholder:text-stone-600 focus:border-sky-300/50 disabled:opacity-60"
              bind:value={draft.gitName}
              aria-invalid={validationErrors.gitName ? 'true' : undefined}
              aria-describedby={validationErrors.gitName ? 'profile-name-error' : undefined}
              placeholder="Octo Cat"
              disabled={savingProfileId !== null}
            />
            {#if validationErrors.gitName}
              <span id="profile-name-error" class="mt-1.5 block text-xs text-rose-300"
                >{validationErrors.gitName}</span
              >
            {/if}
          </div>

          <div class="block">
            <label for="profile-email" class="text-sm font-medium text-stone-200">Git email</label>
            <input
              id="profile-email"
              type="email"
              class="mt-2 w-full rounded-lg border border-white/10 bg-stone-950/70 px-3 py-2.5 text-sm text-white placeholder:text-stone-600 focus:border-sky-300/50 disabled:opacity-60"
              bind:value={draft.gitEmail}
              aria-invalid={validationErrors.gitEmail ? 'true' : undefined}
              aria-describedby={validationErrors.gitEmail ? 'profile-email-error' : undefined}
              placeholder="octo@example.com"
              disabled={savingProfileId !== null}
            />
            {#if validationErrors.gitEmail}
              <span id="profile-email-error" class="mt-1.5 block text-xs text-rose-300"
                >{validationErrors.gitEmail}</span
              >
            {/if}
          </div>

          <div class="block sm:col-span-2">
            <label for="profile-account" class="text-sm font-medium text-stone-200"
              >GitHub account</label
            >
            <select
              id="profile-account"
              class="mt-2 w-full rounded-lg border border-white/10 bg-stone-950/70 px-3 py-2.5 text-sm text-white focus:border-sky-300/50 disabled:opacity-60"
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
              <span id="profile-account-error" class="mt-1.5 block text-xs text-rose-300"
                >{validationErrors.githubAccount}</span
              >
            {:else}
              <span id="profile-account-help" class="mt-1.5 block text-xs text-stone-500">
                Only the account hostname and username are saved.
              </span>
            {/if}
          </div>
        </div>

        <div class="mt-6 flex justify-end">
          <button
            type="submit"
            class="rounded-lg bg-sky-300 px-4 py-2.5 text-sm font-semibold text-stone-950 transition hover:bg-sky-200 disabled:cursor-wait disabled:opacity-60"
            disabled={savingProfileId !== null}
          >
            {savingProfileId !== null
              ? 'Saving…'
              : formMode === 'create'
                ? 'Create profile'
                : 'Save changes'}
          </button>
        </div>
      </form>
    {/if}

    {#if profiles.length > 0}
      <ul class="grid gap-4 p-6 sm:grid-cols-2">
        {#each profiles as profile (profile.id)}
          {@const associatedAccount = profile.githubAccount
            ? findAssociatedAccount(profile.githubAccount, accounts)
            : undefined}
          <li class="rounded-xl border border-white/10 bg-black/10 p-5">
            <div class="flex items-start justify-between gap-4">
              <div class="min-w-0">
                <h4 class="truncate font-semibold text-white">{profile.label}</h4>
                <p class="mt-3 text-sm text-stone-200">{profile.gitName}</p>
                <p class="mt-1 break-all font-mono text-xs text-stone-400">{profile.gitEmail}</p>
              </div>
              <button
                type="button"
                class="shrink-0 text-sm font-medium text-stone-400 transition hover:text-sky-200 disabled:opacity-60"
                onclick={() => startEdit(profile)}
                disabled={operationPending}
                aria-label={`Edit ${profile.label}`}>Edit</button
              >
            </div>

            <div class="mt-5 border-t border-white/10 pt-4">
              <div class="flex flex-wrap items-center gap-2">
                <p class="text-xs text-stone-400">{associationLabel(profile)}</p>
                {#if profile.githubAccount && !associatedAccount}
                  <span
                    class="rounded-full border border-amber-400/20 bg-amber-400/10 px-2 py-0.5 text-[11px] font-semibold text-amber-200"
                  >
                    Not discovered
                  </span>
                {:else if associatedAccount?.active}
                  <span
                    class="rounded-full border border-sky-300/20 bg-sky-300/10 px-2 py-0.5 text-[11px] font-semibold text-sky-200"
                  >
                    Active
                  </span>
                {/if}
              </div>

              {#if confirmingDeleteId === profile.id}
                <div class="mt-4 rounded-lg border border-rose-400/20 bg-rose-400/5 p-3">
                  <p class="text-sm text-rose-100">
                    Delete this local profile? This cannot be undone.
                  </p>
                  <div class="mt-3 flex justify-end gap-2">
                    <button
                      type="button"
                      class="rounded-md px-2.5 py-1.5 text-xs font-semibold text-stone-300 hover:bg-white/5 disabled:opacity-60"
                      onclick={() => (confirmingDeleteId = null)}
                      disabled={deletingProfileId !== null}>Cancel</button
                    >
                    <button
                      type="button"
                      class="rounded-md bg-rose-300 px-2.5 py-1.5 text-xs font-semibold text-stone-950 hover:bg-rose-200 disabled:cursor-wait disabled:opacity-60"
                      onclick={() => confirmDelete(profile.id)}
                      disabled={deletingProfileId !== null}
                      >{deletingProfileId === profile.id ? 'Deleting…' : 'Delete profile'}</button
                    >
                  </div>
                </div>
              {:else}
                <button
                  type="button"
                  class="mt-3 text-xs font-medium text-stone-500 transition hover:text-rose-300 disabled:opacity-60"
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
        <div class="h-44 animate-pulse rounded-xl bg-white/5"></div>
        <div class="h-44 animate-pulse rounded-xl bg-white/5"></div>
      </div>
    {:else if !formMode}
      <div class="px-6 py-10">
        <p class="font-medium text-white">No profiles yet</p>
        <p class="mt-2 max-w-xl text-sm leading-6 text-stone-400">
          Create a reusable Git name and email. Profiles only change repositories in a later
          milestone.
        </p>
      </div>
    {/if}
  </div>
</section>
