<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  type Variant = 'primary' | 'secondary' | 'danger' | 'warning' | 'ghost';
  type Size = 'xs' | 'sm' | 'md';

  type Props = HTMLButtonAttributes & {
    variant?: Variant;
    size?: Size;
    /** When true the button is disabled and shows `pendingLabel` (or its children). */
    pending?: boolean;
    pendingLabel?: string;
    children: Snippet;
  };

  let {
    variant = 'secondary',
    size = 'sm',
    pending = false,
    pendingLabel,
    type = 'button',
    disabled = false,
    class: extra = '',
    children,
    ...rest
  }: Props = $props();

  const variants: Record<Variant, string> = {
    primary: 'bg-accent text-on-accent font-semibold hover:bg-accent-hover',
    secondary:
      'border border-edge bg-hover text-fg font-medium hover:border-edge-hover hover:bg-hover-strong',
    danger: 'bg-danger-solid text-on-danger font-semibold hover:bg-danger-solid-hover',
    warning: 'bg-warning-solid text-on-warning font-semibold hover:brightness-105',
    ghost: 'text-fg-muted font-medium hover:bg-hover hover:text-fg-strong',
  };

  const sizes: Record<Size, string> = {
    xs: 'rounded-md px-2.5 py-1.5 text-xs',
    sm: 'rounded-lg px-3 py-2 text-sm',
    md: 'rounded-lg px-4 py-2.5 text-sm',
  };
</script>

<button
  {type}
  disabled={disabled || pending}
  class={`inline-flex items-center justify-center gap-2 transition disabled:cursor-wait disabled:opacity-60 ${sizes[size]} ${variants[variant]} ${extra}`}
  {...rest}
>
  {#if pending && pendingLabel !== undefined}
    {pendingLabel}
  {:else}
    {@render children()}
  {/if}
</button>
