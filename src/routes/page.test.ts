import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { GitProfile } from '$lib/domain/profiles';
import type { RegisteredRepository } from '$lib/domain/repositories';

import Page from './+page.svelte';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);
const openMock = vi.mocked(open);
const repository: RegisteredRepository = {
  id: 'dc683096-c95b-4b68-b609-b994f98b2d2c',
  path: '/work/project',
  addedAt: '2026-09-03T10:00:00Z',
  profileId: null,
  state: 'available',
  message: null,
  inspection: {
    path: '/work/project',
    remotes: [
      {
        name: 'origin',
        url: 'https://github.com/octo/project.git',
        direction: 'fetch',
      },
    ],
    identity: {
      name: {
        value: 'Octo Cat',
        source: {
          scope: 'global',
          origin: 'file:/home/octo/.gitconfig',
          conditionalInclude: false,
        },
      },
      email: {
        value: 'octo@example.com',
        source: { scope: 'local', origin: 'file:.git/config', conditionalInclude: false },
      },
    },
  },
};

const profile: GitProfile = {
  id: '0db9c44d-e1e1-493b-a14f-0208aa4ef247',
  label: 'Personal',
  gitName: 'Octo Cat',
  gitEmail: 'octo@example.com',
  githubAccount: { hostname: 'github.com', username: 'personal' },
};

type InitialStateOptions = {
  repositories?: RegisteredRepository[];
  welcomeDismissed?: boolean;
  environmentReady?: boolean;
};

function mockInitialState(
  profiles: GitProfile[],
  {
    repositories = [repository],
    welcomeDismissed = true,
    environmentReady = true,
  }: InitialStateOptions = {},
): void {
  invokeMock.mockImplementation((command) => {
    switch (command) {
      case 'get_preferences':
        return Promise.resolve({ theme: 'system', welcomeDismissed });
      case 'set_theme_preference':
        return Promise.resolve({ theme: 'dark', welcomeDismissed });
      case 'set_welcome_dismissed':
        return Promise.resolve({ theme: 'system', welcomeDismissed: true });
      case 'get_environment_status':
        return Promise.resolve({
          git: {
            dependency: 'git',
            state: environmentReady ? 'available' : 'missing',
            version: environmentReady ? '2.47.0' : null,
            message: environmentReady ? 'Git is available.' : 'Git is not installed.',
          },
          githubCli: {
            dependency: 'githubCli',
            state: 'available',
            version: '2.73.0',
            message: 'GitHub CLI is available.',
          },
          isReady: environmentReady,
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
      case 'list_repositories':
        return Promise.resolve(repositories);
      case 'list_directory_rules':
        return Promise.resolve([]);
      case 'register_repository':
        return Promise.resolve(repository);
      case 'assign_repository_profile':
        return Promise.resolve({
          ...repository,
          profileId: profile.id,
          inspection: {
            ...repository.inspection!,
            identity: {
              ...repository.inspection!.identity,
              email: {
                ...repository.inspection!.identity.email,
                value: 'wrong@example.com',
              },
            },
          },
        });
      case 'preview_repository_profile':
        return Promise.resolve({
          repositoryId: repository.id,
          path: repository.path,
          profile,
          changes: [
            {
              key: 'userName',
              current: repository.inspection!.identity.name,
              desired: profile.gitName,
            },
            {
              key: 'userEmail',
              current: {
                ...repository.inspection!.identity.email,
                value: 'wrong@example.com',
              },
              desired: profile.gitEmail,
            },
          ],
        });
      case 'apply_repository_profile':
        return Promise.resolve({ ...repository, profileId: profile.id });
      case 'create_profile':
        return Promise.resolve(profile);
      case 'preview_directory_rule':
        return Promise.resolve({
          ruleId: null,
          directory: '/selected/work',
          profile,
          operation: 'add',
          condition: 'gitdir:/selected/work/',
          identityFilePath: '/app/identities/profile.gitconfig',
          globalConfigPath: '/home/octo/.gitconfig',
          backupRequired: true,
          conflicts: [],
          canApply: true,
        });
      case 'apply_directory_rule':
        return Promise.resolve({
          id: 'rule-id',
          directory: '/selected/work',
          profileId: profile.id,
          state: 'active',
          message: null,
        });
      default:
        return Promise.reject(new Error(`Unexpected command: ${command}`));
    }
  });
}

describe('profiles page flow', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    openMock.mockReset();
  });

  it('loads profiles and exposes the navigation sections', async () => {
    const user = userEvent.setup();
    mockInitialState([profile]);
    render(Page);

    await user.click(screen.getByRole('button', { name: 'Profiles' }));

    expect(await screen.findByRole('heading', { name: 'Reusable identities' })).toBeInTheDocument();
    expect(await screen.findByRole('heading', { name: 'Personal' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Repositories' })).toBeEnabled();
    expect(screen.getByRole('button', { name: 'Rules' })).toBeEnabled();
    expect(screen.getByRole('button', { name: 'Settings' })).toBeEnabled();
    expect(invokeMock).toHaveBeenCalledWith('list_profiles');
  });

  it('navigates the sidebar with arrow keys', async () => {
    const user = userEvent.setup();
    mockInitialState([profile]);
    render(Page);
    await screen.findByRole('heading', { name: 'Identity overview' });

    screen.getByRole('button', { name: 'Overview' }).focus();
    await user.keyboard('{ArrowDown}');
    expect(screen.getByRole('button', { name: 'Profiles' })).toHaveFocus();

    await user.keyboard('{End}');
    expect(screen.getByRole('button', { name: 'Settings' })).toHaveFocus();

    await user.keyboard('{Enter}');
    expect(
      await screen.findByRole('heading', { name: 'Application preferences' }),
    ).toBeInTheDocument();
  });

  it('persists a theme change through the Settings section', async () => {
    const user = userEvent.setup();
    mockInitialState([profile]);
    render(Page);
    await screen.findByRole('heading', { name: 'Identity overview' });

    await user.click(screen.getByRole('button', { name: 'Settings' }));
    await user.click(screen.getByRole('radio', { name: 'Dark' }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('set_theme_preference', { theme: 'dark' });
    });
    expect(document.documentElement.dataset.theme).toBe('dark');
  });

  it('shows the first-run welcome screen until it is skipped', async () => {
    const user = userEvent.setup();
    mockInitialState([], { repositories: [], welcomeDismissed: false });
    render(Page);

    expect(
      await screen.findByRole('heading', { name: 'Let’s set up Git Identity Manager' }),
    ).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Skip for now' }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('set_welcome_dismissed', { dismissed: true });
    });
    expect(await screen.findByRole('heading', { name: 'Identity overview' })).toBeInTheDocument();
  });

  it('shows the welcome screen when a required tool is missing even with data present', async () => {
    mockInitialState([profile], { welcomeDismissed: false, environmentReady: false });
    render(Page);

    expect(
      await screen.findByRole('heading', { name: 'Let’s set up Git Identity Manager' }),
    ).toBeInTheDocument();
    expect(screen.getByText('Git is not installed.')).toBeInTheDocument();
  });

  it('lets the sidebar navigate away from the welcome screen without dismissing it', async () => {
    const user = userEvent.setup();
    mockInitialState([profile], { welcomeDismissed: false, environmentReady: false });
    render(Page);
    await screen.findByRole('heading', { name: 'Let’s set up Git Identity Manager' });

    await user.click(screen.getByRole('button', { name: 'Repositories' }));
    expect(
      await screen.findByRole('heading', { name: 'Repository identities' }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole('heading', { name: 'Let’s set up Git Identity Manager' }),
    ).not.toBeInTheDocument();
    expect(invokeMock).not.toHaveBeenCalledWith('set_welcome_dismissed', expect.anything());

    await user.click(screen.getByRole('button', { name: 'Overview' }));
    expect(
      await screen.findByRole('heading', { name: 'Let’s set up Git Identity Manager' }),
    ).toBeInTheDocument();
  });

  it('opens the repositories view with effective identity and source data', async () => {
    const user = userEvent.setup();
    mockInitialState([]);
    render(Page);

    await user.click(screen.getByRole('button', { name: 'Repositories' }));

    expect(
      await screen.findByRole('heading', { name: 'Repository identities' }),
    ).toBeInTheDocument();
    expect(await screen.findByRole('heading', { name: 'project' })).toBeInTheDocument();
    expect(screen.getByText('Octo Cat')).toBeInTheDocument();
    expect(screen.getByText('octo@example.com')).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith('list_repositories');
  });

  it('registers a directory selected by the native picker and ignores cancellation', async () => {
    const user = userEvent.setup();
    mockInitialState([], { repositories: [] });
    openMock.mockResolvedValueOnce(null).mockResolvedValueOnce('/selected/project');
    render(Page);

    await user.click(screen.getByRole('button', { name: 'Repositories' }));
    await screen.findByText('No repositories registered');
    await user.click(screen.getByRole('button', { name: 'Add repository' }));
    expect(invokeMock).not.toHaveBeenCalledWith('register_repository', expect.anything());

    await user.click(screen.getByRole('button', { name: 'Add repository' }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('register_repository', {
        path: '/selected/project',
      });
    });
    expect(await screen.findByRole('heading', { name: 'project' })).toBeInTheDocument();
  });

  it('assigns, previews, and applies a repository profile through narrow commands', async () => {
    const user = userEvent.setup();
    mockInitialState([profile]);
    render(Page);

    await user.click(screen.getByRole('button', { name: 'Repositories' }));
    await screen.findByRole('heading', { name: 'project' });
    await user.selectOptions(screen.getByLabelText('Expected profile for project'), profile.id);
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('assign_repository_profile', {
        id: repository.id,
        profileId: profile.id,
      });
    });

    await user.click(await screen.findByRole('button', { name: 'Fix Git identity' }));
    expect(await screen.findByRole('dialog')).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith('preview_repository_profile', { id: repository.id });
    await user.click(screen.getByRole('button', { name: 'Apply local identity' }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('apply_repository_profile', { id: repository.id });
    });
    expect(await screen.findByRole('status')).toHaveTextContent('Verified Personal');
  });

  it('previews and applies a directory rule through narrow commands', async () => {
    const user = userEvent.setup();
    mockInitialState([profile]);
    openMock.mockResolvedValue('/selected/work');
    render(Page);

    await user.click(screen.getByRole('button', { name: 'Rules' }));
    expect(
      await screen.findByRole('heading', { name: 'Directory identity rules' }),
    ).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Choose directory…' }));
    await user.selectOptions(screen.getByLabelText('Profile for directory rule'), profile.id);
    await user.click(screen.getByRole('button', { name: 'Preview' }));

    expect(await screen.findByRole('dialog', { name: 'Add directory rule' })).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith('preview_directory_rule', {
      input: { id: null, directory: '/selected/work', profileId: profile.id },
    });
    await user.click(screen.getByRole('button', { name: 'Apply rule' }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('apply_directory_rule', {
        input: { id: null, directory: '/selected/work', profileId: profile.id },
      });
    });
    expect(await screen.findByText('/selected/work/**')).toBeInTheDocument();
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
