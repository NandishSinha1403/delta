import type { DfaTuple } from '../engine/api';

/** A set in the book's notation: {q1, q2, q3}. */
export function formatSet(items: readonly string[]): string {
  return `{${items.join(', ')}}`;
}

/** The tuple's header line, e.g. "M1 = (Q, Σ, δ, q1, F)" (Sipser, Example 1.7). */
export function tupleHeader(m: DfaTuple): string {
  return `${m.name} = (Q, Σ, δ, ${m.start}, F)`;
}

export interface DeltaTable {
  symbols: string[];
  rows: { state: string; cells: string[] }[];
}

/** δ as the book draws it: one row per state, one column per symbol. Missing entries are "?". */
export function deltaTable(m: DfaTuple): DeltaTable {
  const lookup = new Map(m.delta.map((t) => [`${t.from}\u0000${t.symbol}`, t.to]));
  return {
    symbols: [...m.sigma],
    rows: m.states.map((state) => ({
      state,
      cells: m.sigma.map((a) => lookup.get(`${state}\u0000${a}`) ?? '?'),
    })),
  };
}
