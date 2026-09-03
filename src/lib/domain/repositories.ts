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
  state: RepositoryState;
  inspection: RepositoryInspection | null;
  message: string | null;
};

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
