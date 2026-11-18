import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { RecordingRun } from '../src/daemon/mind.js';
import { PlanInspector } from '../src/inspector/PlanInspector.js';
import { daemonApi } from '../src/daemon/api.js';
import { MindPlans } from '../src/results/parts/MindPlans.js';
import { readPlanRanking, readPlans } from '../src/results/plans.js';

const stitch = JSON.parse(
  readFileSync(
    resolve(__dirname, '../../../contracts/fixtures/mind.clip_plan/valid/stitch.json'),
    'utf8',
  ),
);
const RECORDING = stitch.source_fingerprint as string;

function candidates(fingerprint = RECORDING) {
  return {
    schema_version: 'clipmill.mind.candidates.v1',
    source_fingerprint: fingerprint,
    length: { min_ticks: 20 * 90_000, max_ticks: 90 * 90_000 },
    inputs: {},
    candidates: [{ scout: 'link', score: 9, plan: stitch }],
  };
}

afterEach(cleanup);

describe('the mind’s clips', () => {
  it('reads a stitched plan as parts in the order they play', () => {
    const [plan] = readPlans(candidates(), RECORDING);
    expect(plan?.formLabel).toBe('Stitched');
    expect(plan?.parts.map((part) => part.roleLabel)).toEqual(['Question', 'Answer', 'Payoff']);
    expect(plan?.parts[1]?.cuts).toBe(1);
    expect(plan?.summary).toMatch(/^Joins the question at 12:41 to the answer at 12:45/);
    expect(JSON.parse(plan!.planJson)).toEqual(stitch);
  });

  it('shows nothing from another recording', () => {
    expect(readPlans(candidates(`sha256:${'9'.repeat(64)}`), RECORDING)).toEqual([]);
    expect(readPlans({ schema_version: 'something else' }, RECORDING)).toEqual([]);
  });

  it('opens a plan as an edit', () => {
    const plans = readPlans(candidates(), RECORDING);
    const onEdit = vi.fn();
    render(
      <MindPlans plans={plans} sourceDurationTicks={3600 * 90_000} busy={false} onEdit={onEdit} />,
    );
    expect(screen.getByText('Built by the mind')).toBeTruthy();
    expect(screen.getByText(/2\. Answer 12:45–13:12 · 5 s cut out/)).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: /^Edit:/ }));
    expect(onEdit).toHaveBeenCalledWith(plans[0]);
  });
});

describe('the final editorial ranking', () => {
  const quality = {
    verdict: 'accept',
    summary: 'The answer resolves its question.',
    issues: [],
    joins: [{ after: 'p2', verdict: 'pass', note: 'The same event is discussed.' }],
    repaired: true,
  };
  const ranking = {
    schema_version: 'clipmill.ranking.set.v2',
    source_fingerprint: RECORDING,
    requested_count: 2,
    editorial_prompt: 'Pricing mistakes',
    cutoff_reason: 'One distinct cut survived review.',
    clips: [
      {
        plan: stitch,
        keep_score: 82,
        chapter: 'First customers',
        reasons: ['A complete exchange.'],
        quality,
        strength: 0.83,
      },
    ],
  };
  it('retains critic findings and shows both the score and the honest cutoff', () => {
    const result = readPlanRanking(ranking, RECORDING);
    expect(result.cutoff).toBe(ranking.cutoff_reason);
    expect(result.prompt).toBe(ranking.editorial_prompt);
    expect(result.clips[0]?.assessment?.joins[0]?.note).toBe(quality.joins[0]?.note);
    expect(result.clips[0]?.assessment?.repaired).toBe(true);
    const inspect = vi.fn();
    render(
      <MindPlans
        plans={result.clips}
        sourceDurationTicks={null}
        busy={false}
        onInspect={inspect}
        onEdit={vi.fn()}
      />,
    );
    expect(screen.getByText('Keep score 82/100')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: /^Review / }));
    expect(inspect).toHaveBeenCalledWith(result.clips[0]);
    expect(readPlanRanking({ ...ranking, clips: [] }, RECORDING).clips).toEqual([]);
    expect(() => readPlanRanking(ranking, 'another source')).toThrow();
  });
});

it('restores the review reason and saves a revised decision against the selected plan', async () => {
  const run = { projectId: 'project', sourceId: 'source', jobId: 'run' };
  const [clip] = readPlans(candidates(), RECORDING);
  const planDecision = vi.fn(
    async (_run: RecordingRun, _clipId: string, decision?: string, reason?: string) =>
      decision ? { decision, reason: reason ?? '' } : { decision: 'kept', reason: 'Clear payoff.' },
  );
  render(
    <PlanInspector
      clip={clip!}
      run={run}
      api={{ ...daemonApi, planDecision }}
      busy={false}
      preview={null}
      transcript={null}
      notice={null}
      onBack={vi.fn()}
      onEdit={vi.fn()}
      onStarted={undefined}
    />,
  );
  await screen.findByDisplayValue('Clear payoff.');
  expect(screen.getByRole('button', { name: 'Keep' }).getAttribute('aria-pressed')).toBe('true');
  fireEvent.change(screen.getByLabelText('Decision reason (optional)'), {
    target: { value: 'The setup repeats.' },
  });
  fireEvent.click(screen.getByRole('button', { name: 'Reject' }));
  await waitFor(() =>
    expect(screen.getByRole('button', { name: 'Reject' }).getAttribute('aria-pressed')).toBe(
      'true',
    ),
  );
  expect(planDecision).toHaveBeenLastCalledWith(
    run,
    clip!.planId,
    'rejected',
    'The setup repeats.',
  );
});
