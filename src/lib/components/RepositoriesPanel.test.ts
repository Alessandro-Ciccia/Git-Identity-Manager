import { render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { RegisteredRepository } from '$lib/domain/repositories';

import RepositoriesPanel from './RepositoriesPanel.svelte';

type Props = {
  repositories: RegisteredRepository[];
  loading: boolean;
  error: string | null;
  adding: boolean;
  refreshingId: string | null;
  removingId: string | null;
  revealingId: string | null;
  onAdd: () => void | Promise<void>;
  onRefreshAll: () => void | Promise<void>;
  onRefresh: (id: string) => void | Promise<void>;
  onRemove: (id: string) => Promise<boolean>;
  onReveal: (id: string) => void | Promise<void>;
};

const available: RegisteredRepository = {
  id: 'dc683096-c95b-4b68-b609-b994f98b2d2c',
  path: '/work/project',
  addedAt: '2026-09-03T10:00:00Z',
  state: 'available',
  message: null,
  inspection: {
    path: '/work/project',
    remotes: [
      { name: 'origin', url: 'https://github.com/octo/project.git', direction: 'fetch' },
      { name: 'upstream', url: 'github.com:base/project.git', direction: 'push' },
    ],
    identity: {
      name: {
        value: 'Octo Cat',
        source: {
          scope: 'global',
          origin: 'file:/home/octo/.config/git/work.inc',
          conditionalInclude: true,
        },
      },
      email: { value: null, source: null },
    },
  },
};

const missing: RegisteredRepository = {
  id: '30f16421-da51-4822-9b31-c389a118d6b2',
  path: '/work/moved',
  addedAt: '2026-09-03T10:00:00Z',
  state: 'missing',
  inspection: null,
  message: 'This repository folder is missing or has moved.',
};

function renderPanel(overrides: Partial<Props> = {}) {
  return render(RepositoriesPanel, {
    repositories: [available, missing],
    loading: false,
    error: null,
    adding: false,
    refreshingId: null,
    removingId: null,
    revealingId: null,
    onAdd: vi.fn(),
    onRefreshAll: vi.fn(),
    onRefresh: vi.fn(),
    onRemove: vi.fn().mockResolvedValue(true),
    onReveal: vi.fn(),
    ...overrides,
  });
}

describe('RepositoriesPanel', () => {
  it('shows loading and empty states', () => {
    const { unmount } = renderPanel({ repositories: [], loading: true });
    expect(screen.getByLabelText('Loading repositories')).toBeInTheDocument();
    unmount();

    renderPanel({ repositories: [] });
    expect(screen.getByText('No repositories registered')).toBeInTheDocument();
  });

  it('renders effective identity, origins, remotes, and stale registrations', () => {
    renderPanel();

    expect(screen.getByRole('heading', { name: 'project' })).toBeInTheDocument();
    expect(screen.getByText('Octo Cat')).toBeInTheDocument();
    expect(screen.getByText('Conditional include · Global')).toBeInTheDocument();
    expect(screen.getByText('/home/octo/.config/git/work.inc')).toBeInTheDocument();
    expect(screen.getAllByText('Not configured')).toHaveLength(2);
    expect(screen.getByText('https://github.com/octo/project.git')).toBeInTheDocument();
    expect(screen.getByText('This repository folder is missing or has moved.')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Reveal moved' })).toBeDisabled();
  });

  it('calls add, refresh, and reveal actions', async () => {
    const user = userEvent.setup();
    const onAdd = vi.fn();
    const onRefresh = vi.fn();
    const onReveal = vi.fn();
    renderPanel({ onAdd, onRefresh, onReveal });

    await user.click(screen.getByRole('button', { name: 'Add repository' }));
    await user.click(screen.getByRole('button', { name: 'Refresh project' }));
    await user.click(screen.getByRole('button', { name: 'Reveal project' }));

    expect(onAdd).toHaveBeenCalledOnce();
    expect(onRefresh).toHaveBeenCalledWith(available.id);
    expect(onReveal).toHaveBeenCalledWith(available.id);
  });

  it('requires confirmation and explains removal does not modify files', async () => {
    const user = userEvent.setup();
    const onRemove = vi.fn().mockResolvedValue(true);
    renderPanel({ onRemove });

    await user.click(screen.getByRole('button', { name: 'Remove project' }));
    expect(onRemove).not.toHaveBeenCalled();
    const card = screen.getByRole('heading', { name: 'project' }).closest('li');
    expect(card).not.toBeNull();
    expect(
      within(card!).getByText(/files and Git configuration will not be changed/),
    ).toBeInTheDocument();
    await user.click(within(card!).getByRole('button', { name: 'Remove registration' }));
    expect(onRemove).toHaveBeenCalledWith(available.id);
  });

  it('shows safe errors and disables concurrent actions', () => {
    renderPanel({ error: 'Repository could not be inspected.', adding: true });
    expect(screen.getByRole('alert')).toHaveTextContent('could not be inspected');
    expect(screen.getByRole('button', { name: 'Selecting…' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Refresh all' })).toBeDisabled();
  });
});
