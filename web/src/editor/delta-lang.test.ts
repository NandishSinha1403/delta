import { describe, expect, it } from 'vitest';
import { LATEX, tokenize } from './delta-lang';

/** The style of the first token with this text (after `skip` earlier ones). */
function styleOf(src: string, text: string, skip = 0): string | null | undefined {
  return tokenize(src).filter(([t]) => t === text)[skip]?.[1];
}

describe('Delta 2 highlighting', () => {
  const src = `// ends with ab
dfa EndsAB {
  start q0
  accept q2
  q0 -a-> q1;   q0 -b-> q0
  q1 -[a, b]-> q2
}
lang L = { w ∈ {a,b}* | w ends with "ab" }
check EndsAB ≡ re"(a ∪ b)* a b" up to 10
let D = determinize(N)
assert D.start == :q0`;

  it('colors keywords, models and fields', () => {
    expect(styleOf(src, 'dfa')).toBe('keyword');
    expect(styleOf(src, 'start')).toBe('field');
    expect(styleOf(src, 'accept')).toBe('field');
    expect(styleOf(src, 'lang')).toBe('keyword');
    expect(styleOf(src, 'check')).toBe('keyword');
    expect(styleOf(src, 'up')).toBe('keyword');
  });

  it('colors edges, words, regexes and atoms', () => {
    expect(styleOf(src, '-a->')).toBe('symbol');
    expect(styleOf(src, '-[a, b]->')).toBe('symbol');
    expect(styleOf(src, '"ab"')).toBe('string');
    expect(styleOf(src, 're"(a ∪ b)* a b"')).toBe('string');
    expect(styleOf(src, ':q0')).toBe('atom');
    expect(styleOf(src, '// ends with ab')).toBe('comment');
  });

  it('colors library calls as functions and states as names', () => {
    expect(styleOf(src, 'determinize')).toBe('func');
    expect(styleOf(src, 'q0')).toBe('variableName');
    expect(styleOf(src, '10')).toBe('number');
  });

  it('offers LaTeX names for the math symbols', () => {
    expect(LATEX.delta).toBe('δ');
    expect(LATEX.cup).toBe('∪');
    expect(LATEX.epsilon).toBe('ε');
  });
});
