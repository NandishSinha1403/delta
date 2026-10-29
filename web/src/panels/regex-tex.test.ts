import katex from 'katex';
import { describe, expect, it } from 'vitest';
import { regexToTex } from './tuple-format';

// Definition 1.52 / p. 65: union is ∪, star is a superscript. The engine prints plain text
// (`(ab∪a)*`); the Tuple view must typeset it like the book.
describe('regexToTex', () => {
  it('writes union as \\cup and star as a superscript', () => {
    expect(regexToTex('(ab∪a)*')).toBe('{(\\text{a}\\text{b} \\cup \\text{a})}^{*}');
    expect(regexToTex('a*b(a∪b)*')).toBe('{\\text{a}}^{*}\\text{b}{(\\text{a} \\cup \\text{b})}^{*}');
  });

  it('stars only the atom before it, and stacks stars validly', () => {
    expect(regexToTex('ab*')).toBe('\\text{a}{\\text{b}}^{*}');
    expect(regexToTex('(a*)*')).toBe('{({\\text{a}}^{*})}^{*}');
    expect(regexToTex('a+')).toBe('{\\text{a}}^{+}');
  });

  it('uses the book symbols for ε, ∅ and long symbols', () => {
    expect(regexToTex('ε∪∅')).toBe('\\varepsilon \\cup \\emptyset');
    expect(regexToTex('<RESET>*')).toBe('{\\text{<RESET>}}^{*}');
  });

  it('renders in KaTeX as ∪ (never U) with the star in a superscript box', () => {
    const html = katex.renderToString(regexToTex('b*aa*b((bb*a∪a)a*b)*'), { output: 'html', throwOnError: true });
    const text = html.replace(/<[^>]+>/g, '');
    expect(text).toContain('∪');
    expect(text).not.toMatch(/U/);
    // KaTeX puts superscripts in a .msupsub box; one per star (five here).
    expect(html.match(/class="msupsub"/g)?.length).toBe(5);
  });
});
