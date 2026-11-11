import { lazy, Suspense, useEffect, useState } from 'react';
import { EmbedShell } from './shell/Embed';
import { decodeSource, parseHash } from './state/share';
import { CommandPalette } from './shell/CommandPalette';
import { isApple, isPresentShortcut, parseWhat, usePresentStore } from './state/present';
import { DesktopShell, PhoneShell, TabletShell } from './shell/shells';
import { useLayout } from './shell/useLayout';
import { useEngineStore } from './state/engine';
import { execute } from './state/execute';
import { useUiStore } from './state/ui';

// Presentation mode is loaded the first time someone presents.
const Presentation = lazy(() => import('./shell/Presentation'));

// Re-evaluate shortly after typing stops, not on every keystroke.
const DEBOUNCE_MS = 150;

export function App() {
  const layout = useLayout();
  // `#code=…` opens a shared program; `&embed=1` shows the embeddable Pad (03-PRODUCTS.md §5).
  const [hash] = useState(() => parseHash(window.location.hash));
  useEffect(() => {
    const open = (code: string | null) => {
      if (!code) return;
      decodeSource(code)
        .then((source) => useUiStore.getState().openShared(source))
        .catch(() => {});
    };
    open(hash.code);
    // A share link pasted into an open Pad changes only the hash.
    const onHash = () => open(parseHash(window.location.hash).code);
    window.addEventListener('hashchange', onHash);
    return () => window.removeEventListener('hashchange', onHash);
  }, [hash.code]);
  const source = useUiStore((s) => s.source);
  const activeId = useUiStore((s) => s.activeId);
  const auto = useUiStore((s) => s.auto);
  const theme = useUiStore((s) => s.theme);
  const presenting = usePresentStore((s) => s.presenting);
  const evaluate = useEngineStore((s) => s.evaluate);
  const clearRun = useEngineStore((s) => s.clearRun);

  // manual:begin pad-debounce
  // Auto on: live, re-evaluating shortly after typing stops. Off: only Execute updates.
  useEffect(() => {
    if (!auto) return;
    const t = window.setTimeout(() => void evaluate(source), DEBOUNCE_MS);
    return () => window.clearTimeout(t);
  }, [source, auto, evaluate]);
  // manual:end pad-debounce

  // Opening a file (and the first load) always evaluates it; a run belongs to its file.
  useEffect(() => {
    clearRun();
    execute();
  }, [activeId, clearRun]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  // `?present` (or `?present=run|diagram|tuple`) on a share link opens straight into presentation
  // mode, once the shared program has been evaluated.
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    if (!params.has('present') || hash.embed) return;
    const what = parseWhat(params.get('present')) ?? undefined;
    const startWhenReady = () => {
      if (useEngineStore.getState().doc) usePresentStore.getState().start(what);
      else window.setTimeout(startWhenReady, 100);
    };
    startWhenReady();
  }, [hash.embed]);

  // Ctrl+Alt+P (⌘⌥P on a Mac) starts presenting. The browser's own use of the keys (Chrome's
  // Page Setup on a Mac) is cancelled only here, while the Pad has focus.
  useEffect(() => {
    const apple = isApple();
    const onKey = (e: KeyboardEvent) => {
      if (!isPresentShortcut(e, apple)) return;
      e.preventDefault();
      e.stopPropagation();
      const p = usePresentStore.getState();
      if (p.presenting) p.stop();
      else p.start();
    };
    window.addEventListener('keydown', onKey, true);
    return () => window.removeEventListener('keydown', onKey, true);
  }, []);

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

  if (hash.embed) return <EmbedShell panel={hash.panel} />;

  return (
    <>
      {layout === 'desktop' ? <DesktopShell /> : layout === 'tablet' ? <TabletShell /> : <PhoneShell />}
      <CommandPalette />
      {presenting && (
        <Suspense fallback={null}>
          <Presentation />
        </Suspense>
      )}
    </>
  );
}
