// Tauri doesn't have a Node.js server to do proper SSR
// so we use adapter-static with a fallback to index.html to put the site in SPA mode
// See: https://svelte.dev/docs/kit/single-page-apps
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
import { applyThemeHint } from '$lib/state/theme.svelte';

export const ssr = false;

// Apply the persisted light/dark override before the app renders so an explicit theme
// choice does not flash on cold start. "system" writes no hint and is handled by the
// CSS prefers-color-scheme media query with no JavaScript.
applyThemeHint();
