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
    <div className="relative border-t border-border bg-surface px-3 pt-2 pb-1.5">
      {hint && (
        <div
          role="status"
          className="absolute bottom-full left-1/2 mb-2 -translate-x-1/2 rounded-md bg-text px-3 py-1.5 font-mono text-sm text-bg shadow-lg"
        >
          {hint.symbol} = <span className="font-semibold">{hint.ascii}</span> in ASCII
        </div>
      )}
      <div role="toolbar" aria-label="Insert book symbols" className="flex gap-1.5" data-testid="symbol-bar">
        {BOOK_SYMBOLS.map((s) => (
          <button
            key={s.symbol}
            type="button"
            title={`${s.name} (ASCII: ${s.ascii})`}
            aria-label={`Insert ${s.name}, ASCII ${s.ascii}`}
            className="grid h-9 min-w-9 flex-1 place-items-center rounded-md border border-border-strong bg-surface-2 font-mono text-base text-text select-none hover:border-accent hover:text-accent active:bg-navy pointer-coarse:h-11"
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
            {/* The star is shown as the book prints it: a superscript after an expression. */}
            {s.symbol === '*' ? (
              <span className="font-serif text-lg leading-none">
                <span className="text-faint">R</span>
                <sup className="text-xs">∗</sup>
              </span>
            ) : (
              s.symbol
            )}
          </button>
        ))}
      </div>
      <p className="mt-1 hidden text-[11px] text-faint sm:block">ASCII works too: eps · empty · | · -&gt;</p>
    </div>
  );
}
