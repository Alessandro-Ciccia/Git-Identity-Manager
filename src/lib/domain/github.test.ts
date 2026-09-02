import { describe, expect, it } from 'vitest';

import {
  githubAccountKey,
  groupGithubAccounts,
  hasNewGithubAccount,
  type GithubAccount,
} from './github';

const accounts: GithubAccount[] = [
  {
    hostname: 'github.com',
    username: 'Personal',
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
];

describe('GitHub account domain', () => {
  it('creates a case-insensitive host and username key', () => {
    expect(githubAccountKey(accounts[0]!)).toBe('github.com::personal');
  });

  it('groups accounts by host while retaining backend order', () => {
    expect(groupGithubAccounts(accounts)).toEqual([
      {
        hostname: 'github.com',
        accounts: accounts.slice(0, 2),
      },
      {
        hostname: 'github.example.com',
        accounts: accounts.slice(2),
      },
    ]);
  });

  it('detects a newly added account without treating state changes as new identities', () => {
    const previous = { accounts: accounts.slice(0, 1) };

    expect(hasNewGithubAccount(previous, { accounts: accounts.slice(0, 2) })).toBe(true);
    expect(
      hasNewGithubAccount(previous, {
        accounts: [{ ...accounts[0]!, active: false }],
      }),
    ).toBe(false);
  });

  it('returns no groups for an empty account list', () => {
    expect(groupGithubAccounts([])).toEqual([]);
  });
});
