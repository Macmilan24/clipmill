/**
 * Correcting a word from the caption panel.
 *
 * The panel shows the burned-in cues. A correction typed there must be
 * addressed to the word — so it reaches the sidecars too — and a split or a
 * merge must be addressed to the list on screen, since each list numbers its
 * own cues and `hot_1` is nobody in the reading list.
 */
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { PreviewPlan } from '../src/daemon/client.js';
import { Captions } from '../src/editor/Captions.js';
import { mapping } from './support/plan.js';

function plan(): PreviewPlan {
  const program = { frameCount: 60, rateNum: 30, rateDen: 1 };
  return {
    ...mapping(program),
    revision: 1,
    rateNum: 30,
    rateDen: 1,
    frameCount: 60,
    crops: Array.from({ length: 60 }, () => null),
    cues: [
      {
        cueId: 'hot_1',
        firstFrame: 0,
        endFrame: 30,
        region: 'lower_safe',
        karaoke: true,
        leadInCentis: 0,
        lines: [
          [
            { text: 'meet', holdCentis: 50, wordId: 'w1' },
            { text: 'Samy', holdCentis: 50, wordId: 'w2' },
          ],
        ],
      },
      {
        cueId: 'hot_2',
        firstFrame: 30,
        endFrame: 60,
        region: 'lower_safe',
        karaoke: true,
        leadInCentis: 0,
        lines: [[{ text: 'today', holdCentis: 100, wordId: 'w3' }]],
      },
    ],
    gain: [],
    width: 1080,
    height: 1920,
  };
}

function show(current: PreviewPlan = plan()) {
  const onApply = vi.fn();
  render(<Captions plan={current} frame={0} busy={false} onApply={onApply} />);
  return { onApply };
}

describe('correcting a word', () => {
  it('sends the correction to the word, so both presentations get it', () => {
    const { onApply } = show();
    fireEvent.click(screen.getByRole('button', { name: 'Samy' }));
    const field = screen.getByRole('textbox', { name: /correct this word/i });
    expect((field as HTMLInputElement).value).toBe('Samy');
    fireEvent.change(field, { target: { value: 'Sami' } });
    fireEvent.click(screen.getByRole('button', { name: /^correct$/i }));
    expect(onApply).toHaveBeenCalledWith({ op: 'set_word_text', word_id: 'w2', text: 'Sami' });
  });

  it('does not send a correction that changes nothing', () => {
    const { onApply } = show();
    fireEvent.click(screen.getByRole('button', { name: 'Samy' }));
    expect(screen.getByRole('button', { name: /^correct$/i })).toHaveProperty('disabled', true);
    expect(onApply).not.toHaveBeenCalled();
  });

  it('falls back to the cue on screen for a document that predates word ids', () => {
    const legacy = plan();
    const { onApply } = show({
      ...legacy,
      cues: legacy.cues.map((cue) => ({
        ...cue,
        lines: cue.lines.map((line) => line.map((word) => ({ ...word, wordId: '' }))),
      })),
    });
    fireEvent.click(screen.getByRole('button', { name: 'Samy' }));
    fireEvent.change(screen.getByRole('textbox', { name: /correct this word/i }), {
      target: { value: 'Sami' },
    });
    fireEvent.click(screen.getByRole('button', { name: /^correct$/i }));
    expect(onApply).toHaveBeenCalledWith({
      op: 'edit_caption_text',
      cue_id: 'hot_1',
      word_index: 1,
      text: 'Sami',
      presentation: 'burn_in',
    });
  });

  it('addresses a split and a merge to the list on screen', () => {
    const { onApply } = show();
    fireEvent.click(screen.getByRole('button', { name: 'Samy' }));
    fireEvent.click(screen.getByRole('button', { name: /split here/i }));
    expect(onApply).toHaveBeenLastCalledWith({
      op: 'split_cue',
      cue_id: 'hot_1',
      at_word_index: 1,
      new_cue_id: 'hot_1_b',
      presentation: 'burn_in',
    });
    fireEvent.click(screen.getAllByRole('button', { name: /merge with next/i })[0]!);
    expect(onApply).toHaveBeenLastCalledWith({
      op: 'merge_cues',
      first_cue_id: 'hot_1',
      second_cue_id: 'hot_2',
      presentation: 'burn_in',
    });
  });
});
