/**
 * Which end of a daemon list is the newest thing.
 *
 * Every list the daemon returns is sorted, and not all of them the same way.
 * Projects, sources and jobs come back **newest first** — their queries end
 * `ORDER BY created_unix_millis DESC`. Edit documents come back **oldest
 * first**, deliberately: the editor opens the newest, and a document's place in
 * that list is its history.
 *
 * The shell used to take `.at(-1)` for all of them, which is right for exactly
 * one. On an installation with a single project nothing looked wrong; with
 * several, the Results board, the editor and the export screen all opened the
 * *oldest* project — usually one that had never been analyzed, so the board was
 * empty and said so honestly about the wrong recording.
 *
 * So the assumption is written down once, here, with the sort order it depends
 * on named. A helper is worth it precisely because the answer is not the same
 * for every list: `newest` cannot be applied to edit documents, and the type
 * cannot stop you, so the reason is in the name and in this note.
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
 * The most recent item of an oldest-first list, which today is only the edit
 * documents of a project.
 *
 * Named rather than written inline so that a reader meeting `.at(-1)` in this
 * codebase has a reason to check which kind of list it is.
 */
export function oldestFirstNewest<T>(items: readonly T[]): T | null {
  return items.at(-1) ?? null;
}
