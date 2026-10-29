import type { EditorView } from '@codemirror/view';
import { useRef, useState } from 'react';
import { BOOK_SYMBOLS, insertText, type BookSymbol } from './symbols';

const LONG_PRESS_MS = 450;

interface Props {
  view: EditorView | null;
}

/** One tap inserts a symbol; long-press (or hover) shows its ASCII spelling. */
export function SymbolBar({ view }: Props) {
  const [hint, setHint] = useState<BookSymbol | null>(null);
  const timer = useRef<number | undefined>(undefined);
  const longPressed = useRef(false);

  const clear = () => {
    window.clearTimeout(timer.current);
    timer.current = undefined;
  };

  return (
    <div className="relative border-t border-border bg-surface-2">
      {hint && (
        <div
          role="status"
          className="absolute bottom-full left-1/2 mb-2 -translate-x-1/2 rounded-md bg-text px-3 py-1.5 font-mono text-sm text-bg shadow-lg"
        >
          {hint.symbol} = <span className="font-semibold">{hint.ascii}</span> in ASCII
        </div>
      )}
      <div role="toolbar" aria-label="Insert book symbols" className="flex">
        {BOOK_SYMBOLS.map((s) => (
          <button
            key={s.symbol}
            type="button"
            title={`${s.name} (ASCII: ${s.ascii})`}
            aria-label={`Insert ${s.name}, ASCII ${s.ascii}`}
            className="h-11 min-w-11 flex-1 select-none font-mono text-lg text-text hover:bg-border/60 active:bg-border"
            // Keep focus (and the phone keyboard) in the editor.
            onPointerDown={(e) => {
              e.preventDefault();
              longPressed.current = false;
              clear();
              timer.current = window.setTimeout(() => {
                longPressed.current = true;
                setHint(s);
              }, LONG_PRESS_MS);
            }}
            onPointerUp={() => {
              clear();
              setHint(null);
            }}
            onPointerLeave={() => {
              clear();
              setHint(null);
            }}
            onContextMenu={(e) => e.preventDefault()}
            onClick={() => {
              if (longPressed.current || !view) return;
              view.dispatch(insertText(view.state, s.symbol));
              view.focus();
            }}
          >
            {s.symbol}
          </button>
        ))}
      </div>
    </div>
  );
}
