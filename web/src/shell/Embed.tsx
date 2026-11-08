import { useState } from 'react';
import { CodePanel } from '../panels/panels';
import { DiagramPanel } from '../panels/DiagramPanel';
import { ConsolePanel, LangPanel } from '../panels/LangPanel';
import { TestsPanel } from '../panels/TestsPanel';
import { PanelBoundary } from './PanelBoundary';
import { Tabs, TabPanel, type TabItem } from './Tabs';

type EmbedTab = 'diagram' | 'tests' | 'lang' | 'console';

const TABS: readonly TabItem<EmbedTab>[] = [
  { id: 'diagram', label: 'Diagram' },
  { id: 'tests', label: 'Tests' },
  { id: 'lang', label: 'Lang' },
  { id: 'console', label: 'Console' },
];

/** The embeddable Pad (03-PRODUCTS.md §5): code beside one output view, and a link to the Pad. */
export function EmbedShell({ panel }: { panel: string | null }) {
  const first: EmbedTab = TABS.some((t) => t.id === panel) ? (panel as EmbedTab) : 'diagram';
  const [tab, setTab] = useState<EmbedTab>(first);
  const open = () => {
    const url = window.location.href.replace(/&embed=1/, '').replace(/&panel=[^&]*/, '');
    window.open(url, '_blank', 'noopener');
  };
  return (
    <div data-layout="embed" className="flex h-dvh flex-col bg-bg text-[13px]">
      <main className="flex min-h-0 flex-1 flex-col sm:flex-row">
        <section aria-label="Code" className="flex min-h-0 flex-1 flex-col border-border sm:border-r">
          <PanelBoundary name="Code">
            <CodePanel />
          </PanelBoundary>
        </section>
        <section aria-label="Output" className="flex min-h-0 flex-1 flex-col bg-surface">
          <div className="flex items-center border-b border-border pr-2">
            <Tabs idPrefix="embed" label="Output" items={TABS} value={tab} onChange={setTab} className="min-w-0 flex-1 overflow-x-auto" />
            <button type="button" onClick={open} className="shrink-0 text-xs text-accent hover:underline">
              Open in Delta ↗
            </button>
          </div>
          <TabPanel idPrefix="embed" activeId={tab} className="min-h-0 flex-1 overflow-auto">
            <PanelBoundary key={tab} name={tab}>
              {tab === 'diagram' && <DiagramPanel />}
              {tab === 'tests' && <TestsPanel />}
              {tab === 'lang' && <LangPanel />}
              {tab === 'console' && <ConsolePanel />}
            </PanelBoundary>
          </TabPanel>
        </section>
      </main>
    </div>
  );
}
