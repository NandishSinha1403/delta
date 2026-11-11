import { useEffect, useRef, useState } from 'react';
import { shortcutLabel, usePresentStore, WHATS, type PresentTheme } from '../state/present';

const THEMES: readonly [PresentTheme, string][] = [
  ['dark', 'Dark'],
  ['light', 'Light'],
  ['match', 'Match Pad'],
];

function Chip({ on, onClick, children }: { on: boolean; onClick: () => void; children: React.ReactNode }) {
  return (
    <button
      type="button"
      aria-pressed={on}
      onClick={onClick}
      className={`h-8 rounded-full border px-3 text-xs ${on ? 'border-accent bg-navy text-text' : 'border-border-strong bg-surface-2 text-text-2 hover:text-text'}`}
    >
      {children}
    </button>
  );
}

/** Present: a panel with what to show, the projector theme and the code option, then Start. */
export function PresentButton() {
  const [open, setOpen] = useState(false);
  const what = usePresentStore((s) => s.what);
  const theme = usePresentStore((s) => s.theme);
  const showCode = usePresentStore((s) => s.showCode);
  const set = usePresentStore((s) => s.set);
  const start = usePresentStore((s) => s.start);
  const box = useRef<HTMLDivElement>(null);

  // Clicks outside, and Escape, close the panel.
  useEffect(() => {
    if (!open) return;
    const onDown = (e: PointerEvent) => {
      if (!box.current?.contains(e.target as Node)) setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setOpen(false);
    };
    window.addEventListener('pointerdown', onDown);
    window.addEventListener('keydown', onKey);
    return () => {
      window.removeEventListener('pointerdown', onDown);
      window.removeEventListener('keydown', onKey);
    };
  }, [open]);

  return (
    <div ref={box} className="relative shrink-0">
      <button
        type="button"
        aria-haspopup="dialog"
        aria-expanded={open}
        onClick={() => setOpen(!open)}
        className={`flex h-8 items-center gap-2 rounded-md border px-3 text-xs ${open ? 'border-accent bg-navy text-text' : 'border-border-strong text-text hover:bg-surface-2'}`}
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" aria-hidden="true">
          <rect x="1.5" y="2.5" width="13" height="9" rx="1" />
          <path d="M8 11.5v2.5M5 14h6" />
        </svg>
        Present
      </button>
      {open && (
        <div
          role="dialog"
          aria-label="Present"
          className="absolute right-0 z-30 mt-1 flex w-[22rem] flex-col gap-4 rounded-xl border border-border-strong bg-surface-2 p-4 shadow-2xl"
        >
          <div className="flex flex-col gap-1">
            <span className="text-sm font-semibold text-text">Present</span>
            <span className="text-xs text-muted">
              Full screen, large type, no editor chrome. Step with the arrow keys or a clicker.
            </span>
          </div>
          <fieldset className="flex flex-col gap-2">
            <legend className="mb-2 text-[11px] tracking-wider text-muted uppercase">Show</legend>
            <div className="flex flex-wrap gap-1.5">
              {WHATS.map(([id, label]) => (
                <Chip key={id} on={what === id} onClick={() => set({ what: id })}>
                  {label}
                </Chip>
              ))}
            </div>
          </fieldset>
          <fieldset className="flex flex-col gap-2">
            <legend className="mb-2 text-[11px] tracking-wider text-muted uppercase">Projector theme</legend>
            <div className="flex flex-wrap gap-1.5">
              {THEMES.map(([id, label]) => (
                <Chip key={id} on={theme === id} onClick={() => set({ theme: id })}>
                  {label}
                </Chip>
              ))}
            </div>
          </fieldset>
          <label className="flex items-center gap-2.5 text-sm text-text">
            <input
              type="checkbox"
              checked={showCode}
              onChange={(e) => set({ showCode: e.target.checked })}
              className="h-4 w-4 accent-[var(--accent)]"
            />
            Code beside the diagram, current line marked
          </label>
          <div className="flex items-center gap-3">
            <button
              type="button"
              onClick={() => {
                setOpen(false);
                start();
              }}
              className="h-10 flex-1 rounded-lg bg-accent text-sm font-semibold text-on-accent hover:opacity-90"
            >
              Start presenting
            </button>
            <kbd className="font-mono text-[11px] text-muted">{shortcutLabel()}</kbd>
          </div>
        </div>
      )}
    </div>
  );
}
