/**
 * Saved caption styles, key words and the recognizer's non-speech marks.
 */
import type { EditIr } from '@clipmill/contracts';
import { afterEach, describe, expect, it } from 'vitest';

import type { PreviewCue } from '../src/daemon/client.js';
import {
  applyStyle,
  clearKeyWords,
  forgetStyle,
  keyWords,
  lookOptions,
  markKeyWords,
  nonSpeechCount,
  saveStyle,
  savedStyles,
  suggestKeyWords,
} from '../src/editor/captionStyles.js';

afterEach(() => localStorage.clear());

function cue(id: string, words: readonly string[]): PreviewCue {
  return {
    cueId: id,
    firstFrame: 0,
    endFrame: 30,
    region: 'lower_safe',
    karaoke: true,
    leadInCentis: 0,
    lines: [words.map((text, index) => ({ text, holdCentis: 10, wordId: `${id}-${index}` }))],
  };
}

function document(words: readonly (readonly [string, boolean])[]): EditIr {
  return {
    version: 'ir/1',
    timebase: { num: 1, den: 90_000 },
    video: { segments: [] },
    captions: {
      style_ref: 'clipmill.captions.clean.v1',
      cues: [
        {
          cue_id: 'c1',
          start_ticks: 0,
          end_ticks: 90_000,
          region: 'lower_safe',
          anim: 'karaoke',
          lines: [
            {
              words: words.map(([text, emphasis], index) => ({
                text,
                start_ticks: index * 1_000,
                end_ticks: index * 1_000 + 900,
                word_id: `w${index}`,
                ...(emphasis ? { emphasis } : {}),
              })) as [never, ...never[]],
            },
          ],
        },
      ],
    },
    audio: { target_lufs: -14, true_peak_dbtp: -1 },
  } as unknown as EditIr;
}

describe('key words', () => {
  it('picks a number or a name over function words, one per caption', () => {
    expect(suggestKeyWords([cue('a', ['we', 'spent', '400', 'dollars'])])).toEqual(['a-2']);
    expect(suggestKeyWords([cue('b', ['I', 'met', 'Priya', 'there'])])).toEqual(['b-2']);
    // Two words is the whole caption; stressing one stresses nothing.
    expect(suggestKeyWords([cue('c', ['absolutely', 'everything'])])).toEqual([]);
    expect(suggestKeyWords([cue('d', ['and', 'the', 'it', 'was'])])).toEqual([]);
  });

  it('marks only words not already marked, and clears every mark', () => {
    const marked = document([
      ['we', false],
      ['spent', false],
      ['400', true],
    ]);
    expect(keyWords(marked)).toEqual(['w2']);
    expect(clearKeyWords(marked)).toEqual({
      op: 'batch',
      commands: [{ op: 'set_word_emphasis', word_id: 'w2', emphasis: false }],
    });
    expect(clearKeyWords(document([['plain', false]]))).toBeNull();
    expect(markKeyWords([cue('w', ['we', 'spent', '400'])], document([]))).toEqual({
      op: 'batch',
      commands: [{ op: 'set_word_emphasis', word_id: 'w-2', emphasis: true }],
    });
  });
});

describe('non-speech marks', () => {
  it('counts dashes, silence markers and annotations, not words', () => {
    expect(
      nonSpeechCount(
        document([
          ['-', false],
          ['[BLANK_AUDIO]', false],
          ['(laughs)', false],
          ['hello', false],
          ['50%', false],
        ]),
      ),
    ).toBe(3);
  });
});

describe('saved styles', () => {
  it('keeps the look and leaves the clip its own layout', () => {
    saveStyle({
      name: '  Podcast  ',
      styleRef: 'clipmill.captions.boxed.v1',
      options: { font_family: 'Anton', position: { x: 500, y: 300 }, words_on_screen: 2 },
    });
    const [saved] = savedStyles();
    expect(saved?.name).toBe('Podcast');
    expect(saved?.options).toEqual({ font_family: 'Anton' });
    expect(lookOptions({ accent: '#00ff00', words_on_screen: 3 })).toEqual({ accent: '#00ff00' });
    expect(applyStyle(saved!, { position: { x: 10, y: 20 }, font_size: 99 })).toEqual({
      op: 'batch',
      commands: [
        { op: 'set_caption_style', style_ref: 'clipmill.captions.boxed.v1' },
        {
          op: 'set_caption_options',
          options: { font_family: 'Anton', position: { x: 10, y: 20 } },
        },
      ],
    });
    expect(forgetStyle('Podcast')).toEqual([]);
  });
});
