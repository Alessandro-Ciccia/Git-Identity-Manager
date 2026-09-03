<script lang="ts">
  import Badge from '$lib/components/ui/Badge.svelte';
  import Banner from '$lib/components/ui/Banner.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import EmptyState from '$lib/components/ui/EmptyState.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import Panel from '$lib/components/ui/Panel.svelte';
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
</script>

<Panel
  eyebrow="Automatic identity"
  heading="Directory rules"
  headingId="directory-rules-heading"
  description="Apply profiles to repositories below a directory through verified Git conditional includes."
  busy={pending || loading}
>
  {#snippet actions()}
    <Button
      onclick={onRefresh}
      disabled={pending || loading}
      pending={loading}
      pendingLabel="Refreshing…"
    >
      Refresh
    </Button>
  {/snippet}

  {#if error}<Banner tone="danger">{error}</Banner>{/if}
  {#if success}<Banner tone="positive">{success}</Banner>{/if}

  <div class="border-b border-edge p-6">
    <h4 class="font-semibold text-fg-strong">{editingId ? 'Update rule' : 'Add a rule'}</h4>
    <p class="mt-1 text-sm text-fg-muted">
      Select a parent directory and an existing profile. Nothing changes until you review and
      confirm the preview.
    </p>
    <div class="mt-4 grid gap-3 md:grid-cols-[1fr_220px_auto]">
      <button
        type="button"
        class="min-w-0 truncate rounded-lg border border-edge bg-surface-input px-3 py-2 text-left text-sm text-fg transition hover:border-edge-hover disabled:opacity-60"
        onclick={chooseDirectory}
        disabled={pending}>{directory || 'Choose directory…'}</button
      >
      <select
        class="rounded-lg border border-edge bg-surface-input px-3 py-2 text-sm text-fg-strong transition focus:border-accent disabled:opacity-60"
        bind:value={profileId}
        disabled={pending || profiles.length === 0}
        aria-label="Profile for directory rule"
      >
        <option value="">Select profile</option>
        {#each profiles as profile (profile.id)}<option value={profile.id}>{profile.label}</option
          >{/each}
      </select>
      <Button
        variant="primary"
        onclick={requestPreview}
        disabled={pending || !directory || !profileId}
      >
        Preview
      </Button>
    </div>
    {#if editingId}
      <button
        type="button"
        class="mt-3 text-xs text-fg-muted transition hover:text-fg-strong disabled:opacity-60"
        onclick={beginAdd}
        disabled={pending}>Cancel edit</button
      >
    {/if}
  </div>

  {#if rules.length > 0}
    <ul class="space-y-3 p-6">
      {#each rules as rule (rule.id)}
        {@const profile = directoryRuleProfile(rule, profiles)}
        <li class="rounded-xl border border-edge bg-surface-raised p-4">
          <div class="flex flex-wrap items-start justify-between gap-4">
            <div class="min-w-0">
              <div class="flex flex-wrap items-center gap-2">
                <p class="font-medium text-fg-strong">{profile?.label ?? 'Missing profile'}</p>
                <Badge tone={rule.state === 'active' ? 'positive' : 'warning'} uppercase>
                  {directoryRuleStateLabels[rule.state]}
                </Badge>
              </div>
              <p class="mt-2 break-all font-mono text-xs text-fg-muted">{rule.directory}/**</p>
              {#if profile}<p class="mt-1 text-sm text-fg">
                  {profile.gitName} · {profile.gitEmail}
                </p>{/if}
              {#if rule.message}<p class="mt-2 text-xs text-warning">{rule.message}</p>{/if}
            </div>
            <div class="flex gap-3">
              <button
                type="button"
                class="text-xs text-fg-muted transition hover:text-fg-strong disabled:opacity-50"
                onclick={() => beginEdit(rule)}
                disabled={pending || !profile}>Edit / reapply</button
              >
              <button
                type="button"
                class="text-xs text-fg-muted transition hover:text-danger disabled:opacity-50"
                onclick={() => onPreviewRemove(rule.id)}
                disabled={pending}>Remove</button
              >
            </div>
          </div>
        </li>
      {/each}
    </ul>
  {:else if loading}
    <div class="p-6"><div class="h-28 animate-pulse rounded-xl bg-hover"></div></div>
  {:else}
    <EmptyState
      title="No directory rules configured"
      description="Choose a parent directory and a profile above to preview a safe Git conditional include for every repository below it."
    />
  {/if}
</Panel>

{#if preview}
  <Modal
    titleId="rule-preview-heading"
    onClose={onCancelPreview}
    dismissible={!pending}
    class="p-6"
  >
    <h3 id="rule-preview-heading" class="text-xl font-semibold text-fg-strong">
      {directoryRuleOperationLabel(preview.operation)}
    </h3>
    <p class="mt-2 text-sm text-fg">
      Review the exact conditional identity operation before changing Git configuration.
    </p>

    <dl class="mt-5 space-y-3 rounded-xl border border-edge bg-surface-raised p-4 text-sm">
      <div>
        <dt class="text-xs uppercase tracking-wide text-fg-subtle">Directory condition</dt>
        <dd class="mt-1 break-all font-mono text-fg">{preview.condition}</dd>
      </div>
      <div>
        <dt class="text-xs uppercase tracking-wide text-fg-subtle">Identity</dt>
        {#if preview.profile}
          <dd class="mt-1 text-fg">
            user.name = {preview.profile.gitName}<br />user.email = {preview.profile.gitEmail}
          </dd>
        {:else}
          <dd class="mt-1 text-fg-muted">The previously assigned profile is unavailable.</dd>
        {/if}
      </div>
      <div>
        <dt class="text-xs uppercase tracking-wide text-fg-subtle">Generated include file</dt>
        <dd class="mt-1 break-all font-mono text-xs text-fg-muted">
          {preview.identityFilePath}
        </dd>
      </div>
      <div>
        <dt class="text-xs uppercase tracking-wide text-fg-subtle">Global Git config</dt>
        <dd class="mt-1 break-all font-mono text-xs text-fg-muted">
          {preview.globalConfigPath}
        </dd>
      </div>
    </dl>

    {#if preview.backupRequired}<p
        class="mt-4 rounded-lg border border-accent-edge bg-accent-softer p-3 text-sm text-accent-text"
      >
        Existing files will be backed up before modification.
      </p>{/if}
    {#if preview.conflicts.length > 0}
      <div class="mt-4 rounded-lg border border-warning-edge bg-warning-softer p-4">
        <p class="font-medium text-warning">Conflicts detected</p>
        <ul class="mt-2 list-disc space-y-1 pl-5 text-sm text-warning">
          {#each preview.conflicts as conflict (`${conflict.kind}-${conflict.message}`)}<li>
              {conflict.message}
            </li>{/each}
        </ul>
      </div>
    {/if}

    <div class="mt-6 flex justify-end gap-3">
      <Button variant="ghost" onclick={onCancelPreview} disabled={pending}>Cancel</Button>
      <Button
        variant="primary"
        data-autofocus
        onclick={onApply}
        disabled={pending || !preview.canApply}
        {pending}
        pendingLabel="Applying…"
      >
        {preview.operation === 'remove' ? 'Remove rule' : 'Apply rule'}
      </Button>
    </div>
  </Modal>
{/if}
