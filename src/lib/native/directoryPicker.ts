import { open } from '@tauri-apps/plugin-dialog';

export async function selectRepositoryDirectory(): Promise<string | null> {
  const selected = await open({
    directory: true,
    multiple: false,
    title: 'Select a Git repository',
  });
  return selected;
}
