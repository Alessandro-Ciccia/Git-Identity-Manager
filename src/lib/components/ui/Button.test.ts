import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { createRawSnippet } from 'svelte';
import { describe, expect, it, vi } from 'vitest';

import Button from './Button.svelte';

const label = (text: string) => createRawSnippet(() => ({ render: () => `<span>${text}</span>` }));

describe('Button', () => {
  it('renders its children and forwards clicks', async () => {
    const user = userEvent.setup();
    const onclick = vi.fn();
    render(Button, { children: label('Refresh'), onclick });

    await user.click(screen.getByRole('button', { name: 'Refresh' }));
    expect(onclick).toHaveBeenCalledOnce();
  });

  it('shows the pending label and disables while pending', () => {
    render(Button, { children: label('Refresh'), pending: true, pendingLabel: 'Refreshing…' });

    const button = screen.getByRole('button', { name: 'Refreshing…' });
    expect(button).toBeDisabled();
  });

  it('defaults to type="button" and forwards aria-label', () => {
    render(Button, { children: label('x'), 'aria-label': 'Edit Personal' });
    const button = screen.getByRole('button', { name: 'Edit Personal' });
    expect(button).toHaveAttribute('type', 'button');
  });
});
