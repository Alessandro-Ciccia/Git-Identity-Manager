import { describe, expect, it } from 'vitest';

import {
  configOriginLabel,
  configSourceLabel,
  originRemotes,
  repositoryName,
  type GitRemote,
} from './repositories';

const remotes: GitRemote[] = [
  { name: 'origin', url: 'https://github.com/org/project.git', direction: 'fetch' },
  { name: 'upstream', url: 'https://github.com/base/project.git', direction: 'fetch' },
];

describe('repository domain', () => {
  it('derives names from POSIX and Windows paths', () => {
    expect(repositoryName('/work/project')).toBe('project');
    expect(repositoryName('C:\\work\\project')).toBe('project');
  });

  it('describes configuration scope and conditional includes', () => {
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
  });

  it('selects origin remotes without discarding other remotes', () => {
    expect(originRemotes(remotes)).toEqual([remotes[0]]);
    expect(remotes).toHaveLength(2);
  });
});
