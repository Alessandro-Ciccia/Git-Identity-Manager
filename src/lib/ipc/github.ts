import { invoke } from '@tauri-apps/api/core';

import type { GithubAccount, GithubAccountsStatus } from '$lib/domain/github';

const OPEN_GITHUB_LOGIN_PAGE_COMMAND = 'open_github_login_page';
const OPEN_GITHUB_ACCOUNT_PAGE_COMMAND = 'open_github_account_page';
const OPEN_GITHUB_CLI_UPDATE_PAGE_COMMAND = 'open_github_cli_update_page';
const LIST_GITHUB_ACCOUNTS_COMMAND = 'list_github_accounts';
const SWITCH_GITHUB_ACCOUNT_COMMAND = 'switch_github_account';
const LAUNCH_GITHUB_LOGIN_COMMAND = 'launch_github_login';

export function openGithubLoginPage(): Promise<void> {
  return invoke<void>(OPEN_GITHUB_LOGIN_PAGE_COMMAND);
}

export function openGithubAccountPage(account: GithubAccount): Promise<void> {
  return invoke<void>(OPEN_GITHUB_ACCOUNT_PAGE_COMMAND, {
    hostname: account.hostname,
    username: account.username,
  });
}

export function openGithubCliUpdatePage(): Promise<void> {
  return invoke<void>(OPEN_GITHUB_CLI_UPDATE_PAGE_COMMAND);
}

export function listGithubAccounts(): Promise<GithubAccountsStatus> {
  return invoke<GithubAccountsStatus>(LIST_GITHUB_ACCOUNTS_COMMAND);
}

export function switchGithubAccount(account: GithubAccount): Promise<GithubAccountsStatus> {
  return invoke<GithubAccountsStatus>(SWITCH_GITHUB_ACCOUNT_COMMAND, {
    hostname: account.hostname,
    username: account.username,
  });
}

export function launchGithubLogin(hostname: string): Promise<GithubAccountsStatus> {
  return invoke<GithubAccountsStatus>(LAUNCH_GITHUB_LOGIN_COMMAND, { hostname });
}

export function githubCliUpdateRecommended(error: unknown): boolean {
  if (typeof error !== 'object' || error === null || !('code' in error)) {
    return false;
  }

  return (
    error.code === 'githubCliMissing' ||
    error.code === 'githubStatusFailed' ||
    error.code === 'githubStatusMalformed'
  );
}

export function githubErrorMessage(error: unknown): string {
  if (
    typeof error === 'object' &&
    error !== null &&
    'message' in error &&
    typeof error.message === 'string'
  ) {
    return error.message;
  }

  return 'The GitHub account operation could not be completed. Please try again.';
}
