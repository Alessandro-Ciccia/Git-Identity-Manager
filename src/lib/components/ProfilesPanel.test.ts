import { render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { GithubAccount } from '$lib/domain/github';
import type { GitProfile, ProfileInput } from '$lib/domain/profiles';

import ProfilesPanel from './ProfilesPanel.svelte';

type PanelProps = {
  profiles: GitProfile[];
  accounts: GithubAccount[];
  loading: boolean;
  error: string | null;
  savingProfileId: string | null;
  deletingProfileId: string | null;
  onRefresh: () => void | Promise<void>;
  onCreate: (profile: ProfileInput) => Promise<boolean>;
  onUpdate: (id: string, profile: ProfileInput) => Promise<boolean>;
  onDelete: (id: string) => Promise<boolean>;
};

const accounts: GithubAccount[] = [
  {
    hostname: 'github.com',
    username: 'personal',
    email: 'personal@example.com',
    active: true,
    state: 'success',
  },
  {
    hostname: 'github.com',
    username: 'work',
    email: null,
    active: false,
    state: 'success',
  },
];

const profiles: GitProfile[] = [
  {
    id: '0db9c44d-e1e1-493b-a14f-0208aa4ef247',
    label: 'Personal',
    gitName: 'Octo Cat',
    gitEmail: 'octo@example.com',
    githubAccount: { hostname: 'github.com', username: 'personal' },
  },
  {
    id: '6f5818cf-2897-4a15-9709-8ac90a411ce8',
    label: 'Former client',
    gitName: 'Octo Cat',
    gitEmail: 'octo@client.example',
    githubAccount: { hostname: 'github.example.com', username: 'octo-client' },
  },
];

function renderPanel(overrides: Partial<PanelProps> = {}) {
  return render(ProfilesPanel, {
    profiles,
    accounts,
    loading: false,
    error: null,
    savingProfileId: null,
    deletingProfileId: null,
    onRefresh: vi.fn(),
    onCreate: vi.fn().mockResolvedValue(true),
    onUpdate: vi.fn().mockResolvedValue(true),
    onDelete: vi.fn().mockResolvedValue(true),
    ...overrides,
  });
}

describe('ProfilesPanel', () => {
  it('shows loading and empty states', () => {
    const { unmount } = renderPanel({ profiles: [], loading: true });
    expect(screen.getByLabelText('Loading profiles')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Refreshing…' })).toBeDisabled();
    unmount();

    renderPanel({ profiles: [] });
    expect(screen.getByText('No profiles yet')).toBeInTheDocument();
  });

  it('renders profile cards, active associations, and stale associations', () => {
    renderPanel();

    expect(screen.getByRole('heading', { name: 'Personal' })).toBeInTheDocument();
    expect(screen.getByText('octo@example.com')).toBeInTheDocument();
    expect(screen.getByText('@personal · github.com')).toBeInTheDocument();
    expect(screen.getByText('Active')).toBeInTheDocument();
    expect(screen.getByText('@octo-client · github.example.com')).toBeInTheDocument();
    expect(screen.getByText('Not discovered')).toBeInTheDocument();
  });

  it('creates a validated profile with an optional discovered account association', async () => {
    const user = userEvent.setup();
    const onCreate = vi.fn().mockResolvedValue(true);
    renderPanel({ profiles: [], onCreate });

    await user.click(screen.getByRole('button', { name: 'New profile' }));
    await user.type(screen.getByLabelText('Profile label'), 'Work');
    await user.type(screen.getByLabelText('Git name'), 'Octo Cat');
    await user.type(screen.getByLabelText('Git email'), 'octo@company.example');
    await user.selectOptions(screen.getByLabelText('GitHub account'), 'github.com::work');
    await user.click(screen.getByRole('button', { name: 'Create profile' }));

    expect(onCreate).toHaveBeenCalledWith({
      label: 'Work',
      gitName: 'Octo Cat',
      gitEmail: 'octo@company.example',
      githubAccount: { hostname: 'github.com', username: 'work' },
    });
    expect(
      screen.queryByRole('heading', { name: 'Define a Git identity' }),
    ).not.toBeInTheDocument();
  });

  it('shows accessible validation errors without calling create', async () => {
    const user = userEvent.setup();
    const onCreate = vi.fn().mockResolvedValue(true);
    renderPanel({ profiles: [], onCreate });

    await user.click(screen.getByRole('button', { name: 'New profile' }));
    await user.click(screen.getByRole('button', { name: 'Create profile' }));

    expect(onCreate).not.toHaveBeenCalled();
    expect(screen.getByText('Enter a label between 1 and 80 characters.')).toBeInTheDocument();
    expect(screen.getByText('Enter a valid Git email address.')).toBeInTheDocument();
    expect(screen.getByLabelText('Profile label')).toHaveAttribute('aria-invalid', 'true');
  });

  it('edits a profile while retaining a stale account choice', async () => {
    const user = userEvent.setup();
    const onUpdate = vi.fn().mockResolvedValue(true);
    renderPanel({ onUpdate });

    await user.click(screen.getByRole('button', { name: 'Edit Former client' }));
    expect(
      screen.getByRole('option', {
        name: '@octo-client · github.example.com (not currently discovered)',
      }),
    ).toBeInTheDocument();
    await user.clear(screen.getByLabelText('Profile label'));
    await user.type(screen.getByLabelText('Profile label'), 'Archived client');
    await user.click(screen.getByRole('button', { name: 'Save changes' }));

    expect(onUpdate).toHaveBeenCalledWith(profiles[1]!.id, {
      label: 'Archived client',
      gitName: 'Octo Cat',
      gitEmail: 'octo@client.example',
      githubAccount: { hostname: 'github.example.com', username: 'octo-client' },
    });
  });

  it('requires explicit confirmation before deleting', async () => {
    const user = userEvent.setup();
    const onDelete = vi.fn().mockResolvedValue(true);
    renderPanel({ onDelete });

    await user.click(screen.getByRole('button', { name: 'Delete Personal' }));
    expect(onDelete).not.toHaveBeenCalled();
    const card = screen.getByRole('heading', { name: 'Personal' }).closest('li');
    expect(card).not.toBeNull();
    expect(within(card!).getByText(/cannot be undone/)).toBeInTheDocument();
    await user.click(within(card!).getByRole('button', { name: 'Delete profile' }));
    expect(onDelete).toHaveBeenCalledWith(profiles[0]!.id);
  });

  it('shows safe errors, refreshes, and disables duplicate operations', async () => {
    const user = userEvent.setup();
    const onRefresh = vi.fn();
    renderPanel({
      error: 'Profiles could not be read or saved.',
      savingProfileId: profiles[0]!.id,
      onRefresh,
    });

    expect(screen.getByRole('alert')).toHaveTextContent('could not be read or saved');
    expect(screen.getByRole('button', { name: 'Refresh' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'New profile' })).toBeDisabled();
    await user.click(screen.getByRole('button', { name: 'Refresh' }));
    expect(onRefresh).not.toHaveBeenCalled();
  });
});
