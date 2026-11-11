import { describe, expect, it } from 'vitest';
import type { MachineView } from '../engine/api';
import { isPresentShortcut, parseWhat, transitionMarks } from './present';

const key = (k: Partial<Parameters<typeof isPresentShortcut>[0]>) => ({
  code: 'KeyP',
  key: 'p',
  ctrlKey: false,
  metaKey: false,
  altKey: false,
  shiftKey: false,
  ...k,
});

describe('the presentation shortcut', () => {
  it('is ⌘⌥P on a Mac, whatever Option+P types', () => {
    expect(isPresentShortcut(key({ metaKey: true, altKey: true, key: 'π' }), true)).toBe(true);
    expect(isPresentShortcut(key({ ctrlKey: true, altKey: true }), true)).toBe(false);
    // ⌘⇧P and plain ⌘P are someone else's.
    expect(isPresentShortcut(key({ metaKey: true, shiftKey: true }), true)).toBe(false);
    expect(isPresentShortcut(key({ metaKey: true }), true)).toBe(false);
  });

  it('is Ctrl+Alt+P elsewhere, unless AltGr+P types a character', () => {
    expect(isPresentShortcut(key({ ctrlKey: true, altKey: true }), false)).toBe(true);
    expect(isPresentShortcut(key({ ctrlKey: true, altKey: true, key: 'P' }), false)).toBe(true);
    // A layout where AltGr+P is "§": leave it to the editor.
    expect(isPresentShortcut(key({ ctrlKey: true, altKey: true, key: '§' }), false)).toBe(false);
    // Ctrl+Shift+P opens a private window in Firefox: never ours.
    expect(isPresentShortcut(key({ ctrlKey: true, shiftKey: true }), false)).toBe(false);
    expect(isPresentShortcut(key({ ctrlKey: true, altKey: true, code: 'KeyO', key: 'o' }), false)).toBe(false);
  });

  it('reads what to present from a link', () => {
    expect(parseWhat('tuple')).toBe('tuple');
    expect(parseWhat('')).toBeNull();
    expect(parseWhat(null)).toBeNull();
  });
});

describe('transitionMarks', () => {
  const source = '// tour\ndfa EndsAB {\n  start q0\n  accept q2\n  q0 -a-> q1;   q0 -b-> q0\n  q1 -a-> q1;   q1 -b-> q2\n}\n';
  const from = source.indexOf('dfa');
  const to = source.indexOf('}') + 1;
  const machine: MachineView = {
    kind: 'dfa',
    states: ['q0', 'q1', 'q2'].map((name) => ({ name, start: name === 'q0', accept: name === 'q2', trap: false, origin: null })),
    sigma: ['a', 'b'],
    edges: [
      { from: 0, to: 1, label: 'a' },
      { from: 0, to: 0, label: 'b' },
      { from: 1, to: 1, label: 'a' },
      { from: 1, to: 2, label: 'b' },
    ],
    table: { columns: [], rows: [] },
    groups: [],
  };

  it('finds the arrow just taken where it is written', () => {
    const marks = transitionMarks(source, from, to, machine, new Set([3]), 'b');
    expect(marks).toEqual([{ line: 6, start: 16, end: 26 }]);
    const line = source.split('\n')[5] ?? '';
    expect(line.slice(16, 26)).toBe('q1 -b-> q2');
  });

  it('needs the symbol in the written labels, and the right states', () => {
    expect(transitionMarks(source, from, to, machine, new Set([3]), 'a')).toEqual([]);
    expect(transitionMarks(source, from, to, machine, new Set([0]), 'a')).toEqual([{ line: 5, start: 2, end: 12 }]);
    expect(transitionMarks(source, from, to, machine, new Set(), 'a')).toEqual([]);
  });

  it('reads merged labels and does not match a longer state name', () => {
    const src = 'nfa N {\n  q1 -0,1-> q10\n  q1 -0,1-> q1\n}';
    const m: MachineView = { ...machine, states: machine.states.slice(0, 2).map((s, i) => ({ ...s, name: ['q1', 'q10'][i] ?? '' })), edges: [{ from: 0, to: 0, label: '0,1' }] };
    expect(transitionMarks(src, 0, src.length, m, new Set([0]), '1')).toEqual([{ line: 3, start: 2, end: 14 }]);
  });
});
