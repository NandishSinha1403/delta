import { useEngineStore } from '../state/engine';
import { useUiStore } from '../state/ui';
import { applyEdits, openRunView, showString } from './current';
import { EngineStatus, Note } from './Status';

/** Results of the program's `check` and `run` commands (docs/plan/05 §4, batch test panel). */
export function TestsPanel() {
  const results = useEngineStore((s) => s.doc?.results);
  const startRun = useEngineStore((s) => s.startRun);
  const showRun = useEngineStore((s) => s.showRun);
  const select = useUiStore((s) => s.select);
  const revealRange = useUiStore((s) => s.revealRange);
  if (!results) return <EngineStatus empty="" />;
  if (results.length === 0) {
    return <Note>Add a check, e.g. check M accepts "01", "11", to test your machine here.</Note>;
  }

  const cases = results.flatMap((r) => r.cases);
  const passed = cases.filter((c) => c.expected === c.actual).length;

  return (
    <section aria-label="Tests" className="p-3">
      {cases.length > 0 && (
        <p
          data-testid="tests-summary"
          className={`mb-3 text-sm font-semibold ${passed === cases.length ? 'text-accept' : 'text-reject'}`}
        >
          {passed} of {cases.length} passed
        </p>
      )}
      <div className="space-y-4">
        {results.map((r) => {
          if (r.kind === 'run') {
            return (
              <div key={r.from} className="flex items-center gap-2 text-sm">
                <button
                  type="button"
                  onClick={() => revealRange({ from: r.from, to: r.to })}
                  className="font-mono text-muted hover:text-text hover:underline"
                >
                  run {r.object}
                </button>
                {r.run && (
                  <>
                    <span className="font-mono text-text">on {showString(r.run.input.join(''))}</span>
                    <span className={r.run.accepted ? 'text-accept' : 'text-reject'}>
                      {r.run.accepted ? 'accepts' : 'rejects'}
                    </span>
                    <button
                      type="button"
                      onClick={() => {
                        if (!r.run) return;
                        select(r.object);
                        showRun(r.run);
                        openRunView();
                      }}
                      className="ml-auto min-h-8 rounded-md border border-border px-2 text-xs text-text hover:bg-surface-2"
                    >
                      Step through
                    </button>
                  </>
                )}
              </div>
            );
          }
          const expectAccept = r.cases[0]?.expected ?? true;
          return (
            <table key={r.from} className="w-full border-collapse text-sm">
              <caption className="mb-1 text-left">
                <button
                  type="button"
                  onClick={() => revealRange({ from: r.from, to: r.to })}
                  className="font-mono text-xs text-muted hover:text-text hover:underline"
                >
                  check {r.object} {expectAccept ? 'accepts' : 'rejects'}
                </button>
              </caption>
              <thead className="sr-only">
                <tr>
                  <th>Input</th>
                  <th>Expected</th>
                  <th>Result</th>
                </tr>
              </thead>
              <tbody>
                {r.cases.map((c) => {
                  const ok = c.expected === c.actual;
                  return (
                    <tr
                      key={c.from}
                      className="cursor-pointer border-t border-border hover:bg-surface-2"
                      onClick={() => {
                        select(r.object);
                        void startRun(useUiStore.getState().source, r.object, c.input);
                        openRunView();
                      }}
                    >
                      <td className="py-1 pr-2 font-mono text-text">{showString(c.input)}</td>
                      <td className="py-1 pr-2 text-xs text-muted">expected {c.expected ? 'accept' : 'reject'}</td>
                      <td className={`py-1 text-right text-xs font-semibold ${ok ? 'text-accept' : 'text-reject'}`}>
                        {ok ? '✓ pass' : `✗ ${c.actual ? 'accepted' : 'rejected'}`}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          );
        })}
      </div>
    </section>
  );
}

const SEVERITY: Record<string, { icon: string; className: string; label: string }> = {
  error: { icon: '●', className: 'text-reject', label: 'Error' },
  warning: { icon: '▲', className: 'text-warn', label: 'Warning' },
  info: { icon: 'i', className: 'text-accent', label: 'Info' },
  hint: { icon: '·', className: 'text-muted', label: 'Hint' },
};

/** Every diagnostic, with the book line it cites and its quick fixes. */
export function ProblemsPanel() {
  const diagnostics = useEngineStore((s) => s.doc?.diagnostics);
  const docSource = useEngineStore((s) => s.docSource);
  const revealRange = useUiStore((s) => s.revealRange);
  const setSource = useUiStore((s) => s.setSource);
  if (!diagnostics) return <EngineStatus empty="" />;
  if (diagnostics.length === 0) return <Note>No problems. The program matches the book’s definitions.</Note>;
  return (
    <ul aria-label="Problems" className="divide-y divide-border">
      {diagnostics.map((d, i) => {
        const sev = SEVERITY[d.severity] ?? SEVERITY.info;
        return (
          <li key={i} className="px-3 py-2 text-sm">
            <button
              type="button"
              onClick={() => revealRange({ from: d.from, to: d.to })}
              className="flex w-full gap-2 text-left"
            >
              <span aria-label={sev?.label} className={`w-3 shrink-0 text-center ${sev?.className ?? ''}`}>
                {sev?.icon}
              </span>
              <span className="min-w-0 flex-1">
                <span className="font-mono text-xs text-muted">{d.code}</span>{' '}
                <span className="text-text">{d.message}</span>
                {d.book && (
                  <span className="mt-1 block text-xs text-muted italic">
                    {d.book.item}: “{d.book.quote}”
                  </span>
                )}
              </span>
            </button>
            {d.fixes.length > 0 && (
              <div className="mt-1.5 flex flex-wrap gap-1.5 pl-5">
                {d.fixes.map((f) => (
                  <button
                    key={f.title}
                    type="button"
                    onClick={() => {
                      // The fix's offsets refer to the evaluated source; skip it if the code moved on.
                      if (useUiStore.getState().source === docSource) setSource(applyEdits(docSource, f.edits));
                    }}
                    className="min-h-8 rounded-md border border-border px-2 text-xs text-text hover:bg-surface-2"
                  >
                    {f.title}
                  </button>
                ))}
              </div>
            )}
          </li>
        );
      })}
    </ul>
  );
}
