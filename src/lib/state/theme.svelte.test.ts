import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { THEME_HINT_STORAGE_KEY, applyThemeHint, readThemeHint, theme } from './theme.svelte';

describe('theme controller', () => {
  beforeEach(() => {
    globalThis.localStorage.clear();
    delete document.documentElement.dataset.theme;
    theme.dispose();
  });

  afterEach(() => {
    theme.dispose();
  });

  it('leaves data-theme unset for the system preference so CSS drives it', () => {
    document.documentElement.dataset.theme = 'dark';
    theme.init('system');
    expect(theme.preference).toBe('system');
    // The jsdom matchMedia stub reports matches: false -> light.
    expect(theme.resolved).toBe('light');
    expect(document.documentElement.dataset.theme).toBeUndefined();
  });

  it('honours an explicit dark choice and stores a paint-time hint', () => {
    theme.init('system');
    theme.setPreference('dark');

    expect(theme.resolved).toBe('dark');
    expect(document.documentElement.dataset.theme).toBe('dark');
    expect(globalThis.localStorage.getItem(THEME_HINT_STORAGE_KEY)).toBe('dark');
  });

  it('clears the hint when returning to system', () => {
    theme.init('dark');
    expect(globalThis.localStorage.getItem(THEME_HINT_STORAGE_KEY)).toBe('dark');

    theme.setPreference('system');
    expect(globalThis.localStorage.getItem(THEME_HINT_STORAGE_KEY)).toBeNull();
  });

  it('reads and applies only light/dark hints', () => {
    globalThis.localStorage.setItem(THEME_HINT_STORAGE_KEY, 'dark');
    expect(readThemeHint()).toBe('dark');
    applyThemeHint();
    expect(document.documentElement.dataset.theme).toBe('dark');

    globalThis.localStorage.setItem(THEME_HINT_STORAGE_KEY, 'sepia');
    expect(readThemeHint()).toBeNull();
    applyThemeHint();
    expect(document.documentElement.dataset.theme).toBeUndefined();
  });

  it('falls back to system for an invalid stored preference', () => {
    theme.init('bogus' as never);
    expect(theme.preference).toBe('system');
  });
});
