import { describe, expect, it } from 'vitest';
import { tokenize } from './delta-lang';

/** The style of the first token with this text (after `skip` earlier ones). */
function styleOf(src: string, text: string, skip = 0): string | null | undefined {
  return tokenize(src).filter(([t]) => t === text)[skip]?.[1];
}

// Token classes map to VS Code Dark+ / Light+ colors: keyword, field (#9CDCFE), func (#DCDCAA),
// string/symbol (#CE9178), comment, number, and plain text for names and punctuation.
describe('Delta highlighting', () => {
  const dfa = `// M1
dfa M1 {
  Q  = { q1, q2 }
  Σ  = { 0, 1 }
  δ  = table {
          | 0  | 1
       q1 | q1 | q2
       q2 | q1 | q2
  }
  q0 = q1
  F  = { q2 }
}
check M1 accepts "1"
let D = dfa(N)
let U = union(D, M1)`;

  it('colors declarations as keywords and tuple fields as fields', () => {
    expect(styleOf(dfa, 'dfa')).toBe('keyword');
    for (const f of ['Q', 'Σ', 'δ', 'q0', 'F']) expect(styleOf(dfa, f), f).toBe('field');
    expect(styleOf(dfa, 'table')).toBe('keyword');
    expect(styleOf(dfa, 'accepts')).toBe('keyword');
  });

  it('colors commands and calls as functions', () => {
    expect(styleOf(dfa, 'check')).toBe('func');
    expect(styleOf(dfa, 'let')).toBe('func');
    expect(styleOf(dfa, 'dfa', 1)).toBe('func');
    expect(styleOf(dfa, 'union')).toBe('func');
  });

  it('colors strings and alphabet symbols like strings, but not state names', () => {
    expect(styleOf(dfa, '"1"')).toBe('string');
    // Σ = { 0, 1 } and the table header list symbols.
    expect(styleOf(dfa, '0')).toBe('symbol');
    expect(styleOf(dfa, '0', 1)).toBe('symbol');
    // States are plain text, even inside sets and table rows.
    expect(styleOf(dfa, 'q1')).toBe('variableName');
    expect(styleOf(dfa, 'q2', 2)).toBe('variableName');
    expect(styleOf(dfa, '// M1')).toBe('comment');
  });

  it('colors arrow labels and regex bodies as symbols', () => {
    const src = `nfa N {
  δ {
    q1 -[a,ε]-> q2
    q2 -b-> q3
  }
}
regex R over { a, b } = (ab ∪ a)*
regex S = a^3`;
    expect(styleOf(src, 'a')).toBe('symbol');
    expect(styleOf(src, 'ε')).toBe('symbol');
    expect(styleOf(src, 'b')).toBe('symbol');
    expect(styleOf(src, 'q3')).toBe('variableName');
    expect(styleOf(src, 'ab')).toBe('symbol');
    expect(styleOf(src, 'R')).toBe('variableName');
    expect(styleOf(src, '∪')).toBe('operator');
    expect(styleOf(src, 'S')).toBe('variableName');
    expect(styleOf(src, '3')).toBe('number');
  });

  it('colors numbers outside alphabets as numbers', () => {
    expect(styleOf('dfa M {\n  Q = { 1, 2 }\n  q0 = 1\n}', '1')).toBe('number');
  });
});
