import { describe, expect, it } from 'vitest';
import type { MachineView, ObjectView } from '../engine/api';
import { runOverlay, traceFrame } from './overlay';
import { formatSet, texName, texSet, tupleOf } from './tuple-format';

// Sipser, Figure 1.6 / Example 1.7.
const M1: MachineView = {
  kind: 'dfa',
  states: [
    { name: 'q1', start: true, accept: false, trap: false, origin: null },
    { name: 'q2', start: false, accept: true, trap: false, origin: null },
    { name: 'q3', start: false, accept: false, trap: false, origin: null },
  ],
  sigma: ['0', '1'],
  edges: [
    { from: 0, to: 0, label: '0' },
    { from: 0, to: 1, label: '1' },
    { from: 1, to: 2, label: '0' },
    { from: 1, to: 1, label: '1' },
    { from: 2, to: 1, label: '0,1' },
  ],
  table: { columns: ['0', '1'], rows: [] },
};

describe('tuple formatting', () => {
  it('writes the tuple like Example 1.7', () => {
    const t = tupleOf('M1', M1);
    expect(t.header.text).toBe('M1 = (Q, Σ, δ, q1, F)');
    expect(t.header.tex).toBe('M_{1} = (Q, \\Sigma, \\delta, q_{1}, F)');
    expect(t.states.text).toBe('Q = {q1, q2, q3}');
    expect(t.accept.text).toBe('F = {q2}');
    expect(t.definition).toBe('Definition 1.5');
  });

  it('typesets names the way the book does', () => {
    expect(texName('q12')).toBe('q_{12}');
    expect(texName('{q1,q3}')).toBe('\\{q_{1}, q_{3}\\}');
    expect(texName('{}')).toBe('\\text{\\{\\}}');
    expect(texName('qstart')).toBe('q_{\\text{start}}');
    expect(texName('odd_state')).toBe('\\text{odd\\_state}');
    expect(texSet([])).toBe('\\emptyset');
    expect(formatSet([])).toBe('∅');
  });
});

describe('diagram frames', () => {
  const D: ObjectView = {
    name: 'D',
    kind: 'dfa',
    from: 0,
    to: 0,
    derived: null,
    regex: null,
    sigma: ['0', '1'],
    machine: M1,
    trace: {
      title: 'test',
      truncated: false,
      steps: [
        { rule: 'r', message: 'a', add_states: [0], add_edges: [], focus: [0], start: null, accept: null, snapshot: null },
        { rule: 'r', message: 'b', add_states: [1], add_edges: [1], focus: [0, 1], start: null, accept: [1], snapshot: null },
      ],
    },
  };

  it('replays a prefix of the steps on the final machine', () => {
    const f = traceFrame(D, 0);
    expect([...(f?.overlay.hiddenStates ?? [])]).toEqual([1, 2]);
    expect(f?.overlay.hiddenEdges?.size).toBe(5);
    const g = traceFrame(D, 1);
    expect([...(g?.overlay.hiddenStates ?? [])]).toEqual([2]);
    expect(g?.overlay.hiddenEdges?.has(1)).toBe(false);
    expect(g?.overlay.accept).toEqual([1]);
    expect(traceFrame(D, 2)).toBeNull();
  });

  it('colors the run by its verdict at the end only', () => {
    const run = {
      object: 'M1',
      kind: 'dfa',
      input: ['1'],
      accepted: true,
      steps: [
        { pos: 0, active: [0], message: '' },
        { pos: 1, active: [1], message: '' },
      ],
    };
    expect(runOverlay(run, 0)).toEqual({ active: [0], verdict: null });
    expect(runOverlay(run, 1)).toEqual({ active: [1], verdict: 'accept' });
  });
});
