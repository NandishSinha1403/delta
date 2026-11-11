// delta-render's layout (the engine's `diagram_layout`, the same one every download uses) →
// geometry the Pad draws itself, plus the regex → NFA group boxes and the screen-reader text.
import type { MachineView } from '../engine/api';

export interface Point {
  x: number;
  y: number;
}

/** A sub-expression's box (regex → NFA): its rectangle and label position. */
export interface ClusterBox {
  x: number;
  y: number;
  width: number;
  height: number;
  label: string;
  labelAt: Point;
}

/** A placed state: a circle (rx = ry) or an ellipse for a long name. */
export interface RenderNode {
  x: number;
  y: number;
  rx: number;
  ry: number;
}

/** A placed arrow: its curve (ending where the head starts), the head's tip and the point it
 * points away from, and the label's box (centre, width, height). */
export interface RenderEdge {
  path: string;
  tip: Point;
  base: Point;
  label?: { x: number; y: number; w: number; h: number };
}

export interface RenderLayout {
  width: number;
  height: number;
  nodes: RenderNode[];
  /** Same order as `machine.edges`. */
  edges: RenderEdge[];
  start: RenderEdge | null;
  clusters: ClusterBox[];
  /** The top-left corner of the frame (negative when a group box reaches past the drawing). */
  origin: Point;
}

/** Room around a group's states, plus more for each group nested inside it. */
const GROUP_PAD = 10;
const GROUP_STEP = 8;
const GROUP_LABEL = 16;

/** The sub-expression boxes (regex → NFA): each group's states' bounding box, padded so nested
 * groups sit inside their parents, with room for the label on top. */
export function groupBoxes(m: MachineView, nodes: RenderNode[]): ClusterBox[] {
  const groups = m.groups ?? [];
  const depth = (gi: number): number => {
    const kids = groups.flatMap((g, ci) => (g.parent === gi ? [ci] : []));
    return kids.length === 0 ? 0 : 1 + Math.max(...kids.map(depth));
  };
  return groups.flatMap((g, gi) => {
    const boxes = g.states.flatMap((q) => (nodes[q] ? [nodes[q]] : []));
    if (boxes.length === 0) return [];
    const pad = GROUP_PAD + GROUP_STEP * depth(gi);
    const x1 = Math.min(...boxes.map((n) => n.x - n.rx)) - pad;
    const x2 = Math.max(...boxes.map((n) => n.x + n.rx)) + pad;
    const y1 = Math.min(...boxes.map((n) => n.y - n.ry)) - pad - GROUP_LABEL;
    const y2 = Math.max(...boxes.map((n) => n.y + n.ry)) + pad;
    return [{ x: x1, y: y1, width: x2 - x1, height: y2 - y1, label: g.label, labelAt: { x: x1 + 10, y: y1 + 10 } }];
  });
}

/** The engine's layout JSON with the group boxes added, framed so everything is inside. */
export function parseRenderLayout(json: string, m: MachineView): RenderLayout {
  const raw = JSON.parse(json) as Omit<RenderLayout, 'clusters' | 'origin'>;
  const clusters = groupBoxes(m, raw.nodes);
  // Group boxes can reach past the drawing's edge; grow the frame to include them.
  const minX = Math.min(0, ...clusters.map((c) => c.x));
  const minY = Math.min(0, ...clusters.map((c) => c.y));
  const maxX = Math.max(raw.width, ...clusters.map((c) => c.x + c.width));
  const maxY = Math.max(raw.height, ...clusters.map((c) => c.y + c.height));
  return { ...raw, clusters, width: maxX - minX + 8, height: maxY - minY + 8, origin: { x: minX - 4, y: minY - 4 } };
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
