/**
 * What the Results board is entitled to say.
 *
 * These run against the joins rather than the pixels, because the claims worth
 * protecting are claims about meaning: an axis nothing measured must not render
 * as a zero, a shortfall must not be padded away, and the cohort's order is the
 * ranking's decision rather than the interface's.
 */
import { describe, expect, it } from 'vitest';

import type { DiscoveryCandidates, RankingSet } from '@clipmill/contracts';

import type { EditDocSummary } from '../src/daemon/client.js';
import {
  AXES,
  applyFilters,
  clipRows,
  clock,
  duration,
  newestDocumentPerCandidate,
  sortRows,
  summarize,
  tally,
  topFactors,
} from '../src/results/model.js';

const FINGERPRINT = `sha256:${'11'.repeat(32)}`;

function ranking(): RankingSet {
  return {
    schema_version: 'clipmill.ranking.set.v1',
    source_fingerprint: FINGERPRINT,
    inputs: {
      candidates_artifact_id: FINGERPRINT,
      index_artifact_id: FINGERPRINT,
      transcript_artifact_id: FINGERPRINT,
    },
    producer: { stage: 'rank-candidates', implementation: 'test@1' },
    rubric: { scorer: 'test', boundary: 'test', selector: 'test' },
    requested: { count: 4, diversity: 0.3 },
    cohort: [
      {
        candidate_id: 'cand_0000000000000002',
        rank: 2,
        display_score: 71,
        score: 0.71,
        factors: [
          { name: 'hook', available: true, value: 0.8, weight: 0.3, evidence: [] },
          {
            name: 'prompt_relevance',
            available: false,
            unavailable_reason: 'no prompt was given',
          },
        ],
        penalties: [],
        uncertainty: { value: 0.4, band: 'promising', warnings: [] },
        boundary: {
          chosen: { start_ticks: 900_000, end_ticks: 2_700_000 },
          score: 0.5,
          terms: [{ name: 'hook_weight', value: 0.2, weight: 1 }],
        },
        cluster_id: 'clus_0000000000000001',
      },
      {
        candidate_id: 'cand_0000000000000001',
        rank: 1,
        display_score: 92,
        score: 0.92,
        factors: AXES.map((name) => ({
          name,
          available: true,
          value: 0.9,
          weight: 0.2,
          evidence: [],
        })),
        penalties: [{ reason: 'repetition', value: 0.05 }],
        uncertainty: { value: 0.1, band: 'strong', warnings: ['speaker attribution uncertain'] },
        boundary: {
          chosen: { start_ticks: 0, end_ticks: 1_800_000 },
          score: 0.9,
          terms: [{ name: 'hook_weight', value: 0.4, weight: 1 }],
          alternative: { interval: { start_ticks: 90_000, end_ticks: 1_800_000 }, score: 0.8 },
        },
        cluster_id: 'clus_0000000000000002',
      },
    ],
    selected: ['cand_0000000000000001'],
    shortfall: [
      { reason: 'too_few_candidates', count: 3, detail: 'only one moment cleared the bar' },
    ],
    filtered: [],
  } as unknown as RankingSet;
}

function candidates(): DiscoveryCandidates {
  return {
    candidates: [
      {
        id: 'cand_0000000000000001',
        boundary_lattice: { starts: [0, 90_000], ends: [1_800_000], phi_rejects: [] },
      },
    ],
  } as unknown as DiscoveryCandidates;
}

describe('the ranked rows', () => {
  it('are ordered by the ranking rather than re-sorted here', () => {
    const rows = clipRows(ranking(), candidates(), null, []);
    expect(rows.map((row) => row.rank)).toEqual([1, 2]);
    expect(rows[0]?.displayScore).toBe(92);
  });

  it('render an unmeasured axis as its reason and never as a zero', () => {
    const rows = clipRows(ranking(), candidates(), null, []);
    const second = rows.find((row) => row.rank === 2);
    const prompt = second?.axes.find((axis) => axis.axis === 'prompt_relevance');
    expect(prompt?.value).toBeNull();
    expect(prompt?.unavailableReason).toBe('no prompt was given');
  });

  it('keep every axis even when the card did not carry it', () => {
    // A card that lost a factor should show a gap where it used to be, not
    // quietly become a shorter list.
    const rows = clipRows(ranking(), candidates(), null, []);
    expect(rows[1]?.axes).toHaveLength(AXES.length);
  });

  it('carry the boundary alternative when the lattice offered one', () => {
    const rows = clipRows(ranking(), candidates(), null, []);
    expect(rows[0]?.boundary?.alternative).toEqual({ startTicks: 90_000, endTicks: 1_800_000 });
    expect(rows[1]?.boundary?.alternative).toBeNull();
  });

  it('carry the lattice a boundary strip draws, and nothing when there is none', () => {
    const rows = clipRows(ranking(), candidates(), null, []);
    expect(rows[0]?.latticeStarts).toEqual([0, 90_000]);
    expect(rows[1]?.latticeStarts).toEqual([]);
  });

  it('attach a decision the daemon recorded, and ignore one it did not', () => {
    const rows = clipRows(ranking(), candidates(), null, [
      { candidateId: 'cand_0000000000000001', decision: 'approved', decidedUnixMillis: 0 },
      { candidateId: 'cand_0000000000000002', decision: 'unspecified', decidedUnixMillis: 0 },
    ]);
    expect(rows[0]?.decision).toBe('approved');
    expect(rows[1]?.decision).toBeNull();
  });

  it('translate uncertainty into a word rather than shading the score', () => {
    const rows = clipRows(ranking(), candidates(), null, []);
    expect(rows[0]?.bandLabel).toBe('Strong');
    expect(rows[1]?.bandLabel).toBe('Promising');
  });

  it('attach the edit document a clip has, apart from whether it was approved', () => {
    // Approved, and yet no document: the creation failed after the decision.
    // A board that inferred one from the other would send the editor to open
    // nothing.
    const rows = clipRows(
      ranking(),
      candidates(),
      null,
      [{ candidateId: 'cand_0000000000000001', decision: 'approved', decidedUnixMillis: 0 }],
      [edit('edt_a', 'cand_0000000000000002', 10)],
    );
    expect(rows[0]?.decision).toBe('approved');
    expect(rows[0]?.docId).toBeNull();
    expect(rows[1]?.decision).toBeNull();
    expect(rows[1]?.docId).toBe('edt_a');
  });
});

function edit(docId: string, candidateId: string, createdUnixMillis: number): EditDocSummary {
  return {
    docId,
    projectId: 'p1',
    sourceId: 'src_p1',
    candidateId,
    jobId: '',
    revision: 0,
    createdUnixMillis,
    updatedUnixMillis: createdUnixMillis,
  };
}

describe('the document a clip has', () => {
  it('is the newest of several, which is the one the daemon reopens', () => {
    const newest = newestDocumentPerCandidate([
      edit('edt_first', 'cand_1', 10),
      edit('edt_variation', 'cand_1', 20),
      edit('edt_other', 'cand_2', 15),
    ]);
    expect(newest.get('cand_1')?.docId).toBe('edt_variation');
    expect(newest.get('cand_2')?.docId).toBe('edt_other');
  });

  it('breaks a tie the way the daemon does, by the later id', () => {
    const newest = newestDocumentPerCandidate([
      edit('edt_b', 'cand_1', 10),
      edit('edt_a', 'cand_1', 10),
    ]);
    expect(newest.get('cand_1')?.docId).toBe('edt_b');
  });

  it('leaves out a document that names no candidate', () => {
    const newest = newestDocumentPerCandidate([edit('edt_hand', '', 10)]);
    expect(newest.size).toBe(0);
  });
});

describe('the summary', () => {
  it('states the shortfall rather than padding it away', () => {
    const summary = summarize(ranking());
    expect(summary.selected).toBe(1);
    expect(summary.requested).toBe(4);
    expect(summary.shortfall).toEqual(['only one moment cleared the bar']);
  });
});

describe('the filters', () => {
  it('narrow by confidence, by decision, and by score', () => {
    const rows = clipRows(ranking(), candidates(), null, [
      { candidateId: 'cand_0000000000000001', decision: 'rejected', decidedUnixMillis: 0 },
    ]);
    expect(applyFilters(rows, { band: 'strong', decision: 'any', minimumScore: 0 })).toHaveLength(
      1,
    );
    expect(
      applyFilters(rows, { band: 'any', decision: 'undecided', minimumScore: 0 }),
    ).toHaveLength(1);
    expect(applyFilters(rows, { band: 'any', decision: 'any', minimumScore: 80 })).toHaveLength(1);
    expect(applyFilters(rows, { band: 'any', decision: 'approved', minimumScore: 0 })).toHaveLength(
      0,
    );
  });
});

describe('the clock', () => {
  it('reads a tick position the way a person reads a timeline', () => {
    expect(clock(0)).toBe('0:00');
    expect(clock(90_000 * 65)).toBe('1:05');
  });
});

describe('searching the board', () => {
  it('matches the line a person can actually read on the row', () => {
    const rows = clipRows(ranking(), candidates(), null, []);
    const withQuery = (query: string) =>
      applyFilters(rows, { band: 'any', decision: 'any', minimumScore: 0, query });

    // Whatever the fixture's opening lines are, a query drawn from one of them
    // finds that row and a query drawn from nothing finds none.
    const headline = rows[0]?.headline ?? '';
    if (headline !== '') {
      expect(withQuery(headline.slice(0, 6)).length).toBeGreaterThan(0);
    }
    expect(withQuery('zzzzz-no-such-clip')).toHaveLength(0);
  });

  it('treats an absent or blank query as no filter at all', () => {
    const rows = clipRows(ranking(), candidates(), null, []);
    expect(applyFilters(rows, { band: 'any', decision: 'any', minimumScore: 0 })).toHaveLength(
      rows.length,
    );
    expect(
      applyFilters(rows, { band: 'any', decision: 'any', minimumScore: 0, query: '   ' }),
    ).toHaveLength(rows.length);
  });
});

describe('ordering the board', () => {
  it('never adds or drops a row, whatever the order', () => {
    const rows = clipRows(ranking(), candidates(), null, []);
    for (const key of ['rank', 'score', 'longest', 'shortest', 'earliest'] as const) {
      expect(sortRows(rows, key)).toHaveLength(rows.length);
    }
  });

  it("leaves the snapshot's own array untouched", () => {
    const rows = clipRows(ranking(), candidates(), null, []);
    const before = rows.map((row) => row.candidateId);
    sortRows(rows, 'score');
    expect(rows.map((row) => row.candidateId)).toEqual(before);
  });

  it('puts the highest score first when asked for score', () => {
    const rows = clipRows(ranking(), candidates(), null, []);
    const scores = sortRows(rows, 'score').map((row) => row.displayScore);
    expect(scores).toEqual([...scores].sort((left, right) => right - left));
  });
});

describe('the tallies behind the filter chips', () => {
  it('count what pressing the chip would leave', () => {
    const rows = clipRows(ranking(), candidates(), null, [
      { candidateId: 'cand_0000000000000001', decision: 'approved', decidedUnixMillis: 0 },
    ]);
    const counts = tally(rows);
    expect(counts.all).toBe(rows.length);
    expect(counts.approved).toBe(1);
    expect(counts.undecided).toBe(rows.length - 1);
    // A chip's number has to equal what the filter actually returns, or the
    // chip is advertising a result nobody gets.
    expect(applyFilters(rows, { band: 'any', decision: 'approved', minimumScore: 0 })).toHaveLength(
      counts.approved,
    );
    expect(applyFilters(rows, { band: 'strong', decision: 'any', minimumScore: 0 })).toHaveLength(
      counts.strong,
    );
  });
});

describe('why a clip ranked where it did', () => {
  it('orders by weighted contribution rather than by raw value', () => {
    const rows = clipRows(ranking(), candidates(), null, []);
    const row = rows.find((candidate) => candidate.rank === 2);
    expect(row).toBeDefined();
    const factors = topFactors(row!);
    const contribution = factors.map((axis) => (axis.value ?? 0) * (axis.weight ?? 0));
    expect(contribution).toEqual([...contribution].sort((left, right) => right - left));
  });

  it('never offers an axis nobody measured as an explanation', () => {
    const rows = clipRows(ranking(), candidates(), null, []);
    for (const row of rows) {
      expect(topFactors(row).every((axis) => axis.value !== null)).toBe(true);
    }
  });
});

describe('the duration column', () => {
  it('reads as a length rather than a position', () => {
    expect(duration(8)).toBe('0:08');
    expect(duration(75)).toBe('1:15');
  });
});
