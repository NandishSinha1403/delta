import { describe, expect, it } from 'vitest';
import type { MachineView } from '../engine/api';
import { describeMachine, parseLayout, parseSpline, toDot } from './layout';

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

describe('toDot', () => {
  it('draws left to right with a start arrow, double circles and quoted labels', () => {
    const dot = toDot(M);
    expect(dot).toContain('rankdir=LR');
    expect(dot).toContain('__start -> s0;');
    expect(dot).toContain('s1 [label="q\\"2" shape=doublecircle];');
    expect(dot).toContain('s0 -> s1 [label="a,ε"];');
  });
});

describe('clusters', () => {
  it('nests sub-expression groups and puts each state in its innermost box', () => {
    const states = Array.from({ length: 4 }, (_, i) => ({ name: `q${i}`, start: i === 0, accept: false, trap: false, origin: null }));
    const dot = toDot({
      ...M,
      states,
      edges: [],
      groups: [
        { label: '(0∪1)*', states: [0, 1, 2], parent: null },
        { label: '0∪1', states: [1, 2], parent: 0 },
      ],
    });
    const outer = dot.indexOf('subgraph cluster_0');
    const inner = dot.indexOf('subgraph cluster_1');
    expect(outer).toBeGreaterThan(-1);
    expect(inner).toBeGreaterThan(outer);
    // q0 sits in the outer box only, q1 in the inner one, q3 outside both.
    expect(dot.indexOf('s0 [label="q0"]')).toBeLessThan(inner);
    expect(dot.indexOf('s1 [label="q1"]')).toBeGreaterThan(inner);
    expect(dot.match(/s1 \[label/g)).toHaveLength(1);
    // q3 belongs to no group, so it is emitted after the outer cluster closes.
    expect(dot.slice(dot.lastIndexOf('\n  }\n'))).toContain('s3 [label="q3"]');
  });
});

describe('parseLayout', () => {
  // Trimmed Graphviz `json` output for M (bounding box 100 × 50 points).
  const json = JSON.stringify({
    bb: '0,0,100,50',
    objects: [
      { _gvid: 0, name: '__start', pos: '1,25', width: '0.01' },
      { _gvid: 1, name: 's0', pos: '30,25', width: '0.5' },
      { _gvid: 2, name: 's1', pos: '80,25', width: '0.6' },
    ],
    edges: [
      { tail: 0, head: 1, pos: 'e,12,25 1,25 4,25 7,25 10,25' },
      { tail: 1, head: 1, pos: 'e,33,40 25,40 25,48 35,48 33,42', lp: '30,52' },
      { tail: 1, head: 2, pos: 'e,58,25 48,25 51,25 54,25 56,25', lp: '53,30' },
    ],
  });

  it('flips y, pads, and matches edges back to the machine', () => {
    const l = parseLayout(json, M);
    expect(l.width).toBe(124);
    expect(l.height).toBe(74);
    expect(l.nodes[0]).toEqual({ x: 42, y: 37, r: 18 });
    expect(l.nodes[1]?.r).toBeCloseTo(21.6);
    expect(l.start?.tip).toEqual({ x: 24, y: 37 });
    expect(l.edges[0]?.label).toEqual({ x: 42, y: 10 });
    expect(l.edges[1]?.path).toBe('M60,37 C63,37 66,37 68,37');
    expect(l.clusters).toEqual([]);
  });

  it('reads a spline without an arrowhead marker', () => {
    const s = parseSpline('0,0 1,1 2,2 3,3', (p) => p);
    expect(s?.tip).toEqual({ x: 3, y: 3 });
    expect(s?.base).toEqual({ x: 2, y: 2 });
  });
});

describe('describeMachine', () => {
  it('reads the diagram out as sentences', () => {
    expect(describeMachine('N', M)).toBe(
      'NFA N, 2 states, start q1, accept q"2; q1 on a goes to q1; q1 on a,ε goes to q"2',
    );
  });
});
