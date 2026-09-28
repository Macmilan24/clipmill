/**
 * The review station, held to what a reviewer relies on: the clip can be read,
 * its cut can move to any sentence and is what an approval sends, a decision
 * is one click or one key and can be taken back, and nothing the analysis did
 * not measure is drawn as though it had.
 */
import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { TooltipProvider } from '../src/components/ui/tooltip.js';
import type { ClipRow } from '../src/results/model.js';
import type { Transcript } from '../src/results/transcript.js';
import { ClipInspector, type ClipInspectorProps } from '../src/screens/ClipInspector.js';

const SECOND = 90_000;

/**
 * Four sentences, a second of speech each with a pause between: the clip is
 * the middle two (12–16 s), with one sentence of context either side.
 */
function transcript(): Transcript {
  const sentences = [
    ['Most', 'people', 'wait.'],
    ['Confidence', 'is', 'a', 'receipt.'],
    ['Not', 'a', 'ticket.'],
    ['So', 'start', 'small.'],
  ];
  const words: { text: string; startTicks: number; endTicks: number }[] = [];
  const grouped: Transcript['sentences'][number][] = [];
  let at = 10 * SECOND;
  for (const sentence of sentences) {
    const firstWord = words.length;
    for (const text of sentence) {
      words.push({ text, startTicks: at, endTicks: at + 0.3 * SECOND });
      at += 0.35 * SECOND;
    }
    grouped.push({
      startTicks: words[firstWord]!.startTicks,
      endTicks: words.at(-1)!.endTicks,
      firstWord,
      wordCount: sentence.length,
    });
    at += 0.8 * SECOND;
  }
  return { words, sentences: grouped };
}

const WORDS = transcript();
const CLIP_START = WORDS.sentences[1]!.startTicks - 0.05 * SECOND;
const CLIP_END = WORDS.sentences[2]!.endTicks + 0.05 * SECOND;

function row(overrides: Partial<ClipRow> = {}): ClipRow {
  return {
    candidateId: 'cand_1',
    rank: 1,
    displayScore: 92,
    band: 'strong',
    bandLabel: 'Strong',
    warnings: [],
    startTicks: CLIP_START,
    endTicks: CLIP_END,
    durationSeconds: (CLIP_END - CLIP_START) / SECOND,
    headline: 'Confidence is a receipt',
    axes: [
      {
        axis: 'hook',
        label: 'Hook',
        value: 0.8,
        weight: 1.4,
        unavailableReason: null,
        evidence: [{ text: 'Confidence is a receipt.', atTicks: CLIP_START }],
      },
      {
        axis: 'prompt_relevance',
        label: 'Prompt fit',
        value: null,
        weight: null,
        unavailableReason: 'no prompt was given',
        evidence: [],
      },
    ],
    penalties: [],
    boundary: {
      startTicks: CLIP_START,
      endTicks: CLIP_END,
      score: 0.9,
      terms: [{ name: 'hook_weight', value: 0.4 }],
      alternative: {
        startTicks: WORDS.sentences[0]!.startTicks - 0.05 * SECOND,
        endTicks: CLIP_END,
      },
    },
    decision: null,
    docId: null,
    docJobId: null,
    latticeStarts: [CLIP_START],
    latticeEnds: [CLIP_END],
    recommended: true,
    proposer: 'quote',
    clusterId: 'clus_1',
    hook: { text: 'Here is the thing nobody tells you.', atTicks: CLIP_START },
    payoff: null,
    flagged: false,
    ...overrides,
  };
}

function show(overrides: Partial<ClipInspectorProps> = {}) {
  const props: ClipInspectorProps = {
    rows: [row(), row({ candidateId: 'cand_2', rank: 2, headline: 'Start small' })],
    candidateId: 'cand_1',
    proxyUrl: null,
    crop: null,
    peaks: null,
    tileUrl: () => null,
    transcript: { status: 'ready', transcript: WORDS },
    sourceDurationTicks: 120 * SECOND,
    durationTarget: { minTicks: 20 * SECOND, maxTicks: 90 * SECOND },
    busy: false,
    notice: null,
    autoAdvance: true,
    onAutoAdvance: vi.fn(),
    onSelect: vi.fn(),
    onBack: vi.fn(),
    onApprove: vi.fn(),
    onDecide: vi.fn(),
    onUndo: null,
    onOpenEdit: null,
    ...overrides,
  };
  render(
    <TooltipProvider>
      <ClipInspector {...props} />
    </TooltipProvider>,
  );
  return props;
}

/** Radix tabs switch on pointer-down rather than on a synthesised click. */
function openTab(name: RegExp) {
  fireEvent.mouseDown(screen.getByRole('tab', { name }));
}

const word = (text: string) =>
  [...document.querySelectorAll<HTMLElement>('.review-word')].find(
    (element) => element.textContent === text,
  )!;

describe('reading the clip', () => {
  it('shows its words with the sentences around it set back', () => {
    show();
    expect(word('Confidence').dataset.inside).toBe('true');
    expect(word('ticket.').dataset.inside).toBe('true');
    expect(word('wait.').dataset.inside).toBe('false');
    expect(word('small.').dataset.inside).toBe('false');
    expect(screen.getByLabelText('The cut starts here')).toBeTruthy();
    expect(screen.getByLabelText('The cut ends here')).toBeTruthy();
  });

  it('plays from any word', () => {
    show();
    fireEvent.click(word('ticket.'));
    // "ticket." starts at 14.75 s in the recording, three seconds into the
    // cut: the clock reads clip time, the recording's is on hover.
    const clock = screen.getByTestId('timecode');
    expect(clock.textContent).toBe('0:03.0 / 0:03.3');
    expect(clock.getAttribute('title')).toBe('0:14.8 in the recording');
  });

  it('says so when the analysis has no transcript, rather than showing nothing', () => {
    show({ transcript: { status: 'missing', transcript: null } });
    expect(screen.getByText(/has no transcript to read here/i)).toBeTruthy();
  });
});

describe('the cut', () => {
  it('can start a sentence earlier, and says the cut is now the reviewer’s', () => {
    const props = show();
    const context = word('wait.').closest('.review-sentence')!;
    fireEvent.click(within(context as HTMLElement).getByRole('button', { name: 'Start here' }));
    expect(screen.getByText('Your cut')).toBeTruthy();
    expect(word('wait.').dataset.inside).toBe('true');
    fireEvent.click(screen.getByRole('button', { name: 'Approve' }));
    const [cut, open] = vi.mocked(props.onApprove).mock.calls[0]!;
    expect(cut!.startTicks).toBeLessThan(CLIP_START);
    expect(cut!.endTicks).toBe(CLIP_END);
    expect(open).toBe(false);
  });

  it('goes back to the suggested cut, and then approves the search’s own', () => {
    const props = show();
    const context = word('small.').closest('.review-sentence')!;
    fireEvent.click(within(context as HTMLElement).getByRole('button', { name: 'End here' }));
    fireEvent.click(screen.getByRole('button', { name: /back to the suggested cut/i }));
    fireEvent.click(screen.getByRole('button', { name: 'Approve' }));
    expect(props.onApprove).toHaveBeenCalledWith(null, false);
  });

  it('marks the out point at the playhead, between words', () => {
    const props = show();
    fireEvent.click(word('ticket.'));
    fireEvent.keyDown(window, { key: 'o' });
    fireEvent.keyDown(window, { key: 'a' });
    const [cut] = vi.mocked(props.onApprove).mock.calls[0]!;
    const a = WORDS.words.find((entry) => entry.text === 'a' && entry.startTicks > 13 * SECOND)!;
    // The nearest boundary to the start of "ticket." is the pause after "a".
    expect(cut!.endTicks).toBeGreaterThanOrEqual(a.endTicks);
    expect(cut!.endTicks).toBeLessThan(
      WORDS.words.find((entry) => entry.text === 'ticket.')!.startTicks + 1,
    );
  });

  it('lets the runner-up be heard before it is taken', () => {
    const props = show();
    fireEvent.click(screen.getByRole('button', { name: 'Alternative cut' }));
    expect(screen.getByText('Alternative')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Use this cut' }));
    expect(screen.getByText('Your cut')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Approve' }));
    expect(props.onApprove).toHaveBeenCalledWith(row().boundary!.alternative, false);
  });
});

describe('deciding', () => {
  it('approves the search’s cut, with or without opening it', () => {
    const props = show();
    fireEvent.click(screen.getByRole('button', { name: 'Approve' }));
    fireEvent.click(screen.getByRole('button', { name: 'Approve & edit' }));
    expect(props.onApprove).toHaveBeenNthCalledWith(1, null, false);
    expect(props.onApprove).toHaveBeenNthCalledWith(2, null, true);
  });

  it('keeps and rejects in one click, and a second click takes it back', () => {
    const props = show({ rows: [row({ decision: 'kept' })] });
    const keep = screen.getByRole('button', { name: 'Keep for later' });
    expect(keep.getAttribute('aria-pressed')).toBe('true');
    fireEvent.click(keep);
    fireEvent.click(screen.getByRole('button', { name: 'Reject' }));
    expect(props.onDecide).toHaveBeenNthCalledWith(1, null);
    expect(props.onDecide).toHaveBeenNthCalledWith(2, 'rejected');
  });

  it('offers an approved clip’s edit and records nothing more', () => {
    const onOpenEdit = vi.fn();
    const props = show({
      rows: [row({ decision: 'approved', docId: 'edt_1' })],
      onOpenEdit,
    });
    const decide = screen.getByRole('group', { name: /decide about this clip/i });
    expect(within(decide).getByText('Approved')).toBeTruthy();
    fireEvent.click(within(decide).getByRole('button', { name: 'Open edit' }));
    fireEvent.keyDown(window, { key: 'Enter' });
    fireEvent.keyDown(window, { key: 'a' });
    expect(onOpenEdit).toHaveBeenCalledTimes(2);
    expect(props.onApprove).not.toHaveBeenCalled();
  });

  it('makes a moved cut of a clip with an edit a new edit, and says so', () => {
    show({ rows: [row({ decision: 'approved', docId: 'edt_1' })], onOpenEdit: vi.fn() });
    const context = word('wait.').closest('.review-sentence')!;
    fireEvent.click(within(context as HTMLElement).getByRole('button', { name: 'Start here' }));
    expect(screen.getByRole('button', { name: 'Approve as new edit' })).toBeTruthy();
  });

  it('decides from the keyboard, one key a verdict', () => {
    const onUndo = vi.fn();
    const props = show({ rows: [row(), row({ candidateId: 'cand_2', rank: 2 })], onUndo });
    fireEvent.keyDown(window, { key: 'x' });
    fireEvent.keyDown(window, { key: 'h' });
    fireEvent.keyDown(window, { key: 'Enter' });
    fireEvent.keyDown(window, { key: 'ArrowDown' });
    fireEvent.keyDown(window, { key: 'z', metaKey: true });
    expect(props.onDecide).toHaveBeenNthCalledWith(1, 'rejected');
    expect(props.onDecide).toHaveBeenNthCalledWith(2, 'kept');
    expect(props.onApprove).toHaveBeenCalledWith(null, true);
    expect(props.onSelect).toHaveBeenCalledWith('cand_2');
    expect(onUndo).toHaveBeenCalledOnce();
  });

  it('leaves the keys to a control that owns them', () => {
    const props = show();
    const handle = screen.getByRole('slider', { name: 'End of the cut' });
    handle.focus();
    fireEvent.keyDown(handle, { key: 'ArrowDown' });
    const tab = screen.getByRole('tab', { name: /why/i });
    fireEvent.keyDown(tab, { key: 'Enter' });
    expect(props.onSelect).not.toHaveBeenCalled();
    expect(props.onApprove).not.toHaveBeenCalled();
  });

  it('shuts every decision while one is being written', () => {
    const props = show({ busy: true });
    expect(screen.getByRole('button', { name: 'Reject' })).toHaveProperty('disabled', true);
    expect(screen.getByRole('button', { name: 'Keep for later' })).toHaveProperty('disabled', true);
    fireEvent.keyDown(window, { key: 'a' });
    expect(props.onApprove).not.toHaveBeenCalled();
  });

  it('says what the last action did', () => {
    show({ notice: 'Kept for later.' });
    expect(screen.getByText('Kept for later.')).toBeTruthy();
  });

  it('makes the override of a declined moment explicit', () => {
    show({
      rows: [
        row({
          band: 'declined',
          bandLabel: 'Declined by editorial review',
          review: { status: 'rejected', route: 'local', reasons: ['The payoff is missing.'] },
        }),
      ],
    });
    expect(screen.getByRole('button', { name: 'Approve anyway' })).toBeTruthy();
    openTab(/why/i);
    expect(screen.getByText(/did not recommend this moment/i)).toBeTruthy();
    expect(screen.getByText('The payoff is missing.')).toBeTruthy();
  });
});

describe('the queue', () => {
  it('lists every clip with its decision, and hides the decided when asked', () => {
    const props = show({
      rows: [
        row(),
        row({ candidateId: 'cand_2', rank: 2, headline: 'Already rejected', decision: 'rejected' }),
        row({ candidateId: 'cand_3', rank: 3, headline: 'Still to review' }),
      ],
    });
    const queue = screen.getByRole('navigation', { name: /clips in this review/i });
    expect(within(queue).getByText('Already rejected')).toBeTruthy();
    expect(within(queue).getByText('1 of 3 reviewed')).toBeTruthy();
    fireEvent.click(within(queue).getByRole('button', { name: /^to review/i }));
    expect(within(queue).queryByText('Already rejected')).toBeNull();
    fireEvent.click(within(queue).getByText('Still to review'));
    expect(props.onSelect).toHaveBeenCalledWith('cand_3');
  });
});

describe('why and details', () => {
  it('reads an axis nobody measured as its reason, never as a zero', () => {
    show();
    openTab(/why/i);
    expect(screen.getByText('no prompt was given')).toBeTruthy();
    expect(screen.getByText('—')).toBeTruthy();
    expect(screen.getByText(/1 of 2 measured/i)).toBeTruthy();
  });

  it('turns a quote into a place to play from', () => {
    show();
    openTab(/why/i);
    fireEvent.click(screen.getAllByRole('button', { name: /play from/i })[0]!);
    // The hook is quoted at the clip's start, 11.8 s into the recording.
    const clock = screen.getByTestId('timecode');
    expect(clock.textContent).toBe('0:00.0 / 0:03.3');
    expect(clock.getAttribute('title')).toBe('0:11.8 in the recording');
  });

  it('names the clips that cover the same ground', () => {
    const props = show({
      rows: [row(), row({ candidateId: 'cand_2', rank: 2, startTicks: CLIP_START + SECOND })],
    });
    openTab(/details/i);
    fireEvent.click(screen.getByRole('button', { name: 'Clip 02' }));
    expect(props.onSelect).toHaveBeenCalledWith('cand_2');
  });

  it('keeps the search’s numbers out of the way, in diagnostics', () => {
    show();
    openTab(/details/i);
    const diagnostics = screen.getByText('Diagnostics').closest('details')!;
    expect(diagnostics.open).toBe(false);
    expect(within(diagnostics).getByText('hook weight')).toBeTruthy();
  });
});

describe('what cannot be shown', () => {
  it('says there is nothing to preview rather than showing a dead frame', () => {
    show({ proxyUrl: null });
    expect(screen.getByText('Preview unavailable')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Play' })).toHaveProperty('disabled', true);
  });

  it('says a clip is missing from the ranking rather than rendering an empty screen', () => {
    const props = show({ candidateId: 'cand_missing' });
    expect(screen.getByText(/not in the current ranking/i)).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: /back to results/i }));
    expect(props.onBack).toHaveBeenCalled();
  });
});

describe('judging the clip an approval builds', () => {
  it('asks for it once the cut settles, and reads its framing', () => {
    vi.useFakeTimers();
    try {
      const onPreview = vi.fn();
      const plan = {
        rateNum: 30,
        rateDen: 1,
        width: 1080,
        height: 1920,
        crops: [],
        segments: [
          {
            segmentId: 'seg_1',
            sourceFingerprint: 'source',
            inTicks: CLIP_START,
            outTicks: CLIP_END,
            programStartTicks: 0,
            firstFrame: 0,
            endFrame: 30,
            layout: 'two_up',
          },
        ],
        sources: [],
        decisions: ['Two people remain visible in equal portraits.'],
      } as unknown as import('../src/daemon/client.js').PreviewPlan;
      show({ onPreview, preview: plan, proxyUrl: 'http://localhost/proxy.mp4' });
      expect(onPreview).not.toHaveBeenCalled();
      act(() => {
        vi.advanceTimersByTime(300);
      });
      expect(onPreview).toHaveBeenCalledWith(null);
      const note = screen.getByText('Two speakers');
      expect(note.getAttribute('title')).toContain('equal portraits');
    } finally {
      vi.useRealTimers();
    }
  });
});
