import type { ObjectView, RunView } from '../engine/api';
import { pickObject, useEngineStore, useObjects } from '../state/engine';
import { useUiStore } from '../state/ui';

/** The object the views show (see `pickObject`). */
export function useCurrentObject(): ObjectView | null {
  const objects = useObjects();
  const selected = useUiStore((s) => s.selected);
  return pickObject(objects, selected);
}

/** Opens the Run view in whichever layout is active. */
export function openRunView(): void {
  useUiStore.setState({ sheetTab: 'run', sheetOpen: true, phoneView: 'run' });
}

/** Applies a quick fix's edits to the program text (from the end, so offsets stay valid). */
export function applyEdits(source: string, edits: readonly { from: number; to: number; insert: string }[]): string {
  return [...edits]
    .sort((a, b) => b.from - a.from)
    .reduce((text, e) => text.slice(0, e.from) + e.insert + text.slice(e.to), source);
}

/** The empty string shows as ε, as in the book. */
export function showString(s: string): string {
  return s === '' ? 'ε' : s;
}

/**
 * The run shown for `object`: the last one started from the Run panel, else the program's own
 * `run … on …` command (shown at its start until the user steps it).
 */
export function useActiveRun(object: ObjectView | null): { run: RunView | null; pos: number } {
  const run = useEngineStore((s) => s.run);
  const runPos = useEngineStore((s) => s.runPos);
  const results = useEngineStore((s) => s.doc?.results);
  if (!object) return { run: null, pos: 0 };
  if (run && run.object === object.name) return { run, pos: Math.min(runPos, run.input.length) };
  const fromProgram = results?.find((r) => r.kind === 'run' && r.object === object.name)?.run ?? null;
  return { run: fromProgram, pos: 0 };
}
