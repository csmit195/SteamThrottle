export function reorderRules<T extends { priority: number }>(
  rules: T[],
  fromIndex: number,
  toIndex: number,
): T[] {
  if (
    fromIndex < 0 ||
    fromIndex >= rules.length ||
    toIndex < 0 ||
    toIndex >= rules.length ||
    fromIndex === toIndex
  ) {
    return rules;
  }

  const reordered = [...rules];
  const [moved] = reordered.splice(fromIndex, 1);
  reordered.splice(toIndex, 0, moved);
  return reordered.map((rule, priority) => ({ ...rule, priority }));
}
