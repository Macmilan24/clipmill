/**
 * Canonical JSON: recursively sorted keys, two-space indent, trailing
 * newline. Fixtures are stored in this form so byte-comparison is a valid
 * round-trip test in all three contract languages.
 */
export function canonicalJson(value: unknown): string {
  return `${JSON.stringify(sortKeys(value), null, 2)}\n`;
}

function sortKeys(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map(sortKeys);
  }
  if (value !== null && typeof value === 'object') {
    const entries = Object.entries(value as Record<string, unknown>).toSorted(([a], [b]) =>
      a < b ? -1 : a > b ? 1 : 0,
    );
    return Object.fromEntries(entries.map(([key, child]) => [key, sortKeys(child)]));
  }
  return value;
}
