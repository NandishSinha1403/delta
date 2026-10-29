import { useEffect, useRef, useState } from 'react';
import { useEngineStore } from '../state/engine';
import { useUiStore } from '../state/ui';
import { showString, useActiveRun, useCurrentObject } from './current';
import { EngineStatus } from './Status';
import { StepButton } from './StepsPanel';

const STEP_MS = 650;

/** Runs the selected machine on an input, one symbol at a time (docs/plan/05 §4). */
export function RunPanel({ compact = false }: { compact?: boolean }) {
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
  const timer = useRef<number | undefined>(undefined);

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
    timer.current = window.setInterval(() => {
      const { runPos, run: r, setRunPos: set } = useEngineStore.getState();
      if (!r || runPos >= r.input.length) {
        setPlaying(false);
        return;
      }
      set(runPos + 1);
    }, STEP_MS);
    return () => window.clearInterval(timer.current);
  }, [playing]);

  if (!object) return <EngineStatus empty="Declare a dfa or nfa, then run it on an input string." />;
  if (!object.machine || object.machine.kind === 'gnfa') {
    return (
      <p className="p-4 text-sm text-muted">
        {object.name} is a {object.kind === 'regex' ? 'regular expression' : 'GNFA'}. Runs work on DFAs and NFAs: try{' '}
        <code className="font-mono text-text">let N = nfa({object.name})</code>.
      </p>
    );
  }

  const submit = () => {
    setPlaying(false);
    void startRun(useUiStore.getState().source, object.name, input);
  };
  const at = run?.steps[pos];
  const done = run !== null && pos >= n;
  const machine = object.machine;

  return (
    <section
      aria-label={`Run ${object.name}`}
      className={`flex h-full min-h-0 gap-3 p-3 ${compact ? 'flex-row items-start' : 'flex-col'}`}
      onKeyDown={(e) => {
        if (e.target instanceof HTMLInputElement) return;
        if (e.key === 'ArrowRight') go(pos + 1);
        else if (e.key === 'ArrowLeft') go(pos - 1);
        else if (e.key === 'Home') go(0);
        else if (e.key === 'End') go(n);
        else if (e.key === ' ') {
          if (done) go(0);
          setPlaying(!playing);
        } else return;
        e.preventDefault();
      }}
    >
      <div className={`flex shrink-0 flex-col gap-2 ${compact ? 'w-80' : ''}`}>
        <form
          className="flex gap-2"
          onSubmit={(e) => {
            e.preventDefault();
            submit();
          }}
        >
          <label className="flex min-w-0 flex-1 items-center gap-2 rounded-md border border-border bg-editor px-2 focus-within:border-accent">
            <span className="shrink-0 font-mono text-xs text-muted">{object.name} on</span>
            <input
              value={input}
              onChange={(e) => setRunInput(e.target.value)}
              onFocus={() => {
                if (runObject !== object.name) useEngineStore.setState({ runObject: object.name, runInput: input });
              }}
              aria-label={`Input for ${object.name}`}
              placeholder="ε (empty)"
              autoCapitalize="off"
              autoCorrect="off"
              spellCheck={false}
              className="h-9 min-w-0 flex-1 bg-transparent font-mono text-sm text-text outline-none placeholder:text-muted pointer-coarse:h-11"
            />
          </label>
          <button
            type="submit"
            className="h-9 shrink-0 rounded-md bg-accent px-4 text-sm font-semibold text-on-accent hover:opacity-90 pointer-coarse:h-11"
          >
            Run
          </button>
        </form>
        <div className="flex items-center gap-1">
          <StepButton label="Back to start" onClick={() => go(0)} disabled={!run || pending || pos === 0}>
            ⏮
          </StepButton>
          <StepButton label="Step back" onClick={() => go(pos - 1)} disabled={!run || pending || pos === 0}>
            ◀
          </StepButton>
          <StepButton
            label={playing ? 'Pause' : 'Play'}
            onClick={() => {
              if (done) go(0);
              else if (storeRun !== run && run) showRun(run);
              setPlaying(!playing);
            }}
            disabled={!run || pending}
          >
            {playing ? '⏸' : '▶'}
          </StepButton>
          <StepButton label="Step forward" onClick={() => go(pos + 1)} disabled={!run || pending || done}>
            ⏭
          </StepButton>
          <StepButton label="Jump to end" onClick={() => go(n)} disabled={!run || pending || done}>
            ⏩
          </StepButton>
          {done && run && (
            <span
              data-testid="verdict"
              className={`ml-2 rounded-full px-2.5 py-0.5 text-xs font-semibold ${
                run.accepted ? 'bg-accept/15 text-accept' : 'bg-reject/15 text-reject'
              }`}
            >
              {run.accepted ? 'Accepted' : 'Rejected'}
            </span>
          )}
        </div>
      </div>

      <div className="flex min-w-0 flex-1 flex-col gap-2">
        {runError && (
          <p role="alert" className="rounded-md border border-reject/40 bg-reject/10 p-2 text-sm text-text">
            {runError}
          </p>
        )}
        {run && (
          <>
            <Tape input={run.input} pos={pos} />
            <p aria-live="polite" className="font-mono text-[13px] text-text" data-testid="run-message">
              {at?.message}
            </p>
            {machine.kind === 'nfa' && at && (
              <p className="text-xs text-muted">
                Fingers on{' '}
                {at.active.length === 0
                  ? '∅ (every branch died)'
                  : at.active.map((i) => machine.states[i]?.name).join(', ')}
              </p>
            )}
          </>
        )}
        {!run && !runError && (
          <p className="text-sm text-muted">
            Type an input and press Run. Leave it empty to run on {showString('')}.
          </p>
        )}
      </div>
    </section>
  );
}

/** The input tape: read symbols dimmed, the head under the next one. */
function Tape({ input, pos }: { input: readonly string[]; pos: number }) {
  const head = useRef<HTMLLIElement>(null);
  useEffect(() => {
    head.current?.scrollIntoView({ block: 'nearest', inline: 'center' });
  }, [pos]);
  const cells = input.length === 0 ? [] : input;
  return (
    <ol aria-label="Input tape" className="flex overflow-x-auto pb-1 font-mono text-base">
      {cells.map((s, i) => (
        <li
          key={i}
          ref={i === pos ? head : undefined}
          aria-current={i === pos ? 'step' : undefined}
          className={`grid h-10 min-w-10 shrink-0 place-items-center border border-border px-2 ${i > 0 ? '-ml-px' : ''} ${
            i < pos ? 'bg-surface-2 text-muted' : i === pos ? 'border-accent bg-accent/15 text-text' : 'text-text'
          }`}
        >
          {s}
        </li>
      ))}
      <li
        ref={pos >= cells.length ? head : undefined}
        className={`grid h-10 min-w-10 shrink-0 place-items-center border border-dashed px-2 text-muted ${
          cells.length > 0 ? '-ml-px' : ''
        } ${pos >= cells.length ? 'border-accent' : 'border-border'}`}
        aria-label="end of input"
      >
        ⊣
      </li>
    </ol>
  );
}
