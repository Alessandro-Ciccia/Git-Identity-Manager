<script lang="ts">
  import type { Snippet } from 'svelte';

  type Props = {
    eyebrow: string;
    heading: string;
    headingId: string;
    description?: string;
    busy?: boolean;
    actions?: Snippet;
    children: Snippet;
  };

  let {
    eyebrow,
    heading,
    headingId,
    description,
    busy = false,
    actions,
    children,
  }: Props = $props();
</script>

<section
  class="overflow-hidden rounded-2xl border border-edge bg-surface"
  aria-labelledby={headingId}
  aria-busy={busy}
>
  <header class="flex flex-wrap items-start justify-between gap-5 border-b border-edge px-6 py-5">
    <div>
      <p class="text-xs font-semibold uppercase tracking-[0.2em] text-fg-subtle">{eyebrow}</p>
      <h3 id={headingId} class="mt-2 text-lg font-semibold text-fg-strong">{heading}</h3>
      {#if description}
        <p class="mt-1 max-w-2xl text-sm text-fg-muted">{description}</p>
      {/if}
    </div>
    {#if actions}
      <div class="flex items-center gap-2">{@render actions()}</div>
    {/if}
  </header>

  <div aria-live="polite">
    {@render children()}
  </div>
</section>
