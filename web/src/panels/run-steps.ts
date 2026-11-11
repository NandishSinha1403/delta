// Steps of a run. A finite-automaton run has one step per input symbol; other models (pushdown,
// Turing, counter, queue machines) have one step per breadth-first layer of configurations, which
// can be more or fewer than the symbols, and carry structured configurations for the storage views.
import type { RunView } from '../engine/api';

/** One configuration as the engine sends it (`config_json` in crates/delta-lang/src/pad.rs). */
export interface StorageConfig {
  state: string;
  /** Input head (symbols read so far). */
  head: number;
  tapes: { cells: string[]; head: number }[];
  /** Each stack, top first. */
  stacks: string[][];
  counters: number[];
  /** Front first; only for queue machines. */
  queue?: string[];
  output: string;
}

interface StepExtras {
  configs?: StorageConfig[];
  more?: number;
}

/** The index of the last step. */
export function lastStep(run: RunView): number {
  return Math.max(0, run.steps.length - 1);
}

/** True for runs whose steps are configurations rather than one per symbol. */
export function hasStorage(run: RunView): boolean {
  return run.steps.some((s) => (s as StepExtras).configs !== undefined);
}

/** The configurations at step `pos` (the first few), and how many more there are. */
export function configsAt(run: RunView, pos: number): { configs: StorageConfig[]; more: number } {
  const s = run.steps[pos] as (RunView['steps'][number] & StepExtras) | undefined;
  return { configs: s?.configs ?? [], more: s?.more ?? 0 };
}

/** Where the input head is at step `pos`. */
export function inputHead(run: RunView, pos: number): number {
  return run.steps[pos]?.pos ?? pos;
}
