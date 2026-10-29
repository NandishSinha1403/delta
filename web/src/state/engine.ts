import { create } from 'zustand';
import type { DfaTuple } from '../engine/api';
import { engine } from '../engine/client';

export type HelloStatus =
  | { kind: 'loading' }
  | { kind: 'ready'; tuple: DfaTuple }
  | { kind: 'error'; message: string };

interface EngineStore {
  hello: HelloStatus;
  loadHello: () => Promise<void>;
}

export const useEngineStore = create<EngineStore>()((set) => ({
  hello: { kind: 'loading' },
  loadHello: async () => {
    set({ hello: { kind: 'loading' } });
    try {
      set({ hello: { kind: 'ready', tuple: await engine.hello() } });
    } catch (err) {
      set({ hello: { kind: 'error', message: err instanceof Error ? err.message : String(err) } });
    }
  },
}));
