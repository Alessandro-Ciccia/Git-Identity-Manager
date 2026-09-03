<script lang="ts">
  import {
    directoryRuleOperationLabel,
    directoryRuleProfile,
    directoryRuleStateLabels,
    type DirectoryRule,
    type DirectoryRuleInput,
    type DirectoryRulePreview,
  } from '$lib/domain/directoryRules';
  import type { GitProfile } from '$lib/domain/profiles';

  type Props = {
    rules: DirectoryRule[];
    profiles: GitProfile[];
    loading: boolean;
    error: string | null;
    success: string | null;
    pending: boolean;
    preview: DirectoryRulePreview | null;
    onRefresh: () => void | Promise<void>;
    onChooseDirectory: () => Promise<string | null>;
    onPreview: (input: DirectoryRuleInput) => void | Promise<void>;
    onPreviewRemove: (id: string) => void | Promise<void>;
    onCancelPreview: () => void;
    onApply: () => void | Promise<void>;
  };

  let {
    rules,
    profiles,
    loading,
    error,
    success,
    pending,
    preview,
    onRefresh,
    onChooseDirectory,
    onPreview,
    onPreviewRemove,
    onCancelPreview,
    onApply,
  }: Props = $props();

  let editingId = $state<string | null>(null);
  let directory = $state('');
  let profileId = $state('');

  async function chooseDirectory(): Promise<void> {
    const selected = await onChooseDirectory();
    if (selected) directory = selected;
  }

  function beginAdd(): void {
    editingId = null;
    directory = '';
    profileId = profiles[0]?.id ?? '';
  }

  function beginEdit(rule: DirectoryRule): void {
    editingId = rule.id;
    directory = rule.directory;
    profileId = rule.profileId;
  }

  function requestPreview(): void {
    if (!directory || !profileId) return;
    void onPreview({ id: editingId, directory, profileId });
  }

  function stateClass(state: DirectoryRule['state']): string {
    return state === 'active'
      ? 'border-emerald-400/20 bg-emerald-400/10 text-emerald-200'
      : 'border-amber-400/20 bg-amber-400/10 text-amber-200';
  }
</script>

<section
  class="overflow-hidden rounded-2xl border border-white/10 bg-stone-900/50"
  aria-labelledby="directory-rules-heading"
  aria-busy={pending || loading}
>
  <header
    class="flex flex-wrap items-start justify-between gap-5 border-b border-white/10 px-6 py-5"
  >
    <div>
      <p class="text-xs font-semibold uppercase tracking-[0.2em] text-stone-500">
        Automatic identity
      </p>
      <h3 id="directory-rules-heading" class="mt-2 text-lg font-semibold text-white">
        Directory rules
      </h3>
      <p class="mt-1 max-w-2xl text-sm text-stone-400">
        Apply profiles to repositories below a directory through verified Git conditional includes.
      </p>
    </div>
    <button
      type="button"
      class="rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm font-medium text-stone-200 hover:bg-white/10 disabled:opacity-60"
      onclick={onRefresh}
      disabled={pending || loading}>{loading ? 'Refreshing…' : 'Refresh'}</button
    >
  </header>

  <div aria-live="polite">
    {#if error}<div
        class="border-b border-rose-400/20 bg-rose-400/5 px-6 py-4 text-sm text-rose-200"
        role="alert"
      >
        {error}
      </div>{/if}
    {#if success}<div
        class="border-b border-emerald-400/20 bg-emerald-400/5 px-6 py-4 text-sm text-emerald-200"
        role="status"
      >
        {success}
      </div>{/if}

    <div class="border-b border-white/10 p-6">
      <h4 class="font-semibold text-white">{editingId ? 'Update rule' : 'Add a rule'}</h4>
      <p class="mt-1 text-sm text-stone-400">
        Select a parent directory and an existing profile. Nothing changes until you review and
        confirm the preview.
      </p>
      <div class="mt-4 grid gap-3 md:grid-cols-[1fr_220px_auto]">
        <button
          type="button"
          class="min-w-0 truncate rounded-lg border border-white/10 bg-stone-950/50 px-3 py-2 text-left text-sm text-stone-300 hover:border-white/20 disabled:opacity-60"
          onclick={chooseDirectory}
          disabled={pending}>{directory || 'Choose directory…'}</button
        >
        <select
          class="rounded-lg border border-white/10 bg-stone-900 px-3 py-2 text-sm text-stone-100 disabled:opacity-60"
          bind:value={profileId}
          disabled={pending || profiles.length === 0}
          aria-label="Profile for directory rule"
        >
          <option value="">Select profile</option>
          {#each profiles as profile (profile.id)}<option value={profile.id}>{profile.label}</option
            >{/each}
        </select>
        <button
          type="button"
          class="rounded-lg bg-sky-300 px-3 py-2 text-sm font-semibold text-stone-950 hover:bg-sky-200 disabled:opacity-50"
          onclick={requestPreview}
          disabled={pending || !directory || !profileId}>Preview</button
        >
      </div>
      {#if editingId}
        <button
          type="button"
          class="mt-3 text-xs text-stone-400 hover:text-white"
          onclick={beginAdd}
          disabled={pending}>Cancel edit</button
        >
      {/if}
    </div>

    {#if rules.length > 0}
      <ul class="space-y-3 p-6">
        {#each rules as rule (rule.id)}
          {@const profile = directoryRuleProfile(rule, profiles)}
          <li class="rounded-xl border border-white/10 bg-black/10 p-4">
            <div class="flex flex-wrap items-start justify-between gap-4">
              <div class="min-w-0">
                <div class="flex flex-wrap items-center gap-2">
                  <p class="font-medium text-white">{profile?.label ?? 'Missing profile'}</p>
                  <span
                    class={`rounded-full border px-2 py-0.5 text-[11px] font-semibold uppercase ${stateClass(rule.state)}`}
                    >{directoryRuleStateLabels[rule.state]}</span
                  >
                </div>
                <p class="mt-2 break-all font-mono text-xs text-stone-400">{rule.directory}/**</p>
                {#if profile}<p class="mt-1 text-sm text-stone-300">
                    {profile.gitName} · {profile.gitEmail}
                  </p>{/if}
                {#if rule.message}<p class="mt-2 text-xs text-amber-200">{rule.message}</p>{/if}
              </div>
              <div class="flex gap-3">
                <button
                  type="button"
                  class="text-xs text-stone-400 hover:text-white disabled:opacity-50"
                  onclick={() => beginEdit(rule)}
                  disabled={pending || !profile}>Edit / reapply</button
                >
                <button
                  type="button"
                  class="text-xs text-stone-400 hover:text-rose-300 disabled:opacity-50"
                  onclick={() => onPreviewRemove(rule.id)}
                  disabled={pending}>Remove</button
                >
              </div>
            </div>
          </li>
        {/each}
      </ul>
    {:else if loading}
      <div class="p-6"><div class="h-28 animate-pulse rounded-xl bg-white/5"></div></div>
    {:else}
      <div class="px-6 py-8 text-sm text-stone-400">No directory rules configured.</div>
    {/if}
  </div>
</section>

{#if preview}
  <div class="fixed inset-0 z-50 grid place-items-center bg-black/70 p-6" role="presentation">
    <div
      class="max-h-[90vh] w-full max-w-2xl overflow-y-auto rounded-2xl border border-white/15 bg-stone-900 p-6 shadow-2xl"
      role="dialog"
      aria-modal="true"
      aria-labelledby="rule-preview-heading"
    >
      <h3 id="rule-preview-heading" class="text-xl font-semibold text-white">
        {directoryRuleOperationLabel(preview.operation)}
      </h3>
      <p class="mt-2 text-sm text-stone-300">
        Review the exact conditional identity operation before changing Git configuration.
      </p>

      <dl class="mt-5 space-y-3 rounded-xl border border-white/10 bg-black/20 p-4 text-sm">
        <div>
          <dt class="text-xs uppercase tracking-wide text-stone-500">Directory condition</dt>
          <dd class="mt-1 break-all font-mono text-stone-200">{preview.condition}</dd>
        </div>
        <div>
          <dt class="text-xs uppercase tracking-wide text-stone-500">Identity</dt>
          {#if preview.profile}
            <dd class="mt-1 text-stone-200">
              user.name = {preview.profile.gitName}<br />user.email = {preview.profile.gitEmail}
            </dd>
          {:else}
            <dd class="mt-1 text-stone-400">The previously assigned profile is unavailable.</dd>
          {/if}
        </div>
        <div>
          <dt class="text-xs uppercase tracking-wide text-stone-500">Generated include file</dt>
          <dd class="mt-1 break-all font-mono text-xs text-stone-400">
            {preview.identityFilePath}
          </dd>
        </div>
        <div>
          <dt class="text-xs uppercase tracking-wide text-stone-500">Global Git config</dt>
          <dd class="mt-1 break-all font-mono text-xs text-stone-400">
            {preview.globalConfigPath}
          </dd>
        </div>
      </dl>

      {#if preview.backupRequired}<p
          class="mt-4 rounded-lg border border-sky-400/20 bg-sky-400/5 p-3 text-sm text-sky-100"
        >
          Existing files will be backed up before modification.
        </p>{/if}
      {#if preview.conflicts.length > 0}
        <div class="mt-4 rounded-lg border border-amber-400/20 bg-amber-400/5 p-4">
          <p class="font-medium text-amber-100">Conflicts detected</p>
          <ul class="mt-2 list-disc space-y-1 pl-5 text-sm text-amber-200">
            {#each preview.conflicts as conflict (`${conflict.kind}-${conflict.message}`)}<li>
                {conflict.message}
              </li>{/each}
          </ul>
        </div>
      {/if}

      <div class="mt-6 flex justify-end gap-3">
        <button
          type="button"
          class="rounded-lg px-3 py-2 text-sm text-stone-300 hover:bg-white/5"
          onclick={onCancelPreview}
          disabled={pending}>Cancel</button
        >
        <button
          type="button"
          class="rounded-lg bg-sky-300 px-3 py-2 text-sm font-semibold text-stone-950 hover:bg-sky-200 disabled:opacity-50"
          onclick={onApply}
          disabled={pending || !preview.canApply}
          >{pending
            ? 'Applying…'
            : preview.operation === 'remove'
              ? 'Remove rule'
              : 'Apply rule'}</button
        >
      </div>
    </div>
  </div>
{/if}
