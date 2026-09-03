import {
  isThemePreference,
  resolveTheme,
  type ResolvedTheme,
  type ThemePreference,
} from '$lib/domain/preferences';

export const THEME_HINT_STORAGE_KEY = 'gim.theme-hint';
const DARK_QUERY = '(prefers-color-scheme: dark)';

function prefersDark(): boolean {
  return typeof globalThis.matchMedia === 'function' && globalThis.matchMedia(DARK_QUERY).matches;
}

/**
 * Reads the paint-time theme hint written by {@link applyThemeHint}. Only an explicit
 * light/dark choice is stored; `system` is represented by the absence of a hint so the
 * CSS `prefers-color-scheme` media query can drive it with no JavaScript.
 */
export function readThemeHint(): 'light' | 'dark' | null {
  try {
    const value = globalThis.localStorage?.getItem(THEME_HINT_STORAGE_KEY);
    return value === 'light' || value === 'dark' ? value : null;
  } catch {
    return null;
  }
}

/** Applies the stored hint (or clears the override) to the document element. */
export function applyThemeHint(): void {
  if (typeof document === 'undefined') {
    return;
  }
  const hint = readThemeHint();
  if (hint) {
    document.documentElement.dataset.theme = hint;
  } else {
    delete document.documentElement.dataset.theme;
  }
}

function persistHint(preference: ThemePreference): void {
  try {
    const storage = globalThis.localStorage;
    if (!storage) {
      return;
    }
    if (preference === 'system') {
      storage.removeItem(THEME_HINT_STORAGE_KEY);
    } else {
      storage.setItem(THEME_HINT_STORAGE_KEY, preference);
    }
  } catch {
    // A private-mode or blocked storage failure only loses the no-flash hint.
  }
}

function apply(preference: ThemePreference): void {
  if (typeof document === 'undefined') {
    return;
  }
  const root = document.documentElement;
  if (preference === 'system') {
    // Let the CSS prefers-color-scheme media query drive it — no attribute, so the OS
    // theme is followed live with no JavaScript.
    delete root.dataset.theme;
  } else {
    root.dataset.theme = preference;
  }
}

class ThemeController {
  #preference = $state<ThemePreference>('system');
  #systemDark = $state(prefersDark());
  #media: MediaQueryList | null = null;
  #listener: ((event: MediaQueryListEvent) => void) | null = null;

  get preference(): ThemePreference {
    return this.#preference;
  }

  get resolved(): ResolvedTheme {
    return resolveTheme(this.#preference, this.#systemDark);
  }

  /** Called once the backend preference has loaded. Safe to call again on refresh. */
  init(preference: ThemePreference): void {
    this.#preference = isThemePreference(preference) ? preference : 'system';
    this.#startWatching();
    this.#systemDark = prefersDark();
    persistHint(this.#preference);
    apply(this.#preference);
  }

  setPreference(preference: ThemePreference): void {
    this.#preference = preference;
    persistHint(preference);
    apply(preference);
  }

  #startWatching(): void {
    if (this.#media || typeof globalThis.matchMedia !== 'function') {
      return;
    }
    this.#media = globalThis.matchMedia(DARK_QUERY);
    this.#listener = (event) => {
      this.#systemDark = event.matches;
      // Only the CSS media query needs to react; keep `resolved` accurate for display.
      apply(this.#preference);
    };
    this.#media.addEventListener('change', this.#listener);
  }

  /** Test helper: detach the media listener. */
  dispose(): void {
    if (this.#media && this.#listener) {
      this.#media.removeEventListener('change', this.#listener);
    }
    this.#media = null;
    this.#listener = null;
  }
}

export const theme = new ThemeController();
