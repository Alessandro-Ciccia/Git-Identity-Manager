import { invoke } from '@tauri-apps/api/core';

import type { EnvironmentStatus } from '$lib/domain/environment';

const GET_ENVIRONMENT_STATUS_COMMAND = 'get_environment_status';

export function getEnvironmentStatus(): Promise<EnvironmentStatus> {
  return invoke<EnvironmentStatus>(GET_ENVIRONMENT_STATUS_COMMAND);
}

export function environmentErrorMessage(error: unknown): string {
  if (
    typeof error === 'object' &&
    error !== null &&
    'message' in error &&
    typeof error.message === 'string'
  ) {
    return error.message;
  }

  return 'The environment check could not be completed. Please try again.';
}
