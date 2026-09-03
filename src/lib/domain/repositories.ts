import type { GithubAccount, GithubAccountsStatus } from './github';
import type { GitProfile } from './profiles';

export type RepositoryState = 'available' | 'missing' | 'invalid' | 'unavailable';
export type RemoteDirection = 'fetch' | 'push';
export type GitConfigScope = 'system' | 'global' | 'local' | 'worktree' | 'command' | 'unknown';

export type GitRemote = {
  name: string;
  url: string;
  direction: RemoteDirection;
};

export type GitConfigSource = {
  scope: GitConfigScope;
  origin: string;
  conditionalInclude: boolean;
};

export type GitConfigValue = {
  value: string | null;
  source: GitConfigSource | null;
};

export type GitIdentity = {
  name: GitConfigValue;
  email: GitConfigValue;
};

export type RepositoryInspection = {
  path: string;
  remotes: GitRemote[];
  identity: GitIdentity;
};

export type RegisteredRepository = {
  id: string;
  path: string;
  addedAt: string;
  profileId: string | null;
  state: RepositoryState;
  inspection: RepositoryInspection | null;
  message: string | null;
};

export type RepositoryIdentityKey = 'userName' | 'userEmail';

export type RepositoryIdentityChange = {
  key: RepositoryIdentityKey;
  current: GitConfigValue;
  desired: string;
};

export type RepositoryProfilePreview = {
  repositoryId: string;
  path: string;
  profile: GitProfile;
  changes: RepositoryIdentityChange[];
};

export type RepositoryAssignmentStatus =
  | 'unassigned'
  | 'correct'
  | 'gitIdentityMismatch'
  | 'githubAccountMismatch'
  | 'bothMismatch'
  | 'unknown';

export type GithubAccountMatch = 'match' | 'mismatch' | 'notApplicable' | 'unknown';

export type RepositoryAssignmentEvaluation = {
  status: RepositoryAssignmentStatus;
  profile: GitProfile | null;
  gitIdentityMatches: boolean | null;
  githubAccountMatch: GithubAccountMatch;
  activeGithubAccount: GithubAccount | null;
};

export const repositoryAssignmentStatusLabels = {
  unassigned: 'Unassigned',
  correct: 'Correct',
  gitIdentityMismatch: 'Git identity mismatch',
  githubAccountMismatch: 'GitHub account mismatch',
  bothMismatch: 'Both mismatch',
  unknown: 'Unknown',
} as const satisfies Record<RepositoryAssignmentStatus, string>;

const scopeLabels: Record<GitConfigScope, string> = {
  system: 'System',
  global: 'Global',
  local: 'Local',
  worktree: 'Worktree',
  command: 'Command',
  unknown: 'Unknown',
};

export function repositoryName(path: string): string {
  const segments = path.split(/[\\/]/).filter(Boolean);
  return segments.at(-1) ?? path;
}

export function configSourceLabel(source: GitConfigSource | null): string {
  if (!source) {
    return 'Not configured';
  }
  const scope = scopeLabels[source.scope];
  return source.conditionalInclude ? `Conditional include · ${scope}` : scope;
}

export function configOriginLabel(source: GitConfigSource | null): string | null {
  return source?.origin.replace(/^file:/, '') ?? null;
}

export function originRemotes(remotes: GitRemote[]): GitRemote[] {
  return remotes.filter((remote) => remote.name === 'origin');
}

export function repositoryIdentityKeyLabel(key: RepositoryIdentityKey): string {
  return key === 'userName' ? 'user.name' : 'user.email';
}

export function evaluateRepositoryAssignment(
  repository: RegisteredRepository,
  profiles: readonly GitProfile[],
  githubStatus: GithubAccountsStatus | null,
): RepositoryAssignmentEvaluation {
  if (!repository.profileId) {
    return {
      status: 'unassigned',
      profile: null,
      gitIdentityMatches: null,
      githubAccountMatch: 'notApplicable',
      activeGithubAccount: null,
    };
  }

  const profile = profiles.find((candidate) => candidate.id === repository.profileId) ?? null;
  if (!profile || repository.state !== 'available' || !repository.inspection) {
    return {
      status: 'unknown',
      profile,
      gitIdentityMatches: null,
      githubAccountMatch: profile?.githubAccount ? 'unknown' : 'notApplicable',
      activeGithubAccount: null,
    };
  }

  const gitIdentityMatches =
    repository.inspection.identity.name.value === profile.gitName &&
    repository.inspection.identity.email.value === profile.gitEmail;
  const expectedAccount = profile.githubAccount;
  if (!expectedAccount) {
    return {
      status: gitIdentityMatches ? 'correct' : 'gitIdentityMismatch',
      profile,
      gitIdentityMatches,
      githubAccountMatch: 'notApplicable',
      activeGithubAccount: null,
    };
  }

  if (!githubStatus) {
    return {
      status: 'unknown',
      profile,
      gitIdentityMatches,
      githubAccountMatch: 'unknown',
      activeGithubAccount: null,
    };
  }

  const activeGithubAccount =
    githubStatus.accounts.find(
      (account) =>
        account.active &&
        account.state === 'success' &&
        account.hostname.toLowerCase() === expectedAccount.hostname.toLowerCase(),
    ) ?? null;
  const githubMatches =
    activeGithubAccount?.username.toLowerCase() === expectedAccount.username.toLowerCase();

  return {
    status: gitIdentityMatches
      ? githubMatches
        ? 'correct'
        : 'githubAccountMismatch'
      : githubMatches
        ? 'gitIdentityMismatch'
        : 'bothMismatch',
    profile,
    gitIdentityMatches,
    githubAccountMatch: githubMatches ? 'match' : 'mismatch',
    activeGithubAccount,
  };
}
