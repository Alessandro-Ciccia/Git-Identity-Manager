<script lang="ts">
  import type { Snippet } from 'svelte';

  type Props = {
    title: string;
    description: string;
    /** Optional call-to-action rendered below the description. */
    action?: Snippet;
    class?: string;
  };

  let { title, description, action, class: extra = '' }: Props = $props();
</script>

<div class={`px-6 py-10 ${extra}`}>
  <p class="font-medium text-fg-strong">{title}</p>
  <p class="mt-2 max-w-xl text-sm leading-6 text-fg-muted">{description}</p>
  {#if action}
    <div class="mt-4">{@render action()}</div>
  {/if}
</div>
