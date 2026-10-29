// What the diagram shows for a trace step or a run position: pure functions over engine views.
import type { MachineView, ObjectView, RunView } from '../engine/api';
import type { Overlay } from '../diagram/Diagram';

export interface Frame {
  machine: MachineView;
  overlay: Overlay;
}

/**
 * Rebuilds the half-finished machine after trace step `step` (docs/plan/02-CONVERSIONS.md §T).
 * Steps that add states/arrows are replayed on the final machine (unbuilt parts drawn faint);
 * algorithms that remove things (state elimination) carry snapshots instead.
 */
export function traceFrame(object: ObjectView, step: number): Frame | null {
  const steps = object.trace?.steps ?? [];
  const current = steps[step];
  if (!current) return null;

  for (let i = step; i >= 0; i--) {
    const snapshot = steps[i]?.snapshot;
    if (snapshot) return { machine: snapshot, overlay: i === step ? {} : { focus: [] } };
  }

  const m = object.machine;
  if (!m) return null;
  const states = new Set<number>();
  const edges = new Set<number>();
  let start: number | null = null;
  let accept: number[] | null = null;
  for (const s of steps.slice(0, step + 1)) {
    for (const q of s.add_states) states.add(q);
    for (const e of s.add_edges) edges.add(e);
    if (s.start !== null) start = s.start;
    if (s.accept !== null) accept = s.accept;
  }
  const hiddenStates = new Set(m.states.map((_, i) => i).filter((i) => !states.has(i)));
  const hiddenEdges = new Set(m.edges.map((_, i) => i).filter((i) => !edges.has(i)));
  return {
    machine: m,
    overlay: { hiddenStates, hiddenEdges, focus: current.focus, start, accept },
  };
}

/**
 * The run's active states at position `pos`, colored by the verdict once the input is read, and
 * the arrows just taken: from a state active before the last symbol to one active after it, with
 * that symbol in the arrow's label.
 */
export function runOverlay(run: RunView, pos: number, machine?: MachineView): Overlay {
  const at = run.steps[Math.min(pos, run.steps.length - 1)];
  const done = pos >= run.input.length;
  const overlay: Overlay = {
    active: at?.active ?? [],
    verdict: done ? (run.accepted ? 'accept' : 'reject') : null,
  };
  const before = run.steps[pos - 1];
  const symbol = run.input[pos - 1];
  if (machine && before && at && symbol !== undefined) {
    const from = new Set(before.active);
    const to = new Set(at.active);
    overlay.taken = new Set(
      machine.edges.flatMap((e, i) =>
        from.has(e.from) && to.has(e.to) && e.label.split(',').map((l) => l.trim()).includes(symbol) ? [i] : [],
      ),
    );
  }
  return overlay;
}
