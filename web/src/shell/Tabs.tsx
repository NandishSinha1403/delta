import { useRef } from 'react';

export interface TabItem<T extends string> {
  id: T;
  label: string;
  /** Optional glyph shown above the label (phone tab bar). */
  icon?: string;
}

interface Props<T extends string> {
  /** Unique prefix for element ids (tab ↔ panel wiring). */
  idPrefix: string;
  label: string;
  items: readonly TabItem<T>[];
  value: T;
  onChange: (id: T) => void;
  variant?: 'strip' | 'bar';
  className?: string;
}

/** An ARIA tablist with arrow-key navigation. */
export function Tabs<T extends string>({
  idPrefix,
  label,
  items,
  value,
  onChange,
  variant = 'strip',
  className = '',
}: Props<T>) {
  const refs = useRef(new Map<T, HTMLButtonElement>());

  const move = (delta: number) => {
    const i = items.findIndex((t) => t.id === value);
    const next = items[(i + delta + items.length) % items.length];
    if (!next) return;
    onChange(next.id);
    refs.current.get(next.id)?.focus();
  };

  return (
    <div
      role="tablist"
      aria-label={label}
      className={`flex ${variant === 'bar' ? 'justify-around' : 'gap-1'} ${className}`}
      onKeyDown={(e) => {
        if (e.key === 'ArrowRight') move(1);
        else if (e.key === 'ArrowLeft') move(-1);
        else return;
        e.preventDefault();
      }}
    >
      {items.map((t) => {
        const selected = t.id === value;
        return (
          <button
            key={t.id}
            ref={(el) => {
              if (el) refs.current.set(t.id, el);
              else refs.current.delete(t.id);
            }}
            type="button"
            role="tab"
            id={`${idPrefix}-tab-${t.id}`}
            aria-selected={selected}
            aria-controls={`${idPrefix}-panel`}
            tabIndex={selected ? 0 : -1}
            onClick={() => onChange(t.id)}
            className={
              variant === 'bar'
                ? `flex min-h-14 flex-1 flex-col items-center justify-center gap-0.5 text-xs ${
                    selected ? 'font-semibold text-accent' : 'text-muted'
                  }`
                : `min-h-9 rounded-md px-3 text-sm ${
                    selected ? 'bg-surface-2 font-semibold text-text' : 'text-muted hover:text-text'
                  }`
            }
          >
            {t.icon && (
              <span aria-hidden="true" className="font-mono text-base leading-none">
                {t.icon}
              </span>
            )}
            {t.label}
          </button>
        );
      })}
    </div>
  );
}

export function TabPanel({
  idPrefix,
  activeId,
  className = '',
  children,
}: {
  idPrefix: string;
  activeId: string;
  className?: string;
  children: React.ReactNode;
}) {
  return (
    <div
      role="tabpanel"
      id={`${idPrefix}-panel`}
      aria-labelledby={`${idPrefix}-tab-${activeId}`}
      className={className}
    >
      {children}
    </div>
  );
}
