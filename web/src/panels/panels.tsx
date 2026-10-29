import type { EditorView } from '@codemirror/view';
import { useState } from 'react';
import { CodeEditor } from '../editor/CodeEditor';
import { SymbolBar } from '../editor/SymbolBar';
import { Tabs, TabPanel, type TabItem } from '../shell/Tabs';
import { DiagramPanel } from './DiagramPanel';
import { StepsPanel } from './StepsPanel';
import { ProblemsPanel, TestsPanel } from './TestsPanel';
import { DeltaPanel } from './TuplePanel';

export function CodePanel() {
  const [view, setView] = useState<EditorView | null>(null);
  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="min-h-0 flex-1">
        <CodeEditor onView={setView} />
      </div>
      <SymbolBar view={view} />
    </div>
  );
}

type MoreTab = 'delta' | 'steps' | 'tests' | 'problems';

const MORE_TABS: readonly TabItem<MoreTab>[] = [
  { id: 'steps', label: 'Steps' },
  { id: 'delta', label: 'δ-table' },
  { id: 'tests', label: 'Tests' },
  { id: 'problems', label: 'Problems' },
];

/** Phone only: the inspector views that do not get their own tab. */
export function MorePanel() {
  const [tab, setTab] = useState<MoreTab>('steps');
  return (
    <div className="flex h-full min-h-0 flex-col">
      <Tabs
        idPrefix="more"
        label="More views"
        items={MORE_TABS}
        value={tab}
        onChange={setTab}
        className="shrink-0 overflow-x-auto border-b border-border px-2 py-1.5"
      />
      <TabPanel idPrefix="more" activeId={tab} className="min-h-0 flex-1 overflow-auto">
        {tab === 'steps' && (
          <div className="flex h-full min-h-0 flex-col">
            <div className="h-[40%] shrink-0 border-b border-border">
              <DiagramPanel caption={false} />
            </div>
            <div className="min-h-0 flex-1">
              <StepsPanel />
            </div>
          </div>
        )}
        {tab === 'delta' && <DeltaPanel />}
        {tab === 'tests' && <TestsPanel />}
        {tab === 'problems' && <ProblemsPanel />}
      </TabPanel>
    </div>
  );
}
