import { applications, CONVERSIONS } from '../conversions';
import { EXAMPLES } from '../examples';
import { applyConversion } from '../panels/current';
import { TestsPanel } from '../panels/TestsPanel';
import { useEngineStore, useObjects } from '../state/engine';
import { useUiStore, type RailPanel } from '../state/ui';
import { Credit } from './Credit';
import { Icon, type IconName } from './icons';

const ITEMS: readonly { id: RailPanel; label: string; icon: IconName }[] = [
  { id: 'files', label: 'Files', icon: 'files' },
  { id: 'examples', label: 'Book examples', icon: 'book' },
  { id: 'conversions', label: 'Conversions', icon: 'convert' },
  { id: 'tests', label: 'Tests', icon: 'tests' },
];

function RailButton({ id, label, icon }: { id: RailPanel; label: string; icon: IconName }) {
  const rail = useUiStore((s) => s.rail);
  const setRail = useUiStore((s) => s.setRail);
  const open = rail === id;
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      aria-pressed={open}
      onClick={() => setRail(open ? null : id)}
      className={`grid h-[38px] w-[38px] place-items-center rounded-lg ${
        open ? 'bg-surface-3 text-accent' : 'text-muted hover:text-text'
      }`}
    >
      <Icon name={icon} />
    </button>
  );
}

/** The left icon rail: each button opens its drawer beside the editor. */
export function Rail() {
  return (
    <nav aria-label="Tools" className="flex w-[52px] shrink-0 flex-col items-center gap-1 border-r border-border bg-surface pt-2.5">
      {ITEMS.map((i) => (
        <RailButton key={i.id} {...i} />
      ))}
      <div className="flex-1" />
      <div className="mb-2.5">
        <RailButton id="settings" label="Settings" icon="settings" />
      </div>
    </nav>
  );
}

function DrawerHeading({ children }: { children: React.ReactNode }) {
  return <h2 className="px-4 pt-3 pb-2 text-[11px] font-semibold tracking-wider text-muted uppercase">{children}</h2>;
}

function FilesDrawer() {
  const files = useUiStore((s) => s.files);
  const activeId = useUiStore((s) => s.activeId);
  const openFile = useUiStore((s) => s.openFile);
  const newFile = useUiStore((s) => s.newFile);
  return (
    <>
      <DrawerHeading>Files</DrawerHeading>
      <ul className="px-2">
        {files.map((f) => (
          <li key={f.id}>
            <button
              type="button"
              onClick={() => openFile(f.id)}
              className={`w-full rounded-md px-2 py-1.5 text-left font-mono text-xs ${
                f.id === activeId ? 'bg-surface-3 text-text' : 'text-muted hover:text-text'
              }`}
            >
              {f.name}
            </button>
          </li>
        ))}
      </ul>
      <button type="button" onClick={newFile} className="mx-4 mt-2 text-xs text-accent hover:underline">
        + New file
      </button>
      <p className="px-4 pt-3 text-[11px] text-faint">Files are kept in this browser only.</p>
    </>
  );
}

function ExamplesDrawer() {
  const openExample = useUiStore((s) => s.openExample);
  return (
    <>
      <DrawerHeading>Book examples</DrawerHeading>
      <ul className="px-2 pb-3">
        {EXAMPLES.map((ex) => (
          <li key={ex.id}>
            <button
              type="button"
              onClick={() => openExample(ex)}
              className="flex w-full flex-col rounded-md px-2 py-1.5 text-left hover:bg-surface-3"
            >
              <span className="text-[13px] font-medium text-text">{ex.title}</span>
              <span className="text-xs text-muted">{ex.summary}</span>
            </button>
          </li>
        ))}
      </ul>
    </>
  );
}

function ConversionsDrawer() {
  const objects = useObjects();
  return (
    <>
      <DrawerHeading>Conversions</DrawerHeading>
      <ul className="space-y-3 px-4 pb-3">
        {CONVERSIONS.map((c) => {
          const ways = applications(c, objects);
          return (
            <li key={c.fn}>
              <p className="font-mono text-[13px] text-text">
                {c.fn}({c.arity === 1 ? '…' : '…, …'}) <span className="font-sans text-xs text-accent">{c.book}</span>
              </p>
              <p className="text-xs text-muted">{c.doc}</p>
              <div className="mt-1 flex flex-wrap gap-1">
                {ways.length === 0 && <span className="text-[11px] text-faint">No object in this file fits.</span>}
                {ways.slice(0, 6).map((args) => (
                  <button
                    key={args.join(',')}
                    type="button"
                    onClick={() => applyConversion(c, args)}
                    className="rounded-md border border-border-strong px-1.5 py-0.5 font-mono text-[11px] text-text-2 hover:border-accent hover:text-accent"
                  >
                    {c.fn}({args.join(', ')})
                  </button>
                ))}
              </div>
            </li>
          );
        })}
      </ul>
    </>
  );
}

function SettingsDrawer() {
  const theme = useUiStore((s) => s.theme);
  const setTheme = useUiStore((s) => s.setTheme);
  const mode = useEngineStore((s) => s.doc?.mode);
  const version = useEngineStore((s) => s.version);
  return (
    <>
      <DrawerHeading>Settings</DrawerHeading>
      <div className="space-y-4 px-4 pb-4 text-[13px]">
        <fieldset>
          <legend className="mb-1 text-xs text-muted">Theme</legend>
          {(['dark', 'light'] as const).map((t) => (
            <label key={t} className="mr-4 inline-flex items-center gap-1.5 text-text">
              <input type="radio" name="theme" checked={theme === t} onChange={() => setTheme(t)} className="accent-[var(--accent)]" />
              {t === 'dark' ? 'Dark' : 'Light'}
            </label>
          ))}
        </fieldset>
        <div>
          <p className="text-xs text-muted">Checking mode</p>
          <p className="text-text">{mode === 'strict' ? 'Strict' : 'Lenient'}</p>
          <p className="text-[11px] text-faint">
            Write <code className="font-mono">#mode strict</code> at the top of a file to make lenient warnings errors.
          </p>
        </div>
        <div className="text-xs text-muted">
          <p>Engine {version ?? '…'} · WebAssembly</p>
          <Credit />
        </div>
      </div>
    </>
  );
}

/** The drawer for the selected rail item, laid over the editor's left edge. */
export function RailDrawer() {
  const rail = useUiStore((s) => s.rail);
  const setRail = useUiStore((s) => s.setRail);
  if (!rail) return null;
  const label = rail === 'tests' ? 'Tests' : (ITEMS.find((i) => i.id === rail)?.label ?? 'Settings');
  return (
    <aside
      aria-label={label}
      className="absolute inset-y-0 left-0 z-20 w-72 overflow-auto border-r border-border bg-surface shadow-[8px_0_24px_rgb(0_0_0/0.35)]"
      onKeyDown={(e) => {
        if (e.key === 'Escape') setRail(null);
      }}
    >
      {rail === 'files' && <FilesDrawer />}
      {rail === 'examples' && <ExamplesDrawer />}
      {rail === 'conversions' && <ConversionsDrawer />}
      {rail === 'tests' && (
        <>
          <DrawerHeading>Tests</DrawerHeading>
          <TestsPanel />
        </>
      )}
      {rail === 'settings' && <SettingsDrawer />}
    </aside>
  );
}
