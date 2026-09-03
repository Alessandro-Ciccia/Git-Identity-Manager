import { invoke } from '@tauri-apps/api/core';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  applyRepositoryProfile,
  assignRepositoryProfile,
  listRepositories,
  previewRepositoryProfile,
  refreshRepository,
  registerRepository,
  removeRepository,
  removeRepositoryProfile,
  repositoryErrorMessage,
  revealRepository,
} from './repositories';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
const invokeMock = vi.mocked(invoke);

describe('repositories IPC', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
  });

  it('uses fixed typed repository commands', async () => {
    await listRepositories();
    await registerRepository('/work/project');
    await refreshRepository('repository-id');
    await assignRepositoryProfile('repository-id', 'profile-id');
    await removeRepositoryProfile('repository-id');
    await previewRepositoryProfile('repository-id');
    await applyRepositoryProfile('repository-id');
    await removeRepository('repository-id');
    await revealRepository('repository-id');

    expect(invokeMock.mock.calls).toEqual([
      ['list_repositories'],
      ['register_repository', { path: '/work/project' }],
      ['refresh_repository', { id: 'repository-id' }],
      ['assign_repository_profile', { id: 'repository-id', profileId: 'profile-id' }],
      ['remove_repository_profile', { id: 'repository-id' }],
      ['preview_repository_profile', { id: 'repository-id' }],
      ['apply_repository_profile', { id: 'repository-id' }],
      ['remove_repository', { id: 'repository-id' }],
      ['reveal_repository', { id: 'repository-id' }],
    ]);
  });

  it('uses safe structured errors and hides unknown values', () => {
    expect(repositoryErrorMessage({ message: 'Select a valid repository folder.' })).toBe(
      'Select a valid repository folder.',
    );
    expect(repositoryErrorMessage('raw git output')).toBe(
      'The repository operation could not be completed. Please try again.',
    );
  });
});
