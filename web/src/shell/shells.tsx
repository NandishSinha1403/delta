import { CodePanel, MorePanel } from '../panels/panels';
import { DiagramPanel } from '../panels/DiagramPanel';
import { RunPanel } from '../panels/RunPanel';
import { StepsPanel } from '../panels/StepsPanel';
import { ProblemsPanel, TestsPanel } from '../panels/TestsPanel';
import { DeltaPanel, TuplePanel } from '../panels/TuplePanel';
import { useEngineStore } from '../state/engine';
import { useUiStore, type InspectorTab, type PhoneView, type SheetTab } from '../state/ui';
import { Header } from './Header';
import { PanelBoundary } from './PanelBoundary';
import { TabPanel, Tabs, type TabItem } from './Tabs';
import { useKeyboardOpen } from './useLayout';

const INSPECTOR_TABS: readonly TabItem<InspectorTab>[] = [
  { id: 'tuple', label: 'Tuple' },
  { id: 'delta', label: 'δ' },
  { id: 'steps', label: 'Steps' },
  { id: 'tests', label: 'Tests' },
  { id: 'problems', label: 'Problems' },
];

const SHEET_TABS: readonly TabItem<SheetTab>[] = [{ id: 'run', label: 'Run' }, ...INSPECTOR_TABS];

const PHONE_TABS: readonly TabItem<PhoneView>[] = [
  { id: 'code', label: 'Code', icon: '{ }' },
  { id: 'diagram', label: 'Diagram', icon: '◯→' },
  { id: 'tuple', label: 'Tuple', icon: '( )' },
  { id: 'run', label: 'Run', icon: '▶' },
  { id: 'more', label: 'More', icon: '⋯' },
];

function InspectorContent({ tab }: { tab: SheetTab }) {
  return (
    <PanelBoundary key={tab} name={tab}>
      <InspectorView tab={tab} />
    </PanelBoundary>
  );
}

function InspectorView({ tab }: { tab: SheetTab }) {
  switch (tab) {
    case 'tuple':
      return <TuplePanel />;
    case 'delta':
      return <DeltaPanel />;
    case 'steps':
      return <StepsPanel />;
    case 'tests':
      return <TestsPanel />;
    case 'problems':
      return <ProblemsPanel />;
    case 'run':
      return <RunPanel />;
  }
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
      <h2 className="shrink-0 border-b border-border px-4 py-1.5 text-[11px] font-semibold tracking-wider text-muted uppercase">
        {title}
      </h2>
      <div className="min-h-0 flex-1 overflow-auto">
        <PanelBoundary name={title}>{children}</PanelBoundary>
      </div>
    </section>
  );
}

/** Problem counts, test results and the engine version; clicking a count opens its tab. */
function StatusBar() {
  const doc = useEngineStore((s) => s.doc);
  const version = useEngineStore((s) => s.version);
  const status = useEngineStore((s) => s.status);
  const open = (tab: InspectorTab) =>
    useUiStore.setState({ inspectorTab: tab, sheetTab: tab, sheetOpen: true });
  const count = (sev: string) => doc?.diagnostics.filter((d) => d.severity === sev).length ?? 0;
  const cases = doc?.results.flatMap((r) => r.cases) ?? [];
  const passed = cases.filter((c) => c.expected === c.actual).length;
  return (
    <footer className="flex h-7 shrink-0 items-center gap-4 border-t border-border bg-surface px-3 text-xs text-muted">
      <button type="button" onClick={() => open('problems')} className="flex gap-3 hover:text-text">
        <span data-testid="error-count">
          <span className="text-reject">●</span> {count('error')}
        </span>
        <span>
          <span className="text-warn">▲</span> {count('warning')}
        </span>
      </button>
      {cases.length > 0 && (
        <button type="button" onClick={() => open('tests')} className="hover:text-text">
          Tests {passed}/{cases.length}
        </button>
      )}
      <span className="ml-auto">{doc ? `${doc.mode} mode` : ''}</span>
      <span>{status === 'error' ? 'engine error' : version ? `engine ${version}` : 'engine starting…'}</span>
    </footer>
  );
}

/** ≥ 1280 px: Code | Diagram | Inspector, with the Run dock below (§2.1). */
export function DesktopShell() {
  const tab = useUiStore((s) => s.inspectorTab);
  const setTab = useUiStore((s) => s.setInspectorTab);
  return (
    <div data-layout="desktop" className="grid h-dvh grid-rows-[auto_minmax(0,1fr)_auto_auto] bg-bg">
      <Header />
      <main className="grid min-h-0 grid-cols-[minmax(340px,1fr)_minmax(0,1.4fr)_minmax(360px,1fr)] gap-px bg-border">
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
            className="shrink-0 border-b border-border px-2 py-1"
          />
          <TabPanel idPrefix="inspector" activeId={tab} className="min-h-0 flex-1 overflow-auto">
            <InspectorContent tab={tab} />
          </TabPanel>
        </section>
      </main>
      <section aria-label="Run" className="h-36 overflow-auto border-t border-border bg-surface">
        <PanelBoundary name="Run">
          <RunPanel compact />
        </PanelBoundary>
      </section>
      <StatusBar />
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
        className="shrink-0 rounded-t-xl border-t border-border bg-surface shadow-[0_-4px_16px_rgb(0_0_0/0.12)]"
      >
        <div className="flex items-center gap-2 px-2 py-1.5">
          <button
            type="button"
            aria-expanded={open}
            aria-controls="sheet-panel"
            aria-label={open ? 'Collapse panel' : 'Expand panel'}
            onClick={() => setOpen(!open)}
            className="flex h-11 w-11 shrink-0 items-center justify-center rounded-md text-muted hover:bg-surface-2"
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
            className="overflow-x-auto"
          />
        </div>
        {open && (
          <TabPanel idPrefix="sheet" activeId={tab} className="h-[38dvh] overflow-auto border-t border-border">
            <InspectorContent tab={tab} />
          </TabPanel>
        )}
      </section>
      <StatusBar />
    </div>
  );
}

/** < 768 px: one view at a time with a bottom tab bar (§2.3). */
export function PhoneShell() {
  const view = useUiStore((s) => s.phoneView);
  const setView = useUiStore((s) => s.setPhoneView);
  // While typing, the tab bar steps aside so the symbol bar sits right on the keyboard. Focus alone
  // is not enough: Android's Back closes the keyboard without blurring the editor.
  const keyboard = useKeyboardOpen();
  const typing = useUiStore((s) => s.editorFocused) && keyboard && view === 'code';
  return (
    <div data-layout="phone" className="flex h-dvh flex-col bg-bg">
      <Header compact />
      <TabPanel idPrefix="phone" activeId={view} className="min-h-0 flex-1 overflow-auto bg-surface">
        <PanelBoundary key={view} name={view}>
          {view === 'code' && <CodePanel />}
          {view === 'diagram' && <DiagramPanel />}
          {view === 'tuple' && <TuplePanel />}
          {view === 'run' && (
            <div className="flex h-full min-h-0 flex-col">
              <div className="h-[45%] shrink-0 border-b border-border">
                <DiagramPanel />
              </div>
              <div className="min-h-0 flex-1 overflow-auto">
                <RunPanel />
              </div>
            </div>
          )}
          {view === 'more' && <MorePanel />}
        </PanelBoundary>
      </TabPanel>
      {!typing && (
        <nav aria-label="Views" className="shrink-0 border-t border-border bg-surface pb-[env(safe-area-inset-bottom)]">
          <Tabs idPrefix="phone" label="Views" items={PHONE_TABS} value={view} onChange={setView} variant="bar" />
        </nav>
      )}
    </div>
  );
}
