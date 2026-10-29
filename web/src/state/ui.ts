import { create } from 'zustand';
import { DEFAULT_SOURCE } from '../examples';

export type PhoneView = 'code' | 'diagram' | 'tuple' | 'run' | 'more';
export type InspectorTab = 'tuple' | 'delta' | 'steps' | 'tests' | 'problems';
export type SheetTab = InspectorTab | 'run';
export type Theme = 'dark' | 'light';

const SOURCE_KEY = 'delta.source';
const THEME_KEY = 'delta.theme';

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

/** A range of the program the editor should select and scroll to. */
export interface Reveal {
  from: number;
  to: number;
}

interface UiStore {
  /** The program text. Lives here (not in the editor) so switching layouts never loses it. */
  source: string;
  setSource: (source: string) => void;
  theme: Theme;
  setTheme: (theme: Theme) => void;
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
  examplesOpen: boolean;
  setExamplesOpen: (open: boolean) => void;
}

function initialTheme(): Theme {
  const saved = load(THEME_KEY);
  return saved === 'light' || saved === 'dark' ? saved : 'dark';
}

export const useUiStore = create<UiStore>()((set) => ({
  source: load(SOURCE_KEY) ?? DEFAULT_SOURCE,
  setSource: (source) => {
    save(SOURCE_KEY, source);
    set({ source });
  },
  theme: initialTheme(),
  setTheme: (theme) => {
    save(THEME_KEY, theme);
    set({ theme });
  },
  selected: null,
  select: (selected) => set({ selected, step: null }),
  step: null,
  setStep: (step) => set({ step }),
  reveal: null,
  // On a phone the editor is behind a tab, so jumping to code also switches to it.
  revealRange: (reveal) => set({ reveal, phoneView: 'code' }),
  clearReveal: () => set({ reveal: null }),
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
  examplesOpen: false,
  setExamplesOpen: (examplesOpen) => set({ examplesOpen }),
}));
