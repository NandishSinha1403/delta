// Main-thread side of the engine: spawns the worker and wraps it as an EngineApi.
import * as Comlink from 'comlink';
import { StoppedError, type EngineApi } from './api';
import type { WorkerApi } from './worker';

// Every algorithm respects a budget, so a call this slow means something is wrong. WASM cannot be
// interrupted, so the worker is terminated and respawned instead (docs/plan/03-ARCHITECTURE.md §4).
const TIMEOUT_MS = 10_000;

interface Connection {
  worker: Worker;
  remote: Comlink.Remote<WorkerApi>;
  /** Rejects if the worker script or the WASM fails to load, or on Stop; otherwise never settles. */
  crashed: Promise<never>;
  /** Rejects `crashed`, failing every call still waiting on this worker. */
  kill: (reason: Error) => void;
}

function connect(): Connection {
  const worker = new Worker(new URL('./worker.ts', import.meta.url), {
    type: 'module',
    name: 'delta-engine',
  });
  // Without this, Comlink calls to a dead worker would hang forever.
  let kill: (reason: Error) => void = () => {};
  const crashed = new Promise<never>((_, reject) => {
    kill = reject;
    worker.addEventListener(
      'error',
      (event) => reject(new Error(event.message || 'The engine worker failed to start.')),
      { once: true },
    );
  });
  crashed.catch(() => {});
  return { worker, remote: Comlink.wrap<WorkerApi>(worker), crashed, kill };
}

export function createEngine(): EngineApi {
  let conn = connect();

  function call<T>(f: (remote: Comlink.Remote<WorkerApi>) => Promise<T>): Promise<T> {
    const current = conn;
    let timer: number | undefined;
    const timeout = new Promise<never>((_, reject) => {
      timer = window.setTimeout(() => {
        if (conn === current) {
          current.worker.terminate();
          conn = connect();
        }
        reject(new Error('The engine took too long and was restarted.'));
      }, TIMEOUT_MS);
    });
    return Promise.race([f(current.remote), current.crashed, timeout]).finally(() =>
      window.clearTimeout(timer),
    );
  }

  return {
    evaluate: (src) => call((r) => r.evaluate(src)),
    run: (src, name, input) => call((r) => r.run(src, name, input)),
    layout: (dot) => call((r) => r.layout(dot)),
    version: () => call((r) => r.version()),
    restart: () => {
      conn.worker.terminate();
      conn.kill(new StoppedError());
      conn = connect();
    },
  };
}

export const engine: EngineApi = createEngine();
