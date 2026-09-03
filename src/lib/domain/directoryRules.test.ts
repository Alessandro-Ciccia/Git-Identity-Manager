import { describe, expect, it } from 'vitest';

import {
  directoryRuleOperationLabel,
  directoryRuleProfile,
  directoryRuleStateLabels,
  type DirectoryRule,
} from './directoryRules';
import type { GitProfile } from './profiles';

const profile: GitProfile = {
  id: 'profile-1',
  label: 'Work',
  gitName: 'Work Octo',
  gitEmail: 'work@example.com',
  githubAccount: null,
};

const rule: DirectoryRule = {
  id: 'rule-1',
  directory: '/work',
  profileId: profile.id,
  state: 'active',
  message: null,
};

describe('directory rules domain', () => {
  it('resolves profiles without hiding stale references', () => {
    expect(directoryRuleProfile(rule, [profile])).toBe(profile);
    expect(directoryRuleProfile(rule, [])).toBeNull();
  });

  it('provides stable operation and state labels', () => {
    expect(directoryRuleOperationLabel('add')).toBe('Add directory rule');
    expect(directoryRuleOperationLabel('update')).toBe('Update directory rule');
    expect(directoryRuleOperationLabel('remove')).toBe('Remove directory rule');
    expect(directoryRuleStateLabels.needsApply).toBe('Needs reapply');
  });
});
