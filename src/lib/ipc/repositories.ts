import { invoke } from '@tauri-apps/api/core';

import type { RegisteredRepository, RepositoryProfilePreview } from '$lib/domain/repositories';

const LIST_REPOSITORIES_COMMAND = 'list_repositories';
const REGISTER_REPOSITORY_COMMAND = 'register_repository';
const REFRESH_REPOSITORY_COMMAND = 'refresh_repository';
const ASSIGN_REPOSITORY_PROFILE_COMMAND = 'assign_repository_profile';
const REMOVE_REPOSITORY_PROFILE_COMMAND = 'remove_repository_profile';
const PREVIEW_REPOSITORY_PROFILE_COMMAND = 'preview_repository_profile';
const APPLY_REPOSITORY_PROFILE_COMMAND = 'apply_repository_profile';
const REMOVE_REPOSITORY_COMMAND = 'remove_repository';
const REVEAL_REPOSITORY_COMMAND = 'reveal_repository';

export function listRepositories(): Promise<RegisteredRepository[]> {
  return invoke<RegisteredRepository[]>(LIST_REPOSITORIES_COMMAND);
}

export function registerRepository(path: string): Promise<RegisteredRepository> {
  return invoke<RegisteredRepository>(REGISTER_REPOSITORY_COMMAND, { path });
}

export function refreshRepository(id: string): Promise<RegisteredRepository> {
  return invoke<RegisteredRepository>(REFRESH_REPOSITORY_COMMAND, { id });
}

export function assignRepositoryProfile(
  id: string,
  profileId: string,
): Promise<RegisteredRepository> {
  return invoke<RegisteredRepository>(ASSIGN_REPOSITORY_PROFILE_COMMAND, { id, profileId });
}

export function removeRepositoryProfile(id: string): Promise<RegisteredRepository> {
  return invoke<RegisteredRepository>(REMOVE_REPOSITORY_PROFILE_COMMAND, { id });
}

export function previewRepositoryProfile(id: string): Promise<RepositoryProfilePreview> {
  return invoke<RepositoryProfilePreview>(PREVIEW_REPOSITORY_PROFILE_COMMAND, { id });
}

export function applyRepositoryProfile(id: string): Promise<RegisteredRepository> {
  return invoke<RegisteredRepository>(APPLY_REPOSITORY_PROFILE_COMMAND, { id });
}

export function removeRepository(id: string): Promise<void> {
  return invoke<void>(REMOVE_REPOSITORY_COMMAND, { id });
}

export function revealRepository(id: string): Promise<void> {
  return invoke<void>(REVEAL_REPOSITORY_COMMAND, { id });
}

export function repositoryErrorMessage(error: unknown): string {
  if (
    typeof error === 'object' &&
    error !== null &&
    'message' in error &&
    typeof error.message === 'string'
  ) {
    return error.message;
  }
  return 'The repository operation could not be completed. Please try again.';
}
