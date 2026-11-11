// The only engine module React code imports (docs/plan/03-ARCHITECTURE.md §4).
// The engine sits behind this interface so its implementation (Rust/WASM in a worker today)
// can change without touching the UI.
import type { Document as DocumentV1 } from './generated/Document';
import type { ObjectView as ObjectViewV1 } from './generated/ObjectView';
import type { RunResult } from './generated/RunResult';

export type { BookRefView } from './generated/BookRefView';
export type { CheckCaseView } from './generated/CheckCaseView';
export type { CommandResultView } from './generated/CommandResultView';
export type { DiagnosticView } from './generated/DiagnosticView';
export type { EdgeView } from './generated/EdgeView';
export type { FixView } from './generated/FixView';
export type { MachineView } from './generated/MachineView';
export type { OutlineView } from './generated/OutlineView';
export type { RunResult } from './generated/RunResult';
export type { RunStepView } from './generated/RunStepView';
export type { RunView } from './generated/RunView';
export type { StateView } from './generated/StateView';
export type { StepView } from './generated/StepView';
export type { TraceView } from './generated/TraceView';

/** A `check` or `assert` result (02-ENGINE.md §7). */
export interface TestOutput {
  kind: 'test';
  line: number;
  name: string;
  pass: boolean;
  message?: string;
  witness?: string;
  bounded?: number | null;
  verdict?: string;
  assumed?: boolean;
}

/** A `print` line. */
export interface PrintOutput {
  kind: 'print';
  line: number;
  text: string;
}

/** A language object's extra view: class, sample words in and out, decidable questions. */
export interface LangInfo {
  in: string[];
  out: string[];
  questions: { name: string; decidable: string }[];
}

/** A named object of the program (machines, regexes, languages, grammars). */
export type ObjectView = ObjectViewV1 & { class?: string; assumed?: boolean; lang?: LangInfo };

/** Everything the Pad needs after a run of the program. */
export type Document = Omit<DocumentV1, 'objects'> & {
  objects: ObjectView[];
  tests: TestOutput[];
  prints: PrintOutput[];
  outputs: { kind: string; line?: number; [key: string]: unknown }[];
  ok: boolean;
  steps: number;
};

export interface EngineApi {
  /** Lexes, parses, checks and evaluates a whole program. */
  evaluate(src: string): Promise<Document>;
  /** Runs machine `name` of the program on `input`, step by step. */
  run(src: string, name: string, input: string): Promise<RunResult>;
  /** Lays out a Graphviz DOT graph; returns Graphviz's `json` output. */
  layout(dot: string): Promise<string>;
  /** The file a `save` output names: text (svg, tex, txt, json) or bytes (png, pdf). The same
   * renderer as `deltac`, so the Pad's file and the terminal's are identical. `dark` picks the
   * drawing's theme. */
  exportFile(value: unknown, format: string, text: string, dark: boolean): Promise<string | Uint8Array>;
  /** The engine's version, for the status bar and bug reports. */
  version(): Promise<string>;
  /** Stop: terminates the worker (WASM cannot be interrupted) and starts a fresh one. Calls in
   * flight reject with `StoppedError`. */
  restart(): void;
}

/** A call that was cancelled by `restart()` (the user pressed Stop). */
export class StoppedError extends Error {
  constructor() {
    super('Stopped.');
    this.name = 'StoppedError';
  }
}
