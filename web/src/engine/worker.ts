// Runs inside a Web Worker: loads the Rust engine (WASM) and exposes it to the main thread via
// Comlink. All engine work happens here so the UI never blocks.
import * as Comlink from 'comlink';
import type { Graphviz } from '@hpcc-js/wasm-graphviz';
import type { Document } from './api';
import type { RunResult } from './generated/RunResult';
import init, { evaluate, render_file, run, version } from './pkg/delta_wasm';

// Fetches and instantiates delta_wasm_bg.wasm (Vite resolves its URL at build time).
const ready = init();

// Graphviz is ~800 KB, so it is loaded the first time a diagram is drawn, not at startup.
let graphviz: Promise<Graphviz> | undefined;
function loadGraphviz(): Promise<Graphviz> {
  graphviz ??= import('@hpcc-js/wasm-graphviz').then((m) => m.Graphviz.load());
  return graphviz;
}

// The image export module (resvg + svg2pdf, ~900 KB) is loaded the first PNG or PDF download.
type Exporter = typeof import('./export-pkg/delta_export_wasm');
let exporter: Promise<Exporter> | undefined;
function loadExporter(): Promise<Exporter> {
  exporter ??= import('./export-pkg/delta_export_wasm').then(async (m) => {
    await m.default();
    return m;
  });
  return exporter;
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
  /** A `save` output's file: text for svg, tex, txt and json; bytes for png and pdf. */
  async exportFile(value: unknown, format: string, text: string): Promise<string | Uint8Array> {
    await ready;
    const json = JSON.stringify(value);
    if (format === 'png' || format === 'pdf') {
      const svg = render_file(json, 'svg', text, false);
      const m = await loadExporter();
      return format === 'png' ? m.png(svg, 2) : m.pdf(svg);
    }
    return render_file(json, format, text, false);
  },
  async version(): Promise<string> {
    await ready;
    return version();
  },
};

export type WorkerApi = typeof api;

Comlink.expose(api);
