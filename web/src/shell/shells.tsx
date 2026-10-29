import { CodePanel, DiagramPanel, MorePanel, RunPanel, StepsPanel, TestsPanel } from '../panels/panels';
import { DeltaPanel, TuplePanel } from '../panels/TuplePanel';
import { useEngineStore } from '../state/engine';
import {
  useUiStore,
  type InspectorTab,
  type PhoneView,
  type SheetTab,
} from '../state/ui';
import { TabPanel, Tabs, type TabItem } from './Tabs';

const INSPECTOR_TABS: readonly TabItem<InspectorTab>[] = [
  { id: 'tuple', label: 'Tuple' },
  { id: 'delta', label: 'δ' },
  { id: 'steps', label: 'Steps' },
  { id: 'tests', label: 'Tests' },
];

const SHEET_TABS: readonly TabItem<SheetTab>[] = [...INSPECTOR_TABS, { id: 'run', label: 'Run' }];

const PHONE_TABS: readonly TabItem<PhoneView>[] = [
  { id: 'code', label: 'Code', icon: '{ }' },
  { id: 'diagram', label: 'Diagram', icon: '◯→' },
  { id: 'tuple', label: 'Tuple', icon: '( )' },
  { id: 'run', label: 'Run', icon: '▶' },
  { id: 'more', label: 'More', icon: '⋯' },
];

function InspectorContent({ tab }: { tab: SheetTab }) {
  switch (tab) {
    case 'tuple':
      return <TuplePanel />;
    case 'delta':
      return <DeltaPanel />;
    case 'steps':
      return <StepsPanel />;
    case 'tests':
      return <TestsPanel />;
    case 'run':
      return <RunPanel />;
  }
}

function Header({ compact = false }: { compact?: boolean }) {
  const hello = useEngineStore((s) => s.hello);
  return (
    <header className="flex h-12 shrink-0 items-center gap-2 border-b border-border bg-surface px-4">
      <span aria-hidden="true" className="font-serif text-2xl leading-none text-accent italic">
        δ
      </span>
      <h1 className="text-base font-semibold tracking-tight text-text">Delta</h1>
      {!compact && (
        <span className="ml-2 text-sm text-muted">Automata playground · Sipser, 3rd ed.</span>
      )}
      {hello.kind === 'ready' && (
        <span className="ml-auto rounded-full border border-border px-2.5 py-0.5 font-mono text-xs text-muted">
          object: {hello.tuple.name}
        </span>
      )}
    </header>
  );
}

function Pane({
  title,
  className = '',
  children,
}: {
  title: string;
  className?: string;
  children: React.ReactNode;
}) {
  return (
    <section aria-label={title} className={`flex min-h-0 min-w-0 flex-col bg-surface ${className}`}>
      <h2 className="shrink-0 border-b border-border px-4 py-2 text-xs font-semibold tracking-wider text-muted uppercase">
        {title}
      </h2>
      <div className="min-h-0 flex-1 overflow-auto">{children}</div>
    </section>
  );
}

/** ≥ 1280 px: Code | Diagram | Inspector, with the Run dock below (§2.1). */
export function DesktopShell() {
  const tab = useUiStore((s) => s.inspectorTab);
  const setTab = useUiStore((s) => s.setInspectorTab);
  return (
    <div data-layout="desktop" className="grid h-dvh grid-rows-[auto_minmax(0,1fr)_auto] bg-bg">
      <Header />
      <main className="grid min-h-0 grid-cols-[minmax(320px,1fr)_minmax(0,1.5fr)_minmax(340px,1fr)] gap-px bg-border">
        <Pane title="Code">
          <CodePanel />
        </Pane>
        <Pane title="Diagram">
          <DiagramPanel />
        </Pane>
        <section aria-label="Inspector" className="flex min-h-0 min-w-0 flex-col bg-surface">
          <Tabs
            idPrefix="inspector"
            label="Inspector"
            items={INSPECTOR_TABS}
            value={tab}
            onChange={setTab}
            className="shrink-0 border-b border-border px-2 py-1.5"
          />
          <TabPanel idPrefix="inspector" activeId={tab} className="min-h-0 flex-1 overflow-auto">
            <InspectorContent tab={tab} />
          </TabPanel>
        </section>
      </main>
      <section aria-label="Run" className="h-28 border-t border-border bg-surface">
        <RunPanel />
      </section>
    </div>
  );
}

/** 768–1279 px: two panes plus a bottom sheet for Inspector and Run (§2.2). */
export function TabletShell() {
  const tab = useUiStore((s) => s.sheetTab);
  const setTab = useUiStore((s) => s.setSheetTab);
  const open = useUiStore((s) => s.sheetOpen);
  const setOpen = useUiStore((s) => s.setSheetOpen);
  return (
    <div data-layout="tablet" className="flex h-dvh flex-col bg-bg">
      <Header />
      {/* Landscape: Code | Diagram. Portrait: Diagram on top, Code below. */}
      <main className="flex min-h-0 flex-1 flex-col gap-px bg-border landscape:flex-row">
        <Pane title="Code" className="flex-1 portrait:order-2">
          <CodePanel />
        </Pane>
        <Pane title="Diagram" className="flex-1 portrait:order-1">
          <DiagramPanel />
        </Pane>
      </main>
      <section
        aria-label="Inspector and run"
        className="shrink-0 rounded-t-xl border-t border-border bg-surface shadow-[0_-4px_16px_rgb(0_0_0/0.06)]"
      >
        <div className="flex items-center gap-2 px-2 py-1.5">
          <button
            type="button"
            aria-expanded={open}
            aria-controls="sheet-panel"
            aria-label={open ? 'Collapse panel' : 'Expand panel'}
            onClick={() => setOpen(!open)}
            className="flex h-11 w-11 items-center justify-center rounded-md text-muted hover:bg-surface-2"
          >
            <span aria-hidden="true" className="h-1.5 w-8 rounded-full bg-border" />
          </button>
          <Tabs
            idPrefix="sheet"
            label="Inspector and run"
            items={SHEET_TABS}
            value={tab}
            onChange={(t) => {
              setTab(t);
              setOpen(true);
            }}
          />
        </div>
        {open && (
          <TabPanel
            idPrefix="sheet"
            activeId={tab}
            className="h-[38dvh] overflow-auto border-t border-border"
          >
            <InspectorContent tab={tab} />
          </TabPanel>
        )}
      </section>
    </div>
  );
}

/** < 768 px: one view at a time with a bottom tab bar (§2.3). */
export function PhoneShell() {
  const view = useUiStore((s) => s.phoneView);
  const setView = useUiStore((s) => s.setPhoneView);
  // While typing, the tab bar steps aside so the symbol bar sits right on the keyboard.
  const typing = useUiStore((s) => s.editorFocused) && view === 'code';
  return (
    <div data-layout="phone" className="flex h-dvh flex-col bg-bg">
      <Header compact />
      <TabPanel idPrefix="phone" activeId={view} className="min-h-0 flex-1 overflow-auto bg-surface">
        {view === 'code' && <CodePanel />}
        {view === 'diagram' && <DiagramPanel />}
        {view === 'tuple' && <TuplePanel />}
        {view === 'run' && <RunPanel />}
        {view === 'more' && <MorePanel />}
      </TabPanel>
      {!typing && (
        <nav aria-label="Views" className="shrink-0 border-t border-border bg-surface pb-[env(safe-area-inset-bottom)]">
          <Tabs idPrefix="phone" label="Views" items={PHONE_TABS} value={view} onChange={setView} variant="bar" />
        </nav>
      )}
    </div>
  );
}
