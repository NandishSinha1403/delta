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
