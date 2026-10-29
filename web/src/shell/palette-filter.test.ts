import { describe, expect, it } from 'vitest';
import { filterCommands } from './palette-filter';

describe('command palette filter', () => {
  const items = [
    { group: 'Conversions', label: 'dfa(N)', hint: 'Theorem 1.39 · Subset construction' },
    { group: 'Examples', label: 'Theorem 1.49', hint: 'closed under star' },
    { group: 'Go to object', label: 'N', hint: 'NFA' },
  ];
  it('matches every word anywhere in group, label or hint', () => {
    expect(filterCommands(items, '').length).toBe(3);
    expect(filterCommands(items, 'subset').map((i) => i.label)).toEqual(['dfa(N)']);
    expect(filterCommands(items, 'theorem star').map((i) => i.label)).toEqual(['Theorem 1.49']);
    expect(filterCommands(items, 'zzz')).toEqual([]);
  });
});
