import { useEngineStore, useObjects } from '../state/engine';
import { useUiStore } from '../state/ui';
import { useCurrentObject } from '../panels/current';
import { Icon } from './icons';

/** The red δ tile and the wordmark. Brand red appears nowhere else. */
export function Logo({ wordmark = true }: { wordmark?: boolean }) {
  return (
    <div className="flex shrink-0 items-center gap-2.5">
      <div
        aria-hidden="true"
        className="grid h-[30px] w-[30px] place-items-center rounded-lg bg-brand font-serif text-[22px] leading-none font-bold text-[#f1faee] italic"
      >
        δ
      </div>
      {wordmark && <h1 className="font-logo text-xl leading-none tracking-wider text-text">Delta</h1>}
      {!wordmark && <h1 className="sr-only">Delta</h1>}
    </div>
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
      className="grid h-8 w-8 shrink-0 place-items-center rounded-lg border border-border-strong text-muted hover:text-text pointer-coarse:h-10 pointer-coarse:w-10"
    >
      <Icon name={theme === 'dark' ? 'moon' : 'sun'} />
    </button>
  );
}

/** Open files as tabs; the active one is underlined in powder blue. */
function FileTabs() {
  const files = useUiStore((s) => s.files);
  const activeId = useUiStore((s) => s.activeId);
  const openFile = useUiStore((s) => s.openFile);
  const closeFile = useUiStore((s) => s.closeFile);
  const newFile = useUiStore((s) => s.newFile);
  return (
    <nav aria-label="Open files" className="flex h-[52px] min-w-0 items-stretch overflow-x-auto">
      {files.map((f) => {
        const active = f.id === activeId;
        return (
          <div
            key={f.id}
            className={`group flex shrink-0 items-center border-b-2 ${active ? 'border-accent' : 'border-transparent'}`}
          >
            <button
              type="button"
              onClick={() => openFile(f.id)}
              aria-current={active ? 'page' : undefined}
              className={`flex h-full items-center gap-2 pr-1 pl-3.5 font-mono text-xs ${active ? 'text-text' : 'text-muted hover:text-text'}`}
            >
              {active && <span aria-hidden="true">●</span>}
              {f.name}
            </button>
            <button
              type="button"
              onClick={() => closeFile(f.id)}
              aria-label={`Close ${f.name}`}
              className="mr-2 grid h-5 w-5 place-items-center rounded text-faint opacity-0 group-hover:opacity-100 hover:bg-surface-3 hover:text-text focus-visible:opacity-100"
            >
              ×
            </button>
          </div>
        );
      })}
      <button
        type="button"
        onClick={newFile}
        aria-label="New file"
        className="shrink-0 px-3 text-base text-muted hover:text-text"
      >
        +
      </button>
    </nav>
  );
}

function SearchButton({ compact = false }: { compact?: boolean }) {
  const setPaletteOpen = useUiStore((s) => s.setPaletteOpen);
  if (compact) {
    return (
      <button
        type="button"
        onClick={() => setPaletteOpen(true)}
        aria-label="Search commands, examples and objects"
        className="grid h-10 w-10 shrink-0 place-items-center rounded-lg border border-border-strong text-muted"
      >
        <Icon name="search" />
      </button>
    );
  }
  return (
    <button
      type="button"
      onClick={() => setPaletteOpen(true)}
      className="flex h-8 w-full max-w-[420px] items-center gap-2.5 rounded-lg border border-border-strong bg-surface-2 px-3 text-[13px] text-muted hover:text-text"
    >
      <Icon name="search" />
      <span className="flex-1 truncate text-left">Search examples, conversions, objects…</span>
      <kbd className="rounded border border-border-strong px-1.5 py-0.5 font-mono text-[11px] text-text-2">⌘K</kbd>
    </button>
  );
}

/** Desktop and tablet: logo, open files, search, the book badge, theme. */
export function TopBar() {
  return (
    <header className="flex h-[52px] shrink-0 items-center gap-4 border-b border-border bg-surface px-4">
      <Logo />
      <FileTabs />
      <div className="flex min-w-40 flex-1 justify-center">
        <SearchButton />
      </div>
      <span className="hidden shrink-0 rounded-full border border-border-strong px-2 py-1 font-mono text-[11px] text-text-2 lg:inline">
        Sipser 3e · Ch 1.1–1.3
      </span>
      <ThemeToggle />
    </header>
  );
}

/** Phone: logo tile, the object picker, test results, search and theme. */
export function PhoneTopBar() {
  const objects = useObjects();
  const current = useCurrentObject();
  const select = useUiStore((s) => s.select);
  // Select the stable `results` array; deriving inside the selector would re-render forever.
  const cases = useEngineStore((s) => s.doc?.results)?.flatMap((r) => r.cases);
  const passed = cases?.filter((c) => c.expected === c.actual).length ?? 0;
  return (
    <header className="flex h-14 shrink-0 items-center gap-2 border-b border-border bg-surface px-3">
      <Logo wordmark={false} />
      {objects.length > 0 && (
        <select
          aria-label="Object"
          value={current?.name ?? ''}
          onChange={(e) => select(e.target.value)}
          className="h-10 min-w-0 rounded-lg border border-border-strong bg-surface-2 px-2 font-mono text-[13px] text-text"
        >
          {objects.map((o) => (
            <option key={o.name} value={o.name}>
              {o.name} {o.kind.toUpperCase()}
            </option>
          ))}
        </select>
      )}
      <div className="flex-1" />
      {cases && cases.length > 0 && (
        <span className={`font-mono text-xs ${passed === cases.length ? 'text-text' : 'text-reject'}`}>
          {passed === cases.length ? '✓' : '✗'} {passed}/{cases.length}
        </span>
      )}
      <SearchButton compact />
      <ThemeToggle />
    </header>
  );
}
