import { invoke } from '@tauri-apps/api/core';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { ProfileInput } from '$lib/domain/profiles';

import {
  createProfile,
  deleteProfile,
  listProfiles,
  profileErrorMessage,
  updateProfile,
} from './profiles';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);
const profile: ProfileInput = {
  label: 'Personal',
  gitName: 'Octo Cat',
  gitEmail: 'octo@example.com',
  githubAccount: { hostname: 'github.com', username: 'octocat' },
};

describe('profiles IPC', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
  });

  it('lists profiles through a fixed command without arguments', async () => {
    await listProfiles();
    expect(invokeMock).toHaveBeenCalledWith('list_profiles');
  });

  it('creates a profile with only the typed profile payload', async () => {
    await createProfile(profile);
    expect(invokeMock).toHaveBeenCalledWith('create_profile', { profile });
  });

  it('updates a selected profile with its id and typed payload', async () => {
    await updateProfile('profile-id', profile);
    expect(invokeMock).toHaveBeenCalledWith('update_profile', { id: 'profile-id', profile });
  });

  it('deletes only the selected profile id', async () => {
    await deleteProfile('profile-id');
    expect(invokeMock).toHaveBeenCalledWith('delete_profile', { id: 'profile-id' });
  });

  it('uses structured backend messages and hides unknown rejection values', () => {
    expect(
      profileErrorMessage({
        code: 'invalidGitEmail',
        message: 'Enter a valid Git email address.',
      }),
    ).toBe('Enter a valid Git email address.');
    expect(profileErrorMessage('raw storage error')).toBe(
      'The profile operation could not be completed. Please try again.',
    );
  });
});
