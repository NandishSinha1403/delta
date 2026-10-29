import { describe, expect, it } from 'vitest';
import type { DfaTuple } from '../engine/api';
import { deltaTable, formatSet, tupleHeader } from './tuple-format';

// Sipser, Figure 1.6 / Example 1.7 (the same machine delta-core::hello returns).
const M1: DfaTuple = {
  name: 'M1',
  states: ['q1', 'q2', 'q3'],
  sigma: ['0', '1'],
  delta: [
    { from: 'q1', symbol: '0', to: 'q1' },
    { from: 'q1', symbol: '1', to: 'q2' },
    { from: 'q2', symbol: '0', to: 'q3' },
    { from: 'q2', symbol: '1', to: 'q2' },
    { from: 'q3', symbol: '0', to: 'q2' },
    { from: 'q3', symbol: '1', to: 'q2' },
  ],
  start: 'q1',
  accept: ['q2'],
};

describe('tuple formatting', () => {
  it('writes sets and the header like the book', () => {
    expect(formatSet(M1.states)).toBe('{q1, q2, q3}');
    expect(formatSet([])).toBe('{}');
    expect(tupleHeader(M1)).toBe('M1 = (Q, Σ, δ, q1, F)');
  });

  it('lays δ out as the table in Example 1.7', () => {
    expect(deltaTable(M1)).toEqual({
      symbols: ['0', '1'],
      rows: [
        { state: 'q1', cells: ['q1', 'q2'] },
        { state: 'q2', cells: ['q3', 'q2'] },
        { state: 'q3', cells: ['q2', 'q2'] },
      ],
    });
  });

  it('marks a missing transition with ?', () => {
    const partial = { ...M1, delta: M1.delta.slice(1) };
    expect(deltaTable(partial).rows[0]?.cells).toEqual(['?', 'q2']);
  });
});
