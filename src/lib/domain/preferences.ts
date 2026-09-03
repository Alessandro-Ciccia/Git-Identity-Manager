export type ThemePreference = 'system' | 'light' | 'dark';
export type ResolvedTheme = 'light' | 'dark';

export type AppPreferences = {
  theme: ThemePreference;
  welcomeDismissed: boolean;
};

export const themePreferences: readonly ThemePreference[] = ['system', 'light', 'dark'];

export const themePreferenceLabels = {
  system: 'System',
  light: 'Light',
  dark: 'Dark',
} as const satisfies Record<ThemePreference, string>;

export const themePreferenceDescriptions = {
  system: 'Match the operating system appearance.',
  light: 'Always use the light theme.',
  dark: 'Always use the dark theme.',
} as const satisfies Record<ThemePreference, string>;

export function isThemePreference(value: unknown): value is ThemePreference {
  return value === 'system' || value === 'light' || value === 'dark';
}

export function resolveTheme(preference: ThemePreference, prefersDark: boolean): ResolvedTheme {
  if (preference === 'system') {
    return prefersDark ? 'dark' : 'light';
  }
  return preference;
}

export const defaultPreferences: AppPreferences = {
  theme: 'system',
  welcomeDismissed: false,
};
