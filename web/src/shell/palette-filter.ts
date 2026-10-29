/** Keeps commands whose text contains every word of the query (case-insensitive). */
export function filterCommands<T extends { label: string; hint: string; group: string }>(
  items: readonly T[],
  query: string,
): T[] {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  return items.filter((c) => {
    const text = `${c.group} ${c.label} ${c.hint}`.toLowerCase();
    return words.every((w) => text.includes(w));
  });
}
