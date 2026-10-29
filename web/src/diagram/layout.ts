// Machine → Graphviz DOT, and Graphviz's `json` output → plain geometry we draw ourselves.
// Graphviz only places things; the drawing follows the book's conventions (docs/plan/05 §7).
import type { MachineView } from '../engine/api';

export interface Point {
  x: number;
  y: number;
}

export interface NodeBox {
  x: number;
  y: number;
  /** Outer radius in px. */
  r: number;
}

export interface EdgeShape {
  /** SVG path data for the curve (without the arrowhead). */
  path: string;
  /** Arrowhead tip and the point it points away from. */
  tip: Point;
  base: Point;
  label: Point | null;
}

export interface Layout {
  width: number;
  height: number;
  nodes: NodeBox[];
  /** Same order as `machine.edges`. */
  edges: (EdgeShape | null)[];
  /** The start arrow, from the invisible start point to the start state. */
  start: EdgeShape | null;
}

/** Escapes a string for a double-quoted DOT label. */
function quote(s: string): string {
  return `"${s.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`;
}

/** Left-to-right, as the book draws machines. Courier gives Graphviz realistic label widths. */
export function toDot(m: MachineView): string {
  const lines = [
    'digraph {',
    '  rankdir=LR; nodesep=0.35; ranksep=0.45;',
    '  node [shape=circle fontname="Courier" fontsize=14 margin=0.04 width=0.5 height=0.5];',
    '  edge [fontname="Courier" fontsize=13 arrowsize=0.8];',
    '  __start [shape=point style=invis width=0 height=0 label=""];',
  ];
  m.states.forEach((s, i) => {
    lines.push(`  s${i} [label=${quote(s.name)}${s.accept ? ' shape=doublecircle' : ''}];`);
  });
  const start = m.states.findIndex((s) => s.start);
  if (start >= 0) lines.push(`  __start -> s${start};`);
  for (const e of m.edges) lines.push(`  s${e.from} -> s${e.to} [label=${quote(e.label)}];`);
  lines.push('}');
  return lines.join('\n');
}

interface GvObject {
  _gvid: number;
  name: string;
  pos?: string;
  width?: string;
  height?: string;
}

interface GvEdge {
  tail: number;
  head: number;
  pos?: string;
  lp?: string;
}

interface GvJson {
  bb: string;
  objects?: GvObject[];
  edges?: GvEdge[];
}

const PT_PER_INCH = 72;
const PAD = 12;

function nums(s: string): number[] {
  return s.split(',').map(Number);
}

/** Parses a Graphviz spline ("e,x,y x1,y1 …") into a path plus arrowhead, flipping y. */
export function parseSpline(pos: string, flip: (p: Point) => Point): EdgeShape | null {
  const parts = pos.trim().split(/\s+/);
  let end: Point | null = null;
  const pts: Point[] = [];
  for (const part of parts) {
    if (part.startsWith('e,')) {
      const [x = 0, y = 0] = nums(part.slice(2));
      end = flip({ x, y });
    } else if (part.startsWith('s,')) {
      continue;
    } else {
      const [x = 0, y = 0] = nums(part);
      pts.push(flip({ x, y }));
    }
  }
  const first = pts[0];
  const last = pts[pts.length - 1];
  if (!first || !last) return null;
  let d = `M${first.x},${first.y}`;
  for (let i = 1; i + 2 < pts.length; i += 3) {
    const [a, b, c] = [pts[i], pts[i + 1], pts[i + 2]];
    if (a && b && c) d += ` C${a.x},${a.y} ${b.x},${b.y} ${c.x},${c.y}`;
  }
  return { path: d, tip: end ?? last, base: end ? last : (pts[pts.length - 2] ?? first), label: null };
}

/** Turns Graphviz `json` output into our geometry. */
export function parseLayout(json: string, m: MachineView): Layout {
  const g = JSON.parse(json) as GvJson;
  const [, , w = 0, h = 0] = nums(g.bb);
  const flip = (p: Point): Point => ({ x: p.x + PAD, y: h - p.y + PAD });

  const nodes: NodeBox[] = m.states.map(() => ({ x: 0, y: 0, r: 18 }));
  // Graphviz numbers objects by _gvid; our node names are s0, s1, … (and __start).
  const idOf = new Map<number, number | 'start'>();
  for (const o of g.objects ?? []) {
    if (o.name === '__start') {
      idOf.set(o._gvid, 'start');
      continue;
    }
    const i = Number(o.name.slice(1));
    idOf.set(o._gvid, i);
    if (!o.pos || !nodes[i]) continue;
    const [x = 0, y = 0] = nums(o.pos);
    nodes[i] = { ...flip({ x, y }), r: (Number(o.width ?? 0.5) * PT_PER_INCH) / 2 };
  }

  const edges: (EdgeShape | null)[] = m.edges.map(() => null);
  let start: EdgeShape | null = null;
  // DOT lists the start arrow first, then the machine's edges in order; match by endpoints
  // in order so parallel lookups stay stable.
  const used = new Set<number>();
  for (const e of g.edges ?? []) {
    if (!e.pos) continue;
    const shape = parseSpline(e.pos, flip);
    if (!shape) continue;
    if (e.lp) {
      const [x = 0, y = 0] = nums(e.lp);
      shape.label = flip({ x, y });
    }
    if (idOf.get(e.tail) === 'start') {
      start = shape;
      continue;
    }
    const from = idOf.get(e.tail);
    const to = idOf.get(e.head);
    const k = m.edges.findIndex((me, j) => !used.has(j) && me.from === from && me.to === to);
    if (k >= 0) {
      used.add(k);
      edges[k] = shape;
    }
  }
  return { width: w + 2 * PAD, height: h + 2 * PAD, nodes, edges, start };
}

/** A sentence a screen reader can use in place of the picture (docs/plan/05 §8). */
export function describeMachine(name: string, m: MachineView): string {
  const kind = m.kind.toUpperCase();
  const start = m.states.find((s) => s.start)?.name ?? 'none';
  const accept = m.states.filter((s) => s.accept).map((s) => s.name);
  const arrows = m.edges.map((e) => {
    const from = m.states[e.from]?.name ?? '?';
    const to = m.states[e.to]?.name ?? '?';
    return `${from} on ${e.label} goes to ${to}`;
  });
  const n = m.states.length;
  return [
    `${kind} ${name}, ${n} state${n === 1 ? '' : 's'}, start ${start}, accept ${accept.length ? accept.join(', ') : 'none'}`,
    ...arrows,
  ].join('; ');
}
