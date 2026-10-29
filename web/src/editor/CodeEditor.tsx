import { lintGutter, setDiagnostics, type Diagnostic } from '@codemirror/lint';
import { EditorSelection, EditorState } from '@codemirror/state';
import { EditorView } from '@codemirror/view';
import { basicSetup } from 'codemirror';
import { useEffect, useRef } from 'react';
import type { DiagnosticView } from '../engine/api';
import { useEngineStore } from '../state/engine';
import { useUiStore } from '../state/ui';
import { codelens, lensesFor, setLenses } from './codelens';
import { delta, deltaCompletion, deltaHover, deltaLanguage } from './delta-lang';

const theme = EditorView.theme({
  '&': { height: '100%', backgroundColor: 'var(--editor)', color: 'var(--syn-text)' },
  // A fixed 24px line box with no extra spacing between lines (the gutter uses the same box).
  '.cm-scroller': { fontFamily: 'var(--font-mono)', fontSize: '14px', lineHeight: '24px' },
  '.cm-line': { padding: '0 4px', lineHeight: '24px' },
  '.cm-gutterElement': { lineHeight: '24px' },
  '.cm-content': { caretColor: 'var(--accent)' },
  '.cm-cursor, .cm-dropCursor': { borderLeftColor: 'var(--accent)' },
  '.cm-gutters': {
    backgroundColor: 'var(--editor)',
    color: 'var(--muted)',
    border: 'none',
  },
  '.cm-activeLine': { backgroundColor: 'var(--active-line)' },
  '.cm-activeLineGutter': { backgroundColor: 'transparent', color: 'var(--text)' },
  '&.cm-focused': { outline: 'none' },
  '&.cm-focused .cm-selectionBackground, .cm-selectionBackground, ::selection': {
    backgroundColor: 'var(--selection) !important',
  },
  '.cm-tooltip': {
    backgroundColor: 'var(--surface-2)',
    color: 'var(--text)',
    border: '1px solid var(--border)',
    borderRadius: '6px',
  },
  '.cm-tooltip-autocomplete > ul > li[aria-selected]': { backgroundColor: 'var(--selection)', color: 'var(--text)' },
  '.cm-panels': { backgroundColor: 'var(--surface-2)', color: 'var(--text)' },
  '.cm-panels input, .cm-panels button': { fontSize: '13px' },
  '.cm-searchMatch': { backgroundColor: 'var(--search-match)' },
  '.cm-matchingBracket': { backgroundColor: 'var(--selection)', outline: 'none' },
  '.cm-delta-doc': { maxWidth: '360px', padding: '6px 10px', fontFamily: 'var(--font-sans)', fontSize: '13px' },
  '.cm-delta-doc p': { margin: '4px 0 0', color: 'var(--muted)' },
  '.cm-delta-doc p.cm-delta-book': { color: 'var(--accent)', fontSize: '12px' },
  '.cm-diagnostic': { fontFamily: 'var(--font-sans)', fontSize: '13px', maxWidth: '420px' },
  '.cm-diagnostic-book': { display: 'block', marginTop: '4px', color: 'var(--muted)', fontStyle: 'italic' },
});

/** Engine diagnostics → CodeMirror diagnostics, with quick fixes as actions. */
function toDiagnostics(list: readonly DiagnosticView[], length: number): Diagnostic[] {
  return list.map((d) => {
    const from = Math.min(d.from, length);
    const to = Math.min(Math.max(d.to, from), length);
    const severity = d.severity === 'error' || d.severity === 'warning' || d.severity === 'hint' ? d.severity : 'info';
    return {
      from,
      to,
      severity,
      message: d.message,
      renderMessage: () => {
        const el = document.createElement('span');
        el.textContent = `${d.code}: ${d.message}`;
        if (d.book) {
          const book = document.createElement('span');
          book.className = 'cm-diagnostic-book';
          book.textContent = `${d.book.item}: “${d.book.quote}”`;
          el.append(book);
        }
        return el;
      },
      actions: d.fixes.map((fix) => ({
        name: fix.title,
        apply: (view: EditorView) => {
          view.dispatch({ changes: fix.edits.map((e) => ({ from: e.from, to: e.to, insert: e.insert })) });
        },
      })),
    };
  });
}

interface Props {
  onView: (view: EditorView | null) => void;
}

export function CodeEditor({ onView }: Props) {
  const host = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!host.current) return;
    const { source, setSource, setEditorFocused, setCursor } = useUiStore.getState();
    const objects = () => useEngineStore.getState().doc?.objects ?? [];
    const outline = () => useEngineStore.getState().doc?.outline ?? [];
    // "▶ open steps" on a codelens: show that object's construction from step 1.
    const openSteps = (name: string) => {
      useUiStore.setState({
        selected: name,
        step: 0,
        inspectorTab: 'steps',
        sheetTab: 'steps',
        sheetOpen: true,
        phoneView: 'more',
      });
    };
    const extensions = [
      basicSetup,
      delta(),
      deltaLanguage.data.of({ autocomplete: deltaCompletion(outline) }),
      deltaHover(objects),
      lintGutter(),
      codelens(openSteps),
      theme,
      EditorView.contentAttributes.of({ 'aria-label': 'Delta code', autocapitalize: 'off', spellcheck: 'false' }),
      EditorView.updateListener.of((u) => {
        if (u.docChanged) setSource(u.state.doc.toString());
        if (u.focusChanged) setEditorFocused(u.view.hasFocus);
        if (u.selectionSet || u.docChanged) {
          const head = u.state.selection.main.head;
          const line = u.state.doc.lineAt(head);
          setCursor({ line: line.number, col: head - line.from + 1, offset: head });
        }
      }),
    ];
    const view = new EditorView({ parent: host.current, state: EditorState.create({ doc: source, extensions }) });

    // Diagnostics and hints belong to the source they were computed from; skip stale ones.
    const showEngineResults = () => {
      const { doc, docSource } = useEngineStore.getState();
      if (!doc || docSource !== view.state.doc.toString()) return;
      view.dispatch(setDiagnostics(view.state, toDiagnostics(doc.diagnostics, view.state.doc.length)));
      view.dispatch({ effects: setLenses.of(lensesFor(doc)) });
    };
    showEngineResults();
    const unsubEngine = useEngineStore.subscribe((s, prev) => {
      if (s.doc !== prev.doc) showEngineResults();
    });

    // The program can change from outside the editor (quick fixes, Convert buttons, the palette).
    const sync = (text: string) => {
      if (text === view.state.doc.toString()) return;
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: text } });
    };
    const reveal = () => {
      const { reveal: range, clearReveal } = useUiStore.getState();
      if (!range) return;
      const len = view.state.doc.length;
      const from = Math.min(range.from, len);
      const to = Math.min(Math.max(range.to, from), len);
      view.dispatch({ selection: EditorSelection.range(from, to), scrollIntoView: true });
      view.focus();
      clearReveal();
    };
    reveal();
    const unsubUi = useUiStore.subscribe((s, prev) => {
      // Another file: a fresh editor state, so undo history never crosses files.
      if (s.activeId !== prev.activeId) view.setState(EditorState.create({ doc: s.source, extensions }));
      else if (s.source !== prev.source) sync(s.source);
      if (s.reveal && s.reveal !== prev.reveal) reveal();
    });

    onView(view);
    return () => {
      unsubEngine();
      unsubUi();
      onView(null);
      setEditorFocused(false);
      view.destroy();
    };
  }, [onView]);

  return <div ref={host} className="h-full min-h-0 overflow-hidden" />;
}
