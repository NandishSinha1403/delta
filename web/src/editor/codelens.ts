// Codelens hints above `let` and `check` lines (docs/plan/05-PLAYGROUND-UX.md, redesign):
// "Theorem 1.39 · subset construction · 4 states  ▶ open steps" and "✓ 3 / 3 accepted".
// They come from the engine's last evaluation, so they only show when it matches the text.
import { RangeSetBuilder, StateEffect, StateField, type EditorState } from '@codemirror/state';
import { Decoration, EditorView, WidgetType, type DecorationSet } from '@codemirror/view';
import type { Document } from '../engine/api';

export interface Lens {
  /** Offset of the statement the hint sits above. */
  at: number;
  text: string;
  tone: 'info' | 'pass' | 'fail';
  /** The object whose construction steps "open steps" shows. */
  steps?: string;
}

/** The hints for a document: one per derived object, one per `check` command. */
export function lensesFor(doc: Document): Lens[] {
  const out: Lens[] = [];
  for (const o of doc.objects) {
    if (!o.derived) continue;
    const size = o.machine ? ` · ${o.machine.states.length} state${o.machine.states.length === 1 ? '' : 's'}` : '';
    const head = o.trace?.title ?? o.derived;
    const lens: Lens = { at: o.from, text: `${head}${size}`, tone: 'info' };
    if (o.trace && o.trace.steps.length > 0) lens.steps = o.name;
    out.push(lens);
  }
  for (const r of doc.results) {
    if (r.kind !== 'check' || r.cases.length === 0) continue;
    const verb = r.cases[0]?.expected ? 'accepted' : 'rejected';
    const failed = r.cases.filter((c) => c.expected !== c.actual);
    const n = r.cases.length;
    out.push(
      failed.length === 0
        ? { at: r.from, text: `✓ ${n} / ${n} ${verb}`, tone: 'pass' }
        : {
            at: r.from,
            text: `✗ ${failed.length} of ${n} failed: ${failed.map((c) => (c.input === '' ? 'ε' : `"${c.input}"`)).join(', ')}`,
            tone: 'fail',
          },
    );
  }
  return out.sort((a, b) => a.at - b.at);
}

class LensWidget extends WidgetType {
  constructor(
    readonly lens: Lens,
    readonly onOpenSteps: (name: string) => void,
  ) {
    super();
  }

  override eq(other: LensWidget): boolean {
    return (
      other.lens.text === this.lens.text && other.lens.tone === this.lens.tone && other.lens.steps === this.lens.steps
    );
  }

  override toDOM(): HTMLElement {
    const el = document.createElement('div');
    el.className = `cm-lens cm-lens-${this.lens.tone}`;
    el.setAttribute('data-testid', 'codelens');
    const text = document.createElement('span');
    text.textContent = this.lens.text;
    el.append(text);
    const steps = this.lens.steps;
    if (steps) {
      const button = document.createElement('button');
      button.type = 'button';
      button.className = 'cm-lens-action';
      button.textContent = '▶ open steps';
      button.setAttribute('aria-label', `Open the construction steps of ${steps}`);
      button.addEventListener('mousedown', (e) => e.preventDefault());
      button.addEventListener('click', () => this.onOpenSteps(steps));
      el.append(button);
    }
    return el;
  }

  override ignoreEvent(): boolean {
    return false;
  }
}

/** Replaces the hints (sent when a fresh evaluation arrives). */
export const setLenses = StateEffect.define<Lens[]>();

function build(state: EditorState, lenses: readonly Lens[], onOpenSteps: (name: string) => void): DecorationSet {
  const b = new RangeSetBuilder<Decoration>();
  const len = state.doc.length;
  for (const lens of lenses) {
    if (lens.at > len) continue;
    const line = state.doc.lineAt(lens.at);
    b.add(line.from, line.from, Decoration.widget({ widget: new LensWidget(lens, onOpenSteps), block: true, side: -1 }));
  }
  return b.finish();
}

export function codelens(onOpenSteps: (name: string) => void) {
  const field = StateField.define<DecorationSet>({
    create: () => Decoration.none,
    update(deco, tr) {
      for (const e of tr.effects) if (e.is(setLenses)) return build(tr.state, e.value, onOpenSteps);
      // While typing the hints follow the text; a fresh evaluation replaces them.
      return deco.map(tr.changes);
    },
    provide: (f) => EditorView.decorations.from(f),
  });
  return [
    field,
    EditorView.theme({
      '.cm-lens': {
        display: 'flex',
        gap: '10px',
        paddingLeft: '4px',
        fontFamily: 'var(--font-sans)',
        fontSize: '11px',
        lineHeight: '18px',
        color: 'var(--faint)',
      },
      '.cm-lens-pass': { color: 'var(--text)' },
      '.cm-lens-fail': { color: 'var(--reject)' },
      '.cm-lens-action': {
        border: 'none',
        background: 'none',
        padding: 0,
        font: 'inherit',
        color: 'var(--accent)',
        cursor: 'pointer',
      },
      '.cm-lens-action:hover': { textDecoration: 'underline' },
    }),
  ];
}
