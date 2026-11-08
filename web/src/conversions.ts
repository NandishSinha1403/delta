// Library conversions the Pad offers as buttons (01-LANGUAGE.md §12), with their algorithm ids.
// Explain profiles, not the UI, add textbook citations.
// The command palette, the Conversions drawer and the diagram's Convert buttons all read this list,
// so none of them can offer something that does not exist.
import type { ObjectView } from './engine/api';

export interface Conversion {
  fn: 'determinize' | 'minimize' | 'to_nfa' | 'to_gnfa' | 'to_regex' | 'union' | 'intersect' | 'concat' | 'star';
  /** Short label for a button, e.g. "→ DFA". */
  label: string;
  /** The algorithm id, e.g. "determinize/subset". */
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
  { fn: 'determinize', label: '→ DFA', book: 'determinize/subset', doc: 'Subset construction (reachable subsets)', takes: ['nfa', 'regex'], arity: 1, prefix: 'D' },
  { fn: 'minimize', label: 'minimize', book: 'minimize/hopcroft', doc: 'Merge equivalent states', takes: ['dfa'], arity: 1, prefix: 'M' },
  { fn: 'to_nfa', label: '→ NFA', book: 'to_nfa/textbook', doc: 'Regular expression to NFA', takes: ['regex'], arity: 1, prefix: 'N' },
  { fn: 'to_gnfa', label: '→ GNFA', book: 'to_gnfa/special-form', doc: 'Generalized NFA in special form', takes: ['dfa', 'nfa'], arity: 1, prefix: 'G' },
  { fn: 'to_regex', label: '→ regex', book: 'to_regex/state-elimination', doc: 'State elimination', takes: ['dfa', 'nfa'], arity: 1, prefix: 'R' },
  { fn: 'union', label: 'union', book: 'product/pairs', doc: 'Product construction, accepting if either accepts', takes: ['dfa', 'nfa'], arity: 2, prefix: 'U' },
  { fn: 'intersect', label: 'intersect', book: 'product/pairs', doc: 'Product construction, accepting if both accept', takes: ['dfa', 'nfa'], arity: 2, prefix: 'I' },
  { fn: 'concat', label: 'concat', book: 'concat/epsilon', doc: 'Concatenation with ε-edges', takes: ['dfa', 'nfa'], arity: 2, prefix: 'C' },
  { fn: 'star', label: 'star', book: 'star/epsilon', doc: 'Star with a new accepting start state', takes: ['dfa', 'nfa'], arity: 1, prefix: 'S' },
];

/** The Convert buttons above the diagram: only these three, and only where they apply. */
export const DIAGRAM_CONVERSIONS: readonly Conversion['fn'][] = ['determinize', 'minimize', 'to_regex'];

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
