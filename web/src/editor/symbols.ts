import type { EditorState, TransactionSpec } from '@codemirror/state';

export interface BookSymbol {
  /** What gets inserted. */
  symbol: string;
  /** The ASCII spelling the DSL also accepts (docs/plan/04-DSL.md §1). */
  ascii: string;
  /** Spoken name for screen readers. */
  name: string;
}

// The phone symbol bar (docs/plan/05-PLAYGROUND-UX.md §2.3): ε ∅ ∪ * → ⊔ Σ δ.
export const BOOK_SYMBOLS: readonly BookSymbol[] = [
  { symbol: 'ε', ascii: 'eps', name: 'epsilon' },
  { symbol: '∅', ascii: 'empty', name: 'empty set' },
  { symbol: '∪', ascii: '|', name: 'union' },
  { symbol: '*', ascii: '*', name: 'star' },
  { symbol: '→', ascii: '->', name: 'arrow' },
  { symbol: '⊔', ascii: '_', name: 'blank' },
  { symbol: 'Σ', ascii: 'Sigma', name: 'Sigma' },
  { symbol: 'δ', ascii: 'delta', name: 'delta' },
];

/** Replaces every selection with `text` and leaves the cursor after it. */
export function insertText(state: EditorState, text: string): TransactionSpec {
  return { ...state.replaceSelection(text), scrollIntoView: true, userEvent: 'input.type' };
}
