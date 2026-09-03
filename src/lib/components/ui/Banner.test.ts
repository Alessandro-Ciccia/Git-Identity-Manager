import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';

import Banner from './Banner.svelte';

const content = (text: string) =>
  createRawSnippet(() => ({ render: () => `<span>${text}</span>` }));

describe('Banner', () => {
  it('exposes danger banners as alerts', () => {
    render(Banner, { tone: 'danger', children: content('Something failed.') });
    expect(screen.getByRole('alert')).toHaveTextContent('Something failed.');
  });

  it('exposes positive banners as status', () => {
    render(Banner, { tone: 'positive', children: content('Verified.') });
    expect(screen.getByRole('status')).toHaveTextContent('Verified.');
  });

  it('renders a trailing action', () => {
    render(Banner, {
      tone: 'danger',
      children: content('Update required.'),
      action: createRawSnippet(() => ({ render: () => `<button>Update</button>` })),
    });
    expect(screen.getByRole('button', { name: 'Update' })).toBeInTheDocument();
  });
});
