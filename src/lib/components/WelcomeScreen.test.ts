import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { EnvironmentStatus } from '$lib/domain/environment';

import WelcomeScreen from './WelcomeScreen.svelte';

const readyStatus: EnvironmentStatus = {
  git: { dependency: 'git', state: 'available', version: '2.47.0', message: 'Git is available.' },
  githubCli: {
    dependency: 'githubCli',
    state: 'available',
    version: '2.73.0',
    message: 'GitHub CLI is available.',
  },
  isReady: true,
};

function renderScreen(overrides: Record<string, unknown> = {}) {
  return render(WelcomeScreen, {
    status: readyStatus,
    envLoading: false,
    hasProfiles: false,
    hasRepositories: false,
    skipping: false,
    skipError: null,
    onRecheck: vi.fn(),
    onCreateProfile: vi.fn(),
    onAddRepository: vi.fn(),
    onSkip: vi.fn(),
    ...overrides,
  });
}

describe('WelcomeScreen', () => {
  it('guides the three setup steps', () => {
    renderScreen();
    expect(
      screen.getByRole('heading', { name: 'Let’s set up Git Identity Manager' }),
    ).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Create profile' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Add repository' })).toBeInTheDocument();
  });

  it('flags a missing dependency and re-checks on request', async () => {
    const user = userEvent.setup();
    const onRecheck = vi.fn();
    renderScreen({
      status: {
        ...readyStatus,
        git: {
          dependency: 'git',
          state: 'missing',
          version: null,
          message: 'Git is not installed.',
        },
        isReady: false,
      },
      onRecheck,
    });

    expect(screen.getByText('Action needed')).toBeInTheDocument();
    expect(screen.getByText('Git is not installed.')).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Recheck' }));
    expect(onRecheck).toHaveBeenCalledOnce();
  });

  it('reflects completed steps and skips on request', async () => {
    const user = userEvent.setup();
    const onSkip = vi.fn();
    renderScreen({ hasProfiles: true, hasRepositories: true, onSkip });

    expect(screen.getByRole('button', { name: 'Manage profiles' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Manage repositories' })).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Skip for now' }));
    expect(onSkip).toHaveBeenCalledOnce();
  });

  it('surfaces a skip error', () => {
    renderScreen({ skipError: 'The preference could not be updated. Please try again.' });
    expect(screen.getByRole('alert')).toHaveTextContent('could not be updated');
  });
});
