// The conversions and closure operations the engine supports today, with their book references.
// The command palette, the Conversions drawer and the diagram's Convert buttons all read this list,
// so none of them can offer something that does not exist.
import type { ObjectView } from './engine/api';

export interface Conversion {
  fn: 'dfa' | 'nfa' | 'gnfa' | 'regex' | 'union' | 'intersect' | 'concat' | 'star';
  /** Short label for a button, e.g. "→ DFA". */
  label: string;
  /** Book reference, e.g. "Theorem 1.39". */
  book: string;
  /** One line for the palette and drawer. */
  doc: string;
  /** Object kinds each argument may be. */
  takes: readonly string[];
  arity: 1 | 2;
  /** Letter for the new object's name (D, N, G, R, U, I, C, S). */
  prefix: string;
}

export const CONVERSIONS: readonly Conversion[] = [
  { fn: 'dfa', label: '→ DFA', book: 'Theorem 1.39', doc: 'Subset construction (reachable subsets)', takes: ['nfa', 'regex'], arity: 1, prefix: 'D' },
  { fn: 'nfa', label: '→ NFA', book: 'Lemma 1.55', doc: 'Regular expression to NFA, or a DFA viewed as an NFA', takes: ['regex', 'dfa'], arity: 1, prefix: 'N' },
  { fn: 'gnfa', label: '→ GNFA', book: 'Lemma 1.60', doc: 'Generalized NFA in special form', takes: ['dfa', 'nfa'], arity: 1, prefix: 'G' },
  { fn: 'regex', label: '→ regex', book: 'Lemma 1.60', doc: 'State elimination, CONVERT(G)', takes: ['dfa', 'nfa', 'gnfa'], arity: 1, prefix: 'R' },
  { fn: 'union', label: 'union', book: 'Theorem 1.25 / 1.45', doc: 'Product construction on DFAs; NFA union otherwise', takes: ['dfa', 'nfa'], arity: 2, prefix: 'U' },
  { fn: 'intersect', label: 'intersect', book: 'Theorem 1.25', doc: 'Product construction with F = F₁ × F₂ (footnote)', takes: ['dfa'], arity: 2, prefix: 'I' },
  { fn: 'concat', label: 'concat', book: 'Theorem 1.47', doc: 'Concatenation of NFAs', takes: ['dfa', 'nfa'], arity: 2, prefix: 'C' },
  { fn: 'star', label: 'star', book: 'Theorem 1.49', doc: 'Star with a new accepting start state', takes: ['dfa', 'nfa'], arity: 1, prefix: 'S' },
];

/** The Convert buttons above the diagram: only these three, and only where they apply. */
export const DIAGRAM_CONVERSIONS: readonly Conversion['fn'][] = ['dfa', 'gnfa', 'regex'];

export function conversion(fn: Conversion['fn']): Conversion {
  // The list is fixed, so the lookup always succeeds.
  return CONVERSIONS.find((c) => c.fn === fn) ?? (CONVERSIONS[0] as Conversion);
}

/** Every way to apply `c` to the program's objects: [N], [D] … or [D1, D2] … for two arguments. */
export function applications(c: Conversion, objects: readonly ObjectView[]): string[][] {
  const fit = objects.filter((o) => c.takes.includes(o.kind)).map((o) => o.name);
  if (c.arity === 1) return fit.map((n) => [n]);
  const out: string[][] = [];
  for (let i = 0; i < fit.length; i++) {
    for (let j = i + 1; j < fit.length; j++) out.push([fit[i] ?? '', fit[j] ?? '']);
  }
  return out;
}

/** A name for the new object that the program does not use yet: D, D2, D3, … */
export function freshName(prefix: string, taken: readonly string[]): string {
  if (!taken.includes(prefix)) return prefix;
  let i = 2;
  while (taken.includes(`${prefix}${i}`)) i++;
  return `${prefix}${i}`;
}

/** Appends `let NAME = fn(args)` to the program; returns the new text and the new name. */
export function addLet(
  source: string,
  c: Conversion,
  args: readonly string[],
  taken: readonly string[],
): { source: string; name: string } {
  const name = freshName(c.prefix, taken);
  const line = `let ${name} = ${c.fn}(${args.join(', ')})`;
  const sep = source === '' || source.endsWith('\n') ? '' : '\n';
  return { source: `${source}${sep}${line}\n`, name };
}
