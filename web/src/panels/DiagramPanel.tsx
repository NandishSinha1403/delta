import { conversion, DIAGRAM_CONVERSIONS } from '../conversions';
import { Diagram } from '../diagram/Diagram';
import { useEngineStore, useObjects } from '../state/engine';
import { EXECUTE_KEYS, useStale } from '../state/execute';
import { useUiStore } from '../state/ui';
import { applyConversion, useActiveRun, useCurrentObject } from './current';
import { runOverlay, traceFrame } from './overlay';
import { EngineStatus } from './Status';
import { Math } from './TuplePanel';
import { regexLine } from './tuple-format';

/** Graphviz needs seconds for thousands of states, and nobody can read such a picture. */
const MAX_DRAWN_STATES = 300;

/** The program's objects as a segmented control: N NFA · D DFA · R regex. */
function ObjectSwitcher() {
  const objects = useObjects();
  const current = useCurrentObject();
  const select = useUiStore((s) => s.select);
  if (objects.length === 0) return null;
  return (
    <div role="tablist" aria-label="Objects" className="flex min-w-0 gap-0.5 overflow-x-auto rounded-lg border border-border bg-surface-2 p-[3px]">
      {objects.map((o) => {
        const on = o.name === current?.name;
        return (
          <button
            key={o.name}
            type="button"
            role="tab"
            aria-selected={on}
            onClick={() => select(o.name)}
            className={`h-[26px] shrink-0 rounded-md px-2.5 font-mono text-xs ${on ? 'bg-raised text-text' : 'text-text-2 hover:text-text'}`}
          >
            {o.name} <span className={on ? 'text-muted' : 'text-faint'}>{o.kind === 'regex' ? 'regex' : o.kind.toUpperCase()}</span>
          </button>
        );
      })}
    </div>
  );
}

/** → DFA (Thm 1.39), → GNFA, → regex: only the ones that apply to the selected object. */
function ConvertButtons() {
  const current = useCurrentObject();
  if (!current) return null;
  const options = DIAGRAM_CONVERSIONS.map(conversion).filter((c) => c.takes.includes(current.kind));
  if (options.length === 0) return null;
  return (
    <div className="flex shrink-0 items-center gap-1.5">
      <span className="hidden text-xs text-muted xl:inline">Convert</span>
      {options.map((c) => (
        <button
          key={c.fn}
          type="button"
          onClick={() => applyConversion(c, [current.name])}
          title={`${c.book}: ${c.doc}`}
          className="h-7 rounded-md border border-steel/60 bg-accent/10 px-2.5 text-xs text-accent hover:bg-accent/20"
        >
          {c.label}
          {c.fn === 'dfa' && <span className="ml-1 text-steel-soft">Thm 1.39</span>}
        </button>
      ))}
    </div>
  );
}

/** The header above the diagram on desktop and tablet. */
function DiagramHeader() {
  return (
    <div className="flex h-11 shrink-0 items-center gap-3 border-b border-border bg-surface px-3">
      <ObjectSwitcher />
      <div className="flex-1" />
      <ConvertButtons />
    </div>
  );
}

/** Only for slow evaluations (over 300 ms): a thin indeterminate bar, "Evaluating…" and Stop. */
function Loader() {
  const slow = useEngineStore((s) => s.slow);
  const stop = useEngineStore((s) => s.stop);
  if (!slow) return null;
  return (
    <div role="status" data-testid="loader" className="relative shrink-0 border-b border-border bg-surface">
      <div className="h-0.5 overflow-hidden bg-navy-2">
        <div className="h-full w-1/3 animate-[delta-indeterminate_1.1s_ease-in-out_infinite] bg-accent" />
      </div>
      <div className="flex items-center gap-3 px-3 py-1.5 text-xs text-muted">
        <span>Evaluating…</span>
        <button
          type="button"
          onClick={stop}
          className="rounded-md border border-border-strong px-2 py-0.5 text-text hover:border-reject hover:text-reject"
        >
          Stop
        </button>
      </div>
    </div>
  );
}

/** Auto is off and the code changed: say so over the (now stale) diagram. */
function StaleBanner() {
  const stale = useStale();
  if (!stale) return null;
  return (
    <p
      role="status"
      data-testid="stale-banner"
      className="pointer-events-none absolute inset-x-0 top-3 z-10 mx-auto w-fit rounded-full border border-steel/60 bg-navy-2/95 px-3 py-1 text-xs text-accent"
    >
      Code changed — press {EXECUTE_KEYS} to update
    </p>
  );
}

/**
 * The selected object's state diagram. While scrubbing a trace it shows that step; while a run is
 * open on this machine it shows the active states.
 */
export function DiagramPanel({
  caption: showCaption = true,
  header = false,
  legend = true,
}: {
  caption?: boolean;
  header?: boolean;
  legend?: boolean;
}) {
  const object = useCurrentObject();
  const step = useUiStore((s) => s.step);
  const { run, pos } = useActiveRun(object);
  const slow = useEngineStore((s) => s.slow);
  const body = (() => {
    if (!object) return <EngineStatus empty="Declare a dfa or nfa to see its state diagram." />;
    const frame = step !== null ? traceFrame(object, step) : null;
    const caption = step !== null ? object.trace?.steps[step] : undefined;
    const machine = frame?.machine ?? object.machine;

    if (machine && machine.states.length > MAX_DRAWN_STATES) {
      return (
        <div className="flex h-full flex-col items-center justify-center gap-2 p-6 text-center">
          <p className="text-sm text-text">
            {object.name} has {machine.states.length} states, too many to draw.
          </p>
          <p className="max-w-sm text-sm text-muted">
            Diagrams are drawn up to {MAX_DRAWN_STATES} states. The δ-table and the tuple still show all of it.
          </p>
        </div>
      );
    }

    if (frame) {
      return (
        <div className="flex h-full flex-col">
          {showCaption && caption && (
            <p className="shrink-0 border-b border-border bg-surface px-4 py-2 text-xs text-muted">
              <span className="font-semibold text-accent">
                Step {step === null ? '' : step + 1} · {caption.rule}
              </span>{' '}
              <span className="line-clamp-2 whitespace-pre-line">{caption.message}</span>
            </p>
          )}
          <div className="min-h-0 flex-1">
            <Diagram key={`${object.name}-trace`} name={object.name} machine={frame.machine} overlay={frame.overlay} legend={legend} />
          </div>
        </div>
      );
    }

    if (!object.machine) {
      return (
        <div className="flex h-full flex-col items-center justify-center gap-3 bg-canvas p-6 text-center">
          <p className="text-xl text-text">
            <Math line={regexLine(object.name, object.regex ?? '')} />
          </p>
          <p className="max-w-sm text-sm text-muted">
            A regular expression has no state diagram. Convert it with → DFA, or write{' '}
            <code className="font-mono text-text">let N = nfa({object.name})</code> to see Lemma 1.55 build one.
          </p>
        </div>
      );
    }

    const overlay = run ? runOverlay(run, pos, object.machine) : {};
    return <Diagram key={object.name} name={object.name} machine={object.machine} overlay={overlay} legend={legend} />;
  })();

  // While a slow evaluation runs, the previous diagram stays visible but dimmed.
  return (
    <div className="flex h-full min-h-0 flex-col">
      {header && <DiagramHeader />}
      <Loader />
      <div className="relative min-h-0 flex-1">
        <StaleBanner />
        <div className={`h-full transition-opacity ${slow ? 'opacity-40' : ''}`} aria-busy={slow}>
          {body}
        </div>
      </div>
    </div>
  );
}
