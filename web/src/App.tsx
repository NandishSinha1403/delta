import { useEffect } from 'react';
import { CommandPalette } from './shell/CommandPalette';
import { DesktopShell, PhoneShell, TabletShell } from './shell/shells';
import { useLayout } from './shell/useLayout';
import { useEngineStore } from './state/engine';
import { execute } from './state/execute';
import { useUiStore } from './state/ui';

// Re-evaluate shortly after typing stops, not on every keystroke.
const DEBOUNCE_MS = 150;

export function App() {
  const layout = useLayout();
  const source = useUiStore((s) => s.source);
  const activeId = useUiStore((s) => s.activeId);
  const auto = useUiStore((s) => s.auto);
  const theme = useUiStore((s) => s.theme);
  const evaluate = useEngineStore((s) => s.evaluate);
  const clearRun = useEngineStore((s) => s.clearRun);

  // Auto on: live, re-evaluating shortly after typing stops. Off: only Execute updates.
  useEffect(() => {
    if (!auto) return;
    const t = window.setTimeout(() => void evaluate(source), DEBOUNCE_MS);
    return () => window.clearTimeout(t);
  }, [source, auto, evaluate]);

  // Opening a file (and the first load) always evaluates it; a run belongs to its file.
  useEffect(() => {
    clearRun();
    execute();
  }, [activeId, clearRun]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  // ⌘K / Ctrl+K opens the command palette and ⌘↵ / Ctrl+Enter executes, from anywhere. Capture
  // phase, so they win over the editor's own bindings (Mod-Enter would insert a blank line).
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!(e.metaKey || e.ctrlKey)) return;
      if (e.key.toLowerCase() === 'k') {
        e.preventDefault();
        useUiStore.getState().setPaletteOpen(true);
      } else if (e.key === 'Enter') {
        e.preventDefault();
        e.stopPropagation();
        execute();
      }
    };
    window.addEventListener('keydown', onKey, true);
    return () => window.removeEventListener('keydown', onKey, true);
  }, []);

  return (
    <>
      {layout === 'desktop' ? <DesktopShell /> : layout === 'tablet' ? <TabletShell /> : <PhoneShell />}
      <CommandPalette />
    </>
  );
}
