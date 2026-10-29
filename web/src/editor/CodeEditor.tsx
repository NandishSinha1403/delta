import { EditorState } from '@codemirror/state';
import { EditorView } from '@codemirror/view';
import { basicSetup } from 'codemirror';
import { useEffect, useRef } from 'react';
import { useUiStore } from '../state/ui';

const theme = EditorView.theme({
  '&': { height: '100%', backgroundColor: 'var(--surface)', color: 'var(--text)' },
  '.cm-scroller': { fontFamily: 'var(--font-mono)', fontSize: '14px', lineHeight: '1.6' },
  '.cm-content': { caretColor: 'var(--accent)' },
  '.cm-gutters': {
    backgroundColor: 'var(--surface)',
    color: 'var(--muted)',
    borderRight: '1px solid var(--border)',
  },
  '.cm-activeLine, .cm-activeLineGutter': { backgroundColor: 'var(--active-line)' },
  '&.cm-focused': { outline: 'none' },
  '&.cm-focused .cm-selectionBackground, .cm-selectionBackground, ::selection': {
    backgroundColor: 'var(--selection) !important',
  },
});

interface Props {
  onView: (view: EditorView | null) => void;
}

export function CodeEditor({ onView }: Props) {
  const host = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!host.current) return;
    const { source, setSource, setEditorFocused } = useUiStore.getState();
    const view = new EditorView({
      parent: host.current,
      state: EditorState.create({
        doc: source,
        extensions: [
          basicSetup,
          theme,
          EditorView.contentAttributes.of({ 'aria-label': 'Delta code', autocapitalize: 'off' }),
          EditorView.updateListener.of((u) => {
            if (u.docChanged) setSource(u.state.doc.toString());
            if (u.focusChanged) setEditorFocused(u.view.hasFocus);
          }),
        ],
      }),
    });
    onView(view);
    return () => {
      onView(null);
      setEditorFocused(false);
      view.destroy();
    };
  }, [onView]);

  return <div ref={host} className="h-full min-h-0 overflow-hidden" />;
}
