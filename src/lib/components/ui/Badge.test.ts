import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';

import Badge from './Badge.svelte';

const label = (text: string) => createRawSnippet(() => ({ render: () => `<span>${text}</span>` }));

describe('Badge', () => {
  it('renders its content', () => {
    render(Badge, { children: label('Active'), tone: 'accent' });
    expect(screen.getByText('Active')).toBeInTheDocument();
  });

  it('applies uppercase styling only when requested', () => {
    const { container } = render(Badge, { children: label('Correct'), uppercase: true });
    expect(container.querySelector('span.uppercase')).not.toBeNull();
  });
});
