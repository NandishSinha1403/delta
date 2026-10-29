import { useEngineStore } from '../state/engine';
import { useUiStore } from '../state/ui';

/** Why a view has nothing to show yet: the engine is starting, failed, or the program is empty. */
export function EngineStatus({ empty }: { empty: string }) {
  const status = useEngineStore((s) => s.status);
  const error = useEngineStore((s) => s.error);
  const evaluate = useEngineStore((s) => s.evaluate);
  const errors = useEngineStore((s) => s.doc?.diagnostics.filter((d) => d.severity === 'error').length ?? 0);
  if (status === 'loading') return <Note>Starting the engine…</Note>;
  if (status === 'error') {
    return (
      <div role="alert" className="m-4 rounded-md border border-reject/40 bg-reject/10 p-3 text-sm">
        <p className="font-medium text-reject">The engine failed.</p>
        <p className="mt-1 font-mono text-xs text-muted">{error}</p>
        <button
          type="button"
          onClick={() => void evaluate(useUiStore.getState().source)}
          className="mt-2 min-h-11 rounded-md border border-border px-3 text-text hover:bg-surface-2"
        >
          Retry
        </button>
      </div>
    );
  }
  // Objects with errors are not built, so point at the errors rather than the generic hint.
  if (errors > 0) {
    return <Note>Fix the {errors === 1 ? 'error' : `${errors} errors`} in the code first (see Problems).</Note>;
  }
  return <Note>{empty}</Note>;
}

export function Note({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex h-full min-h-24 items-center justify-center p-6 text-center">
      <p className="max-w-xs text-sm text-muted">{children}</p>
    </div>
  );
}
