<script lang="ts">
  import type { Snippet } from 'svelte';

  type Props = {
    /** id of the heading rendered inside `header`, wired to aria-labelledby. */
    titleId: string;
    /** Called on Escape, backdrop click, or a close control — ignored when not dismissible. */
    onClose: () => void;
    /** Set false while an irreversible operation is in flight to lock the modal open. */
    dismissible?: boolean;
    class?: string;
    header?: Snippet;
    children: Snippet;
    footer?: Snippet;
  };

  let {
    titleId,
    onClose,
    dismissible = true,
    class: panelClass = '',
    header,
    children,
    footer,
  }: Props = $props();

  let panel = $state<globalThis.HTMLElement | null>(null);

  const FOCUSABLE =
    'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

  function focusableItems(): globalThis.HTMLElement[] {
    if (!panel) return [];
    return Array.from(panel.querySelectorAll<globalThis.HTMLElement>(FOCUSABLE)).filter(
      (element) =>
        !element.hasAttribute('hidden') && element.getAttribute('aria-hidden') !== 'true',
    );
  }

  function requestClose(): void {
    if (dismissible) {
      onClose();
    }
  }

  function onKeydown(event: globalThis.KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.stopPropagation();
      requestClose();
      return;
    }
    if (event.key !== 'Tab') {
      return;
    }
    const items = focusableItems();
    if (items.length === 0) {
      event.preventDefault();
      panel?.focus();
      return;
    }
    const first = items[0]!;
    const last = items[items.length - 1]!;
    const active = globalThis.document.activeElement as globalThis.HTMLElement | null;
    if (event.shiftKey && (active === first || !panel?.contains(active))) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && active === last) {
      event.preventDefault();
      first.focus();
    }
  }

  function onBackdropClick(event: globalThis.MouseEvent): void {
    if (event.target === event.currentTarget) {
      requestClose();
    }
  }

  $effect(() => {
    const previouslyFocused = globalThis.document.activeElement as globalThis.HTMLElement | null;
    const { overflow } = globalThis.document.body.style;
    globalThis.document.body.style.overflow = 'hidden';

    const target =
      panel?.querySelector<globalThis.HTMLElement>('[data-autofocus]') ??
      focusableItems()[0] ??
      panel ??
      null;
    target?.focus();

    return () => {
      globalThis.document.body.style.overflow = overflow;
      previouslyFocused?.focus?.();
    };
  });
</script>

<svelte:window on:keydown={onKeydown} />

<div
  class="fixed inset-0 z-50 grid place-items-center bg-black/70 p-6"
  role="presentation"
  onclick={onBackdropClick}
>
  <div
    bind:this={panel}
    class={`max-h-[90vh] w-full max-w-2xl overflow-y-auto rounded-2xl border border-edge-strong bg-surface-overlay shadow-2xl outline-none ${panelClass}`}
    role="dialog"
    aria-modal="true"
    aria-labelledby={titleId}
    tabindex="-1"
  >
    {#if header}{@render header()}{/if}
    {@render children()}
    {#if footer}{@render footer()}{/if}
  </div>
</div>
