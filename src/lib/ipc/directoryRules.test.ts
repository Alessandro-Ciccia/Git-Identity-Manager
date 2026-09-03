import { invoke } from '@tauri-apps/api/core';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { DirectoryRuleInput } from '$lib/domain/directoryRules';
import {
  applyDirectoryRule,
  directoryRuleErrorMessage,
  listDirectoryRules,
  previewDirectoryRule,
  previewRemoveDirectoryRule,
  removeDirectoryRule,
} from './directoryRules';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
const invokeMock = vi.mocked(invoke);

const input: DirectoryRuleInput = {
  id: null,
  directory: '/work',
  profileId: 'profile-id',
};

describe('directory rules IPC', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
  });

  it('uses only fixed typed commands and structured inputs', async () => {
    await listDirectoryRules();
    await previewDirectoryRule(input);
    await applyDirectoryRule(input);
    await previewRemoveDirectoryRule('rule-id');
    await removeDirectoryRule('rule-id');

    expect(invokeMock.mock.calls).toEqual([
      ['list_directory_rules'],
      ['preview_directory_rule', { input }],
      ['apply_directory_rule', { input }],
      ['preview_remove_directory_rule', { id: 'rule-id' }],
      ['remove_directory_rule', { id: 'rule-id' }],
    ]);
  });

  it('does not expose unknown error details', () => {
    expect(directoryRuleErrorMessage({ message: 'Resolve conflicts.' })).toBe('Resolve conflicts.');
    expect(directoryRuleErrorMessage('raw config contents')).toBe(
      'The directory rule operation could not be completed. Please try again.',
    );
  });
});
