import { useEngineStore } from '../state/engine';
import { useUiStore, type InspectorTab } from '../state/ui';
import { Credit } from './Credit';

/** Errors, warnings, tests · Ln/Col, mode, engine. Counts open their inspector tab. */
export function StatusBar() {
  const doc = useEngineStore((s) => s.doc);
  const version = useEngineStore((s) => s.version);
  const status = useEngineStore((s) => s.status);
  const cursor = useUiStore((s) => s.cursor);
  const open = (tab: InspectorTab) => useUiStore.setState({ inspectorTab: tab, sheetTab: tab, sheetOpen: true });
  const count = (sev: string) => doc?.diagnostics.filter((d) => d.severity === sev).length ?? 0;
  const errors = count('error');
  const warnings = count('warning');
  const cases = doc?.results.flatMap((r) => r.cases) ?? [];
  const passed = cases.filter((c) => c.expected === c.actual).length;
  return (
    <footer className="flex h-[26px] shrink-0 items-center gap-4 overflow-hidden border-t border-border bg-status px-3.5 font-mono text-[11px] whitespace-nowrap text-muted">
      <button type="button" onClick={() => open('problems')} className="flex gap-4 hover:text-text">
        <span data-testid="error-count" className={errors > 0 ? 'text-reject' : 'text-text'}>
          {errors > 0 ? '●' : '✓'} {errors} error{errors === 1 ? '' : 's'}
        </span>
        <span className={warnings > 0 ? 'text-warn' : ''}>
          {warnings} warning{warnings === 1 ? '' : 's'}
        </span>
      </button>
      {cases.length > 0 && (
        <button
          type="button"
          onClick={() => open('tests')}
          className={passed === cases.length ? 'text-text hover:underline' : 'text-reject hover:underline'}
        >
          tests {passed}/{cases.length}
        </button>
      )}
      <div className="flex-1" />
      <span>
        Ln {cursor.line}, Col {cursor.col}
      </span>
      <span>{doc?.mode ?? 'lenient'}</span>
      <span>{status === 'error' ? 'engine error' : version ? `engine ${version} · wasm` : 'engine starting…'}</span>
      <Credit className="hidden font-sans lg:inline" />
    </footer>
  );
}
