import { useEffect, useRef, useState } from 'react';
import type { MachineView, RunView } from '../engine/api';
import { useEngineStore } from '../state/engine';
import { useUiStore } from '../state/ui';
import { useActiveRun, useCurrentObject } from './current';
import { EngineStatus } from './Status';

const STEP_MS = 650;

/** The active states after `pos` symbols, as the book writes them: q2 for a DFA, {q1,q2} for an NFA. */
function configuration(machine: MachineView, run: RunView, pos: number): string {
  const names = (run.steps[pos]?.active ?? []).map((i) => machine.states[i]?.name ?? '?');
  if (run.kind === 'dfa') return names[0] ?? '?';
  return names.length === 0 ? '∅' : `{${names.join(',')}}`;
}

type Status = 'ready' | 'running' | 'accepted' | 'rejected';

function StatusPill({ status }: { status: Status }) {
  const style: Record<Status, string> = {
    ready: 'bg-surface-3 text-muted',
    running: 'bg-navy-2 text-accent',
    accepted: 'bg-navy text-accent',
    rejected: 'bg-reject/15 text-reject',
  };
  const text: Record<Status, string> = {
    ready: '● ready',
    running: '● running',
    accepted: '✓ accepted',
    rejected: '✗ rejected',
  };
  return (
    <span
      data-testid="verdict"
      data-status={status}
      className={`flex h-7 shrink-0 items-center rounded-full px-3 text-xs font-semibold whitespace-nowrap ${style[status]}`}
    >
      {text[status]}
    </span>
  );
}

/** The input tape: read symbols dimmed, the head on the next one. ε when the input is empty. */
function Tape({ input, pos, big }: { input: readonly string[]; pos: number; big: boolean }) {
  const head = useRef<HTMLLIElement>(null);
  useEffect(() => {
    head.current?.scrollIntoView({ block: 'nearest', inline: 'center' });
  }, [pos]);
  const cell = big ? 'h-12 min-w-12 text-lg' : 'h-9 min-w-[34px] text-base';
  return (
    <ol aria-label="Input tape" className="flex gap-1 overflow-x-auto pb-0.5 font-mono">
      {input.length === 0 && (
        <li
          title="empty input — no symbols to read"
          aria-label="empty input — no symbols to read"
          data-testid="empty-input"
          className={`grid shrink-0 place-items-center rounded-md border border-dashed border-border-strong px-2 text-muted ${cell}`}
        >
          ε
        </li>
      )}
      {input.map((s, i) => (
        <li
          key={i}
          ref={i === pos ? head : undefined}
          aria-current={i === pos ? 'step' : undefined}
          className={`grid shrink-0 place-items-center rounded-md px-2 ${cell} ${
            i < pos
              ? 'bg-surface-2 text-faint'
              : i === pos
                ? 'border-2 border-accent bg-navy text-accent'
                : 'bg-surface-3 text-text'
          }`}
        >
          {s}
        </li>
      ))}
      <li
        ref={pos >= input.length ? head : undefined}
        aria-label="end of input"
        title="end of input"
        className={`grid shrink-0 place-items-center rounded-md border border-dashed px-2 text-muted ${cell} ${
          pos >= input.length ? 'border-accent' : 'border-border-strong'
        }`}
      >
        ⊣
      </li>
    </ol>
  );
}

/** {q1} —a→ {q1,q2} —a→ … : the configurations so far; later ones are still "?". */
function Timeline({ machine, run, pos, go }: { machine: MachineView; run: RunView; pos: number; go: (i: number) => void }) {
  return (
    <ol aria-label="Configurations" className="flex flex-wrap items-center gap-1.5 font-mono text-xs">
      {run.steps.map((_, i) => {
        const known = i <= pos;
        return (
          <li key={i} className="flex items-center gap-1.5">
            {i > 0 && <span className={i <= pos ? 'text-accent' : 'text-dim'}>—{run.input[i - 1]}→</span>}
            <button
              type="button"
              onClick={() => go(i)}
              aria-label={known ? `After ${i} symbols: ${configuration(machine, run, i)}` : `Step ${i}`}
              className={`rounded-md px-2 py-1 ${
                i === pos
                  ? 'border border-accent bg-navy text-accent'
                  : known
                    ? 'bg-surface-2 text-text-2'
                    : 'border border-dashed border-border-strong text-faint'
              }`}
            >
              {known ? configuration(machine, run, i) : '?'}
            </button>
          </li>
        );
      })}
    </ol>
  );
}

function Button({
  label,
  onClick,
  disabled,
  big,
  primary = false,
  children,
}: {
  label: string;
  onClick: () => void;
  disabled: boolean;
  big: boolean;
  primary?: boolean;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={onClick}
      disabled={disabled}
      className={`grid place-items-center rounded-lg font-semibold disabled:opacity-40 ${
        big ? 'h-14 flex-1 text-base' : 'h-9 min-w-9 px-2 text-sm'
      } ${primary ? 'bg-accent text-on-accent hover:opacity-90' : 'border border-border-strong text-text-2 hover:text-text'}`}
    >
      {children}
    </button>
  );
}

/**
 * Runs the selected machine on one input string (docs/plan/05 §4). `strip` is the desktop dock;
 * `stack` is the tablet sheet and the phone's Run tab (48px cells, big buttons).
 */
export function RunPanel({ variant = 'stack' }: { variant?: 'strip' | 'stack' }) {
  const object = useCurrentObject();
  const { run, pos } = useActiveRun(object);
  const storeRun = useEngineStore((s) => s.run);
  const runObject = useEngineStore((s) => s.runObject);
  const runInput = useEngineStore((s) => s.runInput);
  const runError = useEngineStore((s) => s.runError);
  const pending = useEngineStore((s) => s.runPending);
  const setRunInput = useEngineStore((s) => s.setRunInput);
  const setRunPos = useEngineStore((s) => s.setRunPos);
  const startRun = useEngineStore((s) => s.startRun);
  const showRun = useEngineStore((s) => s.showRun);
  const [playing, setPlaying] = useState(false);

  const n = run?.input.length ?? 0;
  // The input box shows what the user typed for this machine, else the program's own run.
  const input = runObject === object?.name ? runInput : (run?.input.join('') ?? '');

  const go = (i: number) => {
    if (!run || pending) return;
    if (storeRun !== run) showRun(run);
    setRunPos(Math.max(0, Math.min(n, i)));
  };

  useEffect(() => {
    if (!playing) return;
    const timer = window.setInterval(() => {
      const { runPos, run: r, setRunPos: set } = useEngineStore.getState();
      if (!r || runPos >= r.input.length) {
        setPlaying(false);
        return;
      }
      set(runPos + 1);
    }, STEP_MS);
    return () => window.clearInterval(timer);
  }, [playing]);

  if (!object) return <EngineStatus empty="Declare a dfa or nfa, then run it on an input string." />;
  const machine = object.machine;
  if (!machine || machine.kind === 'gnfa') {
    return (
      <p className="p-4 text-sm text-muted">
        {object.name} is a {object.kind === 'regex' ? 'regular expression' : 'GNFA'}. Runs work on DFAs and NFAs: try → DFA, or{' '}
        <code className="font-mono text-text">let N = nfa({object.name})</code>.
      </p>
    );
  }

  const submit = () => {
    setPlaying(false);
    void startRun(useUiStore.getState().source, object.name, input);
  };
  const done = run !== null && pos >= n;
  const status: Status = !run ? 'ready' : done ? (run.accepted ? 'accepted' : 'rejected') : pos > 0 || playing ? 'running' : 'ready';
  const at = run?.steps[pos];
  const big = variant === 'stack';
  const togglePlay = () => {
    if (done) go(0);
    else if (storeRun !== run && run) showRun(run);
    setPlaying(!playing);
  };

  const inputBox = (
    <form
      className="flex min-w-0 items-center gap-2"
      onSubmit={(e) => {
        e.preventDefault();
        submit();
      }}
    >
      <label className="flex min-w-0 flex-1 items-center gap-2 font-mono text-xs text-muted">
        <span className="shrink-0">run {object.name} on</span>
        <input
          value={input}
          onChange={(e) => setRunInput(e.target.value)}
          onFocus={() => {
            if (runObject !== object.name) useEngineStore.setState({ runObject: object.name, runInput: input });
          }}
          aria-label={`Input for ${object.name}`}
          placeholder="ε"
          autoCapitalize="off"
          autoCorrect="off"
          spellCheck={false}
          className={`min-w-0 flex-1 rounded-lg border border-border-strong bg-surface-2 px-2.5 font-mono text-sm text-text outline-none placeholder:text-faint focus:border-accent ${
            big ? 'h-11' : 'h-9'
          }`}
        />
      </label>
      <button
        type="submit"
        className={`shrink-0 rounded-lg border border-accent px-3 text-sm font-semibold text-accent hover:bg-accent/10 ${big ? 'h-11' : 'h-9'}`}
      >
        Run
      </button>
    </form>
  );

  const explanation = (
    <div className="flex min-w-0 flex-col gap-1">
      {runError && (
        <p role="alert" className="text-sm text-reject">
          {runError}
        </p>
      )}
      {at && (
        <>
          <p data-testid="run-message" aria-live="polite" className="font-mono text-[12.5px] text-text-2">
            {at.message}
          </p>
          <p data-testid="run-explain" className="text-xs text-muted">
            {at.explain}
          </p>
        </>
      )}
      {!run && !runError && <p className="text-xs text-muted">Type an input and press Run. Leave it empty to run on ε.</p>}
    </div>
  );

  const keys = (e: React.KeyboardEvent) => {
    if (e.target instanceof HTMLInputElement) return;
    if (e.key === 'ArrowRight') go(pos + 1);
    else if (e.key === 'ArrowLeft') go(pos - 1);
    else if (e.key === 'Home') go(0);
    else if (e.key === 'End') go(n);
    else if (e.key === ' ') togglePlay();
    else return;
    e.preventDefault();
  };

  if (variant === 'strip') {
    return (
      <section aria-label={`Run ${object.name}`} onKeyDown={keys} className="flex h-full items-center gap-5 overflow-x-auto px-4">
        <div className="w-72 shrink-0">{inputBox}</div>
        <div className="flex shrink-0 gap-1">
          <Button label="Step back" onClick={() => go(pos - 1)} disabled={!run || pending || pos === 0} big={false}>
            ◀
          </Button>
          <Button label={playing ? 'Pause' : 'Play'} onClick={togglePlay} disabled={!run || pending} big={false} primary>
            {playing ? '❚❚' : '▶'}
          </Button>
          <Button label="Step forward" onClick={() => go(pos + 1)} disabled={!run || pending || done} big={false}>
            ▶|
          </Button>
          <Button label="Jump to end" onClick={() => go(n)} disabled={!run || pending || done} big={false}>
            ⏭
          </Button>
        </div>
        {run && (
          <div className="flex shrink-0 flex-col gap-1">
            <Tape input={run.input} pos={pos} big={false} />
            <span className="text-[11px] text-muted">
              step {pos} of {n}
            </span>
          </div>
        )}
        {run && (
          <div className="min-w-48 shrink">
            <Timeline machine={machine} run={run} pos={pos} go={go} />
          </div>
        )}
        <div className="max-w-[380px] min-w-56 flex-1">{explanation}</div>
        <StatusPill status={status} />
      </section>
    );
  }

  return (
    <section aria-label={`Run ${object.name}`} onKeyDown={keys} className="flex h-full min-h-0 flex-col gap-3 p-4">
      <div className="flex items-center gap-2">
        <div className="min-w-0 flex-1">{inputBox}</div>
        <span className={`shrink-0 rounded-full px-3 py-1 text-xs font-semibold ${run ? 'bg-navy-2 text-accent' : 'bg-surface-3 text-muted'}`}>
          {run ? `step ${pos} / ${n}` : 'no run'}
        </span>
      </div>
      {run && (
        <div className="flex justify-center">
          <Tape input={run.input} pos={pos} big />
        </div>
      )}
      <div className="rounded-xl border border-border bg-surface-2 p-3">{explanation}</div>
      {run && <Timeline machine={machine} run={run} pos={pos} go={go} />}
      <div className="flex-1" />
      <div className="flex items-center gap-2">
        <StatusPill status={status} />
      </div>
      <div className="flex gap-2.5">
        <Button label="Step back" onClick={() => go(pos - 1)} disabled={!run || pending || pos === 0} big>
          ◀
        </Button>
        <Button label={playing ? 'Pause' : 'Play'} onClick={togglePlay} disabled={!run || pending} big primary>
          {playing ? '❚❚ Pause' : '▶ Play'}
        </Button>
        <Button label="Step forward" onClick={() => go(pos + 1)} disabled={!run || pending || done} big>
          ▶|
        </Button>
      </div>
    </section>
  );
}
