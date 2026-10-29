import { Diagram } from '../diagram/Diagram';
import { useUiStore } from '../state/ui';
import { useActiveRun, useCurrentObject } from './current';
import { Math } from './TuplePanel';
import { regexLine } from './tuple-format';
import { runOverlay, traceFrame } from './overlay';
import { EngineStatus } from './Status';

/**
 * The selected object's state diagram. While scrubbing a trace it shows that step; while a run is
 * open on this machine it shows the active states.
 */
export function DiagramPanel({ caption: showCaption = true }: { caption?: boolean }) {
  const object = useCurrentObject();
  const step = useUiStore((s) => s.step);
  const { run, pos } = useActiveRun(object);
  if (!object) return <EngineStatus empty="Declare a dfa or nfa to see its state diagram." />;

  const frame = step !== null ? traceFrame(object, step) : null;
  const caption = step !== null ? object.trace?.steps[step] : undefined;

  if (frame) {
    return (
      <div className="flex h-full flex-col">
        {showCaption && caption && (
          <p className="shrink-0 border-b border-border px-4 py-2 text-xs text-muted">
            <span className="font-semibold text-accent">
              Step {step === null ? '' : step + 1} · {caption.rule}
            </span>{' '}
            <span className="line-clamp-2 whitespace-pre-line">{caption.message}</span>
          </p>
        )}
        <div className="min-h-0 flex-1">
          <Diagram key={`${object.name}-trace`} name={object.name} machine={frame.machine} overlay={frame.overlay} />
        </div>
      </div>
    );
  }

  if (!object.machine) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-3 p-6 text-center">
        <p className="text-xl text-text">
          <Math line={regexLine(object.name, object.regex ?? '')} />
        </p>
        <p className="max-w-sm text-sm text-muted">
          A regular expression has no state diagram. Try <code className="font-mono text-text">let N = nfa({object.name})</code>{' '}
          to see Lemma 1.55 build one.
        </p>
      </div>
    );
  }

  const overlay = run ? runOverlay(run, pos) : {};
  return <Diagram key={object.name} name={object.name} machine={object.machine} overlay={overlay} />;
}
