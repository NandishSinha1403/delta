// Main-thread side of the engine: spawns the worker and wraps it as an EngineApi.
import * as Comlink from 'comlink';
import type { EngineApi } from './api';
import type { WorkerApi } from './worker';

export function createEngine(): EngineApi {
  const worker = new Worker(new URL('./worker.ts', import.meta.url), {
    type: 'module',
    name: 'delta-engine',
  });
  const remote = Comlink.wrap<WorkerApi>(worker);

  // If the worker script or the WASM fails to load, Comlink calls would hang forever;
  // racing them against this promise turns that into a visible error instead.
  const crashed = new Promise<never>((_, reject) => {
    worker.addEventListener(
      'error',
      (event) => reject(new Error(event.message || 'The engine worker failed to start.')),
      { once: true },
    );
  });
  crashed.catch(() => {});

  return {
    hello: () => Promise.race([remote.hello(), crashed]),
  };
}

export const engine: EngineApi = createEngine();
