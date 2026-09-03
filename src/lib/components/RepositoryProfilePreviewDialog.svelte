<script lang="ts">
  import Button from '$lib/components/ui/Button.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import {
    configOriginLabel,
    configSourceLabel,
    repositoryIdentityKeyLabel,
    repositoryName,
    type RepositoryProfilePreview,
  } from '$lib/domain/repositories';

  type Props = {
    preview: RepositoryProfilePreview;
    applying: boolean;
    onCancel: () => void;
    onApply: () => void | Promise<void>;
  };

  let { preview, applying, onCancel, onApply }: Props = $props();
</script>

<Modal titleId="profile-preview-heading" onClose={onCancel} dismissible={!applying}>
  {#snippet header()}
    <header class="border-b border-edge px-6 py-5">
      <p class="text-xs font-semibold uppercase tracking-[0.2em] text-accent-text">
        Confirm change
      </p>
      <h3 id="profile-preview-heading" class="mt-2 text-xl font-semibold text-fg-strong">
        Apply {preview.profile.label} to {repositoryName(preview.path)}
      </h3>
      <p class="mt-2 break-all font-mono text-xs text-fg-subtle">{preview.path}</p>
    </header>
  {/snippet}

  <div class="space-y-5 px-6 py-5">
    <div class="rounded-xl border border-accent-edge bg-accent-softer p-4 text-sm text-fg">
      Git Identity Manager will set only <code>user.name</code> and <code>user.email</code> in this repository’s
      local Git configuration. Global configuration and unrelated repository settings will not be changed.
    </div>

    <div class="space-y-3">
      {#each preview.changes as change (change.key)}
        {@const origin = configOriginLabel(change.current.source)}
        <div class="grid gap-4 rounded-xl border border-edge bg-surface-raised p-4 sm:grid-cols-2">
          <div>
            <p class="text-xs font-semibold uppercase tracking-[0.14em] text-fg-subtle">
              Current {repositoryIdentityKeyLabel(change.key)}
            </p>
            <p class="mt-2 break-all text-sm text-fg-strong">
              {change.current.value ?? 'Not configured'}
            </p>
            <p class="mt-1 text-xs text-fg-muted">
              {configSourceLabel(change.current.source)}
            </p>
            {#if origin}
              <p class="mt-1 break-all font-mono text-[11px] text-fg-faint">{origin}</p>
            {/if}
          </div>
          <div>
            <p class="text-xs font-semibold uppercase tracking-[0.14em] text-fg-subtle">
              New local value
            </p>
            <p class="mt-2 break-all text-sm font-medium text-accent-text">{change.desired}</p>
            <p class="mt-1 text-xs text-fg-muted">Repository local</p>
          </div>
        </div>
      {/each}
    </div>
  </div>

  {#snippet footer()}
    <footer class="flex justify-end gap-3 border-t border-edge px-6 py-4">
      <Button variant="ghost" onclick={onCancel} disabled={applying}>Cancel</Button>
      <Button
        variant="primary"
        size="md"
        data-autofocus
        onclick={onApply}
        pending={applying}
        pendingLabel="Applying and verifying…"
      >
        Apply local identity
      </Button>
    </footer>
  {/snippet}
</Modal>
