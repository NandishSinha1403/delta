// Runs inside a Web Worker: loads the Rust engine (WASM) and exposes it to the main thread via
// Comlink. All engine work happens here so the UI never blocks.
import * as Comlink from 'comlink';
import type { Document } from './api';
import type { RunResult } from './generated/RunResult';
import init, { diagram_layout, evaluate, render_file, run, version } from './pkg/delta_wasm';

// Fetches and instantiates delta_wasm_bg.wasm (Vite resolves its URL at build time).
const ready = init();

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
  // manual:begin worker-evaluate
  async evaluate(src: string): Promise<Document> {
    await ready;
    return JSON.parse(evaluate(src)) as Document;
  },
  // manual:end worker-evaluate
  async run(src: string, name: string, input: string): Promise<RunResult> {
    await ready;
    return JSON.parse(run(src, name, input)) as RunResult;
  },
  /** A `save` output's file: text for svg, tex, txt and json; bytes for png and pdf. */
  async exportFile(value: unknown, format: string, text: string, dark: boolean): Promise<string | Uint8Array> {
    await ready;
    const json = JSON.stringify(value);
    if (format === 'png' || format === 'pdf') {
      const svg = render_file(json, 'svg', text, dark);
      const m = await loadExporter();
      return format === 'png' ? m.png(svg, 2) : m.pdf(svg);
    }
    return render_file(json, format, text, dark);
  },
  /** delta-render's geometry for a machine (the same layout as every download). */
  async diagramLayout(view: unknown): Promise<string> {
    await ready;
    return diagram_layout(JSON.stringify(view));
  },
  async version(): Promise<string> {
    await ready;
    return version();
  },
};

export type WorkerApi = typeof api;

Comlink.expose(api);
