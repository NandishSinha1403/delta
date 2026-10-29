import { create } from 'zustand';

export type PhoneView = 'code' | 'diagram' | 'tuple' | 'run' | 'more';
export type InspectorTab = 'tuple' | 'delta' | 'steps' | 'tests';
export type SheetTab = InspectorTab | 'run';

// Phase 0 seed text. The editor is not wired to the engine yet; it shows what M1 looks like
// in the Delta language (docs/plan/04-DSL.md §2.1).
const SEED = `// Phase 0: the editor is not connected to the engine yet.
// The tuple you see comes from Rust (delta-core) via WASM in a Web Worker.

// Figure 1.6: M1 accepts strings with at least one 1
// and an even number of 0s after the last 1
dfa M1 {
  Q  = { q1, q2, q3 }
  Σ  = { 0, 1 }
  δ  = table {
          | 0  | 1
       q1 | q1 | q2
       q2 | q3 | q2
       q3 | q2 | q2
  }
  q0 = q1
  F  = { q2 }
}
`;

interface UiStore {
  /** The program text. Lives here (not in the editor) so switching layouts never loses it. */
  source: string;
  setSource: (source: string) => void;
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
}

export const useUiStore = create<UiStore>()((set) => ({
  source: SEED,
  setSource: (source) => set({ source }),
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
}));
