import { useEffect, useMemo, useRef, useState } from 'react';
import { applications, CONVERSIONS } from '../conversions';
import { EXAMPLES } from '../examples';
import { applyConversion } from '../panels/current';
import { useObjects } from '../state/engine';
import { useUiStore } from '../state/ui';
import { filterCommands } from './palette-filter';

interface Command {
  id: string;
  group: 'Examples' | 'Conversions' | 'Go to object';
  label: string;
  hint: string;
  run: () => void;
}

/** ⌘K / Ctrl+K: examples, the conversions that apply to this file's objects, and jump-to-object. */
export function CommandPalette() {
  const open = useUiStore((s) => s.paletteOpen);
  const setOpen = useUiStore((s) => s.setPaletteOpen);
  const objects = useObjects();
  const [query, setQuery] = useState('');
  const [index, setIndex] = useState(0);
  const dialog = useRef<HTMLDialogElement>(null);
  const list = useRef<HTMLUListElement>(null);

  useEffect(() => {
    const d = dialog.current;
    if (!d) return;
    if (open && !d.open) d.showModal();
    if (!open && d.open) d.close();
  }, [open]);

  const commands = useMemo<Command[]>(() => {
    const ui = useUiStore.getState();
    const out: Command[] = [];
    for (const o of objects) {
      out.push({
        id: `go-${o.name}`,
        group: 'Go to object',
        label: `${o.name}`,
        hint: o.derived ?? o.kind.toUpperCase(),
        run: () => {
          ui.select(o.name);
          ui.revealRange({ from: o.from, to: o.from });
        },
      });
    }
    for (const c of CONVERSIONS) {
      for (const args of applications(c, objects)) {
        out.push({
          id: `${c.fn}-${args.join(',')}`,
          group: 'Conversions',
          label: `${c.fn}(${args.join(', ')})`,
          hint: `${c.book} · ${c.doc}`,
          run: () => applyConversion(c, args),
        });
      }
    }
    for (const ex of EXAMPLES) {
      out.push({
        id: `ex-${ex.id}`,
        group: 'Examples',
        label: ex.title,
        hint: ex.summary,
        run: () => ui.openExample(ex),
      });
    }
    return out;
  }, [objects]);

  const shown = filterCommands(commands, query);
  const active = Math.min(index, Math.max(0, shown.length - 1));

  const close = () => {
    setOpen(false);
    setQuery('');
    setIndex(0);
  };
  const choose = (c: Command | undefined) => {
    if (!c) return;
    close();
    c.run();
  };

  useEffect(() => {
    list.current?.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: 'nearest' });
  }, [active]);

  return (
    <dialog
      ref={dialog}
      aria-label="Command palette"
      onClose={close}
      onClick={(e) => {
        if (e.target === dialog.current) close();
      }}
      className="mx-auto mt-[12vh] w-[min(36rem,calc(100vw-2rem))] rounded-xl border border-border-strong bg-surface p-0 text-text shadow-2xl backdrop:bg-black/55"
    >
      <input
        autoFocus
        value={query}
        onChange={(e) => {
          setQuery(e.target.value);
          setIndex(0);
        }}
        onKeyDown={(e) => {
          if (e.key === 'ArrowDown') setIndex(Math.min(active + 1, shown.length - 1));
          else if (e.key === 'ArrowUp') setIndex(Math.max(active - 1, 0));
          else if (e.key === 'Enter') choose(shown[active]);
          else return;
          e.preventDefault();
        }}
        role="combobox"
        aria-expanded="true"
        aria-controls="palette-list"
        aria-activedescendant={shown[active] ? `palette-${shown[active].id}` : undefined}
        aria-label="Search examples, conversions and objects"
        placeholder="Search examples, conversions, objects…"
        className="h-12 w-full border-b border-border bg-transparent px-4 text-sm outline-none placeholder:text-muted"
      />
      <ul id="palette-list" ref={list} role="listbox" className="max-h-[50dvh] overflow-auto py-1">
        {shown.length === 0 && <li className="px-4 py-3 text-sm text-muted">Nothing matches.</li>}
        {shown.map((c, i) => (
          <li
            key={c.id}
            id={`palette-${c.id}`}
            role="option"
            aria-selected={i === active}
            onMouseEnter={() => setIndex(i)}
            onClick={() => choose(c)}
            className={`flex cursor-pointer items-baseline gap-3 px-4 py-2 ${i === active ? 'bg-navy-2' : ''}`}
          >
            <span className="w-24 shrink-0 text-[11px] text-faint">{c.group}</span>
            <span className={`shrink-0 font-mono text-[13px] ${i === active ? 'text-accent' : 'text-text'}`}>{c.label}</span>
            <span className="truncate text-xs text-muted">{c.hint}</span>
          </li>
        ))}
      </ul>
    </dialog>
  );
}
