/**
 * The Editor's gestures, end to end through the component: a click only
 * selects or seeks, a drag sends one command on release, edges land between
 * words, and the keys keep working after a mouse click.
 */
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { TooltipProvider } from '../src/components/ui/tooltip.js';
import { Editor } from '../src/screens/Editor.js';
import { TICKS } from './support/plan.js';
import { talk, wordStart } from './support/talk.js';
import { drag, measureTrack, seekTo, xOf } from './support/timeline.js';

function show(fillers: readonly number[] = []) {
  const { plan, document, transcript } = talk(fillers);
  const onApply = vi.fn();
  render(
    <TooltipProvider>
      <Editor
        plan={plan}
        document={document}
        transcript={transcript}
        proxyUrls={new Map(plan.proxies.map((proxy) => [proxy.sourceFingerprint, 'proxy.mp4']))}
        docId="edt_talk"
        labels={{ project: 'Talks', clip: 'A steady speaker' }}
        loading={false}
        problem={null}
        busy={false}
        canUndo={false}
        canRedo={false}
        resolving={false}
        resolveRefusal={null}
        picker={null}
        onOpenResults={() => {}}
        onExport={null}
        onApply={onApply}
        onUndo={() => {}}
        onRedo={() => {}}
        onResolve={() => {}}
      />
    </TooltipProvider>,
  );
  return { plan, onApply };
}

let track: ReturnType<typeof measureTrack> | null = null;
beforeEach(() => {
  vi.spyOn(HTMLMediaElement.prototype, 'play').mockResolvedValue(undefined);
  vi.spyOn(HTMLMediaElement.prototype, 'pause').mockImplementation(() => {});
});
afterEach(() => {
  track?.mockRestore();
  track = null;
  cleanup();
  vi.restoreAllMocks();
});

describe('captions on the timeline', () => {
  it('selects a caption on a click and never moves it', () => {
    const { onApply } = show();
    const third = document.querySelectorAll('.edit-cue')[2]!;
    fireEvent.pointerDown(third, { button: 0 });
    fireEvent.pointerUp(window);
    expect(onApply).not.toHaveBeenCalled();
    expect(screen.getByText('Caption at 0:02.0')).toBeTruthy();
  });

  it('stretches a caption only as far as the next one', () => {
    const { plan, onApply } = show();
    track = measureTrack();
    const edge = screen.getByRole('button', { name: 'End of the caption “word24. word25”' });
    drag(edge, { x: xOf(plan, 261_000) }, { x: xOf(plan, 315_000) });
    expect(onApply).toHaveBeenCalledWith({
      op: 'set_cue_timing',
      cue_id: 'cue_3',
      start_ticks: 180_000,
      end_ticks: 270_000,
      presentation: 'burn_in',
    });
  });
});

describe('the clip’s edges', () => {
  it('selects the clip on a click on its edge and sends nothing', () => {
    const { onApply } = show();
    const start = screen.getByRole('button', { name: 'Start of the clip' });
    fireEvent.pointerDown(start, { button: 0 });
    fireEvent.pointerUp(window);
    expect(onApply).not.toHaveBeenCalled();
    expect(screen.getByRole('tab', { name: /framing/i }).getAttribute('aria-selected')).toBe(
      'true',
    );
  });

  it('pulls the start back into the recording, to the gap before a word', () => {
    const { plan, onApply } = show();
    track = measureTrack();
    drag(
      screen.getByRole('button', { name: 'Start of the clip' }),
      { x: xOf(plan, 0) },
      { x: xOf(plan, -2.3 * TICKS) },
    );
    // 597.7 s is nearest the gap before word 16, which starts at 598 s.
    expect(onApply).toHaveBeenCalledWith({
      op: 'extend_with_captions',
      segment_id: 'seg_1',
      in_ticks: wordStart(16) - 9_000,
      out_ticks: 610 * TICKS,
    });
  });

  it('trims the start to the gap before a word, never inside one', () => {
    const { plan, onApply } = show();
    track = measureTrack();
    drag(
      screen.getByRole('button', { name: 'Start of the clip' }),
      { x: xOf(plan, 0) },
      { x: xOf(plan, 3.1 * TICKS) },
    );
    expect(onApply).toHaveBeenCalledWith({
      op: 'trim',
      segment_id: 'seg_1',
      in_ticks: wordStart(26) - 9_000,
      out_ticks: 610 * TICKS,
    });
  });

  it('splits with the blade between two words, under a fresh section id', () => {
    const { plan, onApply } = show();
    fireEvent.click(screen.getByRole('button', { name: 'Blade' }));
    track = measureTrack();
    fireEvent.pointerDown(document.querySelector('.edit-video')!, {
      clientX: xOf(plan, 4.05 * TICKS),
      button: 0,
    });
    const at = wordStart(28) - 9_000;
    expect(onApply).toHaveBeenCalledWith({
      op: 'split_segment',
      segment_id: 'seg_1',
      at_ticks: at,
      new_segment_id: `seg_1_cut_${at}`,
    });
  });
});

describe('the keyboard', () => {
  it('still plays on Space after a toolbar button was clicked', () => {
    show();
    const snapping = screen.getByRole('button', { name: 'Snapping' });
    fireEvent.pointerDown(snapping, { button: 0 });
    fireEvent.click(snapping);
    snapping.focus();
    fireEvent.keyDown(snapping, { key: ' ' });
    expect(HTMLMediaElement.prototype.play).toHaveBeenCalled();
  });

  it('cuts the selected words, with the pause after them, on Delete', () => {
    const { onApply } = show();
    const word = document.querySelector('[data-position="2"]')!;
    fireEvent.pointerDown(word, { button: 0 });
    fireEvent.keyDown(window, { key: 'Delete' });
    expect(onApply).toHaveBeenCalledWith({
      op: 'ripple_delete',
      start_ticks: 90_000,
      end_ticks: 135_000,
      reflow_edges: true,
    });
  });

  it('marks a range with I and O and deletes it', () => {
    const { plan, onApply } = show();
    seekTo(plan, 30);
    fireEvent.keyDown(window, { key: 'i' });
    seekTo(plan, 60);
    fireEvent.keyDown(window, { key: 'o' });
    fireEvent.keyDown(window, { key: 'Backspace' });
    expect(onApply).toHaveBeenCalledWith({
      op: 'ripple_delete',
      start_ticks: 90_000,
      end_ticks: 180_000,
      reflow_edges: true,
    });
  });
});

describe('the transcript', () => {
  it('includes a sentence from before the clip, captions and all', () => {
    const { onApply } = show();
    const include = screen.getAllByRole('button', { name: 'Include from here' });
    fireEvent.click(include.at(-1)!);
    expect(onApply).toHaveBeenCalledWith({
      op: 'extend_with_captions',
      segment_id: 'seg_1',
      in_ticks: wordStart(15) - 9_000,
      out_ticks: 610 * TICKS,
    });
  });

  it('cuts only the filler words left ticked', () => {
    const { onApply } = show([25, 31]);
    fireEvent.click(screen.getByRole('button', { name: /2 filler words/ }));
    const boxes = screen.getAllByRole('checkbox');
    fireEvent.click(boxes[1]!);
    fireEvent.click(screen.getByRole('button', { name: 'Cut 1' }));
    expect(onApply).toHaveBeenCalledWith({
      op: 'ripple_delete',
      start_ticks: 225_000,
      end_ticks: 270_000,
      reflow_edges: true,
    });
  });

  it('corrects a word everywhere it appears in the captions', () => {
    const { onApply } = show();
    fireEvent.click(screen.getByRole('button', { name: 'Find and replace' }));
    fireEvent.change(screen.getByRole('textbox', { name: 'Find a caption word' }), {
      target: { value: 'word24' },
    });
    expect(screen.getByText('1 in the captions')).toBeTruthy();
    fireEvent.change(screen.getByRole('textbox', { name: 'Replace with' }), {
      target: { value: 'Twenty' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Replace all' }));
    expect(onApply).toHaveBeenCalledWith({
      op: 'batch',
      // The full stop after the word is kept.
      commands: [{ op: 'set_word_text', word_id: 'w24', text: 'Twenty.' }],
    });
  });
});
