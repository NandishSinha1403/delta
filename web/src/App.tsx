import { useEffect } from 'react';
import { CommandPalette } from './shell/CommandPalette';
import { DesktopShell, PhoneShell, TabletShell } from './shell/shells';
import { useLayout } from './shell/useLayout';
import { useEngineStore } from './state/engine';
import { useUiStore } from './state/ui';

// Re-evaluate shortly after typing stops, not on every keystroke.
const DEBOUNCE_MS = 150;

export function App() {
  const layout = useLayout();
  const source = useUiStore((s) => s.source);
  const activeId = useUiStore((s) => s.activeId);
  const theme = useUiStore((s) => s.theme);
  const evaluate = useEngineStore((s) => s.evaluate);
  const clearRun = useEngineStore((s) => s.clearRun);

  useEffect(() => {
    const t = window.setTimeout(() => void evaluate(source), DEBOUNCE_MS);
    return () => window.clearTimeout(t);
  }, [source, evaluate]);

  // A run belongs to the file it was started in.
  useEffect(() => {
    clearRun();
  }, [activeId, clearRun]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  // ⌘K / Ctrl+K opens the command palette from anywhere, including the editor.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        useUiStore.getState().setPaletteOpen(true);
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
