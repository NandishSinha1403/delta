import { create } from 'zustand';
import { StoppedError, type Document, type ObjectView, type RunView } from '../engine/api';
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
  /** An evaluation is in flight. */
  evaluating: boolean;
  /** …and it has taken over SLOW_MS, so the loader shows (never for fast runs). */
  slow: boolean;
  /** How long the last finished evaluation took, for "Executed in N ms". */
  lastMs: number | null;
  /** The last evaluation was cancelled with Stop. */
  stopped: boolean;
  /** Stop: kills and restarts the worker; the previous results stay on screen. */
  stop: () => void;

  /** The Run panel: which machine, what input, the result and the current position in it. */
  runObject: string | null;
  runInput: string;
  run: RunView | null;
  runError: string | null;
  runPos: number;
  /** A run was requested and its result has not arrived yet (stepping now would step the old one). */
  runPending: boolean;
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
/** Evaluations faster than this never show a loader. */
export const SLOW_MS = 300;
let latestRun = 0;

export const useEngineStore = create<EngineStore>()((set, get) => ({
  status: 'loading',
  error: null,
  doc: null,
  docSource: '',
  version: null,
  evaluate: async (source) => {
    const ticket = ++latest;
    const started = performance.now();
    set({ evaluating: true, stopped: false });
    const timer = window.setTimeout(() => {
      if (ticket === latest && get().evaluating) set({ slow: true });
    }, SLOW_MS);
    try {
      const doc = await engine.evaluate(source);
      if (ticket !== latest) return;
      const lastMs = Math.round(performance.now() - started);
      set({ status: 'ready', error: null, doc, docSource: source, evaluating: false, slow: false, lastMs });
      if (get().version === null) set({ version: await engine.version() });
    } catch (err) {
      if (ticket !== latest) return;
      if (err instanceof StoppedError) set({ evaluating: false, slow: false, stopped: true });
      else set({ status: 'error', error: message(err), evaluating: false, slow: false });
    } finally {
      window.clearTimeout(timer);
    }
  },
  evaluating: false,
  slow: false,
  lastMs: null,
  stopped: false,
  stop: () => {
    engine.restart();
    // The in-flight evaluation rejects with StoppedError and clears the flags; a run in flight
    // (if any) is abandoned too.
    latestRun++;
    set({ runPending: false });
  },

  runObject: null,
  runInput: '',
  run: null,
  runError: null,
  runPos: 0,
  runPending: false,
  setRunInput: (runInput) => set({ runInput }),
  setRunPos: (runPos) => set({ runPos }),
  startRun: async (source, object, input) => {
    const ticket = ++latestRun;
    set({ runObject: object, runInput: input, runPending: true });
    try {
      const result = await engine.run(source, object, input);
      if (ticket !== latestRun) return;
      set({ run: result.run, runError: result.error, runPos: 0, runPending: false });
    } catch (err) {
      if (ticket !== latestRun) return;
      set({ run: null, runError: message(err), runPos: 0, runPending: false });
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
