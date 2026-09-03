import { describe, expect, it } from 'vitest';

import type { GithubAccount } from './github';
import {
  findAssociatedAccount,
  githubReferenceKey,
  hasProfileValidationErrors,
  profileDraft,
  profileInputFromDraft,
  validateProfileDraft,
  type GitProfile,
  type ProfileDraft,
} from './profiles';

const validDraft: ProfileDraft = {
  label: 'Personal',
  gitName: 'Octo Cat',
  gitEmail: 'octo@example.com',
  githubAccountKey: 'github.com::octocat',
};

const profile: GitProfile = {
  id: '0db9c44d-e1e1-493b-a14f-0208aa4ef247',
  label: 'Personal',
  gitName: 'Octo Cat',
  gitEmail: 'octo@example.com',
  githubAccount: { hostname: 'github.com', username: 'OctoCat' },
};

const accounts: GithubAccount[] = [
  {
    hostname: 'github.com',
    username: 'octocat',
    email: null,
    active: true,
    state: 'success',
  },
];

describe('profiles domain', () => {
  it('normalizes a valid draft into a secret-free input', () => {
    expect(
      profileInputFromDraft({
        ...validDraft,
        label: ' Personal ',
        gitName: ' Octo Cat ',
        gitEmail: ' octo@example.com ',
        githubAccountKey: 'GitHub.COM::OctoCat',
      }),
    ).toEqual({
      label: 'Personal',
      gitName: 'Octo Cat',
      gitEmail: 'octo@example.com',
      githubAccount: { hostname: 'github.com', username: 'OctoCat' },
    });
  });

  it('reports required, malformed, oversized, and control-character fields', () => {
    const errors = validateProfileDraft({
      label: ' '.repeat(81),
      gitName: 'Octo\nCat',
      gitEmail: 'not-an-email',
      githubAccountKey: 'https://github.com::octocat',
    });

    expect(errors).toEqual({
      label: 'Enter a label between 1 and 80 characters.',
      gitName: 'Enter a Git name between 1 and 200 characters.',
      gitEmail: 'Enter a valid Git email address.',
      githubAccount: 'Select a valid GitHub account.',
    });
    expect(hasProfileValidationErrors(errors)).toBe(true);
    expect(profileInputFromDraft({ ...validDraft, gitEmail: 'invalid' })).toBeNull();
  });

  it('allows a profile without a GitHub association', () => {
    expect(
      profileInputFromDraft({ ...validDraft, githubAccountKey: '' })?.githubAccount,
    ).toBeNull();
  });

  it('creates drafts and case-insensitive association keys from profiles', () => {
    expect(profileDraft(profile)).toEqual({
      label: 'Personal',
      gitName: 'Octo Cat',
      gitEmail: 'octo@example.com',
      githubAccountKey: 'github.com::octocat',
    });
    expect(githubReferenceKey(profile.githubAccount!)).toBe('github.com::octocat');
  });

  it('finds a discovered association case-insensitively and tolerates stale references', () => {
    expect(findAssociatedAccount(profile.githubAccount!, accounts)).toEqual(accounts[0]);
    expect(
      findAssociatedAccount({ hostname: 'github.example.com', username: 'missing' }, accounts),
    ).toBeUndefined();
  });
});
