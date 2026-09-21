/**
 * Daemon list ordering: projects, sources, and jobs are newest-first;
 * edit documents are oldest-first. Use the helper matching the list's order.
 */

/**
 * The most recent item of a newest-first list: projects, sources, or jobs.
 *
 * Not for edit documents — those are oldest-first and their newest is
 * [`oldestFirstNewest`].
 */
export function newest<T>(items: readonly T[]): T | null {
  return items[0] ?? null;
}

/**
 * Newest item from an oldest-first list, such as a project's edit documents.
 */
export function oldestFirstNewest<T>(items: readonly T[]): T | null {
  return items.at(-1) ?? null;
}
