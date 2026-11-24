/**
 * The mind's clips as Results rows: a ranked plan becomes a clip like any
 * other, carrying its parts in the order they play, stored under the id the
 * engine keeps its decisions and edit under.
 */
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { describe, expect, it } from 'vitest';

import { openingTicks, placeLabel, playedSpans, playedTicksAt } from '../src/results/model.js';
import { planRows, planSummary, readPlanRanking } from '../src/results/plans.js';

const stitch = JSON.parse(
  readFileSync(
    resolve(__dirname, '../../../contracts/fixtures/mind.clip_plan/valid/stitch.json'),
    'utf8',
  ),
);
const RECORDING = stitch.source_fingerprint as string;
const STORED = 'cand_0123456789abcdef';

function ranking(overrides: Record<string, unknown> = {}) {
  return {
    schema_version: 'clipmill.ranking.set.v2',
    source_fingerprint: RECORDING,
    requested_count: 2,
    editorial_prompt: '',
    cutoff_reason: 'One distinct cut survived review.',
    clips: [
      {
        plan: stitch,
        keep_score: 82,
        chapter: 'First customers',
        reasons: ['A complete exchange.'],
        quality: {
          verdict: 'accept',
          summary: 'The answer resolves its question.',
          issues: ['The payoff is quieter than the answer.'],
          joins: [
            { after: 'p1', verdict: 'pass', note: 'Continuous.' },
            { after: 'p2', verdict: 'warn', note: 'A jump in energy at the join.' },
          ],
          repaired: false,
        },
        strength: 0.83,
      },
    ],
    ...overrides,
  };
}

describe('a ranked plan as a Results row', () => {
  const read = readPlanRanking(ranking(), RECORDING);
  const [row] = planRows(read, new Map([[stitch.plan_id, STORED]]), [
    { candidateId: STORED, decision: 'approved', decidedUnixMillis: 1 },
  ]);

  it('is named, scored and decided like any clip', () => {
    expect(row?.candidateId).toBe(STORED);
    expect(row?.headline).toBe(stitch.title);
    expect(row?.displayScore).toBe(82);
    expect(row?.decision).toBe('approved');
    expect(row?.review?.status).toBe('accepted');
    expect(row?.review?.summary).toBe('The answer resolves its question.');
    expect(row?.durationSeconds).toBeCloseTo(stitch.duration_ticks / 90_000);
    expect(JSON.parse(row!.planJson!)).toEqual(stitch);
  });

  it('carries its parts in playback order, less what was taken out', () => {
    expect(row?.parts?.map((part) => part.label)).toEqual(['Question', 'Answer', 'Payoff']);
    expect(row?.parts?.[1]?.kept).toEqual([
      { startTicks: 68_850_000, endTicks: 70_290_000 },
      { startTicks: 70_740_000, endTicks: 71_280_000 },
    ]);
    expect(playedSpans(row!)).toHaveLength(4);
    expect(openingTicks(row!)).toBe(68_490_000);
    expect(placeLabel(row!)).toBe('12:41 · 3 parts');
    // Halfway through playback is inside the answer, before its removed tangent.
    expect(playedTicksAt(row!, 0.5)).toBe(70_110_000);
    // Three quarters is in the payoff: what lies between parts never plays.
    expect(playedTicksAt(row!, 0.75)).toBe(158_265_000);
  });

  it('spans the stretch of recording its parts lie in', () => {
    expect(row?.startTicks).toBe(68_490_000);
    expect(row?.endTicks).toBe(159_030_000);
  });

  it('lists the planner’s doubts and the review’s findings as checks', () => {
    expect(row?.flagged).toBe(true);
    expect(row?.warnings).toEqual([
      'A complete exchange.',
      'The payoff is said more calmly; the join may feel like a jump',
      'The payoff is quieter than the answer.',
      'A jump in energy at the join.',
    ]);
  });

  it('is named by its plan when the engine cannot say where it is stored', () => {
    const [fallback] = planRows(read, new Map(), []);
    expect(fallback?.candidateId).toBe(stitch.plan_id);
    expect(fallback?.decision).toBeNull();
  });
});

describe('what the board says above the mind’s clips', () => {
  it('gives the cutoff as the reason when fewer came back than were asked for', () => {
    const summary = planSummary(readPlanRanking(ranking(), RECORDING), 'interview');
    expect(summary.selected).toBe(1);
    expect(summary.requested).toBe(2);
    expect(summary.shortfall).toEqual(['One distinct cut survived review.']);
  });

  it('gives no shortfall when every clip asked for came back', () => {
    const summary = planSummary(
      readPlanRanking(ranking({ requested_count: 1 }), RECORDING),
      'interview',
    );
    expect(summary.shortfall).toEqual([]);
  });

  it('refuses a ranking of another recording', () => {
    expect(() => readPlanRanking(ranking(), `sha256:${'9'.repeat(64)}`)).toThrow();
    expect(readPlanRanking(ranking({ clips: [] }), RECORDING).clips).toEqual([]);
  });
});

describe('what the review found when it set an edit against the moment played whole', () => {
  const compared = (kept: string, reason: string) => {
    const base = ranking();
    const [clip] = base.clips as { quality: Record<string, unknown> }[];
    clip!.quality = {
      ...clip!.quality,
      compared: { plain_plan_id: `plan_${'e'.repeat(24)}`, kept, reason },
    };
    return readPlanRanking(base, RECORDING).clips[0]!.assessment.reasons;
  };

  it('says why an edit was kept', () => {
    expect(compared('edit', 'edit_preferred')).toEqual([
      'A complete exchange.',
      'Preferred to the same moment played whole, both ways round.',
    ]);
  });

  it('says why a moment plays whole', () => {
    expect(compared('plain', 'not_measured').at(-1)).toMatch(/could not be checked/);
    expect(compared('plain', 'awaiting_agent').at(-1)).toMatch(/connected agent/);
  });

  it('says nothing about a clip that was never an edit', () => {
    expect(readPlanRanking(ranking(), RECORDING).clips[0]!.assessment.reasons).toEqual([
      'A complete exchange.',
    ]);
  });
});
