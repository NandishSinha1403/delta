import { describe, expect, it } from 'vitest';
import { decodeSource, encodeSource, parseHash } from './share';

describe('share links', () => {
  it('round-trips a program through the URL hash', async () => {
    const src = 'dfa M {\n  start q0\n  accept q1\n  q0 -a-> q1\n}\nlang L = { w ∈ {a,b}* | w ends with "ab" }\n';
    const code = await encodeSource(src);
    expect(code).toMatch(/^[A-Za-z0-9_-]+$/);
    expect(await decodeSource(code)).toBe(src);
  });

  it('reads the embed options', () => {
    expect(parseHash('#code=abc&embed=1&panel=tests&readonly=1')).toEqual({
      code: 'abc',
      embed: true,
      panel: 'tests',
      readonly: true,
    });
    expect(parseHash('')).toEqual({ code: null, embed: false, panel: null, readonly: false });
  });
});
