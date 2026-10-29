import { useEffect, useId, useMemo, useRef, useState } from 'react';
import type { MachineView } from '../engine/api';
import { StoppedError } from '../engine/api';
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
  /** Arrows the run just took. */
  taken?: ReadonlySet<number>;
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
  // Bumped when Stop cancels a layout in flight, so it is requested again from the new worker.
  const [retry, setRetry] = useState(0);
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
      (err: unknown) => {
        if (!live) return;
        if (err instanceof StoppedError) setRetry((r) => r + 1);
        else setState({ dot, layout: null, error: err instanceof Error ? err.message : String(err) });
      },
    );
    return () => {
      live = false;
    };
  }, [dot, machine, retry]);
  // Until the new layout arrives, keep showing the previous one (no flicker while typing).
  return { layout: state.layout, error: state.dot === dot ? state.error : null };
}

interface View {
  x: number;
  y: number;
  k: number;
}

function fit(layout: Layout, box: { w: number; h: number }): View {
  const k = Math.min(box.w / layout.width, box.h / layout.height, 1.75) * 0.9;
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

/** A state name as the book sets it: q1 → italic q with subscript 1; anything else in mono. */
function StateLabel({ name, x, y }: { name: string; x: number; y: number }) {
  const sub = /^([A-Za-z])(\d+)$/.exec(name);
  if (sub) {
    return (
      <text x={x} y={y} textAnchor="middle" dominantBaseline="central" fill="var(--text)">
        <tspan fontFamily="var(--font-serif)" fontStyle="italic" fontSize={19}>
          {sub[1]}
        </tspan>
        <tspan fontFamily="var(--font-serif)" fontSize={12} dy={5}>
          {sub[2]}
        </tspan>
      </text>
    );
  }
  return (
    <text x={x} y={y} textAnchor="middle" dominantBaseline="central" fontSize={13} fill="var(--text)">
      {name}
    </text>
  );
}

/** Rough width of a mono label, for its pill (Graphviz already left room for the text). */
function pillWidth(label: string): number {
  return [...label].length * 7.6 + 12;
}

interface Props {
  name: string;
  machine: MachineView;
  overlay?: Overlay;
  /** Show the finger/accept/edge legend (off where space is tight). */
  legend?: boolean;
}

/** A state diagram in the book's style: circles, double circles for accept states, a start arrow. */
export function Diagram({ name, machine, overlay = {}, legend = true }: Props) {
  const { layout, error } = useLayout(machine);
  const host = useRef<HTMLDivElement>(null);
  // useId gives ids with ":" which are awkward in url(#…); strip them.
  const glowId = `glow${useId().replace(/[^a-zA-Z0-9]/g, '')}`;
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

  const machineStart = machine.states.findIndex((s) => s.start);
  const accept = new Set(overlay.accept ?? machine.states.flatMap((s, i) => (s.accept ? [i] : [])));
  const start = overlay.start ?? machineStart;
  const focus = new Set(overlay.focus ?? []);
  const active = new Set(overlay.active ?? []);
  const taken = overlay.taken ?? new Set<number>();
  const ring = overlay.verdict === 'reject' ? 'var(--reject)' : 'var(--accent)';

  return (
    <div
      className="relative h-full min-h-32 w-full overflow-hidden bg-canvas"
      style={{
        backgroundImage: 'radial-gradient(var(--grid-dot) 1px, transparent 1px)',
        backgroundSize: '18px 18px',
      }}
    >
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
          const other = [...pointers.current.entries()].find(([id]) => id !== e.pointerId)?.[1];
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
            <defs>
              {/* The soft powder-blue glow around active states. */}
              <filter id={glowId} x="-60%" y="-60%" width="220%" height="220%">
                <feGaussianBlur in="SourceAlpha" stdDeviation="6" result="blur" />
                <feFlood floodColor="#a8dadc" floodOpacity="0.45" />
                <feComposite in2="blur" operator="in" />
                <feMerge>
                  <feMergeNode />
                  <feMergeNode in="SourceGraphic" />
                </feMerge>
              </filter>
            </defs>
            <g transform={`translate(${shown.x},${shown.y}) scale(${shown.k})`}>
              {layout.clusters.map((c, i) => (
                // One box per sub-expression of a regex → NFA construction (Lemma 1.55).
                <g key={`c${i}`} data-cluster={c.label}>
                  <rect
                    x={c.x}
                    y={c.y}
                    width={c.width}
                    height={c.height}
                    rx={10}
                    fill="var(--surface)"
                    fillOpacity={0.55}
                    stroke="var(--border-strong)"
                    strokeDasharray="5 4"
                  />
                  <text x={c.x + 10} y={c.labelAt.y} dominantBaseline="central" fontSize={11} fill="var(--faint)">
                    {c.label}
                  </text>
                </g>
              ))}
              {layout.start && start === machineStart && (
                <g>
                  <path d={layout.start.path} fill="none" stroke="var(--edge)" strokeWidth={1.4} />
                  <polygon points={arrowHead(layout.start.tip, layout.start.base)} fill="var(--edge)" />
                </g>
              )}
              {start >= 0 && start !== machineStart && layout.nodes[start] && <ManualStart node={layout.nodes[start]} />}
              {machine.edges.map((e, i) => {
                const shape = layout.edges[i];
                if (!shape) return null;
                const faint = overlay.hiddenEdges?.has(i) ?? false;
                const hot = taken.has(i);
                const stroke = hot ? 'var(--accent)' : 'var(--edge)';
                const w = pillWidth(e.label);
                // ε-arrows are dashed, so the construction's glue is easy to tell from real input.
                const epsilon = e.label === 'ε';
                return (
                  <g key={i} opacity={faint ? 0.12 : 1} data-edge={i} data-taken={hot || undefined}>
                    <path
                      d={shape.path}
                      fill="none"
                      stroke={stroke}
                      strokeWidth={hot ? 2.2 : 1.4}
                      strokeDasharray={epsilon ? '5 4' : undefined}
                      data-epsilon={epsilon || undefined}
                    />
                    <polygon points={arrowHead(shape.tip, shape.base)} fill={stroke} />
                    {shape.label && (
                      <g>
                        <rect
                          x={shape.label.x - w / 2}
                          y={shape.label.y - 10}
                          width={w}
                          height={20}
                          rx={5}
                          fill={hot ? 'var(--navy)' : 'var(--surface-3)'}
                          stroke={hot ? 'var(--accent)' : 'none'}
                        />
                        <text
                          x={shape.label.x}
                          y={shape.label.y}
                          textAnchor="middle"
                          dominantBaseline="central"
                          fontSize={12}
                          fill={hot ? 'var(--accent)' : 'var(--text-2)'}
                        >
                          {e.label}
                        </text>
                      </g>
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
                const faint = overlay.hiddenStates?.has(i) ?? false;
                const stroke = isActive ? ring : isFocus ? 'var(--accent)' : isAccept ? 'var(--accept)' : 'var(--node)';
                return (
                  <g
                    key={i}
                    opacity={faint ? 0.12 : 1}
                    data-state={s.name}
                    data-active={isActive || undefined}
                    data-focus={isFocus || undefined}
                  >
                    {s.origin && <title>{`${s.name}: ${s.origin}`}</title>}
                    <circle
                      cx={n.x}
                      cy={n.y}
                      r={n.r}
                      fill={isActive ? 'var(--navy)' : 'var(--node-fill)'}
                      stroke={stroke}
                      strokeWidth={isActive || isFocus ? 2.6 : 1.6}
                      strokeDasharray={s.trap ? '4 3' : undefined}
                      filter={isActive ? `url(#${glowId})` : undefined}
                    />
                    {isAccept && <circle cx={n.x} cy={n.y} r={n.r - 4.5} fill="none" stroke={stroke} strokeWidth={1.4} />}
                    <StateLabel name={s.name} x={n.x} y={n.y} />
                    {isActive && (
                      // The finger on this state.
                      <circle
                        cx={n.x + n.r * 0.72}
                        cy={n.y - n.r * 0.72}
                        r={6}
                        fill={ring}
                        stroke="var(--canvas)"
                        strokeWidth={2}
                        data-finger
                      />
                    )}
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
      {legend && layout && (
        <div
          aria-label="Legend"
          className="absolute bottom-3 left-3 hidden gap-3.5 rounded-lg border border-border bg-surface/90 px-3 py-2 text-[11px] text-text-2 sm:flex"
        >
          <span className="flex items-center gap-1.5">
            <span className="h-2.5 w-2.5 rounded-full bg-accent" />
            finger (active)
          </span>
          <span className="flex items-center gap-1.5">
            <span className="h-2.5 w-2.5 rounded-full border-2 border-[var(--accept)]" />
            accept state
          </span>
          <span className="flex items-center gap-1.5">
            <span className="h-0.5 w-4 bg-accent" />
            edge just taken
          </span>
        </div>
      )}
      <div
        role="toolbar"
        aria-label="Zoom"
        className="absolute right-3 bottom-3 flex items-center gap-0.5 rounded-lg border border-border bg-surface/90 p-1"
      >
        <ZoomButton label="Zoom in" onClick={() => zoomAt(1.25, { x: box.w / 2, y: box.h / 2 })}>
          +
        </ZoomButton>
        <ZoomButton label="Zoom out" onClick={() => zoomAt(0.8, { x: box.w / 2, y: box.h / 2 })}>
          −
        </ZoomButton>
        <ZoomButton label="Fit to view" onClick={refit}>
          Fit
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
    <g>
      <path d={`M${base.x},${base.y} L${tip.x - 8},${tip.y}`} stroke="var(--edge)" strokeWidth={1.4} />
      <polygon points={arrowHead(tip, base)} fill="var(--edge)" />
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
      className="grid h-8 min-w-8 place-items-center rounded-md px-2 text-[13px] text-text hover:bg-surface-3 pointer-coarse:h-10 pointer-coarse:min-w-10"
    >
      {children}
    </button>
  );
}
