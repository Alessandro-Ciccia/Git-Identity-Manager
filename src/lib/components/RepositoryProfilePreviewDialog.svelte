<script lang="ts">
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

<div class="fixed inset-0 z-50 grid place-items-center bg-black/70 p-6" role="presentation">
  <div
    class="max-h-[90vh] w-full max-w-2xl overflow-y-auto rounded-2xl border border-white/15 bg-stone-900 shadow-2xl"
    role="dialog"
    aria-modal="true"
    aria-labelledby="profile-preview-heading"
  >
    <header class="border-b border-white/10 px-6 py-5">
      <p class="text-xs font-semibold uppercase tracking-[0.2em] text-sky-300">Confirm change</p>
      <h3 id="profile-preview-heading" class="mt-2 text-xl font-semibold text-white">
        Apply {preview.profile.label} to {repositoryName(preview.path)}
      </h3>
      <p class="mt-2 break-all font-mono text-xs text-stone-500">{preview.path}</p>
    </header>

    <div class="space-y-5 px-6 py-5">
      <div class="rounded-xl border border-sky-300/15 bg-sky-300/5 p-4 text-sm text-stone-300">
        Git Identity Manager will set only <code>user.name</code> and <code>user.email</code> in this
        repository’s local Git configuration. Global configuration and unrelated repository settings will
        not be changed.
      </div>

      <div class="space-y-3">
        {#each preview.changes as change (change.key)}
          {@const origin = configOriginLabel(change.current.source)}
          <div class="grid gap-4 rounded-xl border border-white/10 bg-black/15 p-4 sm:grid-cols-2">
            <div>
              <p class="text-xs font-semibold uppercase tracking-[0.14em] text-stone-500">
                Current {repositoryIdentityKeyLabel(change.key)}
              </p>
              <p class="mt-2 break-all text-sm text-stone-100">
                {change.current.value ?? 'Not configured'}
              </p>
              <p class="mt-1 text-xs text-stone-400">
                {configSourceLabel(change.current.source)}
              </p>
              {#if origin}
                <p class="mt-1 break-all font-mono text-[11px] text-stone-600">{origin}</p>
              {/if}
            </div>
            <div>
              <p class="text-xs font-semibold uppercase tracking-[0.14em] text-stone-500">
                New local value
              </p>
              <p class="mt-2 break-all text-sm font-medium text-sky-100">{change.desired}</p>
              <p class="mt-1 text-xs text-stone-400">Repository local</p>
            </div>
          </div>
        {/each}
      </div>
    </div>

    <footer class="flex justify-end gap-3 border-t border-white/10 px-6 py-4">
      <button
        type="button"
        class="rounded-lg px-3 py-2 text-sm font-medium text-stone-300 transition hover:bg-white/5 hover:text-white disabled:opacity-60"
        onclick={onCancel}
        disabled={applying}>Cancel</button
      >
      <button
        type="button"
        class="rounded-lg bg-sky-300 px-4 py-2 text-sm font-semibold text-stone-950 transition hover:bg-sky-200 disabled:cursor-wait disabled:opacity-60"
        onclick={onApply}
        disabled={applying}
      >
        {applying ? 'Applying and verifying…' : 'Apply local identity'}
      </button>
    </footer>
  </div>
</div>
