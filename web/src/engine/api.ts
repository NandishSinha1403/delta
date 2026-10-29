// The only engine module React code imports (docs/plan/03-ARCHITECTURE.md §4).
// The engine sits behind this interface so its implementation (Rust/WASM in a worker today)
// can change without touching the UI.
import type { Document } from './generated/Document';
import type { RunResult } from './generated/RunResult';

export type { BookRefView } from './generated/BookRefView';
export type { CheckCaseView } from './generated/CheckCaseView';
export type { CommandResultView } from './generated/CommandResultView';
export type { DiagnosticView } from './generated/DiagnosticView';
export type { Document } from './generated/Document';
export type { EdgeView } from './generated/EdgeView';
export type { FixView } from './generated/FixView';
export type { MachineView } from './generated/MachineView';
export type { ObjectView } from './generated/ObjectView';
export type { OutlineView } from './generated/OutlineView';
export type { RunResult } from './generated/RunResult';
export type { RunStepView } from './generated/RunStepView';
export type { RunView } from './generated/RunView';
export type { StateView } from './generated/StateView';
export type { StepView } from './generated/StepView';
export type { TraceView } from './generated/TraceView';

export interface EngineApi {
  /** Lexes, parses, checks and evaluates a whole program. */
  evaluate(src: string): Promise<Document>;
  /** Runs machine `name` of the program on `input`, step by step. */
  run(src: string, name: string, input: string): Promise<RunResult>;
  /** Lays out a Graphviz DOT graph; returns Graphviz's `json` output. */
  layout(dot: string): Promise<string>;
  /** The engine's version, for the status bar and bug reports. */
  version(): Promise<string>;
}
