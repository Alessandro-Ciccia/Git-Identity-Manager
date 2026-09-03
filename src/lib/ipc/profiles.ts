import { invoke } from '@tauri-apps/api/core';

import type { GitProfile, ProfileInput } from '$lib/domain/profiles';

const LIST_PROFILES_COMMAND = 'list_profiles';
const CREATE_PROFILE_COMMAND = 'create_profile';
const UPDATE_PROFILE_COMMAND = 'update_profile';
const DELETE_PROFILE_COMMAND = 'delete_profile';

export function listProfiles(): Promise<GitProfile[]> {
  return invoke<GitProfile[]>(LIST_PROFILES_COMMAND);
}

export function createProfile(profile: ProfileInput): Promise<GitProfile> {
  return invoke<GitProfile>(CREATE_PROFILE_COMMAND, { profile });
}

export function updateProfile(id: string, profile: ProfileInput): Promise<GitProfile> {
  return invoke<GitProfile>(UPDATE_PROFILE_COMMAND, { id, profile });
}

export function deleteProfile(id: string): Promise<void> {
  return invoke<void>(DELETE_PROFILE_COMMAND, { id });
}

export function profileErrorMessage(error: unknown): string {
  if (
    typeof error === 'object' &&
    error !== null &&
    'message' in error &&
    typeof error.message === 'string'
  ) {
    return error.message;
  }

  return 'The profile operation could not be completed. Please try again.';
}
