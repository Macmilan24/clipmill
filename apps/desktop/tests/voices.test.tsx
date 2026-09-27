import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { PlaybackController } from '../src/inspector/playback.js';
import { TranscriptPanel } from '../src/inspector/TranscriptPanel.js';
import type { Transcript } from '../src/results/transcript.js';
import {
  type Voices,
  readVoices,
  recallNames,
  rememberNames,
  voiceAt,
  voiceName,
  voiceOfWords,
} from '../src/results/voices.js';

const SECOND = 90_000;
const RECORDING = `sha256:${'1'.repeat(64)}`;

const voices: Voices = {
  sourceFingerprint: RECORDING,
  ids: ['spk_1', 'spk_2'],
  turns: [
    { startTicks: 0, endTicks: 4 * SECOND, speakerId: 'spk_1' },
    { startTicks: 4.5 * SECOND, endTicks: 9 * SECOND, speakerId: 'spk_2' },
    { startTicks: 9.2 * SECOND, endTicks: 12 * SECOND, speakerId: 'spk_1' },
  ],
};

/** Four sentences of three words: the first two said by one voice, then two more. */
function transcript(): Transcript {
  const words = [0, 1, 2, 3].flatMap((sentence) =>
    [0, 1, 2].map((word) => {
      const start = [0, 2, 5, 9.5][sentence]! * SECOND + word * 0.5 * SECOND;
      return { text: `w${sentence}${word}`, startTicks: start, endTicks: start + 0.4 * SECOND };
    }),
  );
  return {
    words,
    sentences: [0, 1, 2, 3].map((sentence) => ({
      startTicks: words[sentence * 3]!.startTicks,
      endTicks: words[sentence * 3 + 2]!.endTicks,
      firstWord: sentence * 3,
      wordCount: 3,
    })),
    voices,
  };
}

beforeEach(() => localStorage.clear());
afterEach(cleanup);

describe('who speaks when', () => {
  it('reads the voices of this recording and no other', () => {
    const document = {
      schema_version: 'clipmill.speech.speakers.v1' as const,
      source_fingerprint: RECORDING,
      audio_artifact_id: `sha256:${'2'.repeat(64)}`,
      vad_artifact_id: `sha256:${'3'.repeat(64)}`,
      producer: { stage: 'speech-speakers', implementation: 'fixture' },
      clustering: {
        window_ticks: 135_000,
        hop_ticks: 67_500,
        link: 0.5,
        merge: 0.65,
        smallest: 0.02,
      },
      coverage: { start_ticks: 0, end_ticks: 12 * SECOND, analyzed: true },
      speakers: [
        { speaker_id: 'spk_1', speech_ticks: 7 * SECOND, first_ticks: 0 },
        { speaker_id: 'spk_2', speech_ticks: 4 * SECOND, first_ticks: 4.5 * SECOND },
      ],
      turns: [{ start_ticks: 0, end_ticks: 4 * SECOND, speaker_id: 'spk_1' }],
    };
    expect(readVoices(document, RECORDING)).toEqual({
      sourceFingerprint: RECORDING,
      ids: ['spk_1', 'spk_2'],
      turns: [{ startTicks: 0, endTicks: 4 * SECOND, speakerId: 'spk_1' }],
    });
    expect(readVoices(document, `sha256:${'9'.repeat(64)}`)).toBeNull();
  });

  it('finds the voice at a moment, and none in the pause between two', () => {
    expect(voiceAt(voices, 2 * SECOND)).toBe('spk_1');
    expect(voiceAt(voices, 4.2 * SECOND)).toBeNull();
    expect(voiceAt(voices, 4.5 * SECOND)).toBe('spk_2');
    expect(voiceAt(voices, 11.9 * SECOND)).toBe('spk_1');
    expect(voiceAt(voices, 12 * SECOND)).toBeNull();
  });

  it('gives a sentence to the voice heard for most of it', () => {
    const across = [
      { startTicks: 3.5 * SECOND, endTicks: 3.9 * SECOND },
      { startTicks: 4.6 * SECOND, endTicks: 5.5 * SECOND },
      { startTicks: 5.6 * SECOND, endTicks: 6.5 * SECOND },
    ];
    expect(voiceOfWords(voices, across)).toBe('spk_2');
    expect(voiceOfWords(voices, [{ startTicks: 4.1 * SECOND, endTicks: 4.3 * SECOND }])).toBeNull();
  });

  it('names a voice by number until a person names it, per recording', () => {
    expect(voiceName('spk_2', {})).toBe('Speaker 2');
    expect(voiceName('spk_2', { spk_2: '  Andrew ' })).toBe('Andrew');
    rememberNames(RECORDING, { spk_1: 'Host' });
    expect(recallNames(RECORDING)).toEqual({ spk_1: 'Host' });
    expect(recallNames(`sha256:${'9'.repeat(64)}`)).toEqual({});
    rememberNames(RECORDING, {});
    expect(localStorage.getItem(`clipmill.voices.${RECORDING}`)).toBeNull();
  });
});

describe('the Inspector’s transcript', () => {
  function show() {
    render(
      <TranscriptPanel
        state={{ status: 'ready', transcript: transcript() }}
        cut={{ startTicks: 0, endTicks: 12 * SECOND }}
        controller={new PlaybackController(0)}
        onStart={() => {}}
        onEnd={() => {}}
      />,
    );
  }

  it('names the voice where it changes, not on every sentence', () => {
    show();
    expect(
      screen.getAllByRole('button', { name: /^Speaker \d$/ }).map((chip) => chip.textContent),
    ).toEqual(['Speaker 1', 'Speaker 2', 'Speaker 1']);
  });

  it('names a voice for every sentence it says, and keeps the name', () => {
    show();
    fireEvent.click(screen.getAllByRole('button', { name: 'Speaker 1' })[0]!);
    const field = screen.getByRole('textbox', { name: 'Name Speaker 1' });
    fireEvent.change(field, { target: { value: 'Andrew' } });
    fireEvent.keyDown(field, { key: 'Enter' });
    expect(screen.getAllByRole('button', { name: 'Andrew' })).toHaveLength(2);
    expect(recallNames(RECORDING)).toEqual({ spk_1: 'Andrew' });
    // Emptied, it is a number again.
    fireEvent.click(screen.getAllByRole('button', { name: 'Andrew' })[0]!);
    const again = screen.getByRole('textbox', { name: 'Name Andrew' });
    fireEvent.change(again, { target: { value: '' } });
    fireEvent.blur(again);
    expect(screen.getAllByRole('button', { name: 'Speaker 1' })).toHaveLength(2);
    expect(recallNames(RECORDING)).toEqual({});
  });
});
