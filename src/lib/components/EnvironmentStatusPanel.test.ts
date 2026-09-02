import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { EnvironmentStatus } from '$lib/domain/environment';

import EnvironmentStatusPanel from './EnvironmentStatusPanel.svelte';

const readyStatus: EnvironmentStatus = {
  git: {
    dependency: 'git',
    state: 'available',
    version: '2.47.0',
    message: 'Git is available.',
  },
  githubCli: {
    dependency: 'githubCli',
    state: 'available',
    version: '2.73.0',
    message: 'GitHub CLI is available.',
  },
  isReady: true,
};

describe('EnvironmentStatusPanel', () => {
  it('shows a loading state while the first check runs', () => {
    render(EnvironmentStatusPanel, {
      status: null,
      loading: true,
      error: null,
      onRefresh: vi.fn(),
    });

    expect(screen.getByRole('button', { name: 'Checking…' })).toBeDisabled();
    expect(screen.getByLabelText('Checking dependencies')).toBeInTheDocument();
  });

  it('shows available dependencies and normalized versions', () => {
    render(EnvironmentStatusPanel, {
      status: readyStatus,
      loading: false,
      error: null,
      onRefresh: vi.fn(),
    });

    expect(screen.getByText('Your environment is ready.')).toBeInTheDocument();
    expect(screen.getByText('v2.47.0')).toBeInTheDocument();
    expect(screen.getByText('v2.73.0')).toBeInTheDocument();
    expect(screen.getAllByText('Available')).toHaveLength(2);
  });

  it('explains a missing dependency without treating it as a crash', () => {
    const status: EnvironmentStatus = {
      ...readyStatus,
      githubCli: {
        dependency: 'githubCli',
        state: 'missing',
        version: null,
        message:
          'GitHub CLI is not installed or is not available on PATH. Install it, then refresh this check.',
      },
      isReady: false,
    };

    render(EnvironmentStatusPanel, {
      status,
      loading: false,
      error: null,
      onRefresh: vi.fn(),
    });

    expect(screen.getByText('One required dependency is missing.')).toBeInTheDocument();
    expect(screen.getByText('Missing')).toBeInTheDocument();
    expect(screen.getByText(/Install it, then refresh this check/)).toBeInTheDocument();
  });

  it('shows command-level failures and lets the user retry', async () => {
    const user = userEvent.setup();
    const onRefresh = vi.fn();

    render(EnvironmentStatusPanel, {
      status: null,
      loading: false,
      error: 'The environment check could not be completed. Please try again.',
      onRefresh,
    });

    expect(screen.getByRole('alert')).toHaveTextContent('could not be completed');
    await user.click(screen.getByRole('button', { name: 'Refresh' }));
    expect(onRefresh).toHaveBeenCalledOnce();
  });
});
