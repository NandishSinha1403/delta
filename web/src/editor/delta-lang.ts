// Delta 2 language support for CodeMirror: highlighting, completion and hover docs
// (03-PRODUCTS.md §3.1). The engine stays the only parser; this tokenizer is just for colors.
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

/** Statement keywords (01-LANGUAGE.md §8, §15). */
const KEYWORDS = [
  'let', 'var', 'fn', 'if', 'else', 'for', 'while', 'return', 'print', 'show', 'assert', 'check',
  'run', 'model', 'machine', 'lang', 'use', 'budget', 'explore', 'fixpoint', 'cfg', 'and', 'or',
  'not', 'in', 'true', 'false', 'none', 'union', 'inter', 'minus', 'subset', 'equiv', 'union_all',
  'eps', 'empty',
];
/** Words that are keywords only in their place (`check M accepts`, `step R on a`, …). */
const CONTEXTUAL = [
  'accepts', 'rejects', 'up', 'to', 'on', 'from', 'as', 'by', 'over', 'any', 'step', 'edges',
  'table', 'extends', 'strict', 'notation', 'explain', 'ends', 'starts', 'with', 'contains',
];
/** The standard prelude's models (01-LANGUAGE.md §4). */
export const MODELS = ['dfa', 'nfa', 'gnfa', 'pda', 'dpda', 'tm', 'ntm', 'lba', 'mtm', 'twdfa', 'mealy', 'moore', 'fst', 'ctr'];
/** Machine fields and their textbook letters (01-LANGUAGE.md §5.1). */
const FIELDS = ['states', 'alphabet', 'input', 'stack', 'tape', 'output', 'start', 'accept', 'final', 'reject', 'blank', 'Q', 'Σ', 'Γ', 'Δ', 'F', 'δ', 'delta', 'Sigma', 'Gamma'];
/** The library (01-LANGUAGE.md §12). */
export const LIBRARY = [
  'determinize', 'minimize', 'complete', 'trim', 'remove_epsilon', 'reverse', 'complement', 'union',
  'intersect', 'difference', 'concat', 'star', 'product', 'to_nfa', 'to_regex', 'to_gnfa',
  'simplify', 'equivalent', 'subset_of', 'is_empty', 'is_finite', 'is_universal', 'shortest',
  'words', 'count', 'len', 'rev', 'set', 'list', 'sort', 'prefixes', 'suffixes', 'substrings',
  'power_set', 'min', 'max', 'abs', 'sum', 'keys', 'values', 'first', 'last', 'steps',
];
const CLASSES = ['Finite', 'Regular', 'DCF', 'CF', 'CS', 'Decidable', 'Recognizable', 'Any'];

interface State {
  /** Inside a machine body's braces (depth), where `-label->` is an edge. */
  machine: number;
  /** The previous word, to see `dfa M {` open a machine body. */
  prev: string;
  braces: number;
  machineAt: number[];
}

const parser: StreamParser<State> = {
  name: 'delta',
  startState: () => ({ machine: 0, prev: '', braces: 0, machineAt: [] }),
  copyState: (s) => ({ ...s, machineAt: [...s.machineAt] }),
  token(stream: StringStream, state: State): string | null {
    if (stream.eatSpace()) return null;
    if (stream.match('//')) {
      stream.skipToEnd();
      return 'comment';
    }
    if (stream.match('/*')) {
      while (!stream.eol() && !stream.match('*/')) stream.next();
      return 'comment';
    }
    const inMachine = state.machineAt.length > 0;
    // An edge `-label->` or `-[labels]->` inside a machine body.
    if (inMachine && stream.peek() === '-' && !stream.match('->', false)) {
      stream.next();
      if (stream.peek() === '[') {
        let depth = 0;
        while (!stream.eol()) {
          const c = stream.next();
          if (c === '[') depth++;
          else if (c === ']') {
            depth--;
            if (depth === 0 && stream.match('->')) break;
          }
        }
      } else {
        while (!stream.eol() && !stream.match('->')) stream.next();
      }
      return 'symbol';
    }
    if (stream.match(/^re"(\\.|[^"\\])*"?/)) return 'string';
    if (stream.match(/^"(\\.|[^"\\])*"?/)) return 'string';
    if (stream.match(/^'(<[^>']*>|[^'])'/)) return 'symbol';
    if (stream.match(/^:[\p{L}_][\p{L}\p{N}_']*/u) && /[\s([{,=:]|^$/.test(state.prev.slice(-1) || ' ')) {
      return 'atom';
    }
    if (stream.match(/^\d[\d_]*/)) {
      state.prev = '0';
      return 'number';
    }
    const word = stream.match(/^[\p{L}_][\p{L}\p{N}_']*/u);
    if (word && Array.isArray(word)) {
      const w = word[0];
      const prev = state.prev;
      state.prev = w;
      if (w === 'ε' || w === 'λ') return 'symbol';
      if (MODELS.includes(w) && !['==', '.'].includes(prev)) {
        state.prev = `model:${w}`;
        return 'keyword';
      }
      if (KEYWORDS.includes(w)) return w === 'true' || w === 'false' || w === 'none' ? 'atom' : 'keyword';
      if (inMachine && FIELDS.includes(w)) return 'field';
      if (FIELDS.includes(w) && (w === 'δ' || w === 'Σ')) return 'field';
      if (CONTEXTUAL.includes(w)) return 'keyword';
      if (CLASSES.includes(w)) return 'type';
      if (LIBRARY.includes(w) && stream.peek() === '(') return 'func';
      if (stream.peek() === '(' && prev === 'fn') return 'func';
      return 'variableName';
    }
    const c = stream.next() ?? '';
    if (c === '{') {
      state.braces++;
      // `dfa {` or `dfa M {` (or with options) opens a machine body.
      if (state.prev.startsWith('model:') || state.prev === 'machine-name') state.machineAt.push(state.braces);
      state.prev = '{';
      return 'brace';
    }
    if (c === '}') {
      if (state.machineAt[state.machineAt.length - 1] === state.braces) state.machineAt.pop();
      state.braces = Math.max(0, state.braces - 1);
      state.prev = '}';
      return 'brace';
    }
    // A name after a model name keeps the machine-literal context (`dfa M {`).
    if (state.prev.startsWith('model:') && c === '(') {
      while (!stream.eol() && stream.next() !== ')');
      return 'meta';
    }
    state.prev = c;
    return '∪∩∖∁∈∉⊆≡≠≤≥⋃→⊔ε∅⁺·∘=<>+-*/%^|.'.includes(c) ? 'operator' : null;
  },
};

// Keep `dfa M {` recognised: after a model name, the next word is its name.
const baseToken = parser.token;
parser.token = (stream, state) => {
  const before = state.prev;
  const style = baseToken(stream, state);
  if (before.startsWith('model:') && style === 'variableName') state.prev = 'machine-name';
  return style;
};

export const deltaLanguage = StreamLanguage.define(parser);

/** VS Code Dark+ / Light+ token colors (the variables come from index.css). */
export const deltaHighlight = HighlightStyle.define([
  { tag: t.keyword, color: 'var(--tok-keyword)' },
  { tag: t.propertyName, color: 'var(--tok-field)' },
  { tag: t.function(t.variableName), color: 'var(--tok-func)' },
  { tag: [t.string, t.special(t.string)], color: 'var(--tok-string)' },
  { tag: t.comment, color: 'var(--tok-comment)', fontStyle: 'italic' },
  { tag: t.number, color: 'var(--tok-number)' },
  { tag: [t.atom, t.typeName], color: 'var(--tok-func)' },
]);

/** Tokens with their style names, for tests. */
export function tokenize(src: string): [string, string | null][] {
  const out: [string, string | null][] = [];
  const state = parser.startState?.(2) ?? { machine: 0, prev: '', braces: 0, machineAt: [] };
  for (const line of src.split('\n')) {
    const stream = new StringStream(line, 2, 2);
    while (!stream.eol()) {
      const style = parser.token(stream, state);
      const text = stream.current();
      if (text.trim() !== '') out.push([text, style]);
      stream.start = stream.pos;
    }
  }
  return out;
}

// --- Hover docs ---

interface Doc {
  title: string;
  body: string;
}

export const KEYWORD_DOCS: Record<string, Doc> = {
  dfa: { title: 'dfa — deterministic finite automaton', body: 'model dfa { input one-way; deterministic; total; accept by final }. Write `dfa M { … }`; convert with determinize(N).' },
  nfa: { title: 'nfa — nondeterministic finite automaton', body: 'model nfa { input one-way; epsilon; accept by final }. Make one from a regex with to_nfa(R).' },
  pda: { title: 'pda — pushdown automaton', body: 'Labels `a, X → YZ` read a, pop X, push YZ (Y on top). ε means "omit that operation".' },
  tm: { title: 'tm — Turing machine', body: 'Labels `0 → x, R` write and move, `x → R` only moves. A run ends in accept, reject, loops or unknown.' },
  lang: { title: 'lang — a language', body: 'A set of words with a class (Finite ⊂ Regular ⊂ DCF ⊂ CF ⊂ CS ⊂ Decidable ⊂ Recognizable ⊂ Any). Questions are allowed only where they are decidable.' },
  check: { title: 'check', body: 'check M accepts "w", … · check A ≡ B · check A ≡ B up to n · check L.is_empty(). Results appear in Tests.' },
  explore: { title: 'explore — build a machine from successors', body: 'explore { alphabet S; start s₀; step π on a -> e; accept π if φ }. Only reachable states are created.' },
  fixpoint: { title: 'fixpoint — repeat until nothing changes', body: 'fixpoint X from init { expr using X }. Budgeted.' },
  run: { title: 'run M on "w"', body: 'Shows the run step by step in the Run view; the verdict is accept, reject, loops or unknown.' },
  model: { title: 'model — what a machine is made of', body: 'model m { input …; storage …; accept by …; [deterministic] [total] [epsilon] }. Use `extends` to build on another model.' },
  cfg: { title: 'cfg — context-free grammar', body: 'cfg G { S -> a S b | ε }. Variables are the left-hand sides; <Name> is one symbol.' },
  use: { title: 'use', body: 'use strict · use notation "hmu" · use explain "sipser3e" (adds citations, never changes results).' },
  budget: { title: 'budget', body: 'budget { steps: …, states: …, depth: …, output: … }. Hitting a budget is an outcome, not a crash.' },
};

/** Hover docs for keywords and object names; `objects` gives each name's kind and origin. */
export function deltaHover(objects: () => readonly { name: string; kind: string; derived: string | null }[]) {
  return hoverTooltip((view, pos): Tooltip | null => {
    const { from, to, text } = view.state.doc.lineAt(pos);
    let start = pos - from;
    let end = pos - from;
    while (start > 0 && /[\p{L}\p{N}_']/u.test(text[start - 1] ?? '')) start--;
    while (end < text.length && /[\p{L}\p{N}_']/u.test(text[end] ?? '')) end++;
    if (start === end) return null;
    const word = text.slice(start, end);
    const object = objects().find((o) => o.name === word);
    const doc = KEYWORD_DOCS[word];
    if (!object && !doc) return null;
    return {
      pos: from + start,
      end: Math.min(to, from + end),
      above: true,
      create() {
        const dom = document.createElement('div');
        dom.className = 'cm-delta-hover';
        const title = document.createElement('div');
        title.className = 'cm-delta-hover-title';
        const body = document.createElement('div');
        if (object) {
          title.textContent = `${object.name} · ${object.kind}`;
          body.textContent = object.derived ?? 'Declared in this program.';
        } else if (doc) {
          title.textContent = doc.title;
          body.textContent = doc.body;
        }
        dom.append(title, body);
        return { dom };
      },
    };
  });
}

// --- Completion ---

/** LaTeX names → Unicode (03-PRODUCTS.md §3.1: `\delta` ⇥ δ). */
export const LATEX: Record<string, string> = {
  epsilon: 'ε', emptyset: '∅', cup: '∪', cap: '∩', setminus: '∖', complement: '∁', in: '∈',
  notin: '∉', subseteq: '⊆', subset: '⊆', equiv: '≡', neq: '≠', leq: '≤', geq: '≥', bigcup: '⋃',
  to: '→', blank: '⊔', delta: 'δ', Sigma: 'Σ', Gamma: 'Γ', Delta: 'Δ', lambda: 'λ', cdot: '·',
  circ: '∘', plus: '⁺',
};

const SNIPPETS: Completion[] = [
  snippetCompletion('dfa ${M} {\n  start ${q0}\n  accept ${q1}\n  ${q0} -${a}-> ${q1}\n}', { label: 'dfa', detail: 'machine literal', type: 'keyword' }),
  snippetCompletion('check ${M} accepts "${w}"', { label: 'check … accepts', detail: 'test words', type: 'keyword' }),
  snippetCompletion('lang ${L} = { w ∈ {${a}, ${b}}* | ${cond} }', { label: 'lang … set-builder', detail: 'a language', type: 'keyword' }),
  snippetCompletion('explore {\n  alphabet ${S}\n  start ${s0}\n  step ${q} on a -> ${next}\n  accept ${q} if ${cond}\n}', { label: 'explore', detail: 'construction', type: 'keyword' }),
];

/** Keywords, the library, names in scope, the machines' states, and `\latex` names. */
export function deltaCompletion(outline: () => readonly OutlineView[]) {
  return (ctx: CompletionContext): CompletionResult | null => {
    const latex = ctx.matchBefore(/\\[A-Za-z]*/);
    if (latex) {
      return {
        from: latex.from,
        options: Object.entries(LATEX).map(([name, sym]) => ({ label: `\\${name}`, apply: sym, detail: sym, type: 'text' })),
        validFor: /^\\[A-Za-z]*$/,
      };
    }
    const word = ctx.matchBefore(/[\p{L}_][\p{L}\p{N}_']*/u);
    if (!word || (word.from === word.to && !ctx.explicit)) return null;
    const items = outline();
    const options: Completion[] = [
      ...KEYWORDS.map((k) => ({ label: k, type: 'keyword' })),
      ...MODELS.map((m) => ({ label: m, type: 'class', detail: 'model' })),
      ...LIBRARY.map((f) => ({ label: f, type: 'function', apply: `${f}(`, detail: 'library' })),
      ...CLASSES.map((c) => ({ label: c, type: 'type', detail: 'class' })),
      ...items.map((o) => ({ label: o.name, type: 'variable', detail: o.kind })),
      ...items.flatMap((o) => o.states.map((s) => ({ label: s, type: 'constant', detail: `state of ${o.name}` }))),
      ...SNIPPETS,
    ];
    const seen = new Set<string>();
    return {
      from: word.from,
      options: options.filter((o) => !seen.has(o.label) && (seen.add(o.label), true)),
      validFor: /^[\p{L}\p{N}_']*$/u,
    };
  };
}

export function delta(): LanguageSupport {
  return new LanguageSupport(deltaLanguage, [syntaxHighlighting(deltaHighlight)]);
}
