import { describe, expect, it } from 'vitest';

import type { EnvironmentStatus } from './environment';
import { environmentSummary } from './environment';

const readyStatus: EnvironmentStatus = {
  git: {
    dependency: 'git',
    state: 'available',
    version: '2.47.0',
    message: 'Git is available.',
  },
  githubCli: {
    dependency: 'githubCli',
    state: 'available',
    version: '2.73.0',
    message: 'GitHub CLI is available.',
  },
  isReady: true,
};

describe('environment summary', () => {
  it('reports a ready environment', () => {
    expect(environmentSummary(readyStatus)).toBe('Your environment is ready.');
  });

  it('reports a missing dependency', () => {
    const status: EnvironmentStatus = {
      ...readyStatus,
      githubCli: {
        dependency: 'githubCli',
        state: 'missing',
        version: null,
        message: 'GitHub CLI is not installed.',
      },
      isReady: false,
    };

    expect(environmentSummary(status)).toBe('One required dependency is missing.');
  });

  it('distinguishes check errors from missing tools', () => {
    const status: EnvironmentStatus = {
      ...readyStatus,
      git: {
        dependency: 'git',
        state: 'error',
        version: null,
        message: 'Git could not be checked.',
      },
      isReady: false,
    };

    expect(environmentSummary(status)).toBe('One or more dependencies could not be checked.');
  });
});
