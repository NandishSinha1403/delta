import { create } from 'zustand';
import { DEFAULT_SOURCE, type Example } from '../examples';

export type PhoneView = 'code' | 'diagram' | 'tuple' | 'run' | 'more';
export type InspectorTab = 'tuple' | 'delta' | 'steps' | 'tests' | 'problems';
export type SheetTab = InspectorTab | 'run';
export type Theme = 'dark' | 'light';
/** The left rail's drawers. */
export type RailPanel = 'files' | 'examples' | 'conversions' | 'tests' | 'settings';

const FILES_KEY = 'delta.files';
const OLD_SOURCE_KEY = 'delta.source';
const THEME_KEY = 'delta.theme';
const AUTO_KEY = 'delta.auto';

// Browser storage can be missing or throw (private windows, blocked site data); it only holds
// conveniences, so failures are ignored.
function load(key: string): string | null {
  try {
    return window.localStorage.getItem(key);
  } catch {
    return null;
  }
}

function save(key: string, value: string): void {
  try {
    window.localStorage.setItem(key, value);
  } catch {
    // Not persisted; the app still works.
  }
}

/** An open file: a name shown on its tab and its program text. */
export interface DeltaFile {
  id: string;
  name: string;
  source: string;
}

interface Files {
  files: DeltaFile[];
  activeId: string;
}

let counter = 0;
function newId(): string {
  counter += 1;
  return `${Date.now().toString(36)}-${counter}`;
}

function initialFiles(): Files {
  try {
    const saved = JSON.parse(load(FILES_KEY) ?? 'null') as Files | null;
    if (saved && Array.isArray(saved.files) && saved.files.length > 0) {
      const files = saved.files.filter(
        (f): f is DeltaFile => typeof f?.id === 'string' && typeof f.name === 'string' && typeof f.source === 'string',
      );
      const first = files[0];
      if (first) {
        return { files, activeId: files.some((f) => f.id === saved.activeId) ? saved.activeId : first.id };
      }
    }
  } catch {
    // Corrupt storage: start fresh below.
  }
  // Before files existed the program lived under one key; keep it as the first file.
  const id = newId();
  return { files: [{ id, name: 'tour.delta', source: load(OLD_SOURCE_KEY) ?? DEFAULT_SOURCE }], activeId: id };
}

/** A name not already used: base.delta, base-2.delta, … */
export function uniqueName(base: string, taken: readonly string[]): string {
  let name = `${base}.delta`;
  for (let i = 2; taken.includes(name); i++) name = `${base}-${i}.delta`;
  return name;
}

/** A range of the program the editor should select and scroll to. */
export interface Reveal {
  from: number;
  to: number;
}

interface UiStore {
  files: DeltaFile[];
  activeId: string;
  /** The active file's program text (kept in sync with `files`). */
  source: string;
  setSource: (source: string) => void;
  openFile: (id: string) => void;
  newFile: () => void;
  closeFile: (id: string) => void;
  /** Opens an example in its own tab (or switches to it if it is already open). */
  openExample: (example: Example) => void;
  theme: Theme;
  setTheme: (theme: Theme) => void;
  /** Auto: re-evaluate while typing. Off: only Execute (⌘↵ / Ctrl+Enter) updates. */
  auto: boolean;
  setAuto: (auto: boolean) => void;
  /** The object shown in the diagram, tuple, δ-table and steps (by name). */
  selected: string | null;
  select: (name: string) => void;
  /** Trace step shown in the diagram, or null for the finished result. */
  step: number | null;
  setStep: (step: number | null) => void;
  /** A pending "jump to this code" request, consumed by the editor. */
  reveal: Reveal | null;
  revealRange: (range: Reveal) => void;
  clearReveal: () => void;
  /** Cursor position for the status bar (1-based). */
  cursor: { line: number; col: number; offset: number };
  setCursor: (cursor: { line: number; col: number; offset: number }) => void;
  /** Phone only: hides the tab bar while typing so the symbol bar sits on the keyboard. */
  editorFocused: boolean;
  setEditorFocused: (focused: boolean) => void;
  phoneView: PhoneView;
  setPhoneView: (view: PhoneView) => void;
  inspectorTab: InspectorTab;
  setInspectorTab: (tab: InspectorTab) => void;
  sheetTab: SheetTab;
  setSheetTab: (tab: SheetTab) => void;
  sheetOpen: boolean;
  setSheetOpen: (open: boolean) => void;
  rail: RailPanel | null;
  setRail: (panel: RailPanel | null) => void;
  paletteOpen: boolean;
  setPaletteOpen: (open: boolean) => void;
}

function initialTheme(): Theme {
  const saved = load(THEME_KEY);
  return saved === 'light' || saved === 'dark' ? saved : 'dark';
}

function persist(files: DeltaFile[], activeId: string): void {
  save(FILES_KEY, JSON.stringify({ files, activeId }));
}

const start = initialFiles();

export const useUiStore = create<UiStore>()((set, get) => {
  /** Switches to a file: its text becomes the program, and per-program view state resets. */
  const activate = (files: DeltaFile[], activeId: string) => {
    persist(files, activeId);
    const source = files.find((f) => f.id === activeId)?.source ?? '';
    set({ files, activeId, source, selected: null, step: null, reveal: null });
  };

  return {
    files: start.files,
    activeId: start.activeId,
    source: start.files.find((f) => f.id === start.activeId)?.source ?? '',
    setSource: (source) => {
      const { files, activeId } = get();
      const next = files.map((f) => (f.id === activeId ? { ...f, source } : f));
      persist(next, activeId);
      set({ files: next, source });
    },
    openFile: (id) => {
      if (id !== get().activeId) activate(get().files, id);
    },
    newFile: () => {
      const { files } = get();
      const file = { id: newId(), name: uniqueName('untitled', files.map((f) => f.name)), source: '' };
      activate([...files, file], file.id);
    },
    closeFile: (id) => {
      const { files, activeId } = get();
      const i = files.findIndex((f) => f.id === id);
      const rest = files.filter((f) => f.id !== id);
      if (rest.length === 0) {
        const file = { id: newId(), name: 'untitled.delta', source: '' };
        activate([file], file.id);
        return;
      }
      if (id === activeId) activate(rest, (rest[Math.max(0, i - 1)] ?? rest[0])?.id ?? '');
      else {
        persist(rest, activeId);
        set({ files: rest });
      }
    },
    openExample: (example) => {
      const { files } = get();
      const name = `${example.id}.delta`;
      const open = files.find((f) => f.name === name);
      if (open) {
        activate(files, open.id);
        return;
      }
      const file = { id: newId(), name, source: example.source };
      activate([...files, file], file.id);
    },
    theme: initialTheme(),
    setTheme: (theme) => {
      save(THEME_KEY, theme);
      set({ theme });
    },
    auto: load(AUTO_KEY) !== 'off',
    setAuto: (auto) => {
      save(AUTO_KEY, auto ? 'on' : 'off');
      set({ auto });
    },
    selected: null,
    select: (selected) => set({ selected, step: null }),
    step: null,
    setStep: (step) => set({ step }),
    reveal: null,
    // On a phone the editor is behind a tab, so jumping to code also switches to it.
    revealRange: (reveal) => set({ reveal, phoneView: 'code' }),
    clearReveal: () => set({ reveal: null }),
    cursor: { line: 1, col: 1, offset: 0 },
    setCursor: (cursor) => set({ cursor }),
    editorFocused: false,
    setEditorFocused: (editorFocused) => set({ editorFocused }),
    phoneView: 'code',
    setPhoneView: (phoneView) => set({ phoneView }),
    inspectorTab: 'tuple',
    setInspectorTab: (inspectorTab) => set({ inspectorTab }),
    sheetTab: 'tuple',
    setSheetTab: (sheetTab) => set({ sheetTab }),
    sheetOpen: true,
    setSheetOpen: (sheetOpen) => set({ sheetOpen }),
    rail: null,
    setRail: (rail) => set({ rail }),
    paletteOpen: false,
    setPaletteOpen: (paletteOpen) => set({ paletteOpen }),
  };
});
