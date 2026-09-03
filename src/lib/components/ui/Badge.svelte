<script lang="ts">
  import type { Snippet } from 'svelte';

  type Tone = 'neutral' | 'positive' | 'warning' | 'danger' | 'info' | 'accent';

  type Props = {
    tone?: Tone;
    /** Uppercase tracked style used for status pills on cards. */
    uppercase?: boolean;
    class?: string;
    children: Snippet;
  };

  let { tone = 'neutral', uppercase = false, class: extra = '', children }: Props = $props();

  const tones: Record<Tone, string> = {
    neutral: 'border-edge bg-hover text-fg-muted',
    positive: 'border-positive-edge bg-positive-soft text-positive',
    warning: 'border-warning-edge bg-warning-soft text-warning',
    danger: 'border-danger-edge bg-danger-soft text-danger',
    info: 'border-info-edge bg-info-soft text-info',
    accent: 'border-accent-edge bg-accent-soft text-accent-text',
  };
</script>

<span
  class={`inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-semibold ${
    uppercase ? 'uppercase tracking-wide' : ''
  } ${tones[tone]} ${extra}`}
>
  {@render children()}
</span>
