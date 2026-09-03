import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { RepositoryProfilePreview } from '$lib/domain/repositories';

import RepositoryProfilePreviewDialog from './RepositoryProfilePreviewDialog.svelte';

const preview: RepositoryProfilePreview = {
  repositoryId: 'dc683096-c95b-4b68-b609-b994f98b2d2c',
  path: '/work/project',
  profile: {
    id: '0db9c44d-e1e1-493b-a14f-0208aa4ef247',
    label: 'Work',
    gitName: 'Work Octo',
    gitEmail: 'octo@company.example',
    githubAccount: null,
  },
  changes: [
    {
      key: 'userName',
      current: {
        value: 'Personal Octo',
        source: {
          scope: 'global',
          origin: 'file:/home/octo/.gitconfig',
          conditionalInclude: false,
        },
      },
      desired: 'Work Octo',
    },
    {
      key: 'userEmail',
      current: { value: null, source: null },
      desired: 'octo@company.example',
    },
  ],
};

describe('RepositoryProfilePreviewDialog', () => {
  it('shows current, desired, scope, and exact keys before applying', () => {
    render(RepositoryProfilePreviewDialog, {
      preview,
      applying: false,
      onCancel: vi.fn(),
      onApply: vi.fn(),
    });

    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Apply Work to project' })).toBeInTheDocument();
    expect(screen.getByText('Current user.name')).toBeInTheDocument();
    expect(screen.getByText('Current user.email')).toBeInTheDocument();
    expect(screen.getByText('Personal Octo')).toBeInTheDocument();
    expect(screen.getByText('Work Octo')).toBeInTheDocument();
    expect(screen.getByText('octo@company.example')).toBeInTheDocument();
    expect(screen.getAllByText('Repository local')).toHaveLength(2);
    expect(
      screen.getByText(/Global configuration and unrelated repository settings/),
    ).toBeInTheDocument();
  });

  it('supports cancel and explicit apply and locks controls while applying', async () => {
    const user = userEvent.setup();
    const onCancel = vi.fn();
    const onApply = vi.fn();
    const { unmount } = render(RepositoryProfilePreviewDialog, {
      preview,
      applying: false,
      onCancel,
      onApply,
    });

    await user.click(screen.getByRole('button', { name: 'Cancel' }));
    await user.click(screen.getByRole('button', { name: 'Apply local identity' }));
    expect(onCancel).toHaveBeenCalledOnce();
    expect(onApply).toHaveBeenCalledOnce();
    unmount();

    render(RepositoryProfilePreviewDialog, {
      preview,
      applying: true,
      onCancel,
      onApply,
    });
    expect(screen.getByRole('button', { name: 'Cancel' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Applying and verifying…' })).toBeDisabled();
  });
});
