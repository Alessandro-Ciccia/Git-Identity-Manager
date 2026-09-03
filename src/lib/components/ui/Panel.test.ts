import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';

import Panel from './Panel.svelte';

const body = createRawSnippet(() => ({ render: () => `<p>panel body</p>` }));

describe('Panel', () => {
  it('labels the region by its heading and reflects busy state', () => {
    const { container } = render(Panel, {
      eyebrow: 'GitHub CLI',
      heading: 'GitHub accounts',
      headingId: 'github-accounts-heading',
      description: 'Credentials stay managed by GitHub CLI.',
      busy: true,
      children: body,
    });

    const region = container.querySelector('section');
    expect(region).toHaveAttribute('aria-labelledby', 'github-accounts-heading');
    expect(region).toHaveAttribute('aria-busy', 'true');
    expect(screen.getByRole('heading', { name: 'GitHub accounts' })).toHaveAttribute(
      'id',
      'github-accounts-heading',
    );
    expect(screen.getByText('panel body')).toBeInTheDocument();
  });

  it('renders header actions', () => {
    render(Panel, {
      eyebrow: 'Local identities',
      heading: 'Git profiles',
      headingId: 'profiles-heading',
      children: body,
      actions: createRawSnippet(() => ({ render: () => `<button>New profile</button>` })),
    });
    expect(screen.getByRole('button', { name: 'New profile' })).toBeInTheDocument();
  });
});
