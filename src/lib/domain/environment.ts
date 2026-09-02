export type Dependency = 'git' | 'githubCli';
export type DependencyState = 'available' | 'missing' | 'error';

export type DependencyStatus = {
  dependency: Dependency;
  state: DependencyState;
  version: string | null;
  message: string;
};

export type EnvironmentStatus = {
  git: DependencyStatus;
  githubCli: DependencyStatus;
  isReady: boolean;
};

export const dependencyLabels = {
  git: 'Git',
  githubCli: 'GitHub CLI',
} as const satisfies Record<Dependency, string>;

export function environmentSummary(status: EnvironmentStatus): string {
  if (status.isReady) {
    return 'Your environment is ready.';
  }

  const missingCount = [status.git, status.githubCli].filter(
    (dependency) => dependency.state === 'missing',
  ).length;

  if (missingCount > 0) {
    return missingCount === 1
      ? 'One required dependency is missing.'
      : 'Required dependencies are missing.';
  }

  return 'One or more dependencies could not be checked.';
}
