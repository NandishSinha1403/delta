import { useEffect } from 'react';
import { DesktopShell, PhoneShell, TabletShell } from './shell/shells';
import { useLayout } from './shell/useLayout';
import { useEngineStore } from './state/engine';
import { useUiStore } from './state/ui';

// Re-evaluate shortly after typing stops, not on every keystroke.
const DEBOUNCE_MS = 120;

export function App() {
  const layout = useLayout();
  const source = useUiStore((s) => s.source);
  const theme = useUiStore((s) => s.theme);
  const evaluate = useEngineStore((s) => s.evaluate);

  useEffect(() => {
    const t = window.setTimeout(() => void evaluate(source), DEBOUNCE_MS);
    return () => window.clearTimeout(t);
  }, [source, evaluate]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  if (layout === 'desktop') return <DesktopShell />;
  if (layout === 'tablet') return <TabletShell />;
  return <PhoneShell />;
}
