import { useEffect, useRef } from 'react';
import { EXAMPLES } from '../examples';
import { useEngineStore, useObjects } from '../state/engine';
import { useUiStore } from '../state/ui';
import { useCurrentObject } from '../panels/current';

function ObjectPicker() {
  const objects = useObjects();
  const current = useCurrentObject();
  const select = useUiStore((s) => s.select);
  if (objects.length === 0) return null;
  return (
    <label className="flex min-w-0 items-center gap-1.5 text-xs text-muted">
      <span className="hidden sm:inline">Object</span>
      <select
        aria-label="Object"
        value={current?.name ?? ''}
        onChange={(e) => select(e.target.value)}
        className="h-8 min-w-0 rounded-md border border-border bg-surface-2 px-2 font-mono text-sm text-text pointer-coarse:h-10"
      >
        {objects.map((o) => (
          <option key={o.name} value={o.name}>
            {o.name} : {o.kind.toUpperCase()}
          </option>
        ))}
      </select>
    </label>
  );
}

function ThemeToggle() {
  const theme = useUiStore((s) => s.theme);
  const setTheme = useUiStore((s) => s.setTheme);
  const next = theme === 'dark' ? 'light' : 'dark';
  return (
    <button
      type="button"
      onClick={() => setTheme(next)}
      aria-label={`Switch to ${next} theme`}
      title={`Switch to ${next} theme`}
      className="grid h-8 w-8 place-items-center rounded-md text-muted hover:bg-surface-2 hover:text-text pointer-coarse:h-10 pointer-coarse:w-10"
    >
      <span aria-hidden="true">{theme === 'dark' ? '☾' : '☀'}</span>
    </button>
  );
}

/** The example gallery, by book chapter and example number (docs/plan/05 §5). */
function ExamplesDialog() {
  const open = useUiStore((s) => s.examplesOpen);
  const setOpen = useUiStore((s) => s.setExamplesOpen);
  const setSource = useUiStore((s) => s.setSource);
  const dialog = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const d = dialog.current;
    if (!d) return;
    if (open && !d.open) d.showModal();
    if (!open && d.open) d.close();
  }, [open]);

  return (
    <dialog
      ref={dialog}
      aria-labelledby="examples-title"
      onClose={() => setOpen(false)}
      onClick={(e) => {
        if (e.target === dialog.current) setOpen(false);
      }}
      className="m-auto w-[min(32rem,calc(100vw-2rem))] rounded-xl border border-border bg-surface p-0 text-text shadow-2xl backdrop:bg-black/50"
    >
      <div className="flex items-center justify-between border-b border-border px-4 py-3">
        <h2 id="examples-title" className="text-base font-semibold">
          Examples from the book
        </h2>
        <button
          type="button"
          onClick={() => setOpen(false)}
          aria-label="Close"
          className="grid h-9 w-9 place-items-center rounded-md text-muted hover:bg-surface-2"
        >
          ✕
        </button>
      </div>
      <ul className="max-h-[70dvh] overflow-auto py-1">
        {EXAMPLES.map((ex) => (
          <li key={ex.id}>
            <button
              type="button"
              onClick={() => {
                setSource(ex.source);
                useUiStore.setState({ selected: null, step: null, phoneView: 'code' });
                useEngineStore.getState().clearRun();
                setOpen(false);
              }}
              className="flex w-full flex-col px-4 py-2.5 text-left hover:bg-surface-2"
            >
              <span className="text-sm font-semibold">{ex.title}</span>
              <span className="text-sm text-muted">{ex.summary}</span>
            </button>
          </li>
        ))}
      </ul>
      <p className="border-t border-border px-4 py-2 text-xs text-muted">
        Opening an example replaces the code. Undo with Ctrl/⌘ + Z in the editor.
      </p>
    </dialog>
  );
}

export function Header({ compact = false }: { compact?: boolean }) {
  const setExamplesOpen = useUiStore((s) => s.setExamplesOpen);
  return (
    <header className="flex h-12 shrink-0 items-center gap-2 border-b border-border bg-surface px-3 sm:px-4">
      <span aria-hidden="true" className="font-serif text-2xl leading-none text-accent italic">
        δ
      </span>
      <h1 className="font-logo text-lg leading-none tracking-wide text-text">Delta</h1>
      {!compact && <span className="ml-1 hidden text-sm text-muted lg:inline">Automata playground · Sipser, 3rd ed.</span>}
      <div className="ml-auto flex min-w-0 items-center gap-1.5">
        <ObjectPicker />
        <button
          type="button"
          onClick={() => setExamplesOpen(true)}
          className="h-8 shrink-0 rounded-md border border-border px-2.5 text-sm text-text hover:bg-surface-2 pointer-coarse:h-10"
        >
          Examples
        </button>
        <ThemeToggle />
      </div>
      <ExamplesDialog />
    </header>
  );
}
