import { useEffect, useRef } from 'react';
import { useUiStore } from '../state/ui';
import { useCurrentObject } from './current';
import { EngineStatus } from './Status';

/** Scrubs through the trace of the conversion that made the selected object (docs/plan/05 §3). */
export function StepsPanel() {
  const object = useCurrentObject();
  const step = useUiStore((s) => s.step);
  const setStep = useUiStore((s) => s.setStep);
  const current = useRef<HTMLLIElement>(null);

  useEffect(() => {
    current.current?.scrollIntoView({ block: 'nearest' });
  }, [step]);

  if (!object) return <EngineStatus empty="Convert something, e.g. let D = dfa(N), to see its steps." />;
  const trace = object.trace;
  if (!trace || trace.steps.length === 0) {
    return (
      <p className="p-4 text-sm text-muted">
        {object.name} was written by hand, so there are no construction steps. Objects made with{' '}
        <code className="font-mono text-text">let</code> (e.g. <code className="font-mono text-text">let D = dfa(N)</code>) can be
        scrubbed step by step.
      </p>
    );
  }

  const n = trace.steps.length;
  // Position n is "the finished result" (step = null).
  const index = step ?? n;
  const go = (i: number) => {
    const clamped = Math.max(0, Math.min(n, i));
    setStep(clamped === n ? null : clamped);
  };
  const shown = step === null ? null : trace.steps[step];

  return (
    <section
      aria-label="Construction steps"
      className="h-full overflow-auto"
      onKeyDown={(e) => {
        if (e.target instanceof HTMLInputElement) return;
        if (e.key === 'ArrowRight') go(index + 1);
        else if (e.key === 'ArrowLeft') go(index - 1);
        else if (e.key === 'Home') go(0);
        else if (e.key === 'End') go(n);
        else return;
        e.preventDefault();
      }}
    >
      <div className="sticky top-0 z-10 space-y-2 border-b border-border bg-surface p-3">
        <p className="text-sm font-semibold text-text">{trace.title}</p>
        <div className="flex items-center gap-1">
          <StepButton label="First step" onClick={() => go(0)} disabled={index === 0}>
            ⏮
          </StepButton>
          <StepButton label="Previous step" onClick={() => go(index - 1)} disabled={index === 0}>
            ◀
          </StepButton>
          <input
            type="range"
            min={0}
            max={n}
            value={index}
            aria-label="Step"
            aria-valuetext={step === null ? 'Result' : `Step ${step + 1} of ${n}`}
            onChange={(e) => go(Number(e.target.value))}
            className="mx-1 min-w-0 flex-1 accent-[var(--accent)]"
          />
          {/* At the finished result, "next" starts the walkthrough from step 1. */}
          <StepButton
            label={index === n ? 'Start from step 1' : 'Next step'}
            onClick={() => go(index === n ? 0 : index + 1)}
          >
            ▶
          </StepButton>
          <StepButton label="Result" onClick={() => go(n)} disabled={index === n}>
            ⏭
          </StepButton>
        </div>
        <p aria-live="polite" className="text-xs text-muted" data-testid="step-counter">
          {step === null ? `Result · ${n} steps` : `Step ${step + 1} of ${n}`}
          {trace.truncated && ' · trace truncated by the step budget; the result is complete'}
        </p>
        {shown && (
          <div className="rounded-md bg-surface-2 p-2.5" data-testid="step-message">
            <p className="text-xs font-semibold text-accent">{shown.rule}</p>
            <p className="mt-1 font-mono text-[13px] whitespace-pre-line text-text">{shown.message}</p>
          </div>
        )}
      </div>
      <ol className="py-1">
        {trace.steps.map((s, i) => (
          <li key={i} ref={i === step ? current : undefined}>
            <button
              type="button"
              onClick={() => go(i)}
              aria-current={i === step ? 'step' : undefined}
              className={`flex w-full gap-2 px-3 py-1.5 text-left text-[13px] ${
                i === step ? 'bg-selection text-text' : 'text-muted hover:bg-surface-2 hover:text-text'
              }`}
            >
              <span className="w-6 shrink-0 text-right font-mono tabular-nums">{i + 1}</span>
              <span className="line-clamp-2 font-mono whitespace-pre-line">{s.message}</span>
            </button>
          </li>
        ))}
      </ol>
    </section>
  );
}

export function StepButton({
  label,
  onClick,
  disabled = false,
  children,
}: {
  label: string;
  onClick: () => void;
  disabled?: boolean;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={onClick}
      disabled={disabled}
      className="grid h-9 w-9 shrink-0 place-items-center rounded-md border border-border text-sm text-text hover:bg-surface-2 disabled:opacity-40 pointer-coarse:h-11 pointer-coarse:w-11"
    >
      {children}
    </button>
  );
}
