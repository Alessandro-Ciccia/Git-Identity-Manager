export type GithubAuthenticationState = 'success' | 'error' | 'timeout';

export type GithubAccount = {
  hostname: string;
  username: string;
  email: string | null;
  active: boolean;
  state: GithubAuthenticationState;
};

export type GithubAccountsStatus = {
  accounts: GithubAccount[];
};

export type GithubAccountGroup = {
  hostname: string;
  accounts: GithubAccount[];
};

export const githubAuthenticationLabels = {
  success: 'Ready',
  error: 'Authentication issue',
  timeout: 'Check timed out',
} as const satisfies Record<GithubAuthenticationState, string>;

export function githubAccountKey(account: GithubAccount): string {
  return `${account.hostname.toLowerCase()}::${account.username.toLowerCase()}`;
}

export function hasNewGithubAccount(
  previous: GithubAccountsStatus,
  current: GithubAccountsStatus,
): boolean {
  const previousAccounts = new Set(previous.accounts.map(githubAccountKey));
  return current.accounts.some((account) => !previousAccounts.has(githubAccountKey(account)));
}

export function groupGithubAccounts(accounts: readonly GithubAccount[]): GithubAccountGroup[] {
  const groups = new Map<string, GithubAccount[]>();

  for (const account of accounts) {
    const group = groups.get(account.hostname);
    if (group) {
      group.push(account);
    } else {
      groups.set(account.hostname, [account]);
    }
  }

  return Array.from(groups, ([hostname, groupedAccounts]) => ({
    hostname,
    accounts: groupedAccounts,
  }));
}
