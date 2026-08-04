/**
 * Which end of a daemon list holds the newest thing.
 *
 * This is a one-line helper guarding a bug that hid for four screens: the shell
 * took `.at(-1)` from every list the daemon returned, and three of the four are
 * sorted newest-first. On an installation with one project nothing looked
 * wrong. With several, Results, the editor and Export all opened the *oldest*
 * project — normally one that had never been analyzed — so the board was empty
 * and honestly said so about the wrong recording.
 *
 * The orderings asserted here are the daemon's, quoted from its own queries:
 *
 *   projects   ORDER BY created_unix_millis DESC, project_id DESC
 *   sources    ORDER BY s.created_unix_millis DESC, s.source_id DESC
 *   jobs       ORDER BY created_unix_millis DESC, job_id DESC
 *   edit_docs  ORDER BY created_unix_millis ASC, doc_id ASC
 */
import { describe, expect, it } from 'vitest';

import { newest, oldestFirstNewest } from '../src/daemon/ordering.js';

describe('newest', () => {
  it('takes the head of a newest-first list', () => {
    // As the daemon returns projects, sources and jobs.
    expect(newest(['08-03', '08-02', '07-31'])).toBe('08-03');
  });

  it('is null for an empty list rather than undefined', () => {
    // Callers store this in state typed `T | null`; undefined would render as
    // a missing prop rather than an empty screen.
    expect(newest([])).toBeNull();
  });

  it('does not mutate what it was given', () => {
    const items = ['a', 'b'];
    newest(items);
    expect(items).toEqual(['a', 'b']);
  });
});

describe('oldestFirstNewest', () => {
  it('takes the tail of an oldest-first list', () => {
    // As the daemon returns a project's edit documents.
    expect(oldestFirstNewest(['first', 'second', 'latest'])).toBe('latest');
  });

  it('is null for an empty list', () => {
    expect(oldestFirstNewest([])).toBeNull();
  });

  it('disagrees with newest on any list longer than one', () => {
    // The whole point: these are different answers, and picking the wrong one
    // is silent. A list of one cannot catch it, which is why the bug survived.
    const several = [1, 2, 3];
    expect(newest(several)).not.toBe(oldestFirstNewest(several));
    expect(newest([7])).toBe(oldestFirstNewest([7]));
  });
});
