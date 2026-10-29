// The formal tuple in the book's notation, as TeX for KaTeX and as plain text for screen readers.
import type { MachineView } from '../engine/api';

const TEX_SPECIAL = /[\\{}$&#^_%~]/g;

function texText(s: string): string {
  return `\\text{${s.replace(TEX_SPECIAL, (c) => (c === '\\' ? '\\textbackslash{}' : `\\${c}`))}}`;
}

/** A state or object name as the book typesets it: q1 → q₁, {q1,q2} → {q₁, q₂}, qstart → q_start. */
export function texName(name: string): string {
  const sub = /^([A-Za-z])(\d+)$/.exec(name);
  if (sub) return `${sub[1]}_{${sub[2]}}`;
  if (name === 'qstart' || name === 'qaccept') return `q_{\\text{${name.slice(1)}}}`;
  if (name === '∅') return '\\emptyset';
  if (name.length > 2 && name.startsWith('{') && name.endsWith('}')) {
    const inner = name.slice(1, -1).trim();
    if (inner === '') return '\\emptyset';
    return `\\{${inner.split(',').map((p) => texName(p.trim())).join(', ')}\\}`;
  }
  if (/^[A-Za-z]$/.test(name)) return name;
  return texText(name);
}

/** An alphabet symbol: ε gets the book's ε, everything else is set upright. */
export function texSymbol(s: string): string {
  if (s === 'ε') return '\\varepsilon';
  return texText(s);
}

/** A set in TeX: \{a, b\}, or ∅ when empty. */
export function texSet(items: readonly string[], each: (s: string) => string = texName): string {
  return items.length === 0 ? '\\emptyset' : `\\{${items.map(each).join(', ')}\\}`;
}

/** A set in plain text, the way the book writes it: {q1, q2, q3}. */
export function formatSet(items: readonly string[]): string {
  return items.length === 0 ? '∅' : `{${items.join(', ')}}`;
}

export interface TupleLine {
  /** KaTeX source. */
  tex: string;
  /** The same line as plain text (accessible name, tests). */
  text: string;
}

export interface Tuple {
  header: TupleLine;
  states: TupleLine;
  sigma: TupleLine;
  start: TupleLine;
  accept: TupleLine;
  /** Where the definition comes from. */
  definition: string;
}

/** The parts of the formal tuple, phrased as in Example 1.7 (DFA), 1.38 (NFA) or Def 1.64 (GNFA). */
export function tupleOf(name: string, m: MachineView): Tuple {
  const names = m.states.map((s) => s.name);
  const start = m.states.find((s) => s.start)?.name ?? '?';
  const accept = m.states.filter((s) => s.accept).map((s) => s.name);
  const sigma = { tex: `\\Sigma = ${texSet(m.sigma, texSymbol)}`, text: `Σ = ${formatSet(m.sigma)}` };
  const states = { tex: `Q = ${texSet(names)}`, text: `Q = ${formatSet(names)}` };
  if (m.kind === 'gnfa') {
    const accepting = accept[0] ?? '?';
    return {
      header: {
        tex: `${texName(name)} = (Q, \\Sigma, \\delta, ${texName(start)}, ${texName(accepting)})`,
        text: `${name} = (Q, Σ, δ, ${start}, ${accepting})`,
      },
      states,
      sigma,
      start: { tex: texName(start), text: start },
      accept: { tex: texName(accepting), text: accepting },
      definition: 'Definition 1.64',
    };
  }
  return {
    header: {
      tex: `${texName(name)} = (Q, \\Sigma, \\delta, ${texName(start)}, F)`,
      text: `${name} = (Q, Σ, δ, ${start}, F)`,
    },
    states,
    sigma,
    start: { tex: texName(start), text: start },
    accept: { tex: `F = ${texSet(accept)}`, text: `F = ${formatSet(accept)}` },
    definition: m.kind === 'nfa' ? 'Definition 1.37' : 'Definition 1.5',
  };
}

/**
 * A regular expression as the book typesets it: ∪ as \cup, star and plus as superscripts on the
 * atom or parenthesized group before them, ε and ∅ as math symbols (Definition 1.52, p. 65).
 * The input is the engine's printed form (minimal parentheses, `<name>` for long symbols).
 */
export function regexToTex(r: string): string {
  const out: string[] = [];
  // Where each open group starts in `out`, and where the last complete atom starts.
  const groups: number[] = [];
  let atom = -1;
  const chars = [...r];
  for (let i = 0; i < chars.length; i++) {
    const c = chars[i] ?? '';
    if (c === '(') {
      groups.push(out.length);
      out.push('(');
      atom = -1;
    } else if (c === ')') {
      out.push(')');
      atom = groups.pop() ?? 0;
    } else if (c === '*' || c === '+' || c === '⁺') {
      if (atom < 0) {
        out.push(texText(c));
        continue;
      }
      // Wrap the whole atom so stacked stars stay valid TeX: {{a}^{*}}^{*}.
      const base = out.splice(atom).join('');
      out.push(`{${base}}^{${c === '*' ? '*' : '+'}}`);
    } else if (c === '∪') {
      out.push(' \\cup ');
      atom = -1;
    } else if (c === '∘') {
      out.push(' \\circ ');
      atom = -1;
    } else if (c === ' ') {
      continue;
    } else {
      atom = out.length;
      if (c === 'ε') out.push('\\varepsilon');
      else if (c === '∅') out.push('\\emptyset');
      else if (c === '<') {
        const end = chars.indexOf('>', i);
        const name = end < 0 ? chars.slice(i).join('') : chars.slice(i, end + 1).join('');
        out.push(texText(name));
        i = end < 0 ? chars.length : end;
      } else out.push(texText(c));
    }
  }
  return out.join('');
}

/** `R = (ab ∪ a)*`, typeset; the accessible name keeps the plain form. */
export function regexLine(name: string, regex: string): TupleLine {
  return { tex: `${texName(name)} = ${regexToTex(regex)}`, text: `${name} = ${regex}` };
}
