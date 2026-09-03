import { render, screen } from '@testing-library/svelte';
import { tick } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import SplashScreen from './SplashScreen.svelte';

describe('SplashScreen', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('shows the branded icon then calls ondone after the duration', async () => {
    const ondone = vi.fn();
    render(SplashScreen, { duration: 2000, ondone });

    const overlay = screen.getByTestId('splash-screen');
    expect(overlay).toBeInTheDocument();
    expect(overlay.querySelector('img')?.getAttribute('src')).toBe('/splash-icon.png');

    await vi.advanceTimersByTimeAsync(1700);
    await tick();
    expect(overlay.classList.contains('splash--leaving')).toBe(true);
    expect(ondone).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(300);
    expect(ondone).toHaveBeenCalledOnce();
  });

  it('clears its timers on unmount', async () => {
    const ondone = vi.fn();
    const { unmount } = render(SplashScreen, { duration: 2000, ondone });
    unmount();
    await vi.advanceTimersByTimeAsync(5000);
    expect(ondone).not.toHaveBeenCalled();
  });
});
