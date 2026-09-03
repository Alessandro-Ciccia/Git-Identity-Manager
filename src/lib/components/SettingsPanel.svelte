<script lang="ts">
  import Banner from '$lib/components/ui/Banner.svelte';
  import Panel from '$lib/components/ui/Panel.svelte';
  import {
    themePreferenceDescriptions,
    themePreferenceLabels,
    themePreferences,
    type ResolvedTheme,
    type ThemePreference,
  } from '$lib/domain/preferences';

  type Props = {
    theme: ThemePreference;
    resolved: ResolvedTheme;
    pending: boolean;
    error: string | null;
    onSetTheme: (theme: ThemePreference) => void | Promise<void>;
  };

  let { theme, resolved, pending, error, onSetTheme }: Props = $props();

  let group = $state<globalThis.HTMLDivElement | null>(null);

  function selectRelative(offset: number): void {
    const current = themePreferences.indexOf(theme);
    const next =
      themePreferences[(current + offset + themePreferences.length) % themePreferences.length];
    if (next) {
      void onSetTheme(next);
      const buttons = group?.querySelectorAll<globalThis.HTMLButtonElement>('button[role="radio"]');
      buttons?.[themePreferences.indexOf(next)]?.focus();
    }
  }

  function onKeydown(event: globalThis.KeyboardEvent): void {
    if (event.key === 'ArrowRight' || event.key === 'ArrowDown') {
      event.preventDefault();
      selectRelative(1);
    } else if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') {
      event.preventDefault();
      selectRelative(-1);
    }
  }
</script>

<Panel
  eyebrow="Preferences"
  heading="Settings"
  headingId="settings-heading"
  description="Local application preferences. Nothing here is synced, and none of it contains secrets."
  busy={pending}
>
  {#if error}<Banner tone="danger">{error}</Banner>{/if}

  <div class="space-y-8 p-6">
    <fieldset>
      <legend class="text-sm font-medium text-fg">Appearance</legend>
      <p class="mt-1 text-sm text-fg-muted">
        Choose how Git Identity Manager looks. “System” follows your operating system.
      </p>
      <div
        bind:this={group}
        role="radiogroup"
        aria-label="Theme"
        tabindex="-1"
        class="mt-3 inline-flex rounded-lg border border-edge bg-surface-raised p-1"
        onkeydown={onKeydown}
      >
        {#each themePreferences as option (option)}
          <button
            type="button"
            role="radio"
            aria-checked={theme === option}
            tabindex={theme === option ? 0 : -1}
            class={`rounded-md px-3 py-1.5 text-sm font-medium transition ${
              theme === option ? 'bg-accent text-on-accent' : 'text-fg-muted hover:text-fg-strong'
            }`}
            onclick={() => onSetTheme(option)}
            disabled={pending}
          >
            {themePreferenceLabels[option]}
          </button>
        {/each}
      </div>
      <p class="mt-2 text-xs text-fg-subtle">
        {themePreferenceDescriptions[theme]}
        {#if theme === 'system'}
          Currently showing the {resolved} theme.
        {/if}
      </p>
    </fieldset>

    <div class="rounded-xl border border-edge bg-surface-raised p-4 text-sm text-fg-muted">
      <p class="font-medium text-fg">Where your data lives</p>
      <ul class="mt-2 list-disc space-y-1 pl-5">
        <li>
          Profiles, registered repositories, directory rules, and these preferences are stored
          locally in your operating system’s application-data directory.
        </li>
        <li>
          GitHub tokens, passwords, credential-helper secrets, and SSH keys are never stored by Git
          Identity Manager.
        </li>
      </ul>
    </div>
  </div>
</Panel>
