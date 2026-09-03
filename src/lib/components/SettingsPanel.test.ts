import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { ResolvedTheme, ThemePreference } from '$lib/domain/preferences';

import SettingsPanel from './SettingsPanel.svelte';

type PanelProps = {
  theme: ThemePreference;
  resolved: ResolvedTheme;
  pending: boolean;
  error: string | null;
  onSetTheme: (theme: ThemePreference) => void | Promise<void>;
};

function renderPanel(overrides: Partial<PanelProps> = {}) {
  return render(SettingsPanel, {
    theme: 'system',
    resolved: 'dark',
    pending: false,
    error: null,
    onSetTheme: vi.fn(),
    ...overrides,
  });
}

describe('SettingsPanel', () => {
  it('marks the active theme in an accessible radio group', () => {
    renderPanel({ theme: 'light' });
    const group = screen.getByRole('radiogroup', { name: 'Theme' });
    expect(group).toBeInTheDocument();
    expect(screen.getByRole('radio', { name: 'Light' })).toHaveAttribute('aria-checked', 'true');
    expect(screen.getByRole('radio', { name: 'System' })).toHaveAttribute('aria-checked', 'false');
  });

  it('selects a theme on click', async () => {
    const user = userEvent.setup();
    const onSetTheme = vi.fn();
    renderPanel({ onSetTheme });

    await user.click(screen.getByRole('radio', { name: 'Dark' }));
    expect(onSetTheme).toHaveBeenCalledWith('dark');
  });

  it('moves the selection with arrow keys', async () => {
    const user = userEvent.setup();
    const onSetTheme = vi.fn();
    renderPanel({ theme: 'system', onSetTheme });

    screen.getByRole('radio', { name: 'System' }).focus();
    await user.keyboard('{ArrowRight}');
    expect(onSetTheme).toHaveBeenCalledWith('light');
  });

  it('shows a safe error and states data is stored locally', () => {
    renderPanel({ error: 'Preferences could not be read or saved.' });
    expect(screen.getByRole('alert')).toHaveTextContent('could not be read or saved');
    expect(screen.getByText(/never stored by/i)).toBeInTheDocument();
  });
});
