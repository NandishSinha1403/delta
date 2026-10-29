import { useEffect, useMemo, useRef, useState } from 'react';
import type { MachineView } from '../engine/api';
import { engine } from '../engine/client';
import { describeMachine, parseLayout, toDot, type Layout, type Point } from './layout';

/** What to emphasize on top of the plain machine (a run position or a trace step). */
export interface Overlay {
  /** States and arrows not built yet at this trace step (drawn faint). */
  hiddenStates?: ReadonlySet<number>;
  hiddenEdges?: ReadonlySet<number>;
  /** States the current step is about. */
  focus?: readonly number[];
  /** The run's active states ("fingers"). */
  active?: readonly number[];
  /** Overrides for constructions whose start/accept states change as they go. */
  start?: number | null;
  accept?: readonly number[] | null;
  /** Colors the active states once a run has finished. */
  verdict?: 'accept' | 'reject' | null;
}

// Layouts are pure functions of the DOT text, so they are cached across renders and objects.
const cache = new Map<string, Promise<string>>();

function useLayout(machine: MachineView): { layout: Layout | null; error: string | null } {
  const dot = useMemo(() => toDot(machine), [machine]);
  const [state, setState] = useState<{ dot: string; layout: Layout | null; error: string | null }>({
    dot: '',
    layout: null,
    error: null,
  });
  useEffect(() => {
    let live = true;
    let pending = cache.get(dot);
    if (!pending) {
      pending = engine.layout(dot);
      cache.set(dot, pending);
      pending.catch(() => cache.delete(dot));
    }
    pending.then(
      (json) => live && setState({ dot, layout: parseLayout(json, machine), error: null }),
      (err: unknown) =>
        live && setState({ dot, layout: null, error: err instanceof Error ? err.message : String(err) }),
    );
    return () => {
      live = false;
    };
  }, [dot, machine]);
  // Until the new layout arrives, keep showing the previous one (no flicker while typing).
  return { layout: state.layout, error: state.dot === dot ? state.error : null };
}

interface View {
  x: number;
  y: number;
  k: number;
}

function fit(layout: Layout, box: { w: number; h: number }): View {
  const k = Math.min(box.w / layout.width, box.h / layout.height, 1.75) * 0.94;
  return { k, x: (box.w - layout.width * k) / 2, y: (box.h - layout.height * k) / 2 };
}

function arrowHead(tip: Point, base: Point): string {
  const dx = tip.x - base.x;
  const dy = tip.y - base.y;
  const len = Math.hypot(dx, dy) || 1;
  const [ux, uy] = [dx / len, dy / len];
  const size = 9;
  const half = 4.2;
  const bx = tip.x - ux * size;
  const by = tip.y - uy * size;
  return `${tip.x},${tip.y} ${bx - uy * half},${by + ux * half} ${bx + uy * half},${by - ux * half}`;
}

interface Props {
  name: string;
  machine: MachineView;
  overlay?: Overlay;
  selectedState?: number | null;
  onSelectState?: (i: number) => void;
}

/** A state diagram in the book's style: circles, double circles for accept states, a start arrow. */
export function Diagram({ name, machine, overlay = {}, selectedState = null, onSelectState }: Props) {
  const { layout, error } = useLayout(machine);
  const host = useRef<HTMLDivElement>(null);
  const [box, setBox] = useState({ w: 0, h: 0 });
  // null = fit to the box; refits on new layouts and resizes until the user pans or zooms.
  const [view, setView] = useState<View | null>(null);
  const pointers = useRef(new Map<number, Point>());

  useEffect(() => {
    const el = host.current;
    if (!el) return;
    const ro = new ResizeObserver(([entry]) => {
      if (entry) setBox({ w: entry.contentRect.width, h: entry.contentRect.height });
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const shown = layout && box.w > 0 && box.h > 0 ? (view ?? fit(layout, box)) : null;

  const zoomAt = (factor: number, at: Point) => {
    if (!shown) return;
    const k = Math.min(6, Math.max(0.2, shown.k * factor));
    const f = k / shown.k;
    setView({ k, x: at.x - (at.x - shown.x) * f, y: at.y - (at.y - shown.y) * f });
  };

  const refit = () => setView(null);

  const local = (e: { clientX: number; clientY: number }): Point => {
    const r = host.current?.getBoundingClientRect();
    return { x: e.clientX - (r?.left ?? 0), y: e.clientY - (r?.top ?? 0) };
  };

  // Wheel zoom needs a non-passive listener so the page does not scroll.
  useEffect(() => {
    const el = host.current;
    if (!el) return;
    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      zoomAt(Math.exp(-e.deltaY * 0.0015), local(e));
    };
    el.addEventListener('wheel', onWheel, { passive: false });
    return () => el.removeEventListener('wheel', onWheel);
  });

  const accept = new Set(overlay.accept ?? machine.states.flatMap((s, i) => (s.accept ? [i] : [])));
  const start = overlay.start ?? machine.states.findIndex((s) => s.start);
  const focus = new Set(overlay.focus ?? []);
  const active = new Set(overlay.active ?? []);
  const hiddenStates = overlay.hiddenStates;
  const hiddenEdges = overlay.hiddenEdges;
  const verdictColor =
    overlay.verdict === 'accept' ? 'var(--accept)' : overlay.verdict === 'reject' ? 'var(--reject)' : 'var(--accent)';

  return (
    <div className="relative h-full min-h-32 w-full overflow-hidden">
      <div
        ref={host}
        className="absolute inset-0 cursor-grab touch-none select-none active:cursor-grabbing"
        onDoubleClick={refit}
        onPointerDown={(e) => {
          (e.target as Element).setPointerCapture?.(e.pointerId);
          pointers.current.set(e.pointerId, local(e));
        }}
        onPointerMove={(e) => {
          const prev = pointers.current.get(e.pointerId);
          if (!prev || !shown) return;
          const now = local(e);
          const others = [...pointers.current.entries()].filter(([id]) => id !== e.pointerId);
          const other = others[0]?.[1];
          if (other) {
            // Pinch: scale by the change in finger distance, around the midpoint.
            const before = Math.hypot(prev.x - other.x, prev.y - other.y) || 1;
            const after = Math.hypot(now.x - other.x, now.y - other.y) || 1;
            zoomAt(after / before, { x: (now.x + other.x) / 2, y: (now.y + other.y) / 2 });
          } else if (Math.hypot(now.x - prev.x, now.y - prev.y) > 0) {
            setView({ ...shown, x: shown.x + now.x - prev.x, y: shown.y + now.y - prev.y });
          }
          pointers.current.set(e.pointerId, now);
        }}
        onPointerUp={(e) => pointers.current.delete(e.pointerId)}
        onPointerCancel={(e) => pointers.current.delete(e.pointerId)}
      >
        {layout && shown && (
          <svg
            role="img"
            aria-label={describeMachine(name, machine)}
            data-testid="diagram"
            width={box.w}
            height={box.h}
            className="block font-mono"
          >
            <g transform={`translate(${shown.x},${shown.y}) scale(${shown.k})`}>
              {layout.start && start === machine.states.findIndex((s) => s.start) && (
                <g className="text-muted">
                  <path d={layout.start.path} fill="none" stroke="currentColor" strokeWidth={1.4} />
                  <polygon points={arrowHead(layout.start.tip, layout.start.base)} fill="currentColor" />
                </g>
              )}
              {start >= 0 && start !== machine.states.findIndex((s) => s.start) && layout.nodes[start] && (
                <ManualStart node={layout.nodes[start]} />
              )}
              {machine.edges.map((e, i) => {
                const shape = layout.edges[i];
                if (!shape) return null;
                const faint = hiddenEdges?.has(i) ?? false;
                return (
                  <g key={i} opacity={faint ? 0.12 : 1} data-edge={i}>
                    <path d={shape.path} fill="none" stroke="var(--edge)" strokeWidth={1.4} />
                    <polygon points={arrowHead(shape.tip, shape.base)} fill="var(--edge)" />
                    {shape.label && (
                      <text
                        x={shape.label.x}
                        y={shape.label.y}
                        textAnchor="middle"
                        dominantBaseline="central"
                        fontSize={13}
                        fill="var(--text)"
                        stroke="var(--canvas)"
                        strokeWidth={4}
                        paintOrder="stroke"
                      >
                        {e.label}
                      </text>
                    )}
                  </g>
                );
              })}
              {machine.states.map((s, i) => {
                const n = layout.nodes[i];
                if (!n) return null;
                const isAccept = accept.has(i);
                const isActive = active.has(i);
                const isFocus = focus.has(i);
                const faint = hiddenStates?.has(i) ?? false;
                const ring = isActive ? verdictColor : isFocus || selectedState === i ? 'var(--accent)' : 'var(--node)';
                const r = n.r;
                return (
                  <g
                    key={i}
                    opacity={faint ? 0.12 : 1}
                    data-state={s.name}
                    data-active={isActive || undefined}
                    data-focus={isFocus || undefined}
                    onClick={() => onSelectState?.(i)}
                    className={onSelectState ? 'cursor-pointer' : undefined}
                  >
                    {s.origin && <title>{`${s.name}: ${s.origin}`}</title>}
                    <circle
                      cx={n.x}
                      cy={n.y}
                      r={r}
                      fill={isActive ? `color-mix(in srgb, ${verdictColor} 22%, var(--canvas))` : 'var(--canvas)'}
                      stroke={ring}
                      strokeWidth={isActive || isFocus ? 2.6 : 1.4}
                      strokeDasharray={s.trap ? '4 3' : undefined}
                    />
                    {isAccept && (
                      <circle cx={n.x} cy={n.y} r={r - 4} fill="none" stroke={ring} strokeWidth={1.2} />
                    )}
                    <text
                      x={n.x}
                      y={n.y}
                      textAnchor="middle"
                      dominantBaseline="central"
                      fontSize={14}
                      fill="var(--text)"
                    >
                      {s.name}
                    </text>
                  </g>
                );
              })}
            </g>
          </svg>
        )}
      </div>
      {!layout && !error && <p className="absolute inset-0 grid place-items-center text-sm text-muted">Laying out…</p>}
      {error && (
        <p role="alert" className="absolute inset-x-4 top-4 rounded-md border border-reject/40 bg-reject/10 p-3 text-sm">
          The diagram could not be laid out: {error}
        </p>
      )}
      <div className="absolute right-2 bottom-2 flex gap-1">
        <ZoomButton label="Zoom in" onClick={() => zoomAt(1.25, { x: box.w / 2, y: box.h / 2 })}>
          +
        </ZoomButton>
        <ZoomButton label="Zoom out" onClick={() => zoomAt(0.8, { x: box.w / 2, y: box.h / 2 })}>
          −
        </ZoomButton>
        <ZoomButton label="Fit to view" onClick={refit}>
          ⤢
        </ZoomButton>
      </div>
    </div>
  );
}

/** A start arrow for a state that is only the start state part-way through a construction. */
function ManualStart({ node }: { node: { x: number; y: number; r: number } }) {
  const tip = { x: node.x - node.r, y: node.y };
  const base = { x: tip.x - 26, y: tip.y };
  return (
    <g className="text-muted">
      <path d={`M${base.x},${base.y} L${tip.x - 8},${tip.y}`} stroke="currentColor" strokeWidth={1.4} />
      <polygon points={arrowHead(tip, base)} fill="currentColor" />
    </g>
  );
}

function ZoomButton({
  label,
  onClick,
  children,
}: {
  label: string;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={onClick}
      className="grid h-9 w-9 place-items-center rounded-md border border-border bg-surface/90 font-mono text-base text-text hover:bg-surface-2 pointer-coarse:h-11 pointer-coarse:w-11"
    >
      {children}
    </button>
  );
}
