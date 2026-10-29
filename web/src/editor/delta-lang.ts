// Delta language support for CodeMirror: highlighting, completion and hover docs
// (docs/plan/04-DSL.md). The engine stays the only parser; this tokenizer is just for colors.
import {
  snippetCompletion,
  type Completion,
  type CompletionContext,
  type CompletionResult,
} from '@codemirror/autocomplete';
import {
  HighlightStyle,
  LanguageSupport,
  StreamLanguage,
  StringStream,
  syntaxHighlighting,
  type StreamParser,
} from '@codemirror/language';
import { hoverTooltip, type Tooltip } from '@codemirror/view';
import { tags as t } from '@lezer/highlight';
import type { OutlineView } from '../engine/api';

const DECLARATIONS = ['dfa', 'nfa', 'gnfa', 'regex'];
const COMMANDS = ['let', 'check', 'run'];
const OPERATIONS = ['union', 'intersect', 'concat', 'star'];
const WORDS = [
  'accepts',
  'rejects',
  'on',
  'over',
  'table',
  'partial',
  'mode',
  'order',
  'strict',
  'lenient',
  'simplified',
  'faithful',
  'full',
  'true',
  'false',
];
/** The names of a tuple's components (Q Σ Γ δ q0 F start accept reject, plus ASCII spellings). */
const FIELDS = ['Q', 'F', 'q0', 'Sigma', 'Gamma', 'delta', 'start', 'accept', 'reject'];
const ASCII_SYMBOLS = ['eps', 'empty'];
/** Fields whose set lists alphabet symbols rather than states. */
const SET_OF_SYMBOLS = ['Σ', 'Sigma', 'Γ', 'Gamma'];

/**
 * What the next words are: plain names, or symbols of the alphabet (inside `Σ = { … }` and
 * `over { … }`, between `-` and `->` on an arrow, in a δ table's header row, and in a regex body).
 */
interface State {
  ctx: 'none' | 'set' | 'label' | 'regex' | 'header';
  /** The previous significant token, to recognise `Σ =`, `over`, `table {` and `regex R … =`. */
  prev: string;
  /** Inside `regex NAME …` before its `=`. */
  regexHead: boolean;
  /** A `table {` was just opened, so the next line starting with `|` is the header. */
  tableOpen: boolean;
  /** The word before the last `=`, so `{` after `Σ =` opens a set of symbols but `Q = {` does not. */
  field: string;
}

const parser: StreamParser<State> = {
  name: 'delta',
  startState: () => ({ ctx: 'none', prev: '', regexHead: false, tableOpen: false, field: '' }),
  copyState: (s) => ({ ...s }),
  token(stream, state) {
    if (stream.sol()) {
      // Statements are lines: a regex body or a table header ends with its line.
      if (state.ctx === 'regex' || state.ctx === 'header' || state.ctx === 'label') state.ctx = 'none';
      if (state.tableOpen && stream.match(/^\s*\|/, false)) {
        state.ctx = 'header';
        state.tableOpen = false;
      }
    }
    if (stream.eatSpace()) return null;
    const style = next(stream, state);
    const text = stream.current();
    if (style !== 'comment') state.prev = text;
    return style;
  },
  tokenTable: {
    field: t.propertyName,
    func: t.function(t.variableName),
    symbol: t.string,
  },
  languageData: {
    commentTokens: { line: '//', block: { open: '/*', close: '*/' } },
    closeBrackets: { brackets: ['(', '[', '{', '"'] },
  },
};

function next(stream: StringStream, state: State): string | null {
  if (stream.match('//')) {
    stream.skipToEnd();
    return 'comment';
  }
  if (stream.match('/*')) {
    if (!stream.skipTo('*/')) stream.skipToEnd();
    else stream.match('*/');
    return 'comment';
  }
  if (stream.match(/^"(?:[^"\\]|\\.)*"?/)) return 'string';
  const symbols = state.ctx !== 'none';
  if (stream.match(/^<[A-Za-z0-9_]+>/)) return 'symbol';
  // Arrows: `-a->`, `-[a,b]->`, `→`. The label between `-` and `->` is made of symbols.
  if (stream.match(/^(?:->|→)/)) {
    if (state.ctx === 'label') state.ctx = 'none';
    return 'operator';
  }
  if (stream.match(/^-(?!>)/)) {
    if (state.ctx === 'none' && state.prev !== '') state.ctx = 'label';
    return 'operator';
  }
  if (stream.match(/^[Σδ]/)) {
    if (stream.current() === 'Σ' && state.ctx === 'regex') return 'symbol';
    return 'field';
  }
  if (stream.match(/^Γ/)) return 'field';
  if (stream.match(/^[ε∅]/)) return symbols ? 'symbol' : 'keyword';
  if (stream.match(/^[∪∘*+|^]/)) return 'operator';
  if (stream.match('{')) {
    if (state.prev === '=' && SET_OF_SYMBOLS.includes(state.field)) state.ctx = 'set';
    if (state.prev === 'over') state.ctx = 'set';
    if (state.prev === 'table') state.tableOpen = true;
    return 'brace';
  }
  if (stream.match('}')) {
    if (state.ctx === 'set') state.ctx = 'none';
    return 'brace';
  }
  if (stream.match('=')) {
    if (state.regexHead) {
      state.regexHead = false;
      state.ctx = 'regex';
    }
    state.field = state.prev;
    return 'operator';
  }
  if (stream.match(/^[A-Za-z0-9_']+/)) {
    const w = stream.current();
    if (state.ctx === 'set' || state.ctx === 'label' || state.ctx === 'header') return 'symbol';
    // In R^k the k is a count, not a symbol.
    if (state.ctx === 'regex' && state.prev === '^' && /^\d+$/.test(w)) return 'number';
    if (state.ctx === 'regex') return w === 'Sigma' ? 'symbol' : w === 'eps' || w === 'empty' ? 'keyword' : 'symbol';
    if (w === 'regex' && state.prev !== '=' && !stream.match(/^\s*\(/, false)) state.regexHead = true;
    // A word followed by `(` is a function call: dfa(N), union(A, B).
    if (stream.match(/^\s*\(/, false)) return 'func';
    if (COMMANDS.includes(w) || OPERATIONS.includes(w)) return 'func';
    if (DECLARATIONS.includes(w) || WORDS.includes(w) || ASCII_SYMBOLS.includes(w)) return 'keyword';
    if (FIELDS.includes(w)) return 'field';
    if (/^\d+$/.test(w)) return 'number';
    return 'variableName';
  }
  stream.next();
  return 'punctuation';
}

export const deltaLanguage = StreamLanguage.define(parser);

/** VS Code Dark+ / Light+ token colors (the UI palette is not used for code). */
export const deltaHighlight = HighlightStyle.define([
  { tag: t.comment, color: 'var(--syn-comment)', fontStyle: 'italic' },
  { tag: t.keyword, color: 'var(--syn-keyword)' },
  { tag: t.propertyName, color: 'var(--syn-field)' },
  { tag: t.function(t.variableName), color: 'var(--syn-function)' },
  { tag: t.string, color: 'var(--syn-string)' },
  { tag: t.number, color: 'var(--syn-number)' },
  { tag: [t.operator, t.punctuation, t.brace, t.variableName], color: 'var(--syn-text)' },
]);

/** Splits source into (text, token style) pairs with the real tokenizer; for tests. */
export function tokenize(src: string): [string, string | null][] {
  const state = parser.startState?.(2) ?? { ctx: 'none', prev: '', regexHead: false, tableOpen: false, field: '' };
  const out: [string, string | null][] = [];
  for (const line of src.split('\n')) {
    const stream = new StringStream(line, 2, 2);
    while (!stream.eol()) {
      const style = parser.token(stream, state);
      const text = stream.current();
      if (text.trim()) out.push([text, style]);
      stream.start = stream.pos;
    }
  }
  return out;
}

// --- Hover docs (docs/plan/05-PLAYGROUND-UX.md §5): the book line behind each keyword. ---

interface Doc {
  title: string;
  body: string;
}

export const KEYWORD_DOCS: Record<string, Doc> = {
  dfa: {
    title: 'Definition 1.5 — finite automaton',
    body: 'A finite automaton is a 5-tuple (Q, Σ, δ, q₀, F). As a function, dfa(N) is the subset construction of Theorem 1.39: every NFA has an equivalent DFA.',
  },
  nfa: {
    title: 'Definition 1.37 — nondeterministic finite automaton',
    body: 'A 5-tuple (Q, Σ, δ, q₀, F) with δ : Q × Σ_ε → P(Q). As a function, nfa(R) is Lemma 1.55 (regex → NFA) or the trivial DFA ⊆ NFA view.',
  },
  gnfa: {
    title: 'Definition 1.64 — generalized NFA',
    body: 'A 5-tuple (Q, Σ, δ, q_start, q_accept) whose arrows are labeled by regular expressions. gnfa(D) adds the new start and accept states of Lemma 1.60.',
  },
  regex: {
    title: 'Definition 1.52 — regular expression',
    body: 'R is a, ε, ∅, (R₁ ∪ R₂), (R₁ ∘ R₂) or (R₁*). As a function, regex(D) is Lemma 1.60: state elimination (CONVERT(G)).',
  },
  δ: { title: 'Definition 1.5 — transition function', body: 'δ : Q × Σ → Q for a DFA; δ : Q × Σ_ε → P(Q) for an NFA (Definition 1.37).' },
  delta: { title: 'Definition 1.5 — transition function', body: 'ASCII spelling of δ.' },
  Q: { title: 'Definition 1.5', body: 'Q is a finite set called the states.' },
  Σ: { title: 'Definition 1.5', body: 'Σ is a finite set called the alphabet.' },
  Sigma: { title: 'Definition 1.5', body: 'ASCII spelling of Σ, a finite set called the alphabet.' },
  q0: { title: 'Definition 1.5', body: 'q₀ ∈ Q is the start state.' },
  F: { title: 'Definition 1.5', body: 'F ⊆ Q is the set of accept states. F can be ∅ (p. 36).' },
  ε: { title: '§0.2 — the empty string', body: 'The string of length zero, written ε. ASCII: eps.' },
  eps: { title: '§0.2 — the empty string', body: 'ASCII spelling of ε.' },
  '∅': { title: 'Definition 1.52', body: '∅ is the regular expression for the empty language. ASCII: empty.' },
  empty: { title: 'Definition 1.52', body: 'ASCII spelling of ∅, the empty language.' },
  '∪': { title: 'Definition 1.52 — union', body: '(R₁ ∪ R₂). ASCII: |. Note: + means R⁺ = RR*, not union (p. 65).' },
  let: { title: 'let', body: 'Names the result of a conversion: let D = dfa(N).' },
  union: {
    title: 'Theorem 1.25 / Theorem 1.45 — union',
    body: 'union(D1, D2) on two DFAs is the product construction: Q₁ × Q₂ (reachable pairs), start (q₁, q₂), F = (F₁ × Q₂) ∪ (Q₁ × F₂). With an NFA it is Theorem 1.45: a new start with ε-arrows to both starts.',
  },
  intersect: {
    title: 'Theorem 1.25 (footnote) — intersection',
    body: 'intersect(D1, D2) on two DFAs: the product construction with F = F₁ × F₂.',
  },
  concat: {
    title: 'Theorem 1.47 — concatenation',
    body: 'concat(N1, N2): start q₁, ε-arrows from every accept state of N1 to q₂, F = F₂.',
  },
  star: {
    title: 'Theorem 1.49 — star',
    body: 'star(N1): a new start q₀ that is also accepting, an ε-arrow to q₁, ε-arrows from F₁ back to q₁. Not "make the start accepting" (Exercise 1.15).',
  },
  check: { title: 'check', body: 'check M accepts "w", … / check M rejects "w", … — results appear in the Tests tab.' },
  run: { title: 'run', body: 'run M on "w" — open it in the Run panel to step through it.' },
  table: { title: 'δ as a table', body: 'Write δ the way the book draws it: one row per state, one column per symbol.' },
};

function wordAt(text: string, pos: number): { from: number; to: number; word: string } | null {
  const isWord = (c: string | undefined) => !!c && /[A-Za-z0-9_'εΣδ∅∪]/.test(c);
  let from = pos;
  let to = pos;
  while (from > 0 && isWord(text[from - 1])) from--;
  while (to < text.length && isWord(text[to])) to++;
  if (from === to) return null;
  return { from, to, word: text.slice(from, to) };
}

/** Hover docs for keywords and object names; `objects` gives each name's kind and origin. */
export function deltaHover(objects: () => readonly { name: string; kind: string; derived: string | null }[]) {
  return hoverTooltip((view, pos): Tooltip | null => {
    const line = view.state.doc.lineAt(pos);
    const hit = wordAt(line.text, pos - line.from);
    if (!hit) return null;
    const doc = KEYWORD_DOCS[hit.word];
    const object = objects().find((o) => o.name === hit.word);
    if (!doc && !object) return null;
    return {
      pos: line.from + hit.from,
      end: line.from + hit.to,
      above: true,
      create: () => {
        const dom = document.createElement('div');
        dom.className = 'cm-delta-doc';
        const title = document.createElement('strong');
        const body = document.createElement('p');
        if (doc) {
          title.textContent = doc.title;
          body.textContent = doc.body;
        } else if (object) {
          title.textContent = `${object.name} : ${object.kind.toUpperCase()}`;
          body.textContent = object.derived ?? 'Declared in this program.';
        }
        dom.append(title, body);
        return { dom };
      },
    };
  });
}

// --- Completion ---

const SNIPPETS: Completion[] = [
  snippetCompletion(
    'dfa ${M} {\n  Q  = { ${q1}, ${q2} }\n  Σ  = { ${0}, ${1} }\n  δ  = table {\n         | 0  | 1\n      q1 | q1 | q2\n      q2 | q1 | q2\n  }\n  q0 = q1\n  F  = { q2 }\n}',
    { label: 'dfa', detail: 'Definition 1.5', type: 'keyword', boost: 2 },
  ),
  snippetCompletion(
    'nfa ${N} {\n  Q = { ${q1}, ${q2} }\n  Σ = { ${a}, ${b} }\n  δ {\n    q1 -[a,b]-> q1\n    q1 -a-> q2\n  }\n  q0 = q1\n  F  = { q2 }\n}',
    { label: 'nfa', detail: 'Definition 1.37', type: 'keyword', boost: 2 },
  ),
  snippetCompletion('regex ${R} over { ${a}, ${b} } = ${(a ∪ b)*}', {
    label: 'regex',
    detail: 'Definition 1.52',
    type: 'keyword',
    boost: 2,
  }),
  snippetCompletion('let ${D} = dfa(${N})', { label: 'let … = dfa(…)', detail: 'Theorem 1.39', type: 'keyword' }),
  snippetCompletion('let ${N} = nfa(${R})', { label: 'let … = nfa(…)', detail: 'Lemma 1.55', type: 'keyword' }),
  snippetCompletion('let ${G} = gnfa(${D})', { label: 'let … = gnfa(…)', detail: 'Lemma 1.60', type: 'keyword' }),
  snippetCompletion('let ${R} = regex(${D}, mode: simplified)', {
    label: 'let … = regex(…)',
    detail: 'Lemma 1.60',
    type: 'keyword',
  }),
  snippetCompletion('let ${M} = union(${M1}, ${M2})', { label: 'let … = union(…, …)', detail: 'Theorem 1.25 / 1.45', type: 'keyword' }),
  snippetCompletion('let ${M} = intersect(${D1}, ${D2})', {
    label: 'let … = intersect(…, …)',
    detail: 'Theorem 1.25',
    type: 'keyword',
  }),
  snippetCompletion('let ${N} = concat(${N1}, ${N2})', { label: 'let … = concat(…, …)', detail: 'Theorem 1.47', type: 'keyword' }),
  snippetCompletion('let ${N} = star(${N1})', { label: 'let … = star(…)', detail: 'Theorem 1.49', type: 'keyword' }),
  snippetCompletion('check ${M} accepts "${w}"', { label: 'check … accepts', type: 'keyword' }),
  snippetCompletion('check ${M} rejects "${w}"', { label: 'check … rejects', type: 'keyword' }),
  snippetCompletion('run ${M} on "${w}"', { label: 'run … on', type: 'keyword' }),
];

const PLAIN: Completion[] = [...OPERATIONS, ...WORDS, ...FIELDS, ...ASCII_SYMBOLS, 'Σ', 'δ', 'ε', '∅'].map((w) => ({
  label: w,
  type: 'keyword',
  ...(KEYWORD_DOCS[w] ? { info: KEYWORD_DOCS[w].body } : {}),
}));

/** Keywords, snippets, the program's objects, and the states/symbols of the machine under the cursor. */
export function deltaCompletion(outline: () => readonly OutlineView[]) {
  return (ctx: CompletionContext): CompletionResult | null => {
    const word = ctx.matchBefore(/[A-Za-z0-9_']+|[εΣδ∅]/);
    if (!word && !ctx.explicit) return null;
    const from = word?.from ?? ctx.pos;
    const items = outline();
    const here = items.find((o) => o.from <= ctx.pos && ctx.pos <= o.to);
    const objects: Completion[] = items.map((o) => ({ label: o.name, type: 'class', detail: o.kind }));
    const local: Completion[] = here
      ? [
          ...here.states.map((s) => ({ label: s, type: 'variable', detail: 'state' })),
          ...here.symbols.map((s) => ({ label: s, type: 'constant', detail: 'symbol' })),
        ]
      : [];
    const atLineStart = /^\s*$/.test(ctx.state.doc.lineAt(from).text.slice(0, from - ctx.state.doc.lineAt(from).from));
    return {
      from,
      options: [...(atLineStart ? SNIPPETS : []), ...local, ...objects, ...PLAIN],
      validFor: /^[A-Za-z0-9_']*$/,
    };
  };
}

export function delta(): LanguageSupport {
  return new LanguageSupport(deltaLanguage, [syntaxHighlighting(deltaHighlight)]);
}
