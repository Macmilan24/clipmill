/**
 * The clip's words with neighbouring sentences set back. Any word seeks the
 * player; any sentence can become the cut's first or last. A clip of parts
 * shows each part's words in the order they play, with what was taken out of
 * a part set back; its parts are changed in the editor.
 */
import { useEffect, useRef, useState, useSyncExternalStore } from 'react';

import { Skeleton } from '../components/ui/skeleton.js';
import type { ClipPart } from '../results/model.js';
import {
  type Transcript,
  type TranscriptWord,
  endAfter,
  sentencesAround,
  startBefore,
  wordAt,
  wordInside,
} from '../results/transcript.js';
import type { TranscriptState } from '../results/useResults.js';
import { useVoiceNames, voiceOfWords } from '../results/voices.js';
import { VoiceChip } from './VoiceChip.js';
import type { PlaybackController } from './playback.js';
import { type Cut, clockTenths } from './review.js';

export interface TranscriptPanelProps {
  readonly state: TranscriptState;
  readonly cut: Cut;
  readonly controller: PlaybackController;
  readonly onStart: (ticks: number) => void;
  readonly onEnd: (ticks: number) => void;
  /** A clip of parts, in the order they play; its words are shown part by part. */
  readonly parts?: readonly ClipPart[] | null;
}

/** Sentences shown either side of the cut before anyone asks for more. */
const CONTEXT = 2;
/** How many more each "earlier" or "later" adds. */
const MORE = 3;

export function TranscriptPanel({
  state,
  cut,
  controller,
  onStart,
  onEnd,
  parts = null,
}: TranscriptPanelProps) {
  if (state.status !== 'ready') {
    return state.status === 'missing' ? (
      <p className="review-empty-note">
        This analysis has no transcript to read here. Listen to the clip, or analyze the recording
        again to add one.
      </p>
    ) : (
      <div className="review-transcript" aria-busy="true" aria-label="Loading the transcript">
        {[92, 78, 88, 64, 81].map((width, index) => (
          // eslint-disable-next-line react/no-array-index-key -- placeholder lines
          <Skeleton key={index} className="h-3.5" style={{ width: `${width}%` }} />
        ))}
      </div>
    );
  }
  if (parts) {
    return <PartWords transcript={state.transcript} parts={parts} controller={controller} />;
  }
  return (
    <Words
      transcript={state.transcript}
      cut={cut}
      controller={controller}
      onStart={onStart}
      onEnd={onEnd}
    />
  );
}

function Words({
  transcript,
  cut,
  controller,
  onStart,
  onEnd,
}: {
  readonly transcript: Transcript;
  readonly cut: Cut;
  readonly controller: PlaybackController;
  readonly onStart: (ticks: number) => void;
  readonly onEnd: (ticks: number) => void;
}) {
  const [earlier, setEarlier] = useState(0);
  const [later, setLater] = useState(0);
  const live = useSyncExternalStore(controller.subscribe, () =>
    wordAt(transcript, controller.getState().ticks),
  );
  const playing = useSyncExternalStore(controller.subscribe, () => controller.getState().playing);
  const list = useRef<HTMLDivElement>(null);
  const touched = useRef(0);

  const around = sentencesAround(transcript, cut.startTicks, cut.endTicks, CONTEXT);
  const first = Math.max(0, around.first - earlier);
  const end = Math.min(transcript.sentences.length, around.end + later);
  const sentences = transcript.sentences.slice(first, end);
  const inside = transcript.words.filter((word) =>
    wordInside(word, cut.startTicks, cut.endTicks),
  ).length;
  const voices = transcript.voices ?? null;
  const [names, rename] = useVoiceNames(voices?.sourceFingerprint ?? null);
  // Who says each sentence shown, named where it changes.
  const speaking = sentences.map((sentence) =>
    voices
      ? voiceOfWords(
          voices,
          transcript.words.slice(sentence.firstWord, sentence.firstWord + sentence.wordCount),
        )
      : null,
  );

  // Follow the spoken sentence while playing, unless the reviewer has just
  // scrolled somewhere to read — the panel must not snatch the page back.
  useEffect(() => {
    if (!playing || live < 0 || Date.now() - touched.current < 4_000) return;
    list.current
      ?.querySelector(`[data-word="${live}"]`)
      ?.closest('.review-sentence')
      ?.scrollIntoView?.({ block: 'nearest' });
  }, [live, playing]);

  if (transcript.sentences.length === 0) {
    return <p className="review-empty-note">Nothing was said in this part of the recording.</p>;
  }

  return (
    <div
      ref={list}
      className="review-transcript"
      onWheel={() => {
        touched.current = Date.now();
      }}
      onClick={(event) => {
        const word = (event.target as HTMLElement).closest<HTMLElement>('[data-word]');
        const at = word ? transcript.words[Number(word.dataset.word)] : undefined;
        if (at) controller.seek(at.startTicks);
      }}
    >
      <p className="review-transcript-summary">
        <span className="mono">{inside}</span> words in the cut
      </p>
      {first > 0 && (
        <button type="button" className="review-more" onClick={() => setEarlier(earlier + MORE)}>
          Show earlier
        </button>
      )}
      {sentences.map((sentence, shown) => {
        const words = transcript.words.slice(
          sentence.firstWord,
          sentence.firstWord + sentence.wordCount,
        );
        const voice = speaking[shown] ?? null;
        const newVoice = voice !== null && (shown === 0 || speaking[shown - 1] !== voice);
        const lastWord = sentence.firstWord + sentence.wordCount - 1;
        const startAt = startBefore(transcript, sentence.firstWord);
        const endAt = endAfter(transcript, lastWord);
        const state = words.every((word) => wordInside(word, cut.startTicks, cut.endTicks))
          ? 'inside'
          : words.some((word) => wordInside(word, cut.startTicks, cut.endTicks))
            ? 'partial'
            : 'outside';
        return (
          <div key={sentence.firstWord} className="review-sentence" data-state={state}>
            {voices && newVoice && (
              <div className="review-voice-line">
                <VoiceChip id={voice} voices={voices} names={names} onRename={rename} />
              </div>
            )}
            <div className="review-sentence-head">
              <span className="mono">{clockTenths(sentence.startTicks)}</span>
              <span className="review-sentence-actions">
                <button
                  type="button"
                  disabled={startAt >= cut.endTicks || startAt === cut.startTicks}
                  onClick={() => onStart(startAt)}
                >
                  Start here
                </button>
                <button
                  type="button"
                  disabled={endAt <= cut.startTicks || endAt === cut.endTicks}
                  onClick={() => onEnd(endAt)}
                >
                  End here
                </button>
              </span>
            </div>
            <p>
              {words.map((word, offset) => {
                const position = sentence.firstWord + offset;
                const within = wordInside(word, cut.startTicks, cut.endTicks);
                const opens =
                  within &&
                  (position === 0 ||
                    !wordInside(transcript.words[position - 1]!, cut.startTicks, cut.endTicks));
                const closes =
                  within &&
                  (position === transcript.words.length - 1 ||
                    !wordInside(transcript.words[position + 1]!, cut.startTicks, cut.endTicks));
                return (
                  <span key={position}>
                    {opens && (
                      <span className="review-edge" aria-label="The cut starts here">
                        In
                      </span>
                    )}
                    <span
                      className="review-word"
                      data-word={position}
                      data-inside={within ? 'true' : 'false'}
                      data-live={position === live ? 'true' : undefined}
                    >
                      {word.text}
                    </span>
                    {closes && (
                      <span className="review-edge" aria-label="The cut ends here">
                        Out
                      </span>
                    )}{' '}
                  </span>
                );
              })}
            </p>
          </div>
        );
      })}
      {end < transcript.sentences.length && (
        <button type="button" className="review-more" onClick={() => setLater(later + MORE)}>
          Show later
        </button>
      )}
    </div>
  );
}

/** Whether a word plays in a part: inside it, and not in what was taken out. */
function plays(word: TranscriptWord, part: ClipPart) {
  return part.kept.some((span) => wordInside(word, span.startTicks, span.endTicks));
}

/** A clip of parts, read part by part in the order they play. */
function PartWords({
  transcript,
  parts,
  controller,
}: {
  readonly transcript: Transcript;
  readonly parts: readonly ClipPart[];
  readonly controller: PlaybackController;
}) {
  const live = useSyncExternalStore(controller.subscribe, () =>
    wordAt(transcript, controller.getState().ticks),
  );
  const playing = useSyncExternalStore(controller.subscribe, () => controller.getState().playing);
  const list = useRef<HTMLDivElement>(null);
  const touched = useRef(0);
  const voices = transcript.voices ?? null;
  const [names, rename] = useVoiceNames(voices?.sourceFingerprint ?? null);
  const spoken = transcript.words.filter((word) => parts.some((part) => plays(word, part))).length;

  useEffect(() => {
    if (!playing || live < 0 || Date.now() - touched.current < 4_000) return;
    list.current
      ?.querySelector(`[data-word="${live}"]`)
      ?.closest('.review-sentence')
      ?.scrollIntoView?.({ block: 'nearest' });
  }, [live, playing]);

  return (
    <div
      ref={list}
      className="review-transcript"
      onWheel={() => {
        touched.current = Date.now();
      }}
      onClick={(event) => {
        const word = (event.target as HTMLElement).closest<HTMLElement>('[data-word]');
        const at = word ? transcript.words[Number(word.dataset.word)] : undefined;
        if (at) controller.seek(at.startTicks);
      }}
    >
      <p className="review-transcript-summary">
        <span className="mono">{spoken}</span> words in {parts.length}{' '}
        {parts.length === 1 ? 'part' : 'parts'}
      </p>
      {parts.map((part, index) => {
        const around = sentencesAround(transcript, part.startTicks, part.endTicks, 0);
        const sentences = transcript.sentences.slice(around.first, around.end);
        return (
          <section key={`${part.startTicks}:${index}`} aria-label={`Part ${index + 1}`}>
            <p className="review-part-head">
              <span className="mono">{index + 1}</span> {part.label}
              <span className="mono"> · {clockTenths(part.startTicks)}</span>
            </p>
            {sentences.map((sentence, shown) => {
              const words = transcript.words.slice(
                sentence.firstWord,
                sentence.firstWord + sentence.wordCount,
              );
              const voice = voices ? voiceOfWords(voices, words) : null;
              const previous =
                shown > 0 && voices
                  ? voiceOfWords(
                      voices,
                      transcript.words.slice(
                        sentences[shown - 1]!.firstWord,
                        sentences[shown - 1]!.firstWord + sentences[shown - 1]!.wordCount,
                      ),
                    )
                  : null;
              const state = words.every((word) => plays(word, part))
                ? 'inside'
                : words.some((word) => plays(word, part))
                  ? 'partial'
                  : 'outside';
              return (
                <div key={sentence.firstWord} className="review-sentence" data-state={state}>
                  {voices && voice !== null && (shown === 0 || previous !== voice) && (
                    <div className="review-voice-line">
                      <VoiceChip id={voice} voices={voices} names={names} onRename={rename} />
                    </div>
                  )}
                  <p>
                    {words.map((word, offset) => {
                      const position = sentence.firstWord + offset;
                      return (
                        <span key={position}>
                          <span
                            className="review-word"
                            data-word={position}
                            data-inside={plays(word, part) ? 'true' : 'false'}
                            data-live={position === live ? 'true' : undefined}
                          >
                            {word.text}
                          </span>{' '}
                        </span>
                      );
                    })}
                  </p>
                </div>
              );
            })}
          </section>
        );
      })}
    </div>
  );
}
