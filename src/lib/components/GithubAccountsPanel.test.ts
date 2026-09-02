import { render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { GithubAccount, GithubAccountsStatus } from '$lib/domain/github';

import GithubAccountsPanel from './GithubAccountsPanel.svelte';

type PanelProps = {
  status: GithubAccountsStatus | null;
  loading: boolean;
  error: string | null;
  switchingAccountKey: string | null;
  viewingAccountKey: string | null;
  loginLoading: boolean;
  loginInProgress: boolean;
  showUpdateAction: boolean;
  updateLoading: boolean;
  onRefresh: () => void | Promise<void>;
  onSwitch: (account: GithubAccount) => void | Promise<void>;
  onView: (account: GithubAccount) => void | Promise<void>;
  onLogin: () => void | Promise<void>;
  onOpenLoginPage: () => void | Promise<void>;
  onUpdate: () => void | Promise<void>;
};

const accountStatus: GithubAccountsStatus = {
  accounts: [
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
    {
      hostname: 'github.example.com',
      username: 'enterprise',
      email: 'enterprise@example.com',
      active: true,
      state: 'timeout',
    },
  ],
};

function renderPanel(overrides: Partial<PanelProps> = {}) {
  return render(GithubAccountsPanel, {
    status: accountStatus,
    loading: false,
    error: null,
    switchingAccountKey: null,
    viewingAccountKey: null,
    loginLoading: false,
    loginInProgress: false,
    showUpdateAction: false,
    updateLoading: false,
    onRefresh: vi.fn(),
    onSwitch: vi.fn(),
    onView: vi.fn(),
    onLogin: vi.fn(),
    onOpenLoginPage: vi.fn(),
    onUpdate: vi.fn(),
    ...overrides,
  });
}

describe('GithubAccountsPanel', () => {
  it('shows loading and empty states', () => {
    const { unmount } = renderPanel({ status: null, loading: true });

    expect(screen.getByLabelText('Loading GitHub accounts')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Refreshing…' })).toBeDisabled();
    unmount();

    renderPanel({ status: { accounts: [] } });
    expect(screen.getByText('No GitHub accounts found')).toBeInTheDocument();
  });

  it('groups hosts and distinguishes active and unhealthy accounts', () => {
    renderPanel();

    expect(screen.getByRole('heading', { name: 'github.com' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'github.example.com' })).toBeInTheDocument();
    expect(screen.getAllByText('Active')).toHaveLength(2);
    expect(screen.getByText('Check timed out')).toBeInTheDocument();
    expect(screen.getByText('Unavailable')).toBeInTheDocument();
    expect(screen.getByText('personal@example.com')).toBeInTheDocument();
    expect(screen.getByText('No public email')).toBeInTheDocument();
  });

  it('offers browser viewing for every account, including unhealthy accounts', async () => {
    const user = userEvent.setup();
    const onView = vi.fn();
    renderPanel({ onView });

    expect(screen.getAllByRole('button', { name: /in browser$/ })).toHaveLength(3);
    expect(screen.getAllByText('View on browser')).toHaveLength(3);
    await user.click(
      screen.getByRole('button', {
        name: 'View @enterprise on github.example.com in browser',
      }),
    );
    expect(onView).toHaveBeenCalledWith(accountStatus.accounts[2]);
  });

  it('requests a switch for a healthy inactive account', async () => {
    const user = userEvent.setup();
    const onSwitch = vi.fn();
    renderPanel({ onSwitch });

    await user.click(screen.getByRole('button', { name: 'Switch to @work on github.com' }));

    expect(onSwitch).toHaveBeenCalledWith(accountStatus.accounts[1]);
    expect(
      screen.queryByRole('button', { name: 'Switch to @enterprise on github.example.com' }),
    ).not.toBeInTheDocument();
  });

  it('shows browser-login guidance and can explicitly reopen the browser', async () => {
    const user = userEvent.setup();
    const onOpenLoginPage = vi.fn();
    renderPanel({ loginInProgress: true, onOpenLoginPage });

    expect(screen.getByText(/one-time code is in your clipboard/)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Login in progress' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Refresh' })).toBeEnabled();
    await user.click(screen.getByRole('button', { name: 'Open browser' }));
    expect(onOpenLoginPage).toHaveBeenCalledOnce();
  });

  it('launches login and refreshes through accessible controls', async () => {
    const user = userEvent.setup();
    const onLogin = vi.fn();
    const onRefresh = vi.fn();
    renderPanel({ onLogin, onRefresh });

    await user.click(screen.getByRole('button', { name: 'Add account' }));
    await user.click(screen.getByRole('button', { name: 'Refresh' }));

    expect(onLogin).toHaveBeenCalledOnce();
    expect(onRefresh).toHaveBeenCalledOnce();
  });

  it('offers the safe update action when structured account status is unavailable', async () => {
    const user = userEvent.setup();
    const onUpdate = vi.fn();
    renderPanel({
      status: null,
      error:
        'GitHub CLI could not provide structured account status. Install or update GitHub CLI, then try again.',
      showUpdateAction: true,
      onUpdate,
    });

    expect(screen.getByRole('alert')).toHaveTextContent('Install or update GitHub CLI');
    expect(screen.getByRole('button', { name: 'Add account' })).toBeDisabled();
    await user.click(screen.getByRole('button', { name: 'Update GitHub CLI' }));
    expect(onUpdate).toHaveBeenCalledOnce();
  });

  it('shows safe errors and disables duplicate operations', () => {
    renderPanel({
      error: 'GitHub CLI could not provide structured account status.',
      switchingAccountKey: 'github.com::work',
    });

    expect(screen.getByRole('alert')).toHaveTextContent('structured account status');
    expect(screen.getByRole('button', { name: 'Refresh' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Add account' })).toBeDisabled();
    const workRow = screen.getByText('@work').closest('li');
    expect(workRow).not.toBeNull();
    expect(
      within(workRow!).getByRole('button', { name: 'Switch to @work on github.com' }),
    ).toBeDisabled();
    expect(within(workRow!).getByText('Switching…')).toBeInTheDocument();
  });
});
