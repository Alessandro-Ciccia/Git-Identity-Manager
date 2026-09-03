import { describe, expect, it } from 'vitest';

import type { GithubAccountsStatus } from './github';
import type { GitProfile } from './profiles';
import {
  configOriginLabel,
  configSourceLabel,
  evaluateRepositoryAssignment,
  originRemotes,
  repositoryIdentityKeyLabel,
  repositoryName,
  type GitRemote,
  type RegisteredRepository,
} from './repositories';

const remotes: GitRemote[] = [
  { name: 'origin', url: 'https://github.com/org/project.git', direction: 'fetch' },
  { name: 'upstream', url: 'https://github.com/base/project.git', direction: 'fetch' },
];

const profile: GitProfile = {
  id: '0db9c44d-e1e1-493b-a14f-0208aa4ef247',
  label: 'Work',
  gitName: 'Work Octo',
  gitEmail: 'octo@company.example',
  githubAccount: { hostname: 'github.com', username: 'octo-work' },
};

const repository: RegisteredRepository = {
  id: 'dc683096-c95b-4b68-b609-b994f98b2d2c',
  path: '/work/project',
  addedAt: '2026-09-03T10:00:00Z',
  profileId: profile.id,
  state: 'available',
  message: null,
  inspection: {
    path: '/work/project',
    remotes,
    identity: {
      name: { value: profile.gitName, source: null },
      email: { value: profile.gitEmail, source: null },
    },
  },
};

function githubStatus(username: string): GithubAccountsStatus {
  return {
    accounts: [
      {
        hostname: 'github.com',
        username,
        email: null,
        active: true,
        state: 'success',
      },
    ],
  };
}

describe('repository domain', () => {
  it('derives names from POSIX and Windows paths', () => {
    expect(repositoryName('/work/project')).toBe('project');
    expect(repositoryName('C:\\work\\project')).toBe('project');
  });

  it('describes configuration scope, keys, and conditional includes', () => {
    expect(configSourceLabel(null)).toBe('Not configured');
    expect(
      configSourceLabel({
        scope: 'global',
        origin: 'file:/home/octo/.config/git/work.inc',
        conditionalInclude: true,
      }),
    ).toBe('Conditional include · Global');
    expect(
      configOriginLabel({ scope: 'local', origin: 'file:.git/config', conditionalInclude: false }),
    ).toBe('.git/config');
    expect(repositoryIdentityKeyLabel('userName')).toBe('user.name');
    expect(repositoryIdentityKeyLabel('userEmail')).toBe('user.email');
  });

  it('selects origin remotes without discarding other remotes', () => {
    expect(originRemotes(remotes)).toEqual([remotes[0]]);
    expect(remotes).toHaveLength(2);
  });

  it('evaluates every documented mismatch combination', () => {
    expect(
      evaluateRepositoryAssignment(repository, [profile], githubStatus('octo-work')).status,
    ).toBe('correct');

    const wrongGit = structuredClone(repository);
    wrongGit.inspection!.identity.email.value = 'personal@example.com';
    expect(
      evaluateRepositoryAssignment(wrongGit, [profile], githubStatus('octo-work')).status,
    ).toBe('gitIdentityMismatch');
    expect(
      evaluateRepositoryAssignment(repository, [profile], githubStatus('personal')).status,
    ).toBe('githubAccountMismatch');
    expect(evaluateRepositoryAssignment(wrongGit, [profile], githubStatus('personal')).status).toBe(
      'bothMismatch',
    );
  });

  it('handles unassigned, stale, unavailable, and Git-only profiles without false matches', () => {
    expect(
      evaluateRepositoryAssignment(
        { ...repository, profileId: null },
        [profile],
        githubStatus('octo-work'),
      ).status,
    ).toBe('unassigned');
    expect(evaluateRepositoryAssignment(repository, [], githubStatus('octo-work')).status).toBe(
      'unknown',
    );
    expect(evaluateRepositoryAssignment(repository, [profile], null).status).toBe('unknown');
    expect(
      evaluateRepositoryAssignment(
        { ...repository, state: 'missing', inspection: null },
        [profile],
        githubStatus('octo-work'),
      ).status,
    ).toBe('unknown');

    const gitOnly = { ...profile, githubAccount: null };
    expect(evaluateRepositoryAssignment(repository, [gitOnly], null).status).toBe('correct');
  });
});
