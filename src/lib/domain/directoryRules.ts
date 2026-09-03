import type { GitProfile } from './profiles';

export type DirectoryRuleState =
  'active' | 'needsApply' | 'missingDirectory' | 'missingProfile' | 'unavailable';

export type DirectoryRule = {
  id: string;
  directory: string;
  profileId: string;
  state: DirectoryRuleState;
  message: string | null;
};

export type DirectoryRuleInput = {
  id: string | null;
  directory: string;
  profileId: string;
};

export type DirectoryRuleOperation = 'add' | 'update' | 'remove';
export type DirectoryRuleConflictKind =
  'overlappingRule' | 'existingConditionalInclude' | 'repositoryOverride';

export type DirectoryRuleConflict = {
  kind: DirectoryRuleConflictKind;
  message: string;
  blocking: boolean;
};

export type DirectoryRulePreview = {
  ruleId: string | null;
  directory: string;
  profile: GitProfile | null;
  operation: DirectoryRuleOperation;
  condition: string;
  identityFilePath: string;
  globalConfigPath: string;
  backupRequired: boolean;
  conflicts: DirectoryRuleConflict[];
  canApply: boolean;
};

export const directoryRuleStateLabels = {
  active: 'Active',
  needsApply: 'Needs reapply',
  missingDirectory: 'Missing directory',
  missingProfile: 'Missing profile',
  unavailable: 'Unavailable',
} as const satisfies Record<DirectoryRuleState, string>;

export function directoryRuleProfile(
  rule: DirectoryRule,
  profiles: readonly GitProfile[],
): GitProfile | null {
  return profiles.find((profile) => profile.id === rule.profileId) ?? null;
}

export function directoryRuleOperationLabel(operation: DirectoryRuleOperation): string {
  if (operation === 'remove') return 'Remove directory rule';
  if (operation === 'update') return 'Update directory rule';
  return 'Add directory rule';
}
