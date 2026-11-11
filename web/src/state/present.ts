// Presentation mode: full screen, large type, no editor chrome, stepped with the keyboard or a
// clicker. What it shows and how it looks are remembered per browser.
import { create } from 'zustand';
import type { MachineView } from '../engine/api';

/** What is presented: the current run step by step, the diagram alone, or the tuple and δ-table. */
export type PresentWhat = 'run' | 'diagram' | 'tuple';
/** The three views, as the Present panel and the command palette name them. */
export const WHATS: readonly [PresentWhat, string][] = [
  ['run', 'The run, step by step'],
  ['diagram', 'Diagram only'],
  ['tuple', 'Tuple and δ-table'],
];

/** The command palette's entries: one per view. */
export const PRESENT_COMMANDS: readonly { what: PresentWhat; label: string }[] = WHATS.map(([what, label]) => ({
  what,
  label: `Present: ${label.toLowerCase()}`,
}));

/** The projector themes (DESIGN.md), or whatever the Pad shows. */
export type PresentTheme = 'dark' | 'light' | 'match';

const KEY = 'delta.present';

interface Settings {
  what: PresentWhat;
  theme: PresentTheme;
  showCode: boolean;
}

const DEFAULTS: Settings = { what: 'run', theme: 'dark', showCode: true };

function loadSettings(): Settings {
  try {
    const raw = JSON.parse(window.localStorage.getItem(KEY) ?? 'null') as Partial<Settings> | null;
    return {
      what: parseWhat(raw?.what ?? null) ?? DEFAULTS.what,
      theme: raw?.theme === 'light' || raw?.theme === 'match' || raw?.theme === 'dark' ? raw.theme : DEFAULTS.theme,
      showCode: typeof raw?.showCode === 'boolean' ? raw.showCode : DEFAULTS.showCode,
    };
  } catch {
    return DEFAULTS;
  }
}

function saveSettings(s: Settings): void {
  try {
    window.localStorage.setItem(KEY, JSON.stringify(s));
  } catch {
    // Not persisted; the setting still applies for this visit.
  }
}

/** `run`, `diagram` or `tuple`; anything else is null. */
export function parseWhat(value: string | null): PresentWhat | null {
  return value === 'run' || value === 'diagram' || value === 'tuple' ? value : null;
}

interface PresentStore extends Settings {
  presenting: boolean;
  /** B: the screen goes blank (the projector background) until the next key. */
  blank: boolean;
  laser: boolean;
  set: (patch: Partial<Settings>) => void;
  /** Starts presenting, optionally showing something other than the saved choice. */
  start: (what?: PresentWhat) => void;
  stop: () => void;
  setBlank: (blank: boolean) => void;
  setLaser: (laser: boolean) => void;
}

export const usePresentStore = create<PresentStore>()((set, get) => ({
  ...loadSettings(),
  presenting: false,
  blank: false,
  laser: false,
  set: (patch) => {
    const { what, theme, showCode } = { ...get(), ...patch };
    saveSettings({ what, theme, showCode });
    set(patch);
  },
  start: (what) => {
    if (what) get().set({ what });
    set({ presenting: true, blank: false, laser: false });
    // Full screen needs a user gesture; opened from a link it simply fills the window.
    document.documentElement.requestFullscreen?.().catch(() => {});
  },
  stop: () => {
    set({ presenting: false, blank: false, laser: false });
    if (document.fullscreenElement) document.exitFullscreen?.().catch(() => {});
  },
  setBlank: (blank) => set({ blank }),
  setLaser: (laser) => set({ laser }),
}));

/** True on Apple platforms, where the shortcut uses ⌘ instead of Ctrl. */
export function isApple(): boolean {
  const nav = navigator as Navigator & { userAgentData?: { platform?: string } };
  return /mac|iphone|ipad/i.test(nav.userAgentData?.platform ?? navigator.platform ?? '');
}

/** The shortcut as the user's keyboard labels it. */
export function shortcutLabel(apple = isApple()): string {
  return apple ? '⌘⌥P' : 'Ctrl+Alt+P';
}

interface KeyLike {
  code: string;
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
}

/**
 * Ctrl+Alt+P (⌘⌥P on a Mac) starts presenting. The physical P key is matched, so Option+P typing
 * "π" on a Mac doesn't matter. On Windows and Linux, Ctrl+Alt is AltGr on many keyboard layouts;
 * when AltGr+P types a character there (the event's key is that character, not "p"), the keys
 * are left alone so the character can still be typed.
 */
export function isPresentShortcut(e: KeyLike, apple: boolean): boolean {
  if (e.code !== 'KeyP' || !e.altKey || e.shiftKey) return false;
  if (apple) return e.metaKey && !e.ctrlKey;
  if (!e.ctrlKey || e.metaKey) return false;
  return e.key.toLowerCase() === 'p' || e.key.length !== 1;
}

/** A piece of a program line to mark: `line` is 1-based, `start`/`end` are columns in it. */
export interface Mark {
  line: number;
  start: number;
  end: number;
}

function escape(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * Where in the program the arrows just taken are written: each `from -labels-> to` in the
 * machine's declaration (`declFrom`..`declTo`) whose labels include `symbol`. Several marks for an
 * NFA; none when the arrow isn't written out (a machine built by a construction).
 */
export function transitionMarks(
  source: string,
  declFrom: number,
  declTo: number,
  machine: MachineView,
  taken: ReadonlySet<number>,
  symbol: string | undefined,
): Mark[] {
  if (symbol === undefined || taken.size === 0) return [];
  const before = source.slice(0, declFrom);
  const firstLine = before.split('\n').length;
  const lines = source.slice(before.lastIndexOf('\n') + 1, declTo).split('\n');
  const marks: Mark[] = [];
  for (const i of taken) {
    const e = machine.edges[i];
    const from = machine.states[e?.from ?? -1]?.name;
    const to = machine.states[e?.to ?? -1]?.name;
    if (!e || from === undefined || to === undefined) continue;
    const re = new RegExp(`(?<![\\w'])${escape(from)}\\s*-([^>\\n]*?)->\\s*${escape(to)}(?![\\w'])`, 'g');
    lines.forEach((text, j) => {
      for (const m of text.matchAll(re)) {
        const labels = (m[1] ?? '').split(',').map((l) => l.trim());
        if (!labels.includes(symbol)) continue;
        const line = firstLine + j;
        const start = m.index ?? 0;
        if (!marks.some((k) => k.line === line && k.start === start)) marks.push({ line, start, end: start + m[0].length });
      }
    });
  }
  return marks.sort((a, b) => a.line - b.line || a.start - b.start);
}
