import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import type { DirectoryRulePreview } from '$lib/domain/directoryRules';
import type { GitProfile } from '$lib/domain/profiles';
import DirectoryRulesPanel from './DirectoryRulesPanel.svelte';

const profile: GitProfile = {
  id: 'profile-id',
  label: 'Work',
  gitName: 'Work Octo',
  gitEmail: 'work@example.com',
  githubAccount: null,
};

const baseProps = {
  rules: [
    {
      id: 'rule-id',
      directory: '/work',
      profileId: profile.id,
      state: 'active' as const,
      message: null,
    },
  ],
  profiles: [profile],
  loading: false,
  error: null,
  success: null,
  pending: false,
  preview: null,
  onRefresh: vi.fn(),
  onChooseDirectory: vi.fn(async () => '/clients'),
  onPreview: vi.fn(),
  onPreviewRemove: vi.fn(),
  onCancelPreview: vi.fn(),
  onApply: vi.fn(),
};

describe('DirectoryRulesPanel', () => {
  it('lists rules and requests a structured preview after directory selection', async () => {
    const onPreview = vi.fn();
    render(DirectoryRulesPanel, { props: { ...baseProps, onPreview } });

    expect(screen.getByText('/work/**')).toBeInTheDocument();
    expect(screen.getByText('Work Octo · work@example.com')).toBeInTheDocument();
    await fireEvent.click(screen.getByRole('button', { name: 'Choose directory…' }));
    await fireEvent.change(screen.getByLabelText('Profile for directory rule'), {
      target: { value: profile.id },
    });
    await fireEvent.click(screen.getByRole('button', { name: 'Preview' }));

    expect(onPreview).toHaveBeenCalledWith({
      id: null,
      directory: '/clients',
      profileId: profile.id,
    });
  });

  it('shows exact paths, backups, conflicts, and blocks unsafe apply', () => {
    const preview: DirectoryRulePreview = {
      ruleId: null,
      directory: '/work',
      profile,
      operation: 'add',
      condition: 'gitdir:/work/',
      identityFilePath: '/app/identities/profile-id.gitconfig',
      globalConfigPath: '/home/octo/.gitconfig',
      backupRequired: true,
      conflicts: [
        {
          kind: 'existingConditionalInclude',
          message: 'An existing include may apply.',
          blocking: true,
        },
      ],
      canApply: false,
    };
    render(DirectoryRulesPanel, { props: { ...baseProps, preview } });

    expect(screen.getByRole('dialog', { name: 'Add directory rule' })).toBeInTheDocument();
    expect(screen.getByText('gitdir:/work/')).toBeInTheDocument();
    expect(screen.getByText('/home/octo/.gitconfig')).toBeInTheDocument();
    expect(screen.getByText(/backed up before modification/)).toBeInTheDocument();
    expect(screen.getByText('An existing include may apply.')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Apply rule' })).toBeDisabled();
  });

  it('requires a preview before removal', async () => {
    const onPreviewRemove = vi.fn();
    render(DirectoryRulesPanel, { props: { ...baseProps, onPreviewRemove } });
    await fireEvent.click(screen.getByRole('button', { name: 'Remove' }));
    expect(onPreviewRemove).toHaveBeenCalledWith('rule-id');
  });
});
