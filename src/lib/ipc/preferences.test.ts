import { invoke } from '@tauri-apps/api/core';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  getPreferences,
  preferenceErrorMessage,
  setThemePreference,
  setWelcomeDismissed,
} from './preferences';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);

describe('preferences IPC', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue({ theme: 'system', welcomeDismissed: false });
  });

  it('reads preferences through a fixed command without arguments', async () => {
    await getPreferences();
    expect(invokeMock).toHaveBeenCalledWith('get_preferences');
  });

  it('sets the theme with only the typed enum value', async () => {
    await setThemePreference('dark');
    expect(invokeMock).toHaveBeenCalledWith('set_theme_preference', { theme: 'dark' });
  });

  it('sets the welcome flag with only a boolean', async () => {
    await setWelcomeDismissed(true);
    expect(invokeMock).toHaveBeenCalledWith('set_welcome_dismissed', { dismissed: true });
  });

  it('uses structured backend messages and hides unknown rejection values', () => {
    expect(
      preferenceErrorMessage({
        code: 'preferenceStorageFailed',
        message:
          'Preferences could not be read or saved. Check application data permissions and try again.',
      }),
    ).toBe(
      'Preferences could not be read or saved. Check application data permissions and try again.',
    );
    expect(preferenceErrorMessage('raw error')).toBe(
      'The preference could not be updated. Please try again.',
    );
  });
});
