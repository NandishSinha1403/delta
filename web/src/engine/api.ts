// The only engine module React code imports (docs/plan/03-ARCHITECTURE.md §4).
// The engine sits behind this interface so its implementation (Rust/WASM in a worker today)
// can change without touching the UI.
import type { DfaTuple } from './generated/DfaTuple';

export type { DfaTuple } from './generated/DfaTuple';
export type { Transition } from './generated/Transition';

export interface EngineApi {
  /** Phase 0 pipeline check: Sipser's M1 (Figure 1.6) as its formal 5-tuple. */
  hello(): Promise<DfaTuple>;
}
