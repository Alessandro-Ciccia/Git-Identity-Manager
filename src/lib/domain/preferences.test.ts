import { describe, expect, it } from 'vitest';

import {
  defaultPreferences,
  isThemePreference,
  resolveTheme,
  themePreferenceLabels,
  themePreferences,
} from './preferences';

describe('preferences domain', () => {
  it('defaults to the system theme and an undismissed welcome', () => {
    expect(defaultPreferences).toEqual({ theme: 'system', welcomeDismissed: false });
  });

  it('lists exactly the three supported theme preferences with labels', () => {
    expect(themePreferences).toEqual(['system', 'light', 'dark']);
    expect(themePreferenceLabels).toEqual({ system: 'System', light: 'Light', dark: 'Dark' });
  });

  it('validates theme preference values', () => {
    expect(isThemePreference('system')).toBe(true);
    expect(isThemePreference('dark')).toBe(true);
    expect(isThemePreference('sepia')).toBe(false);
    expect(isThemePreference(null)).toBe(false);
  });

  it('resolves system against the OS preference and honours explicit choices', () => {
    expect(resolveTheme('system', true)).toBe('dark');
    expect(resolveTheme('system', false)).toBe('light');
    expect(resolveTheme('light', true)).toBe('light');
    expect(resolveTheme('dark', false)).toBe('dark');
  });
});
