import { open } from '@tauri-apps/plugin-dialog';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { selectRepositoryDirectory } from './directoryPicker';

vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }));
const openMock = vi.mocked(open);

describe('repository directory picker', () => {
  beforeEach(() => openMock.mockReset());

  it('selects exactly one directory', async () => {
    openMock.mockResolvedValue('/work/project');
    await expect(selectRepositoryDirectory()).resolves.toBe('/work/project');
    expect(openMock).toHaveBeenCalledWith({
      directory: true,
      multiple: false,
      title: 'Select a Git repository',
    });
  });

  it('returns null when selection is cancelled', async () => {
    openMock.mockResolvedValue(null);
    await expect(selectRepositoryDirectory()).resolves.toBeNull();
  });
});
