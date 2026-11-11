// The full-screen presentation view, loaded the first time someone presents (it is not needed at
// startup). The Present button and its settings panel live in Present.tsx.
import { useEffect, useRef, useState } from 'react';
import { Diagram } from '../diagram/Diagram';
import type { MachineView, ObjectView, RunView } from '../engine/api';
import { useActiveRun, useCurrentObject } from '../panels/current';
import { runOverlay, traceFrame } from '../panels/overlay';
import { hasStorage, inputHead, lastStep } from '../panels/run-steps';
import { TuplePanel } from '../panels/TuplePanel';
import { useEngineStore, useObjects } from '../state/engine';
import { transitionMarks, usePresentStore, type Mark } from '../state/present';
import { useUiStore } from '../state/ui';

/** The declaration of `object` as numbered lines, with the arrows just taken marked. */
function CodeAside({ object, marks }: { object: ObjectView; marks: Mark[] }) {
  const source = useEngineStore((s) => s.docSource);
  const lineStart = source.lastIndexOf('\n', object.from - 1) + 1;
  const first = source.slice(0, lineStart).split('\n').length;
  const lines = source.slice(lineStart, object.to).split('\n');
  return (
    <aside
      aria-label={`Code of ${object.name}`}
      className="flex w-[34%] max-w-[44rem] shrink-0 flex-col overflow-auto border-l border-border px-[2.5vw] py-[3vh] font-mono text-[max(16px,1.5vw)] leading-[1.55]"
    >
      {lines.map((text, i) => {
        const line = first + i;
        const here = marks.filter((m) => m.line === line);
        let at = 0;
        const parts: React.ReactNode[] = [];
        for (const m of here) {
          parts.push(text.slice(at, m.start));
          parts.push(
            <mark
              key={m.start}
              data-testid="code-mark"
              className="rounded-md bg-navy px-1 text-text outline-2 outline-accent outline-solid"
            >
              {text.slice(m.start, m.end)}
            </mark>,
          );
          at = m.end;
        }
        parts.push(text.slice(at));
        return (
          <div key={line} className="whitespace-pre text-text">
            {parts}
            {text === '' ? ' ' : null}
          </div>
        );
      })}
    </aside>
  );
}

/** The input tape, very large: read symbols in the muted color, the head on the next one. */
function BigTape({ run, pos }: { run: RunView; pos: number }) {
  const head = inputHead(run, pos);
  const cells = [...run.input, '⊣'];
  return (
    <ol aria-label="Input tape" className="flex max-w-full gap-[0.4vw] overflow-x-auto font-mono">
      {cells.map((s, i) => {
        const end = i === run.input.length;
        const here = i === head;
        return (
          <li
            key={i}
            aria-current={here ? 'step' : undefined}
            className={`grid h-[min(84px,8vh)] min-w-[min(84px,8vh)] shrink-0 place-items-center rounded-md px-2 text-[min(44px,4.2vh)] ${
              here ? 'border-[3px] border-accent bg-navy text-text' : end ? 'border border-dashed border-border-strong text-muted' : 'border border-border-strong'
            } ${i < head ? 'text-muted' : here ? '' : 'text-text'}`}
          >
            {s}
          </li>
        );
      })}
    </ol>
  );
}

interface Stage {
  title: string;
  counter: string | null;
  body: React.ReactNode;
  footer: React.ReactNode;
  /** ← and →, Home and End. */
  go: (to: 'prev' | 'next' | 'first' | 'last') => void;
}

/** Everything the run mode shows for one object. */
function useRunStage(object: ObjectView | null, showCode: boolean): Stage | null {
  const { run, pos } = useActiveRun(object);
  const storeRun = useEngineStore((s) => s.run);
  const showRun = useEngineStore((s) => s.showRun);
  const setRunPos = useEngineStore((s) => s.setRunPos);
  const source = useEngineStore((s) => s.docSource);
  if (!object?.machine) return null;
  const machine: MachineView = object.machine;
  if (!run) {
    return {
      title: `${object.name}: no run yet`,
      counter: null,
      body: <Diagram key={object.name} name={object.name} machine={machine} legend={false} presenting />,
      footer: (
        <p className="text-[max(16px,1.4vw)] text-muted">
          Add <code className="font-mono text-text">run {object.name} on "…"</code> to the program, or start one in the Run panel.
        </p>
      ),
      go: () => {},
    };
  }
  const n = lastStep(run);
  const go = (to: 'prev' | 'next' | 'first' | 'last') => {
    if (storeRun !== run) showRun(run);
    const next = to === 'first' ? 0 : to === 'last' ? n : to === 'prev' ? pos - 1 : pos + 1;
    setRunPos(Math.max(0, Math.min(n, next)));
  };
  const overlay = runOverlay(run, pos, machine);
  const marks =
    showCode && !hasStorage(run)
      ? transitionMarks(source, object.from, object.to, machine, overlay.taken ?? new Set(), run.input[pos - 1])
      : [];
  const at = run.steps[pos];
  const done = pos >= n;
  return {
    title: `run ${object.name} on "${run.input.join('')}"`,
    counter: `${pos} / ${n}`,
    body: (
      <div className="flex h-full min-h-0">
        <div className="min-w-0 flex-1">
          <Diagram key={object.name} name={object.name} machine={machine} overlay={overlay} legend={false} presenting />
        </div>
        {showCode && <CodeAside object={object} marks={marks} />}
      </div>
    ),
    footer: (
      <div className="flex min-w-0 flex-1 flex-wrap items-center gap-x-[3vw] gap-y-3">
        <BigTape run={run} pos={pos} />
        <div className="flex min-w-0 flex-1 flex-col gap-1">
          <p data-testid="present-message" aria-live="polite" className="font-mono text-[max(18px,2.1vw)] leading-tight text-text">
            {at?.message}
            {done && <span className="ml-3 text-accent">{run.accepted ? '✓ accepted' : '✗ rejected'}</span>}
          </p>
          <p className="text-[max(15px,1.25vw)] text-muted">{at?.explain}</p>
        </div>
      </div>
    ),
    go,
  };
}

/** Diagram only: a construction's trace steps when it has one, else just the machine. */
function useDiagramStage(object: ObjectView | null): Stage | null {
  const step = useUiStore((s) => s.step);
  const setStep = useUiStore((s) => s.setStep);
  if (!object?.machine) return null;
  const steps = object.trace?.steps ?? [];
  const frame = step !== null ? traceFrame(object, step) : null;
  const caption = step !== null ? steps[step] : undefined;
  // Trace positions: 0…k-1 are the steps, k (= null) is the finished machine.
  const k = steps.length;
  const at = step ?? k;
  const go = (to: 'prev' | 'next' | 'first' | 'last') => {
    if (k === 0) return;
    const next = to === 'first' ? 0 : to === 'last' ? k : to === 'prev' ? at - 1 : at + 1;
    const clamped = Math.max(0, Math.min(k, next));
    setStep(clamped === k ? null : clamped);
  };
  return {
    title: `${object.name} · ${object.kind.toUpperCase()}`,
    counter: k > 0 ? `${at} / ${k}` : null,
    body: (
      <Diagram
        key={`${object.name}-${frame ? 'trace' : 'final'}`}
        name={object.name}
        machine={frame?.machine ?? object.machine}
        overlay={frame?.overlay ?? {}}
        legend={false}
        presenting
      />
    ),
    footer: (
      <p className="text-[max(16px,1.5vw)] text-text">
        {caption ? (
          <>
            <span className="font-semibold text-accent">{caption.title}</span>{' '}
            <span className="text-muted">{caption.message}</span>
          </>
        ) : (
          <span className="text-muted">{object.derived ?? `${object.machine.states.length} states`}</span>
        )}
      </p>
    ),
    go,
  };
}

/** The tuple and δ-table, set large; ← → move between the program's objects. */
function useTupleStage(object: ObjectView | null): Stage | null {
  const objects = useObjects();
  const select = useUiStore((s) => s.select);
  if (!object) return null;
  const i = objects.findIndex((o) => o.name === object.name);
  const go = (to: 'prev' | 'next' | 'first' | 'last') => {
    const next = to === 'first' ? 0 : to === 'last' ? objects.length - 1 : to === 'prev' ? i - 1 : i + 1;
    const o = objects[Math.max(0, Math.min(objects.length - 1, next))];
    if (o) select(o.name);
  };
  return {
    title: `${object.name} · ${object.kind === 'regex' ? 'regex' : object.kind.toUpperCase()}`,
    counter: objects.length > 1 ? `${i + 1} / ${objects.length}` : null,
    body: (
      <div className="h-full overflow-auto px-[4vw] py-[3vh] text-[max(18px,1.7vw)]">
        <TuplePanel />
      </div>
    ),
    // No caption here; the footer still carries the step buttons for touch screens.
    footer: <div className="flex-1" />,
    go,
  };
}

/** Full screen: the chosen view, large, with the keys from DESIGN.md and an optional laser. */
export default function Presentation() {
  const what = usePresentStore((s) => s.what);
  const theme = usePresentStore((s) => s.theme);
  const showCode = usePresentStore((s) => s.showCode);
  const blank = usePresentStore((s) => s.blank);
  const laser = usePresentStore((s) => s.laser);
  const { stop, setBlank, setLaser } = usePresentStore.getState();
  const object = useCurrentObject();
  const file = useUiStore((s) => s.files.find((f) => f.id === s.activeId)?.name ?? '');
  const [dot, setDot] = useState<{ x: number; y: number } | null>(null);

  const runStage = useRunStage(object, showCode);
  const diagramStage = useDiagramStage(object);
  const tupleStage = useTupleStage(object);
  const stage = what === 'run' ? (runStage ?? tupleStage) : what === 'diagram' ? (diagramStage ?? tupleStage) : tupleStage;

  // Keys, in the capture phase so nothing underneath (the editor, the Run panel) sees them.
  const goRef = useRef(stage?.go);
  useEffect(() => {
    goRef.current = stage?.go;
  });
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.metaKey || e.ctrlKey || e.altKey) return;
      const p = usePresentStore.getState();
      if (p.blank) {
        p.setBlank(false);
        e.preventDefault();
        e.stopPropagation();
        return;
      }
      const go = goRef.current;
      const k = e.key;
      if (k === 'ArrowRight' || k === 'PageDown' || k === 'ArrowDown') go?.('next');
      else if (k === 'ArrowLeft' || k === 'PageUp' || k === 'ArrowUp') go?.('prev');
      else if (k === 'Home') go?.('first');
      else if (k === 'End') go?.('last');
      else if (k === 'b' || k === 'B' || k === '.') p.setBlank(true);
      else if (k === 'l' || k === 'L') p.setLaser(!p.laser);
      else if (k === 'Escape') p.stop();
      else return;
      e.preventDefault();
      e.stopPropagation();
    };
    window.addEventListener('keydown', onKey, true);
    return () => window.removeEventListener('keydown', onKey, true);
  }, []);

  // Leaving full screen with the browser's own Esc also ends the presentation.
  useEffect(() => {
    let was = Boolean(document.fullscreenElement);
    const onChange = () => {
      const now = Boolean(document.fullscreenElement);
      if (was && !now) usePresentStore.getState().stop();
      was = now;
    };
    document.addEventListener('fullscreenchange', onChange);
    return () => document.removeEventListener('fullscreenchange', onChange);
  }, []);

  const themeAttr = theme === 'match' ? undefined : theme;
  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Presentation"
      data-testid="presentation"
      data-present={themeAttr}
      data-what={what}
      onPointerMove={(e) => laser && setDot({ x: e.clientX, y: e.clientY })}
      className={`fixed inset-0 z-50 flex flex-col bg-bg text-text ${laser ? 'cursor-none' : ''}`}
    >
      <header className="flex h-[max(56px,8vh)] shrink-0 items-center gap-[1.2vw] border-b border-border px-[3vw] font-mono text-[max(15px,1.35vw)]">
        <span className="text-muted">{file}</span>
        <span className="text-muted" aria-hidden="true">
          ›
        </span>
        <span className="truncate font-semibold">{stage?.title ?? 'Nothing to present yet'}</span>
        <span className="flex-1" />
        <span aria-hidden="true" className="hidden gap-4 text-[max(11px,0.85vw)] text-muted xl:flex">
          <span>← → step</span>
          <span>Home/End</span>
          <span>L laser</span>
          <span>B blank</span>
          <span>Esc exit</span>
        </span>
        {stage?.counter && (
          <span data-testid="present-counter" className="font-semibold tabular-nums">
            <span className="mr-2 font-normal text-muted">step</span>
            {stage.counter}
          </span>
        )}
        <button
          type="button"
          onClick={stop}
          aria-label="Exit presentation (Esc)"
          className="ml-[1vw] grid h-10 w-10 place-items-center rounded-lg border border-border-strong text-muted hover:text-text"
        >
          ×
        </button>
      </header>
      <main className="min-h-0 flex-1">{stage?.body}</main>
      {stage?.footer && (
        <footer className="flex shrink-0 items-center gap-[2vw] border-t border-border px-[3vw] py-[2.5vh]">
          {stage.footer}
          <div className="flex shrink-0 gap-2">
            <button
              type="button"
              aria-label="Previous step"
              onClick={() => stage.go('prev')}
              className="h-[max(44px,6vh)] w-[max(44px,6vh)] rounded-lg border border-border-strong text-[max(18px,1.6vw)] text-text"
            >
              ←
            </button>
            <button
              type="button"
              aria-label="Next step"
              onClick={() => stage.go('next')}
              className="h-[max(44px,6vh)] w-[max(44px,6vh)] rounded-lg bg-accent text-[max(18px,1.6vw)] text-on-accent"
            >
              →
            </button>
            <button
              type="button"
              aria-pressed={laser}
              onClick={() => setLaser(!laser)}
              className={`h-[max(44px,6vh)] rounded-lg border px-4 text-[max(14px,1.15vw)] ${laser ? 'border-brand text-text' : 'border-border-strong text-text'}`}
            >
              Laser
            </button>
          </div>
        </footer>
      )}
      {laser && dot && (
        <div
          aria-hidden="true"
          data-testid="laser"
          className="pointer-events-none fixed z-[60] h-7 w-7 -translate-x-1/2 -translate-y-1/2 rounded-full bg-brand"
          style={{ left: dot.x, top: dot.y, boxShadow: '0 0 18px 8px rgb(230 57 70 / 0.55)' }}
        />
      )}
      {blank && (
        <button
          type="button"
          aria-label="Blank screen: press any key or click to come back"
          data-testid="blank"
          onClick={() => setBlank(false)}
          className="fixed inset-0 z-[70] cursor-none bg-bg"
        />
      )}
    </div>
  );
}
