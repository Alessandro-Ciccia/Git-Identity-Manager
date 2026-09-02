import { invoke } from '@tauri-apps/api/core';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { GithubAccount } from '$lib/domain/github';

import {
  githubCliUpdateRecommended,
  githubErrorMessage,
  launchGithubLogin,
  listGithubAccounts,
  openGithubAccountPage,
  openGithubCliUpdatePage,
  openGithubLoginPage,
  switchGithubAccount,
} from './github';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);
const account: GithubAccount = {
  hostname: 'github.example.com',
  username: 'octocat',
  email: 'octocat@example.com',
  active: false,
  state: 'success',
};

describe('GitHub IPC', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue({ accounts: [] });
  });

  it('opens the fixed GitHub device login page without URL input', async () => {
    await openGithubLoginPage();

    expect(invokeMock).toHaveBeenCalledWith('open_github_login_page');
  });

  it('opens a verified account page using only the selected account identity', async () => {
    await openGithubAccountPage(account);

    expect(invokeMock).toHaveBeenCalledWith('open_github_account_page', {
      hostname: 'github.example.com',
      username: 'octocat',
    });
  });

  it('opens the fixed GitHub CLI update page command without URL input', async () => {
    await openGithubCliUpdatePage();

    expect(invokeMock).toHaveBeenCalledWith('open_github_cli_update_page');
  });

  it('lists accounts through a fixed command without arguments', async () => {
    await listGithubAccounts();

    expect(invokeMock).toHaveBeenCalledWith('list_github_accounts');
  });

  it('switches using only the selected account identity', async () => {
    await switchGithubAccount(account);

    expect(invokeMock).toHaveBeenCalledWith('switch_github_account', {
      hostname: 'github.example.com',
      username: 'octocat',
    });
  });

  it('launches login for a fixed host value', async () => {
    await launchGithubLogin('github.com');

    expect(invokeMock).toHaveBeenCalledWith('launch_github_login', {
      hostname: 'github.com',
    });
  });

  it('recommends an update only for missing or incompatible GitHub CLI errors', () => {
    expect(githubCliUpdateRecommended({ code: 'githubCliMissing' })).toBe(true);
    expect(githubCliUpdateRecommended({ code: 'githubStatusFailed' })).toBe(true);
    expect(githubCliUpdateRecommended({ code: 'githubStatusMalformed' })).toBe(true);
    expect(githubCliUpdateRecommended({ code: 'githubLoginFailed' })).toBe(false);
    expect(githubCliUpdateRecommended('raw error')).toBe(false);
  });

  it('uses a structured backend message and hides unknown rejection values', () => {
    expect(
      githubErrorMessage({
        code: 'githubSwitchNotVerified',
        message: 'The selected account did not become active.',
      }),
    ).toBe('The selected account did not become active.');
    expect(githubErrorMessage('raw CLI output')).toBe(
      'The GitHub account operation could not be completed. Please try again.',
    );
  });
});
