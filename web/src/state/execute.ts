import { useEngineStore } from './engine';
import { useUiStore } from './ui';

/** Execute: re-evaluate the whole file now (diagram, tuple, δ-table, steps, every check and run). */
export function execute(): void {
  void useEngineStore.getState().evaluate(useUiStore.getState().source);
}

/** Auto is off and the code changed since the last execute. */
export function useStale(): boolean {
  const auto = useUiStore((s) => s.auto);
  const source = useUiStore((s) => s.source);
  const docSource = useEngineStore((s) => s.docSource);
  const hasDoc = useEngineStore((s) => s.doc !== null);
  return !auto && hasDoc && source !== docSource;
}

/** The Execute shortcut as written on this platform. */
export const EXECUTE_KEYS =
  typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent) ? '⌘↵' : 'Ctrl+Enter';
