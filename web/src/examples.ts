// The example gallery (docs/plan/05-PLAYGROUND-UX.md §5). The same `examples/*.delta` files are the
// engine's golden-test corpus, so the gallery can never drift from what the tests check.
const files = import.meta.glob<string>('../../examples/*.delta', {
  query: '?raw',
  import: 'default',
  eager: true,
});

export interface Example {
  id: string;
  /** e.g. "Example 1.41". */
  title: string;
  /** The rest of the first comment line. */
  summary: string;
  source: string;
}

// Gallery order: the tour first, then by book order.
const ORDER = [
  'tour',
  'fig1-6-M1',
  'thm1-25-union',
  'thm1-25-intersect',
  'ex1-38-N1',
  'ex1-41-N4',
  'thm1-45-union',
  'thm1-47-concat',
  'thm1-49-star',
  'ex1-56',
  'ex1-58',
  'ex1-66',
];

/** Splits "// Example 1.41 — N4 … " into a title and a summary. */
export function describe(id: string, source: string): Example {
  const first = (source.split('\n')[0] ?? '').replace(/^\/\/\s*/, '');
  const [title, ...rest] = first.split(/\s+—\s+/);
  if (id === 'tour' || rest.length === 0) {
    return { id, title: 'Tour', summary: first.replace(/^A tour of Phase 1:\s*/i, ''), source };
  }
  return { id, title: title ?? id, summary: rest.join(' — '), source };
}

export const EXAMPLES: readonly Example[] = Object.entries(files)
  .map(([path, source]) => describe(path.replace(/^.*\/|\.delta$/g, ''), source))
  .sort((a, b) => rank(a.id) - rank(b.id));

function rank(id: string): number {
  const i = ORDER.indexOf(id);
  return i === -1 ? ORDER.length : i;
}

export const DEFAULT_SOURCE: string = EXAMPLES.find((e) => e.id === 'tour')?.source ?? '';
