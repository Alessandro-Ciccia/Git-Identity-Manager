import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { createRawSnippet } from 'svelte';
import { describe, expect, it, vi } from 'vitest';

import Modal from './Modal.svelte';

const header = createRawSnippet(() => ({
  render: () => `<h2 id="modal-title">Confirm change</h2>`,
}));

const bodyWithControls = createRawSnippet(() => ({
  render: () =>
    `<div><button data-autofocus>First</button><input aria-label="middle" /><button>Last</button></div>`,
}));

function renderModal(overrides: Record<string, unknown> = {}) {
  return render(Modal, {
    titleId: 'modal-title',
    onClose: vi.fn(),
    header,
    children: bodyWithControls,
    ...overrides,
  });
}

describe('Modal', () => {
  it('is a labelled modal dialog', () => {
    renderModal();
    const dialog = screen.getByRole('dialog', { name: 'Confirm change' });
    expect(dialog).toHaveAttribute('aria-modal', 'true');
  });

  it('moves focus to the autofocus target and locks body scroll', () => {
    renderModal();
    expect(screen.getByRole('button', { name: 'First' })).toHaveFocus();
    expect(document.body.style.overflow).toBe('hidden');
  });

  it('closes on Escape when dismissible', async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    renderModal({ onClose });

    await user.keyboard('{Escape}');
    expect(onClose).toHaveBeenCalledOnce();
  });

  it('ignores Escape and backdrop clicks when not dismissible', async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    const { container } = renderModal({ onClose, dismissible: false });

    await user.keyboard('{Escape}');
    await user.click(container.querySelector('[role="presentation"]') as HTMLElement);
    expect(onClose).not.toHaveBeenCalled();
  });

  it('traps Tab focus within the dialog', async () => {
    const user = userEvent.setup();
    renderModal();

    // Focus starts on the autofocus button ("First"); Shift+Tab should wrap to "Last".
    await user.tab({ shift: true });
    expect(screen.getByRole('button', { name: 'Last' })).toHaveFocus();

    await user.tab();
    expect(screen.getByRole('button', { name: 'First' })).toHaveFocus();
  });

  it('restores focus to the opener on unmount', () => {
    const opener = document.createElement('button');
    opener.textContent = 'Open';
    document.body.appendChild(opener);
    opener.focus();

    const { unmount } = renderModal();
    expect(screen.getByRole('button', { name: 'First' })).toHaveFocus();

    unmount();
    expect(opener).toHaveFocus();
    opener.remove();
  });
});
