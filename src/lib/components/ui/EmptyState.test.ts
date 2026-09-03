import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';

import EmptyState from './EmptyState.svelte';

describe('EmptyState', () => {
  it('renders a title and description', () => {
    render(EmptyState, {
      title: 'No directory rules configured',
      description: 'Add a rule to resolve a profile for repositories below a directory.',
    });
    expect(screen.getByText('No directory rules configured')).toBeInTheDocument();
    expect(screen.getByText(/resolve a profile for repositories/)).toBeInTheDocument();
  });

  it('renders an optional action', () => {
    render(EmptyState, {
      title: 'No profiles yet',
      description: 'Create your first reusable identity.',
      action: createRawSnippet(() => ({ render: () => `<button>New profile</button>` })),
    });
    expect(screen.getByRole('button', { name: 'New profile' })).toBeInTheDocument();
  });
});
