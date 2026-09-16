/**
 * The Inspector's honesty, held to.
 *
 * Every claim here is one the screen could quietly get wrong in a way that looks
 * fine: an axis nobody measured drawn as a zero, an alternative cut offered when
 * the lattice held only one legal pair, a clip with nothing recorded against it
 * shown as though it had been checked and cleared.
 */
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { TooltipProvider } from '../src/components/ui/tooltip.js';
import { ClipInspector } from '../src/screens/ClipInspector.js';
import type { ClipRow } from '../src/results/model.js';

const SECOND = 90_000;

function row(overrides: Partial<ClipRow> = {}): ClipRow {
  return {
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
    axes: [
      {
        axis: 'hook',
        label: 'Hook',
        value: 0.8,
        weight: 1.4,
        unavailableReason: null,
        evidence: [{ text: 'Most founders price from fear.', atTicks: 12 * SECOND }],
      },
      {
        axis: 'prompt_relevance',
        label: 'Prompt fit',
        value: null,
        weight: null,
        unavailableReason: 'no prompt was given; prompt retrieval is not one of this phase',
        evidence: [],
      },
    ],
    penalties: [],
    boundary: null,
    decision: null,
    latticeStarts: [9 * SECOND, 10 * SECOND],
    latticeEnds: [40 * SECOND, 42 * SECOND],
    recommended: true,
    proposer: 'quote',
    clusterId: 'clus_1',
    hook: { text: 'Here is the thing nobody tells you.', atTicks: 10 * SECOND },
    payoff: null,
    flagged: false,
    ...overrides,
  };
}

/**
 * Move to a tab the way a person does.
 *
 * Radix switches on pointer-down rather than on a synthesised `click`, so a
 * bare click leaves the panel where it was and the assertion that follows fails
 * for a reason that has nothing to do with the screen.
 */
function openTab(name: RegExp) {
  const tab = screen.getByRole('tab', { name });
  fireEvent.mouseDown(tab);
  return tab;
}

function show(overrides: Partial<Parameters<typeof ClipInspector>[0]> = {}) {
  const props = {
    rows: [row()],
    candidateId: 'cand_1',
    proxyUrl: null,
    crop: null,
    cues: [],
    peaks: null,
    busy: false,
    notice: null,
    onSelect: () => {},
    onBack: () => {},
    onDecide: () => {},
    onUseAlternative: () => {},
    onTakeCut: () => {},
    ...overrides,
  };
  // Wrapped as `App` wraps it: the transport's tooltips need the provider the
  // shell mounts once at the root, and a test that rendered without it would be
  // testing a tree the product never builds.
  render(
    <TooltipProvider>
      <ClipInspector {...props} />
    </TooltipProvider>,
  );
}

describe('the score panel', () => {
  it('says why an axis was not measured instead of drawing it as a zero', () => {
    show();
    expect(screen.getByText(/no prompt was given/i)).toBeTruthy();
    // The value reads as absent, never as a number that could be mistaken
    // for a scored zero.
    expect(screen.getByText('—')).toBeTruthy();
  });

  it('counts how many axes were measured, so a thin card cannot look full', () => {
    show();
    expect(screen.getByText(/1 of 2 measured/i)).toBeTruthy();
  });

  it('leads with the axes that moved the total, as the design draws them', () => {
    show();
    // Hook is the only measured axis here, so it is the hero bar and Prompt
    // fit falls to the detailed grid.
    expect(screen.getAllByText('Hook').length).toBeGreaterThan(0);
    expect(screen.getByText('Prompt fit')).toBeTruthy();
  });

  it('turns a quote into a place the player can go', () => {
    show({ proxyUrl: 'clipmill-media://proxy/proxy.mp4' });
    const video = document.querySelector('video')!;
    // jsdom never loads media, so it reports no metadata forever; a browser
    // that has fired loadedmetadata reports at least HAVE_METADATA, which is
    // what the seek waits for.
    Object.defineProperty(video, 'readyState', { value: 1, configurable: true });
    fireEvent.loadedMetadata(video);
    // The hook was said at 10s; the ranker's evidence at 12s. Jumping to the
    // evidence must move the player there.
    fireEvent.click(screen.getAllByRole('button', { name: '0:12' })[0]!);
    expect(video.currentTime).toBeCloseTo(12, 3);
  });
});

describe('the boundary panel', () => {
  it('offers the runner-up only when the lattice held one', () => {
    show({
      rows: [
        row({
          boundary: {
            startTicks: 10 * SECOND,
            endTicks: 40 * SECOND,
            score: 0.5,
            terms: [{ name: 'pronoun_open', value: -0.2 }],
            alternative: null,
          },
        }),
      ],
    });
    openTab(/boundary/i);
    expect(screen.getByText(/offered one legal pair/i)).toBeTruthy();
    expect(screen.queryByRole('button', { name: /use the alternative/i })).toBeNull();
  });

  it('says so when the candidate carries no boundary record at all', () => {
    show();
    openTab(/boundary/i);
    expect(screen.getByText(/carries no boundary record/i)).toBeTruthy();
  });

  it('rebuilds from the alternative when there is one to take', () => {
    let asked = false;
    show({
      rows: [
        row({
          boundary: {
            startTicks: 10 * SECOND,
            endTicks: 40 * SECOND,
            score: 0.5,
            terms: [{ name: 'pronoun_open', value: -0.2 }],
            alternative: { startTicks: 9 * SECOND, endTicks: 42 * SECOND },
          },
        }),
      ],
      onUseAlternative: () => {
        asked = true;
      },
    });
    openTab(/boundary/i);
    fireEvent.click(screen.getByRole('button', { name: /use the alternative/i }));
    expect(asked).toBe(true);
  });
});

describe('the risk panel', () => {
  it('says nothing was recorded rather than implying a clean bill', () => {
    show();
    openTab(/risk/i);
    expect(screen.getByText(/recorded nothing against this clip/i)).toBeTruthy();
  });

  it('shows each penalty with the score it cost', () => {
    show({ rows: [row({ penalties: [{ reason: 'repetition', value: 7 }] })] });
    openTab(/risk/i);
    expect(screen.getByText('repetition')).toBeTruthy();
    expect(screen.getByText('−7')).toBeTruthy();
  });
});

describe('deciding', () => {
  it('records each of the three decisions', () => {
    const seen: string[] = [];
    show({ onDecide: (decision) => seen.push(decision) });
    fireEvent.click(screen.getByRole('button', { name: /approve for the editor/i }));
    fireEvent.click(screen.getByRole('button', { name: /keep for later/i }));
    fireEvent.click(screen.getByRole('button', { name: /reject/i }));
    expect(seen).toEqual(['approved', 'kept', 'rejected']);
  });

  it('shuts every decision while one is in flight', () => {
    show({ busy: true });
    expect(screen.getByRole('button', { name: /working/i })).toHaveProperty('disabled', true);
    expect(screen.getByRole('button', { name: /keep for later/i })).toHaveProperty(
      'disabled',
      true,
    );
  });

  it('shows what the last action said', () => {
    show({ notice: 'Sent to the editor.' });
    expect(screen.getByRole('status')).toBeTruthy();
    expect(screen.getByText('Sent to the editor.')).toBeTruthy();
  });
});

describe('a candidate that is not in the ranking', () => {
  it('says so rather than rendering an empty inspector', () => {
    show({ candidateId: 'cand_missing' });
    expect(screen.getByText(/not in the current ranking/i)).toBeTruthy();
  });
});

describe('the timeline', () => {
  it('moves a handle to the next legal edge rather than anywhere', () => {
    // The fixture's legal starts are 9s and 10s and the cut begins at 10s, so
    // stepping back has exactly one place it may land.
    show();
    const handle = screen.getByRole('button', { name: /^in point at/i });
    fireEvent.keyDown(handle, { key: 'ArrowLeft' });
    expect(screen.getByRole('button', { name: /^in point at 00:00:09;00/i })).toBeTruthy();
  });

  it('refuses to step past the last legal edge', () => {
    show();
    const handle = screen.getByRole('button', { name: /^in point at/i });
    fireEvent.keyDown(handle, { key: 'ArrowLeft' });
    fireEvent.keyDown(handle, { key: 'ArrowLeft' });
    // 9s is the earliest start the lattice holds; there is nowhere further back.
    expect(screen.getByRole('button', { name: /^in point at 00:00:09;00/i })).toBeTruthy();
  });

  it('offers to take the cut only once a boundary has actually moved', () => {
    show();
    expect(screen.queryByRole('button', { name: /take this cut/i })).toBeNull();
    fireEvent.keyDown(screen.getByRole('button', { name: /^in point at/i }), { key: 'ArrowLeft' });
    expect(screen.getByRole('button', { name: /take this cut/i })).toBeTruthy();
  });

  it('sends the moved window, not the one the ranker chose', () => {
    const taken: number[][] = [];
    show({ onTakeCut: (start, end) => taken.push([start, end]) });
    fireEvent.keyDown(screen.getByRole('button', { name: /^in point at/i }), { key: 'ArrowLeft' });
    fireEvent.click(screen.getByRole('button', { name: /take this cut/i }));
    expect(taken).toEqual([[9 * SECOND, 40 * SECOND]]);
  });

  it('says the daemon may move the cut, rather than implying it is final', () => {
    show();
    fireEvent.keyDown(screen.getByRole('button', { name: /^in point at/i }), { key: 'ArrowLeft' });
    expect(screen.getByText(/snaps a cut to the lattice/i)).toBeTruthy();
  });

  it('puts a moved boundary back where the ranker had it', () => {
    show();
    const handle = screen.getByRole('button', { name: /^in point at/i });
    fireEvent.keyDown(handle, { key: 'ArrowLeft' });
    fireEvent.click(screen.getByRole('button', { name: /reset/i }));
    expect(screen.queryByRole('button', { name: /take this cut/i })).toBeNull();
    expect(screen.getByRole('button', { name: /^in point at 00:00:10;00/i })).toBeTruthy();
  });

  it('drops the draft when a different clip is opened', () => {
    const { rerender } = render(<div />);
    void rerender;
    // Two candidates, so the second can be opened after the first is dragged.
    const rows = [row(), row({ candidateId: 'cand_2', headline: 'Another clip' })];
    const props = {
      rows,
      candidateId: 'cand_1',
      proxyUrl: null,
      crop: null,
      cues: [],
      peaks: null,
      busy: false,
      notice: null,
      onSelect: () => {},
      onBack: () => {},
      onDecide: () => {},
      onUseAlternative: () => {},
      onTakeCut: () => {},
    };
    const view = render(
      <TooltipProvider>
        <ClipInspector {...props} />
      </TooltipProvider>,
    );
    fireEvent.keyDown(screen.getAllByRole('button', { name: /^in point at/i })[0]!, {
      key: 'ArrowLeft',
    });
    view.rerender(
      <TooltipProvider>
        <ClipInspector {...props} candidateId="cand_2" />
      </TooltipProvider>,
    );
    expect(screen.queryByRole('button', { name: /take this cut/i })).toBeNull();
  });
});

describe('the player', () => {
  it('says there is nothing to preview rather than showing a dead frame', () => {
    show();
    expect(screen.getByText(/published no proxy/i)).toBeTruthy();
  });

  it('carries transport an editor can drive, once there is something to drive', () => {
    show({ proxyUrl: 'clipmill-media://proxy/proxy.mp4' });
    for (const name of [
      /play/i,
      /back one frame/i,
      /forward one frame/i,
      /jump to the in point/i,
    ]) {
      expect(screen.getByRole('button', { name })).toBeTruthy();
    }
  });

  it('offers no transport at all when there is no proxy behind it', () => {
    // Dead controls over an absent video are the failure this screen is being
    // rebuilt to remove, so their absence is the assertion.
    show();
    expect(screen.queryByRole('button', { name: /^play$/i })).toBeNull();
  });
});

describe('opening the clip', () => {
  it('seeks the proxy once it has metadata, not before', () => {
    // A `currentTime` written before the element knows its duration is dropped,
    // which is what made the player open at the top of the whole recording.
    show({ proxyUrl: 'clipmill-media://proxy/proxy.mp4' });
    const video = document.querySelector('video');
    expect(video).toBeTruthy();
    fireEvent.loadedMetadata(video!);
    expect(video!.currentTime).toBeCloseTo(10, 3);
  });

  it('opens at the clip it was given, not at the recording', () => {
    show({
      proxyUrl: 'clipmill-media://proxy/proxy.mp4',
      rows: [row({ startTicks: 90 * SECOND, endTicks: 120 * SECOND })],
    });
    const video = document.querySelector('video');
    fireEvent.loadedMetadata(video!);
    expect(video!.currentTime).toBeCloseTo(90, 3);
  });
});
