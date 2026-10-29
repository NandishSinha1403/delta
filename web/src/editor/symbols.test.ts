import { EditorSelection, EditorState } from '@codemirror/state';
import { describe, expect, it } from 'vitest';
import { BOOK_SYMBOLS, insertText } from './symbols';

describe('symbol bar', () => {
  it('offers the eight book symbols in order', () => {
    expect(BOOK_SYMBOLS.map((s) => s.symbol).join(' ')).toBe('ε ∅ ∪ * → ⊔ Σ δ');
  });

  it('inserts at the cursor and moves the cursor past the symbol', () => {
    const state = EditorState.create({ doc: 'q1 - q2', selection: { anchor: 4 } });
    const next = state.update(insertText(state, '→')).state;
    expect(next.doc.toString()).toBe('q1 -→ q2');
    expect(next.selection.main.head).toBe(5);
  });

  it('replaces a selection', () => {
    const state = EditorState.create({
      doc: 'a + b',
      selection: EditorSelection.single(2, 3),
      extensions: EditorState.allowMultipleSelections.of(true),
    });
    expect(state.update(insertText(state, '∪')).state.doc.toString()).toBe('a ∪ b');
  });
});
