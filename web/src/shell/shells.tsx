import { CodePanel, MorePanel } from '../panels/panels';
import { DiagramPanel } from '../panels/DiagramPanel';
import { RunPanel } from '../panels/RunPanel';
import { StepsPanel } from '../panels/StepsPanel';
import { ProblemsPanel, TestsPanel } from '../panels/TestsPanel';
import { DeltaPanel, TuplePanel } from '../panels/TuplePanel';
import { useEngineStore } from '../state/engine';
import { useUiStore, type InspectorTab, type PhoneView, type SheetTab } from '../state/ui';
import { PanelBoundary } from './PanelBoundary';
import { Rail, RailDrawer } from './Rail';
import { StatusBar } from './StatusBar';
import { TabPanel, Tabs, type TabItem } from './Tabs';
import { PhoneTopBar, TopBar } from './TopBar';
import { useKeyboardOpen } from './useLayout';

/** Tuple · δ table · Steps · Tests n/n · Problems. */
function useInspectorTabs(): TabItem<InspectorTab>[] {
  // Select the stable `results` array; deriving inside the selector would re-render forever.
  const cases = useEngineStore((s) => s.doc?.results)?.flatMap((r) => r.cases);
  const passed = cases?.filter((c) => c.expected === c.actual).length ?? 0;
  return [
    { id: 'tuple', label: 'Tuple' },
    { id: 'delta', label: 'δ table' },
    { id: 'steps', label: 'Steps' },
    { id: 'tests', label: cases && cases.length > 0 ? `Tests ${passed}/${cases.length}` : 'Tests' },
    { id: 'problems', label: 'Problems' },
  ];
}

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

function Region({ label, className = '', children }: { label: string; className?: string; children: React.ReactNode }) {
  return (
    <section aria-label={label} className={`flex min-h-0 min-w-0 flex-col ${className}`}>
      <PanelBoundary name={label}>{children}</PanelBoundary>
    </section>
  );
}

/** ≥ 1280 px: rail | code | diagram | inspector, the run strip and the status bar (redesign). */
export function DesktopShell() {
  const tab = useUiStore((s) => s.inspectorTab);
  const setTab = useUiStore((s) => s.setInspectorTab);
  const tabs = useInspectorTabs();
  return (
    <div data-layout="desktop" className="grid h-dvh grid-rows-[auto_minmax(0,1fr)_auto_auto] bg-bg text-[13px]">
      <TopBar />
      <main className="flex min-h-0">
        <Rail />
        <div className="relative flex min-h-0 w-[clamp(340px,32vw,480px)] shrink-0 flex-col border-r border-border">
          <RailDrawer />
          <Region label="Code" className="h-full">
            <CodePanel />
          </Region>
        </div>
        <Region label="Diagram" className="flex-1 border-r border-border">
          <DiagramPanel header />
        </Region>
        <section aria-label="Inspector" className="flex min-h-0 w-[340px] shrink-0 flex-col bg-surface">
          <Tabs
            idPrefix="inspector"
            label="Inspector"
            items={tabs}
            value={tab}
            onChange={setTab}
            className="shrink-0 overflow-x-auto border-b border-border px-1"
          />
          <TabPanel idPrefix="inspector" activeId={tab} className="min-h-0 flex-1 overflow-auto">
            <InspectorContent tab={tab} />
          </TabPanel>
        </section>
      </main>
      <section aria-label="Run" className="h-[88px] border-t border-border bg-surface">
        <PanelBoundary name="Run">
          <RunPanel variant="strip" />
        </PanelBoundary>
      </section>
      <StatusBar />
    </div>
  );
}

/** 768–1279 px: code | diagram, with a bottom sheet for the inspector and the run (§2.2). */
export function TabletShell() {
  const tab = useUiStore((s) => s.sheetTab);
  const setTab = useUiStore((s) => s.setSheetTab);
  const open = useUiStore((s) => s.sheetOpen);
  const setOpen = useUiStore((s) => s.setSheetOpen);
  const sheetTabs: TabItem<SheetTab>[] = [{ id: 'run', label: 'Run' }, ...useInspectorTabs()];
  return (
    <div data-layout="tablet" className="flex h-dvh flex-col bg-bg text-[13px]">
      <TopBar />
      {/* Landscape: Code | Diagram. Portrait: Diagram on top, Code below. */}
      <main className="flex min-h-0 flex-1 flex-col landscape:flex-row">
        <Region label="Code" className="flex-1 border-border portrait:order-2 portrait:border-t landscape:border-r">
          <CodePanel />
        </Region>
        <Region label="Diagram" className="flex-1 portrait:order-1">
          <DiagramPanel header />
        </Region>
      </main>
      <section aria-label="Inspector and run" className="shrink-0 rounded-t-xl border-t border-border bg-surface shadow-[0_-8px_24px_rgb(0_0_0/0.3)]">
        <div className="flex items-center gap-1 px-2">
          <button
            type="button"
            aria-expanded={open}
            aria-controls="sheet-panel"
            aria-label={open ? 'Collapse panel' : 'Expand panel'}
            onClick={() => setOpen(!open)}
            className="flex h-11 w-11 shrink-0 items-center justify-center rounded-md"
          >
            <span aria-hidden="true" className="h-1.5 w-8 rounded-full bg-raised" />
          </button>
          <Tabs
            idPrefix="sheet"
            label="Inspector and run"
            items={sheetTabs}
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

/** < 768 px: one view at a time with a bottom tab bar (§2.3); Run is its own tab. */
export function PhoneShell() {
  const view = useUiStore((s) => s.phoneView);
  const setView = useUiStore((s) => s.setPhoneView);
  // While typing, the tab bar steps aside so the symbol bar sits right on the keyboard. Focus alone
  // is not enough: Android's Back closes the keyboard without blurring the editor.
  const keyboard = useKeyboardOpen();
  const typing = useUiStore((s) => s.editorFocused) && keyboard && view === 'code';
  return (
    <div data-layout="phone" className="flex h-dvh flex-col bg-bg text-[13px]">
      <PhoneTopBar />
      <TabPanel idPrefix="phone" activeId={view} className="min-h-0 flex-1 overflow-auto bg-surface">
        <PanelBoundary key={view} name={view}>
          {view === 'code' && <CodePanel />}
          {view === 'diagram' && <DiagramPanel />}
          {view === 'tuple' && <TuplePanel />}
          {view === 'run' && (
            <div className="flex h-full min-h-0 flex-col">
              <div className="h-[38%] shrink-0 border-b border-border">
                <DiagramPanel legend={false} />
              </div>
              <div className="min-h-0 flex-1 overflow-auto">
                <RunPanel variant="stack" />
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
