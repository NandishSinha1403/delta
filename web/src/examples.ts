// The Pad's samples (03-PRODUCTS.md §3.1): `samples/*.delta` at the repository root. A Rust test runs
// every one of them, so the gallery can never show a program that fails.
const files = import.meta.glob<string>('../../samples/*.delta', {
  query: '?raw',
  import: 'default',
  eager: true,
});

export interface Example {
  id: string;
  /** e.g. "Subset construction". */
  title: string;
  /** The rest of the first comment line. */
  summary: string;
  source: string;
}

/** Splits "// Title — summary" into a title and a summary. */
export function describe(id: string, source: string): Example {
  const first = (source.split('\n')[0] ?? '').replace(/^\/\/\s*/, '');
  const [title, ...rest] = first.split(/\s+—\s+/);
  return { id, title: title ?? id, summary: rest.join(' — '), source };
}

// File names start with a number, so sorting by id gives the gallery order.
export const EXAMPLES: readonly Example[] = Object.entries(files)
  .map(([path, source]) => describe(path.replace(/^.*\/|\.delta$/g, ''), source))
  .sort((a, b) => a.id.localeCompare(b.id));

export const DEFAULT_SOURCE: string = EXAMPLES[0]?.source ?? '';
