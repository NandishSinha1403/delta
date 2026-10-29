// Runs inside a Web Worker: loads the Rust engine (WASM) and exposes it to the main thread via
// Comlink. All engine work happens here so the UI never blocks.
import * as Comlink from 'comlink';
import type { DfaTuple } from './generated/DfaTuple';
import init, { hello } from './pkg/delta_wasm';

// Fetches and instantiates delta_wasm_bg.wasm (Vite resolves its URL at build time).
const ready = init();

const api = {
  async hello(): Promise<DfaTuple> {
    await ready;
    return JSON.parse(hello()) as DfaTuple;
  },
};

export type WorkerApi = typeof api;

Comlink.expose(api);
