import { describe, expect, it } from 'vitest';
import type { MachineView } from '../engine/api';
import { describeMachine, groupBoxes } from './layout';

const M: MachineView = {
  kind: 'nfa',
  states: [
    { name: 'q1', start: true, accept: false, trap: false, origin: null },
    { name: 'q"2', start: false, accept: true, trap: false, origin: null },
  ],
  sigma: ['a'],
  edges: [
    { from: 0, to: 0, label: 'a' },
    { from: 0, to: 1, label: 'a,ε' },
  ],
  table: { columns: [], rows: [] },
  groups: [],
};

describe('groupBoxes', () => {
  it('nests sub-expression boxes: an outer group is padded more than the one inside it', () => {
    const nodes = [0, 1, 2, 3].map((i) => ({ x: 100 * i, y: 50, rx: 20, ry: 20 }));
    const [outer, inner] = groupBoxes(
      {
        ...M,
        groups: [
          { label: '(0∪1)*', states: [0, 1, 2], parent: null },
          { label: '0∪1', states: [1, 2], parent: 0 },
        ],
      },
      nodes,
    );
    expect(inner?.label).toBe('0∪1');
    expect(outer && inner && outer.x < inner.x && outer.y < inner.y).toBe(true);
    expect(outer && inner && outer.x + outer.width > inner.x + inner.width).toBe(true);
    // q3 belongs to no group, so no box reaches it.
    expect(outer && outer.x + outer.width < 300 - 20).toBe(true);
  });
});

describe('describeMachine', () => {
  it('reads the diagram out as sentences', () => {
    expect(describeMachine('N', M)).toBe(
      'NFA N, 2 states, start q1, accept q"2; q1 on a goes to q1; q1 on a,ε goes to q"2',
    );
  });
});
