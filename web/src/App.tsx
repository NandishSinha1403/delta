import { useEffect } from 'react';
import { DesktopShell, PhoneShell, TabletShell } from './shell/shells';
import { useLayout } from './shell/useLayout';
import { useEngineStore } from './state/engine';

export function App() {
  const layout = useLayout();
  const loadHello = useEngineStore((s) => s.loadHello);

  useEffect(() => {
    void loadHello();
  }, [loadHello]);

  if (layout === 'desktop') return <DesktopShell />;
  if (layout === 'tablet') return <TabletShell />;
  return <PhoneShell />;
}
