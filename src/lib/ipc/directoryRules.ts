import { invoke } from '@tauri-apps/api/core';

import type {
  DirectoryRule,
  DirectoryRuleInput,
  DirectoryRulePreview,
} from '$lib/domain/directoryRules';

const LIST_DIRECTORY_RULES_COMMAND = 'list_directory_rules';
const PREVIEW_DIRECTORY_RULE_COMMAND = 'preview_directory_rule';
const APPLY_DIRECTORY_RULE_COMMAND = 'apply_directory_rule';
const PREVIEW_REMOVE_DIRECTORY_RULE_COMMAND = 'preview_remove_directory_rule';
const REMOVE_DIRECTORY_RULE_COMMAND = 'remove_directory_rule';

export function listDirectoryRules(): Promise<DirectoryRule[]> {
  return invoke<DirectoryRule[]>(LIST_DIRECTORY_RULES_COMMAND);
}

export function previewDirectoryRule(input: DirectoryRuleInput): Promise<DirectoryRulePreview> {
  return invoke<DirectoryRulePreview>(PREVIEW_DIRECTORY_RULE_COMMAND, { input });
}

export function applyDirectoryRule(input: DirectoryRuleInput): Promise<DirectoryRule> {
  return invoke<DirectoryRule>(APPLY_DIRECTORY_RULE_COMMAND, { input });
}

export function previewRemoveDirectoryRule(id: string): Promise<DirectoryRulePreview> {
  return invoke<DirectoryRulePreview>(PREVIEW_REMOVE_DIRECTORY_RULE_COMMAND, { id });
}

export function removeDirectoryRule(id: string): Promise<void> {
  return invoke<void>(REMOVE_DIRECTORY_RULE_COMMAND, { id });
}

export function directoryRuleErrorMessage(error: unknown): string {
  if (
    typeof error === 'object' &&
    error !== null &&
    'message' in error &&
    typeof error.message === 'string'
  ) {
    return error.message;
  }
  return 'The directory rule operation could not be completed. Please try again.';
}
