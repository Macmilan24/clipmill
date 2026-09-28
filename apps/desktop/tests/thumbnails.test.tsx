import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { ClipRow } from '../src/results/model.js';
import { CandidateGrid, reasonOf } from '../src/results/parts/CandidateGrid.js';
import { thumbnailPosition } from '../src/results/thumbnails.js';

const SECOND = 90_000;
const row = (overrides: Partial<ClipRow> = {}): ClipRow =>
  Object.assign(
    {
      candidateId: 'cand_1',
      rank: 1,
      displayScore: 92,
      band: 'strong',
      bandLabel: 'Strong',
      warnings: [],
      startTicks: 10 * SECOND,
      endTicks: 40 * SECOND,
      durationSeconds: 30,
      headline: 'Your first pricing model is probably backwards',
      axes: [],
      penalties: [],
      boundary: null,
      decision: null,
      docId: null,
      docJobId: null,
      latticeStarts: [],
      latticeEnds: [],
      recommended: false,
      proposer: null,
      clusterId: null,
      hook: null,
      payoff: null,
      flagged: false,
    },
    overrides,
  ) as ClipRow;

describe('a vertical thumbnail placed on the face', () => {
  it('centres the point the camera would follow, as far as the still allows', () => {
    expect(thumbnailPosition(0.5)).toBe('50.0% 50%');
    // 16:9 is 3.16 times as wide as 9:16: a face a quarter of the way in sits
    // 13% of the overflow from the left.
    expect(thumbnailPosition(0.25)).toBe('13.4% 50%');
    expect(thumbnailPosition(0.02)).toBe('0.0% 50%');
    expect(thumbnailPosition(0.99)).toBe('100.0% 50%');
    expect(thumbnailPosition(0.3, 9 / 16)).toBe('50% 50%');
  });
});

describe('a card on the board', () => {
  const rows = [
    row({ hook: { text: 'Charge more than feels comfortable.', atTicks: 10 * SECOND } }),
    row({
      candidateId: 'cand_2',
      rank: 2,
      startTicks: 15 * SECOND,
      endTicks: 38 * SECOND,
      review: {
        status: 'accepted',
        route: 'local',
        reasons: ['It ends on the takeaway.'],
        summary: 'A complete answer with a clear takeaway.',
      },
    }),
  ];
  const show = (tileUrl = vi.fn((ticks: number) => `tile-${ticks}`)) => {
    render(
      <CandidateGrid
        rows={rows}
        framing={new Map([['cand_1', 0.25]])}
        focusedId={null}
        checked={new Set()}
        tileUrl={tileUrl}
        onFocus={() => {}}
        onToggle={() => {}}
        onOpen={() => {}}
      />,
    );
    return tileUrl;
  };

  it('says why in one line and names the clip it repeats', () => {
    show();
    expect(reasonOf(rows[0]!)).toBe('“Charge more than feels comfortable.”');
    expect(screen.getByText('A complete answer with a clear takeaway.')).toBeTruthy();
    expect(screen.getByText(/Covers 01/)).toBeTruthy();
    expect(screen.queryAllByText(/Covers/)).toHaveLength(1);
  });

  it('frames the still on the face and scrubs through the clip under the pointer', () => {
    const tileUrl = show();
    const [first] = document.querySelectorAll<HTMLElement>('.results-card-picture');
    const picture = first!;
    expect(picture.querySelector('img')!.style.objectPosition).toBe('13.4% 50%');
    picture.getBoundingClientRect = () =>
      ({ left: 0, width: 100, top: 0, height: 178, right: 100, bottom: 178 }) as DOMRect;
    fireEvent.pointerMove(picture, { clientX: 50 });
    expect(tileUrl).toHaveBeenCalledWith(25 * SECOND);
    fireEvent.pointerLeave(picture);
    expect(tileUrl).toHaveBeenCalledWith(10 * SECOND);
  });
});
