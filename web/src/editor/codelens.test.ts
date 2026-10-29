import { describe, expect, it } from 'vitest';
import type { Document } from '../engine/api';
import { lensesFor } from './codelens';

describe('codelens hints', () => {
  it('summarises conversions and checks like the redesign', () => {
    const doc = {
      objects: [
        { name: 'N', from: 0, derived: null, machine: null, trace: null },
        {
          name: 'D',
          from: 50,
          derived: 'dfa(N) · Theorem 1.39',
          machine: { states: [{}, {}, {}, {}] },
          trace: { title: 'Theorem 1.39 · subset construction', steps: [{}], truncated: false },
        },
      ],
      results: [
        { kind: 'check', from: 70, cases: [{ input: 'ab', expected: true, actual: true }, { input: 'b', expected: true, actual: true }, { input: 'aab', expected: true, actual: true }] },
        { kind: 'check', from: 90, cases: [{ input: '', expected: false, actual: true }] },
        { kind: 'run', from: 99, cases: [] },
      ],
    } as unknown as Document;
    expect(lensesFor(doc)).toEqual([
      { at: 50, text: 'Theorem 1.39 · subset construction · 4 states', tone: 'info', steps: 'D' },
      { at: 70, text: '✓ 3 / 3 accepted', tone: 'pass' },
      { at: 90, text: '✗ 1 of 1 failed: ε', tone: 'fail' },
    ]);
  });
});
