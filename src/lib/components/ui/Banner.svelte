<script lang="ts">
  import type { Snippet } from 'svelte';

  type Tone = 'danger' | 'positive' | 'info';

  type Props = {
    tone: Tone;
    /** `bar` spans the panel width with a bottom border; `inset` is a rounded card. */
    variant?: 'bar' | 'inset';
    /** Trailing action(s), e.g. a retry or update button. */
    action?: Snippet;
    class?: string;
    children: Snippet;
  };

  let { tone, variant = 'bar', action, class: extra = '', children }: Props = $props();

  const tones: Record<Tone, string> = {
    danger: 'border-danger-edge bg-danger-softer text-danger',
    positive: 'border-positive-edge bg-positive-softer text-positive',
    info: 'border-info-edge bg-info-softer text-info',
  };

  const role = $derived(tone === 'danger' ? 'alert' : tone === 'positive' ? 'status' : undefined);
  const layout = $derived(variant === 'bar' ? 'border-b px-6 py-4' : 'rounded-xl border p-4');
</script>

<div
  class={`text-sm ${layout} ${tones[tone]} ${
    action ? 'flex flex-wrap items-center justify-between gap-4' : ''
  } ${extra}`}
  {role}
>
  <span>{@render children()}</span>
  {#if action}
    <span class="shrink-0">{@render action()}</span>
  {/if}
</div>
