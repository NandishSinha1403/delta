import { useSyncExternalStore } from 'react';

export type Layout = 'desktop' | 'tablet' | 'phone';

// Breakpoints from docs/plan/05-PLAYGROUND-UX.md §2 (same as Tailwind's `xl` and `md`).
const DESKTOP = '(min-width: 1280px)';
const TABLET = '(min-width: 768px)';

function getLayout(): Layout {
  if (window.matchMedia(DESKTOP).matches) return 'desktop';
  if (window.matchMedia(TABLET).matches) return 'tablet';
  return 'phone';
}

function subscribe(onChange: () => void): () => void {
  const queries = [window.matchMedia(DESKTOP), window.matchMedia(TABLET)];
  for (const q of queries) q.addEventListener('change', onChange);
  return () => {
    for (const q of queries) q.removeEventListener('change', onChange);
  };
}

/** Which shell to render. Each layout is designed for its screen, not shrunk from desktop. */
export function useLayout(): Layout {
  return useSyncExternalStore(subscribe, getLayout);
}

// The tallest viewport seen so far; the on-screen keyboard shrinks the viewport well below it.
let tallest = 0;

function viewportHeight(): number {
  const h = window.visualViewport?.height ?? window.innerHeight;
  tallest = Math.max(tallest, h);
  return h;
}

function subscribeViewport(onChange: () => void): () => void {
  const vv = window.visualViewport;
  vv?.addEventListener('resize', onChange);
  window.addEventListener('resize', onChange);
  return () => {
    vv?.removeEventListener('resize', onChange);
    window.removeEventListener('resize', onChange);
  };
}

/** True while a phone's on-screen keyboard is open (the viewport lost over a quarter of its height). */
export function useKeyboardOpen(): boolean {
  return useSyncExternalStore(subscribeViewport, () => viewportHeight() < tallest * 0.75);
}
