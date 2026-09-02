import { invoke } from '@tauri-apps/api/core';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { environmentErrorMessage, getEnvironmentStatus } from './environment';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);

describe('environment IPC', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('invokes only the fixed environment status command', async () => {
    invokeMock.mockResolvedValue({ isReady: false });

    await getEnvironmentStatus();

    expect(invokeMock).toHaveBeenCalledWith('get_environment_status');
  });

  it('uses a structured backend message when present', () => {
    expect(
      environmentErrorMessage({
        code: 'environmentCheckFailed',
        message: 'The environment check could not be completed.',
      }),
    ).toBe('The environment check could not be completed.');
  });

  it('does not expose unknown rejection values', () => {
    expect(environmentErrorMessage('raw process output')).toBe(
      'The environment check could not be completed. Please try again.',
    );
  });
});
