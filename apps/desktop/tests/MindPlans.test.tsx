import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { MindPlans } from '../src/results/parts/MindPlans.js';
import { readPlans } from '../src/results/plans.js';

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
