import type { EditorView } from '@codemirror/view';
import { useState } from 'react';
import { CodeEditor } from '../editor/CodeEditor';
import { SymbolBar } from '../editor/SymbolBar';
import { EmptyPanel } from './EmptyPanel';

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

export function DiagramPanel() {
  return (
    <EmptyPanel title="Diagram">
      The state diagram (book conventions, Graphviz layout) arrives in Phase 1.
    </EmptyPanel>
  );
}

export function RunPanel() {
  return (
    <EmptyPanel title="Run">
      Step-by-step runs on input strings (tape, fingers, stack) arrive in Phase 1.
    </EmptyPanel>
  );
}

export function StepsPanel() {
  return <EmptyPanel title="Steps">Conversion traces you can scrub through arrive in Phase 1.</EmptyPanel>;
}

export function TestsPanel() {
  return <EmptyPanel title="Tests">Results of check commands arrive in Phase 1.</EmptyPanel>;
}

export function MorePanel() {
  return (
    <EmptyPanel title="More">
      The δ-table, Steps, Tests, examples and settings will live here.
    </EmptyPanel>
  );
}
