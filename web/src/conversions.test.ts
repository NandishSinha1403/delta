import { describe, expect, it } from 'vitest';
import type { ObjectView } from './engine/api';
import { addLet, applications, conversion, freshName } from './conversions';

const obj = (name: string, kind: string) => ({ name, kind }) as ObjectView;

describe('conversions', () => {
  const objects = [obj('N', 'nfa'), obj('M1', 'dfa'), obj('M2', 'dfa'), obj('R', 'regex')];

  it('offers each conversion only on objects it accepts', () => {
    expect(applications(conversion('dfa'), objects)).toEqual([['N'], ['R']]);
    expect(applications(conversion('intersect'), objects)).toEqual([['M1', 'M2']]);
    expect(applications(conversion('union'), objects)).toEqual([
      ['N', 'M1'],
      ['N', 'M2'],
      ['M1', 'M2'],
    ]);
  });

  it('appends a let line with a fresh name', () => {
    expect(freshName('D', ['D', 'D2'])).toBe('D3');
    expect(addLet('nfa N {}', conversion('dfa'), ['N'], ['N'])).toEqual({
      source: 'nfa N {}\nlet D = dfa(N)\n',
      name: 'D',
    });
    expect(addLet('', conversion('union'), ['A', 'B'], []).source).toBe('let U = union(A, B)\n');
  });
});
