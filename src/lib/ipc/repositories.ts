import { invoke } from '@tauri-apps/api/core';

import type { RegisteredRepository } from '$lib/domain/repositories';

const LIST_REPOSITORIES_COMMAND = 'list_repositories';
const REGISTER_REPOSITORY_COMMAND = 'register_repository';
const REFRESH_REPOSITORY_COMMAND = 'refresh_repository';
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
