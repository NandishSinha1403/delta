import type { DfaTuple } from '../engine/api';
import { useEngineStore } from '../state/engine';
import { deltaTable, formatSet, tupleHeader } from './tuple-format';

/** Shows the engine's answer, or why there isn't one yet. */
function useTuple(): { tuple: DfaTuple } | { pending: React.ReactNode } {
  const hello = useEngineStore((s) => s.hello);
  const loadHello = useEngineStore((s) => s.loadHello);
  if (hello.kind === 'ready') return { tuple: hello.tuple };
  if (hello.kind === 'loading') {
    return { pending: <p className="p-4 text-sm text-muted">Starting the engine…</p> };
  }
  return {
    pending: (
      <div role="alert" className="m-4 rounded-md border border-reject/40 bg-reject/10 p-3 text-sm">
        <p className="font-medium text-reject">The engine failed to load.</p>
        <p className="mt-1 font-mono text-xs text-muted">{hello.message}</p>
        <button
          type="button"
          onClick={() => void loadHello()}
          className="mt-2 min-h-11 rounded-md border border-border px-3 text-text hover:bg-surface-2"
        >
          Retry
        </button>
      </div>
    ),
  };
}

export function DeltaTable({ tuple }: { tuple: DfaTuple }) {
  const table = deltaTable(tuple);
  return (
    <table className="border-collapse font-mono text-sm">
      <caption className="sr-only">Transition function δ of {tuple.name}</caption>
      <thead>
        <tr>
          <th scope="col" className="border-b border-r border-border px-3 py-1">
            <span className="sr-only">state</span>
          </th>
          {table.symbols.map((a) => (
            <th key={a} scope="col" className="border-b border-border px-3 py-1 font-semibold">
              {a}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {table.rows.map((row) => (
          <tr key={row.state}>
            <th scope="row" className="border-r border-border px-3 py-1 text-left font-semibold">
              {row.state}
            </th>
            {row.cells.map((to, i) => (
              <td key={table.symbols[i]} className="px-3 py-1">
                {to}
              </td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}

/** The formal tuple, phrased the way Sipser writes it in Example 1.7. */
export function TuplePanel() {
  const state = useTuple();
  if ('pending' in state) return state.pending;
  const m = state.tuple;
  return (
    <section data-testid="tuple" aria-label={`Formal tuple of ${m.name}`} className="p-4">
      <p className="font-mono text-base font-semibold text-text">{tupleHeader(m)}, where</p>
      <ol className="mt-3 list-decimal space-y-2 pl-6 font-mono text-sm text-text marker:text-muted">
        <li>Q = {formatSet(m.states)},</li>
        <li>Σ = {formatSet(m.sigma)},</li>
        <li>
          δ is described as
          <div className="mt-2">
            <DeltaTable tuple={m} />
          </div>
        </li>
        <li>{m.start} is the start state, and</li>
        <li>F = {formatSet(m.accept)}.</li>
      </ol>
      <p className="mt-4 text-xs text-muted">
        Computed by delta-core (Rust → WebAssembly) in a Web Worker. Definition 1.5.
      </p>
    </section>
  );
}

export function DeltaPanel() {
  const state = useTuple();
  if ('pending' in state) return state.pending;
  return (
    <div className="p-4">
      <DeltaTable tuple={state.tuple} />
    </div>
  );
}
