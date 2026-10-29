import { create } from 'zustand';
import type { Document, ObjectView, RunView } from '../engine/api';
import { engine } from '../engine/client';

export type EngineStatus = 'loading' | 'ready' | 'error';

interface EngineStore {
  status: EngineStatus;
  /** Why the engine failed (status 'error'). */
  error: string | null;
  /** The last evaluated program, and the source it came from (diagnostic offsets refer to it). */
  doc: Document | null;
  docSource: string;
  version: string | null;
  evaluate: (source: string) => Promise<void>;

  /** The Run panel: which machine, what input, the result and the current position in it. */
  runObject: string | null;
  runInput: string;
  run: RunView | null;
  runError: string | null;
  runPos: number;
  setRunInput: (input: string) => void;
  setRunPos: (pos: number) => void;
  startRun: (source: string, object: string, input: string) => Promise<void>;
  showRun: (run: RunView) => void;
  clearRun: () => void;
}

function message(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

// Typing fires many evaluations; only the newest answer may be shown.
let latest = 0;
let latestRun = 0;

export const useEngineStore = create<EngineStore>()((set, get) => ({
  status: 'loading',
  error: null,
  doc: null,
  docSource: '',
  version: null,
  evaluate: async (source) => {
    const ticket = ++latest;
    try {
      const doc = await engine.evaluate(source);
      if (ticket !== latest) return;
      set({ status: 'ready', error: null, doc, docSource: source });
      if (get().version === null) set({ version: await engine.version() });
    } catch (err) {
      if (ticket !== latest) return;
      set({ status: 'error', error: message(err) });
    }
  },

  runObject: null,
  runInput: '',
  run: null,
  runError: null,
  runPos: 0,
  setRunInput: (runInput) => set({ runInput }),
  setRunPos: (runPos) => set({ runPos }),
  startRun: async (source, object, input) => {
    const ticket = ++latestRun;
    set({ runObject: object, runInput: input });
    try {
      const result = await engine.run(source, object, input);
      if (ticket !== latestRun) return;
      set({ run: result.run, runError: result.error, runPos: 0 });
    } catch (err) {
      if (ticket !== latestRun) return;
      set({ run: null, runError: message(err), runPos: 0 });
    }
  },
  showRun: (run) =>
    set({ runObject: run.object, runInput: run.input.join(''), run, runError: null, runPos: 0 }),
  clearRun: () => set({ run: null, runError: null, runPos: 0 }),
}));

/** Objects that can be drawn or listed, in program order. */
export function useObjects(): ObjectView[] {
  return useEngineStore((s) => s.doc?.objects) ?? EMPTY;
}

const EMPTY: ObjectView[] = [];

/** The object the views show: the user's choice if it still exists, else the first one. */
export function pickObject(objects: readonly ObjectView[], selected: string | null): ObjectView | null {
  return objects.find((o) => o.name === selected) ?? objects[0] ?? null;
}
