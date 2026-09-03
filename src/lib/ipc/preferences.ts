import { invoke } from '@tauri-apps/api/core';

import type { AppPreferences, ThemePreference } from '$lib/domain/preferences';

const GET_PREFERENCES_COMMAND = 'get_preferences';
const SET_THEME_PREFERENCE_COMMAND = 'set_theme_preference';
const SET_WELCOME_DISMISSED_COMMAND = 'set_welcome_dismissed';

export function getPreferences(): Promise<AppPreferences> {
  return invoke<AppPreferences>(GET_PREFERENCES_COMMAND);
}

export function setThemePreference(theme: ThemePreference): Promise<AppPreferences> {
  return invoke<AppPreferences>(SET_THEME_PREFERENCE_COMMAND, { theme });
}

export function setWelcomeDismissed(dismissed: boolean): Promise<AppPreferences> {
  return invoke<AppPreferences>(SET_WELCOME_DISMISSED_COMMAND, { dismissed });
}

export function preferenceErrorMessage(error: unknown): string {
  if (
    typeof error === 'object' &&
    error !== null &&
    'message' in error &&
    typeof error.message === 'string'
  ) {
    return error.message;
  }

  return 'The preference could not be updated. Please try again.';
}
