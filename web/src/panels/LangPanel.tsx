// The Lang panel (a language's class, sample words, decidable questions) and the Console
// (print output, shown values, runtime errors and budget outcomes): 03-PRODUCTS.md §3.1.
import { useState } from 'react';
import { engine } from '../engine/client';
import { useEngineStore } from '../state/engine';
import { useUiStore } from '../state/ui';
import { lineRange, showString, useCurrentObject } from './current';
import { EngineStatus, Note } from './Status';

/** The class lattice, smallest first (01-LANGUAGE.md §6). */
const LATTICE = ['Finite', 'Regular', 'DCF', 'CF', 'CS', 'Decidable', 'Recognizable', 'Any'] as const;

const DECIDABLE_LABEL: Record<string, string> = {
  yes: '✓ decidable',
  'not-implemented': '◐ decidable, not implemented (use up to n)',
  no: '✗ undecidable (use up to n)',
};

export function LangPanel() {
  const object = useCurrentObject();
  const status = useEngineStore((s) => s.status);
  if (status !== 'ready') return <EngineStatus empty="" />;
  if (!object || !object.class) {
    return <Note>Pick a machine, regex, grammar or lang to see its language class.</Note>;
  }
  const at = LATTICE.indexOf(object.class as (typeof LATTICE)[number]);
  return (
    <section aria-label="Language" className="space-y-4 p-3 text-sm">
      <div>
        <p className="mb-2">
          <span className="font-mono text-text">{object.name}</span> is{' '}
          <span data-testid="lang-class" className="font-semibold text-accent">
            {object.class}
          </span>
          {object.assumed && <span className="ml-2 rounded bg-surface-2 px-1.5 text-[11px] text-warn">assumed</span>}
        </p>
        <ol aria-label="Class lattice" className="flex flex-wrap items-center gap-1 font-mono text-xs">
          {LATTICE.map((c, i) => (
            <li key={c} className="flex items-center gap-1">
              <span
                className={`rounded px-1.5 py-0.5 ${
                  i === at ? 'bg-accent text-on-accent' : i < at ? 'text-dim' : 'text-muted'
                }`}
              >
                {c}
              </span>
              {i < LATTICE.length - 1 && <span className="text-dim">⊂</span>}
            </li>
          ))}
        </ol>
        <p className="mt-1 text-xs text-muted">The class is what Delta can guarantee: an upper bound.</p>
      </div>
      {object.lang && (
        <>
          <div className="grid grid-cols-2 gap-3">
            <WordList title="In the language" words={object.lang.in} tone="text-accept" />
            <WordList title="Not in it" words={object.lang.out} tone="text-reject" />
          </div>
          <div>
            <h3 className="mb-1 text-xs font-semibold text-muted">Questions</h3>
            <ul className="space-y-0.5 text-xs">
              {object.lang.questions.map((q) => (
                <li key={q.name} className="flex justify-between gap-2">
                  <span className="font-mono text-text">{q.name}</span>
                  <span className={q.decidable === 'yes' ? 'text-accept' : 'text-muted'}>
                    {DECIDABLE_LABEL[q.decidable] ?? q.decidable}
                  </span>
                </li>
              ))}
            </ul>
          </div>
        </>
      )}
    </section>
  );
}

function WordList({ title, words, tone }: { title: string; words: string[]; tone: string }) {
  return (
    <div>
      <h3 className="mb-1 text-xs font-semibold text-muted">{title}</h3>
      {words.length === 0 ? (
        <p className="text-xs text-dim">none up to length 3</p>
      ) : (
        <ul className={`flex flex-wrap gap-1 font-mono text-xs ${tone}`}>
          {words.map((w) => (
            <li key={w} className="rounded bg-surface-2 px-1.5">
              {showString(w)}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

/** `print` lines, shown values, and the diagnostic that stopped the run, in program order. */
export function ConsolePanel() {
  const doc = useEngineStore((s) => s.doc);
  const source = useEngineStore((s) => s.docSource);
  const lastMs = useEngineStore((s) => s.lastMs);
  const revealRange = useUiStore((s) => s.revealRange);
  if (!doc) return <EngineStatus empty="" />;
  const lines = doc.outputs.filter((o) => o.kind === 'print' || o.kind === 'show' || o.kind === 'save');
  const runtime = doc.diagnostics.filter((d) => /^(E5|B6)/.test(d.code));
  return (
    <section aria-label="Console" className="p-3 font-mono text-xs">
      {lines.length === 0 && runtime.length === 0 && <p className="text-dim">No output. Use print or show.</p>}
      <ul className="space-y-1">
        {lines.map((o, i) => {
          if (o.kind === 'save') {
            return (
              <li key={i} className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={() => o.line && revealRange(lineRange(source, o.line))}
                  className="w-8 shrink-0 text-right text-dim hover:text-text"
                >
                  {o.line}
                </button>
                <SaveButton output={o} />
              </li>
            );
          }
          const text =
            o.kind === 'print'
              ? String(o.text ?? '')
              : String((o.value as { text?: string } | undefined)?.text ?? JSON.stringify(o.value));
          return (
            <li key={i} className="flex gap-2">
              <button
                type="button"
                onClick={() => o.line && revealRange(lineRange(source, o.line))}
                className="w-8 shrink-0 text-right text-dim hover:text-text"
              >
                {o.line}
              </button>
              <pre data-testid="console-line" className="min-w-0 flex-1 whitespace-pre-wrap text-text">
                {text}
              </pre>
            </li>
          );
        })}
        {runtime.map((d, i) => (
          <li key={`d${i}`} className={d.code.startsWith('B6') ? 'text-warn' : 'text-reject'}>
            {d.code}: {d.message}
          </li>
        ))}
      </ul>
      <p className="mt-3 text-dim">
        {doc.steps.toLocaleString()} steps{lastMs !== null ? ` · ${lastMs} ms` : ''}
      </p>
    </section>
  );
}

const MIME: Record<string, string> = {
  svg: 'image/svg+xml',
  png: 'image/png',
  pdf: 'application/pdf',
  tex: 'text/x-tex',
  txt: 'text/plain',
  json: 'application/json',
};

/** A `save x to "file"` output: the Pad never writes files, it offers the download. */
function SaveButton({ output }: { output: { [key: string]: unknown } }) {
  const path = String(output.path ?? 'delta');
  const format = String(output.format ?? 'txt');
  const name = path.split('/').pop() ?? path;
  const [state, setState] = useState<'idle' | 'busy' | 'error'>('idle');
  const padTheme = useUiStore((s) => s.theme);
  const downloadTheme = useUiStore((s) => s.downloadTheme);
  // `save … (theme: dark)` in the program wins; else the Downloads setting (Match Pad by default).
  const written = output.theme === 'light' || output.theme === 'dark' ? output.theme : null;
  const theme = written ?? (downloadTheme === 'match' ? padTheme : downloadTheme);
  const download = async () => {
    setState('busy');
    try {
      const data = await engine.exportFile(output.value, format, String(output.text ?? ''), theme === 'dark');
      // A Blob copies the bytes, so the worker's buffer can be reused.
      const blob = new Blob([typeof data === 'string' ? data : new Uint8Array(data)], {
        type: MIME[format] ?? 'application/octet-stream',
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = name;
      a.click();
      setTimeout(() => URL.revokeObjectURL(url), 1000);
      setState('idle');
    } catch {
      setState('error');
    }
  };
  return (
    <button
      type="button"
      data-testid="save-download"
      onClick={() => void download()}
      disabled={state === 'busy'}
      className="rounded border border-border px-2 py-0.5 text-text hover:bg-surface-3 disabled:opacity-60"
    >
      {state === 'busy' ? `Preparing ${name}…` : state === 'error' ? `Couldn't make ${name}, try again` : `Download ${name}`}
    </button>
  );
}
