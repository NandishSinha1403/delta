// Runs inside a Web Worker: loads the Rust engine (WASM) and exposes it to the main thread via
// Comlink. All engine work happens here so the UI never blocks.
import * as Comlink from 'comlink';
import type { Graphviz } from '@hpcc-js/wasm-graphviz';
import type { Document } from './generated/Document';
import type { RunResult } from './generated/RunResult';
import init, { evaluate, run, version } from './pkg/delta_wasm';

// Fetches and instantiates delta_wasm_bg.wasm (Vite resolves its URL at build time).
const ready = init();

// Graphviz is ~800 KB, so it is loaded the first time a diagram is drawn, not at startup.
let graphviz: Promise<Graphviz> | undefined;
function loadGraphviz(): Promise<Graphviz> {
  graphviz ??= import('@hpcc-js/wasm-graphviz').then((m) => m.Graphviz.load());
  return graphviz;
}

const api = {
  async evaluate(src: string): Promise<Document> {
    await ready;
    return JSON.parse(evaluate(src)) as Document;
  },
  async run(src: string, name: string, input: string): Promise<RunResult> {
    await ready;
    return JSON.parse(run(src, name, input)) as RunResult;
  },
  async layout(dot: string): Promise<string> {
    return (await loadGraphviz()).layout(dot, 'json', 'dot');
  },
  async version(): Promise<string> {
    await ready;
    return version();
  },
};

export type WorkerApi = typeof api;

Comlink.expose(api);
