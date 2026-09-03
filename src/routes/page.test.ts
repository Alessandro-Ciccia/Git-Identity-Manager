import { invoke } from '@tauri-apps/api/core';
import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { GitProfile } from '$lib/domain/profiles';

import Page from './+page.svelte';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);
const profile: GitProfile = {
  id: '0db9c44d-e1e1-493b-a14f-0208aa4ef247',
  label: 'Personal',
  gitName: 'Octo Cat',
  gitEmail: 'octo@example.com',
  githubAccount: { hostname: 'github.com', username: 'personal' },
};

function mockInitialState(profiles: GitProfile[]): void {
  invokeMock.mockImplementation((command) => {
    switch (command) {
      case 'get_environment_status':
        return Promise.resolve({
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
        });
      case 'list_github_accounts':
        return Promise.resolve({
          accounts: [
            {
              hostname: 'github.com',
              username: 'personal',
              email: null,
              active: true,
              state: 'success',
            },
          ],
        });
      case 'list_profiles':
        return Promise.resolve(profiles);
      case 'create_profile':
        return Promise.resolve(profile);
      default:
        return Promise.reject(new Error(`Unexpected command: ${command}`));
    }
  });
}

describe('profiles page flow', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('loads profiles and exposes only implemented navigation sections', async () => {
    const user = userEvent.setup();
    mockInitialState([profile]);
    render(Page);

    await user.click(screen.getByRole('button', { name: 'Profiles' }));

    expect(await screen.findByRole('heading', { name: 'Reusable identities' })).toBeInTheDocument();
    expect(await screen.findByRole('heading', { name: 'Personal' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Repositories' })).toBeDisabled();
    expect(invokeMock).toHaveBeenCalledWith('list_profiles');
  });

  it('creates a profile through the typed command and updates the visible collection', async () => {
    const user = userEvent.setup();
    mockInitialState([]);
    render(Page);

    await user.click(screen.getByRole('button', { name: 'Profiles' }));
    await screen.findByText('No profiles yet');
    await user.click(screen.getByRole('button', { name: 'New profile' }));
    await user.type(screen.getByLabelText('Profile label'), 'Personal');
    await user.type(screen.getByLabelText('Git name'), 'Octo Cat');
    await user.type(screen.getByLabelText('Git email'), 'octo@example.com');
    await user.selectOptions(screen.getByLabelText('GitHub account'), 'github.com::personal');
    await user.click(screen.getByRole('button', { name: 'Create profile' }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('create_profile', {
        profile: {
          label: 'Personal',
          gitName: 'Octo Cat',
          gitEmail: 'octo@example.com',
          githubAccount: { hostname: 'github.com', username: 'personal' },
        },
      });
    });
    expect(await screen.findByRole('heading', { name: 'Personal' })).toBeInTheDocument();
  });
});
