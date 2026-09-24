/**
 * The clip's words, read like the Inspector's transcript: sentences with their
 * time, the talk around the clip set back and ready to include, and words cut
 * from the middle struck through. Select words to cut them, hide them from the
 * captions or correct them; fillers and long pauses are gathered for review.
 */
import type { EditIr } from '@clipmill/contracts';
import { Search, X } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';

import { Button } from '../components/ui/button.js';
import { Checkbox } from '../components/ui/checkbox.js';
import { Input } from '../components/ui/input.js';
import type { EditCommandJson, PreviewPlan } from '../daemon/client.js';
import { clockTenths } from '../inspector/review.js';
import { TipButton } from '../inspector/TipButton.js';
import { type Transcript, endAfter, startBefore } from '../results/transcript.js';
import { extendWithCaptions } from './commands.js';
import { type WordRange, positions } from './selection.js';
import { frameOfTicks } from './timeline.js';
import {
  type CaptionWordRef,
  captionRefs,
  correctCaptionWord,
  countMatches,
  cutPauses,
  cutWords,
  hideInCaptions,
  isFiller,
  longPauses,
  programWords,
  replaceCaptionWords,
  shownCues,
} from './transcript.js';

/** Sentences shown either side of the clip before anyone asks for more. */
const CONTEXT = 2;
const MORE = 3;

export interface EditorTranscriptProps {
  readonly plan: PreviewPlan;
  readonly document: EditIr | null;
  readonly transcript: Transcript | null;
  readonly frame: number;
  readonly playing: boolean;
  readonly busy: boolean;
  readonly selected: WordRange | null;
  /** Bumped to open find and replace from the keyboard. */
  readonly findSignal: number;
  readonly onSelect: (range: WordRange | null) => void;
  readonly onSeek: (frame: number) => void;
  readonly onApply: (command: EditCommandJson) => void;
}

type Review = 'fillers' | 'pauses' | null;

export function EditorTranscript({
  plan,
  document,
  transcript,
  frame,
  playing,
  busy,
  selected,
  findSignal,
  onSelect,
  onSeek,
  onApply,
}: EditorTranscriptProps) {
  const words = useMemo(() => programWords(plan, transcript), [plan, transcript]);
  const refs = useMemo(
    () => captionRefs(words, shownCues(plan, document)),
    [words, plan, document],
  );
  const [finding, setFinding] = useState(false);
  const [review, setReview] = useState<Review>(null);
  useEffect(() => {
    if (findSignal > 0) setFinding(true);
  }, [findSignal]);

  const fillers = useMemo(
    () => words.flatMap((word, at) => (isFiller(word.text) ? [at] : [])),
    [words],
  );
  const pauses = useMemo(() => longPauses(words), [words]);

  return (
    <aside className="review-side edit-side edit-transcript-side" aria-label="Transcript">
      <div className="edit-panel-head">
        <span className="edit-panel-title">Transcript</span>
        <span className="review-spacer" />
        <TipButton
          label="Find and replace"
          pressed={finding}
          disabled={!transcript}
          onClick={() => setFinding(!finding)}
        >
          <Search className="size-4" />
        </TipButton>
      </div>
      {finding && (
        <FindReplace plan={plan} busy={busy} onApply={onApply} onClose={() => setFinding(false)} />
      )}
      {transcript && words.length > 0 && (
        <div className="edit-cleanup" role="group" aria-label="Clean up">
          <button
            type="button"
            className="edit-cleanup-chip"
            aria-pressed={review === 'fillers'}
            disabled={fillers.length === 0}
            onClick={() => setReview(review === 'fillers' ? null : 'fillers')}
          >
            <span className="mono">{fillers.length}</span>{' '}
            {fillers.length === 1 ? 'filler word' : 'filler words'}
          </button>
          <button
            type="button"
            className="edit-cleanup-chip"
            aria-pressed={review === 'pauses'}
            disabled={pauses.length === 0}
            onClick={() => setReview(review === 'pauses' ? null : 'pauses')}
          >
            <span className="mono">{pauses.length}</span>{' '}
            {pauses.length === 1 ? 'long pause' : 'long pauses'}
          </button>
        </div>
      )}
      {review === 'fillers' && (
        <FillerReview
          key={plan.revision}
          plan={plan}
          words={words}
          refs={refs}
          fillers={fillers}
          busy={busy}
          onApply={onApply}
          onSeek={onSeek}
          onClose={() => setReview(null)}
        />
      )}
      {review === 'pauses' && (
        <PauseReview
          key={plan.revision}
          plan={plan}
          words={words}
          pauses={pauses}
          busy={busy}
          onApply={onApply}
          onSeek={onSeek}
          onClose={() => setReview(null)}
        />
      )}
      {!transcript ? (
        <p className="review-empty-note">
          This analysis has no transcript. Analyze the recording again to edit by word.
        </p>
      ) : (
        <Words
          plan={plan}
          transcript={transcript}
          words={words}
          refs={refs}
          frame={frame}
          playing={playing}
          busy={busy}
          selected={selected}
          onSelect={onSelect}
          onSeek={onSeek}
          onApply={onApply}
        />
      )}
      {selected && words[selected.first] && (
        <SelectionBar
          plan={plan}
          words={words}
          refs={refs}
          selected={selected}
          busy={busy}
          onApply={onApply}
          onClear={() => onSelect(null)}
        />
      )}
    </aside>
  );
}

function Words({
  plan,
  transcript,
  words,
  refs,
  frame,
  playing,
  busy,
  selected,
  onSelect,
  onSeek,
  onApply,
}: {
  readonly plan: PreviewPlan;
  readonly transcript: Transcript;
  readonly words: ReturnType<typeof programWords>;
  readonly refs: readonly (CaptionWordRef | null)[];
  readonly frame: number;
  readonly playing: boolean;
  readonly busy: boolean;
  readonly selected: WordRange | null;
  readonly onSelect: (range: WordRange | null) => void;
  readonly onSeek: (frame: number) => void;
  readonly onApply: (command: EditCommandJson) => void;
}) {
  const [earlier, setEarlier] = useState(0);
  const [later, setLater] = useState(0);
  const list = useRef<HTMLDivElement>(null);
  const touched = useRef(0);
  const anchor = useRef<number | null>(null);
  const dragged = useRef(false);

  // Where each spoken word landed in the program, by its place in the transcript.
  const placed = useMemo(() => {
    const map = new Map<number, number>();
    words.forEach((word, at) => map.set(word.sourceIndex, at));
    return map;
  }, [words]);
  const first = plan.segments[0];
  const last = plan.segments.at(-1);
  const firstSource = words[0]?.sourceIndex ?? -1;
  const lastSource = words.at(-1)?.sourceIndex ?? -1;
  const sentences = transcript.sentences;
  const opening = sentences.findIndex(
    (sentence) => sentence.firstWord + sentence.wordCount > firstSource,
  );
  const closing = sentences.findLastIndex((sentence) => sentence.firstWord <= lastSource);
  const from = Math.max(0, (opening < 0 ? 0 : opening) - CONTEXT - earlier);
  const to = Math.min(
    sentences.length,
    (closing < 0 ? sentences.length - 1 : closing) + 1 + CONTEXT + later,
  );

  const ticks = (frame * plan.rateDen * 90_000) / Math.max(1, plan.rateNum);
  const live = words.findIndex((word) => ticks >= word.startTicks && ticks < word.endTicks);

  useEffect(() => {
    if (!playing || live < 0 || Date.now() - touched.current < 4_000) return;
    list.current
      ?.querySelector(`[data-position="${live}"]`)
      ?.closest('.review-sentence')
      ?.scrollIntoView?.({ block: 'nearest' });
  }, [live, playing]);

  useEffect(() => {
    const release = () => {
      anchor.current = null;
    };
    window.addEventListener('pointerup', release);
    return () => window.removeEventListener('pointerup', release);
  }, []);

  if (words.length === 0) {
    return <p className="review-empty-note">No spoken words remain in this edit.</p>;
  }

  const inRange = (at: number) => selected !== null && at >= selected.first && at <= selected.last;

  return (
    <div
      ref={list}
      className="review-transcript edit-transcript"
      onWheel={() => {
        touched.current = Date.now();
      }}
    >
      <p className="review-transcript-summary">
        <span className="mono">{words.length}</span> words in the clip
      </p>
      {from > 0 && (
        <button type="button" className="review-more" onClick={() => setEarlier(earlier + MORE)}>
          Show earlier
        </button>
      )}
      {sentences.slice(from, to).map((sentence) => {
        const sourceWords = transcript.words.slice(
          sentence.firstWord,
          sentence.firstWord + sentence.wordCount,
        );
        const inside = sourceWords.filter((_, offset) =>
          placed.has(sentence.firstWord + offset),
        ).length;
        const before =
          sentence.firstWord + sentence.wordCount <= firstSource ||
          (inside === 0 && sentence.firstWord < firstSource);
        const after = sentence.firstWord > lastSource;
        const partialHead = !before && sentence.firstWord < firstSource;
        const partialTail = !after && sentence.firstWord + sentence.wordCount - 1 > lastSource;
        const state =
          inside === sourceWords.length ? 'inside' : inside === 0 ? 'outside' : 'partial';
        const firstPlaced = sourceWords.findIndex((_, offset) =>
          placed.has(sentence.firstWord + offset),
        );
        const label =
          firstPlaced >= 0
            ? clockTenths(words[placed.get(sentence.firstWord + firstPlaced)!]!.startTicks)
            : before
              ? 'Before the clip'
              : after
                ? 'After the clip'
                : 'Cut';
        const includeStart =
          first && (before || partialHead) ? startBefore(transcript, sentence.firstWord) : null;
        const includeEnd =
          last && (after || partialTail)
            ? endAfter(transcript, sentence.firstWord + sentence.wordCount - 1)
            : null;
        return (
          <div key={sentence.firstWord} className="review-sentence" data-state={state}>
            <div className="review-sentence-head">
              <span className="mono">{label}</span>
              {(includeStart !== null || includeEnd !== null) && (
                <span className="review-sentence-actions">
                  <button
                    type="button"
                    disabled={busy}
                    onClick={() => {
                      if (includeStart !== null && first)
                        onApply(extendWithCaptions(first.segmentId, includeStart, first.outTicks));
                      else if (includeEnd !== null && last)
                        onApply(extendWithCaptions(last.segmentId, last.inTicks, includeEnd));
                    }}
                  >
                    {includeStart !== null ? 'Include from here' : 'Include to here'}
                  </button>
                </span>
              )}
            </div>
            <p>
              {sourceWords.map((word, offset) => {
                const at = placed.get(sentence.firstWord + offset);
                if (at === undefined) {
                  const between =
                    sentence.firstWord + offset > firstSource &&
                    sentence.firstWord + offset < lastSource;
                  return (
                    <span key={offset}>
                      <span
                        className="review-word edit-word"
                        data-inside="false"
                        data-cut={between ? 'true' : undefined}
                      >
                        {word.text}
                      </span>{' '}
                    </span>
                  );
                }
                const ref = refs[at];
                return (
                  <span key={offset}>
                    <span
                      className="review-word edit-word"
                      data-position={at}
                      data-inside="true"
                      data-live={at === live ? 'true' : undefined}
                      data-selected={inRange(at) ? 'true' : undefined}
                      data-hidden={ref === null ? 'true' : undefined}
                      data-filler={isFiller(word.text) ? 'true' : undefined}
                      title={ref === null ? 'Hidden from the captions' : undefined}
                      onPointerDown={(event) => {
                        if (event.button > 0) return;
                        event.preventDefault();
                        dragged.current = false;
                        if (event.shiftKey && selected) {
                          onSelect({
                            first: Math.min(selected.first, at),
                            last: Math.max(selected.last, at),
                          });
                          return;
                        }
                        anchor.current = at;
                        onSelect({ first: at, last: at });
                      }}
                      onPointerEnter={() => {
                        if (anchor.current === null || anchor.current === at) return;
                        dragged.current = true;
                        onSelect({
                          first: Math.min(anchor.current, at),
                          last: Math.max(anchor.current, at),
                        });
                      }}
                      onClick={(event) => {
                        if (!dragged.current && !event.shiftKey) {
                          onSeek(frameOfTicks(plan, words[at]!.startTicks));
                        }
                      }}
                    >
                      {ref?.text ?? word.text}
                    </span>{' '}
                  </span>
                );
              })}
            </p>
          </div>
        );
      })}
      {to < sentences.length && (
        <button type="button" className="review-more" onClick={() => setLater(later + MORE)}>
          Show later
        </button>
      )}
    </div>
  );
}

function SelectionBar({
  plan,
  words,
  refs,
  selected,
  busy,
  onApply,
  onClear,
}: {
  readonly plan: PreviewPlan;
  readonly words: ReturnType<typeof programWords>;
  readonly refs: readonly (CaptionWordRef | null)[];
  readonly selected: WordRange;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onClear: () => void;
}) {
  const chosen = positions(selected);
  const cut = cutWords(plan, words, chosen);
  const hide = hideInCaptions(
    plan,
    chosen.map((at) => refs[at] ?? null),
  );
  const single = chosen.length === 1 ? (refs[selected.first] ?? null) : null;
  const [correcting, setCorrecting] = useState(false);
  const [draft, setDraft] = useState(single?.text ?? '');
  useEffect(() => {
    setCorrecting(false);
    setDraft(single?.text ?? '');
  }, [selected.first, selected.last, single?.text]);
  const length = (words[selected.last]!.endTicks - words[selected.first]!.startTicks) / 90_000;
  return (
    <div className="edit-selection-bar" role="toolbar" aria-label="Selected words">
      {correcting && single ? (
        <form
          className="edit-inline"
          onSubmit={(event) => {
            event.preventDefault();
            const text = draft.trim();
            if (!text || text === single.text || busy) return;
            onApply(correctCaptionWord(plan, single, text));
            setCorrecting(false);
          }}
        >
          <Input
            aria-label="Correct this word"
            value={draft}
            autoFocus
            onChange={(event) => setDraft(event.target.value)}
            className="h-8"
          />
          <Button
            type="submit"
            size="sm"
            disabled={busy || !draft.trim() || draft.trim() === single.text}
          >
            Correct
          </Button>
          <Button type="button" size="sm" variant="ghost" onClick={() => setCorrecting(false)}>
            Cancel
          </Button>
        </form>
      ) : (
        <>
          <div className="edit-selection-head">
            <span className="edit-selection-count">
              <span className="mono">{chosen.length}</span> {chosen.length === 1 ? 'word' : 'words'}
              <span className="edit-selection-length mono"> · {length.toFixed(1)} s</span>
            </span>
            <button
              type="button"
              className="edit-icon-button"
              aria-label="Clear the selection"
              onClick={onClear}
            >
              <X className="size-3.5" aria-hidden="true" />
            </button>
          </div>
          <div className="edit-inline">
            <Button size="sm" disabled={busy || !cut} onClick={() => cut && onApply(cut)}>
              Cut
            </Button>
            <Button
              size="sm"
              variant="outline"
              disabled={busy || !hide}
              onClick={() => hide && onApply(hide)}
            >
              Hide in captions
            </Button>
            {single && (
              <Button
                size="sm"
                variant="outline"
                disabled={busy}
                onClick={() => setCorrecting(true)}
              >
                Correct
              </Button>
            )}
          </div>
        </>
      )}
    </div>
  );
}

function FindReplace({
  plan,
  busy,
  onApply,
  onClose,
}: {
  readonly plan: PreviewPlan;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onClose: () => void;
}) {
  const [find, setFind] = useState('');
  const [replace, setReplace] = useState('');
  const matches = countMatches(plan, find);
  const command = replaceCaptionWords(plan, find, replace);
  return (
    <form
      className="edit-find"
      aria-label="Find and replace"
      onSubmit={(event) => {
        event.preventDefault();
        if (command && !busy) {
          onApply(command);
          setFind('');
          setReplace('');
        }
      }}
      onKeyDown={(event) => {
        if (event.key === 'Escape') {
          event.preventDefault();
          event.stopPropagation();
          onClose();
        }
      }}
    >
      <Input
        aria-label="Find a caption word"
        placeholder="Find a word"
        value={find}
        autoFocus
        onChange={(event) => setFind(event.target.value)}
        className="h-8"
      />
      <Input
        aria-label="Replace with"
        placeholder="Replace with"
        value={replace}
        onChange={(event) => setReplace(event.target.value)}
        className="h-8"
      />
      <div className="edit-inline">
        <span className="review-footnote">
          {find.trim() ? `${matches} in the captions` : 'Fixes a name everywhere at once'}
        </span>
        <span className="review-spacer" />
        <Button type="submit" size="sm" disabled={busy || !command}>
          Replace all
        </Button>
      </div>
    </form>
  );
}

function FillerReview({
  plan,
  words,
  refs,
  fillers,
  busy,
  onApply,
  onSeek,
  onClose,
}: {
  readonly plan: PreviewPlan;
  readonly words: ReturnType<typeof programWords>;
  readonly refs: readonly (CaptionWordRef | null)[];
  readonly fillers: readonly number[];
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onSeek: (frame: number) => void;
  readonly onClose: () => void;
}) {
  const [skipped, setSkipped] = useState<readonly number[]>([]);
  const chosen = fillers.filter((at) => !skipped.includes(at));
  const cut = cutWords(plan, words, chosen);
  const hide = hideInCaptions(
    plan,
    chosen.map((at) => refs[at] ?? null),
  );
  return (
    <div className="edit-review" aria-label="Filler words">
      <p className="review-footnote">
        Cutting removes them from the picture and the sound. Untick any you want to keep.
      </p>
      <ul>
        {fillers.map((at) => (
          <li key={at}>
            <Checkbox
              aria-label={`Cut “${words[at]!.text}” at ${clockTenths(words[at]!.startTicks)}`}
              checked={!skipped.includes(at)}
              onCheckedChange={() =>
                setSkipped((old) =>
                  old.includes(at) ? old.filter((item) => item !== at) : [...old, at],
                )
              }
            />
            <button type="button" onClick={() => onSeek(frameOfTicks(plan, words[at]!.startTicks))}>
              “{words[at]!.text}”
            </button>
            <span className="mono">{clockTenths(words[at]!.startTicks)}</span>
          </li>
        ))}
      </ul>
      <div className="edit-inline">
        <Button
          size="sm"
          disabled={busy || !cut}
          onClick={() => {
            if (cut) onApply(cut);
            onClose();
          }}
        >
          Cut {chosen.length}
        </Button>
        <Button
          size="sm"
          variant="outline"
          disabled={busy || !hide}
          onClick={() => {
            if (hide) onApply(hide);
            onClose();
          }}
        >
          Hide in captions
        </Button>
        <span className="review-spacer" />
        <Button size="sm" variant="ghost" onClick={onClose}>
          Close
        </Button>
      </div>
    </div>
  );
}

function PauseReview({
  plan,
  words,
  pauses,
  busy,
  onApply,
  onSeek,
  onClose,
}: {
  readonly plan: PreviewPlan;
  readonly words: ReturnType<typeof programWords>;
  readonly pauses: ReturnType<typeof longPauses>;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onSeek: (frame: number) => void;
  readonly onClose: () => void;
}) {
  const [skipped, setSkipped] = useState<readonly number[]>([]);
  const chosen = pauses.filter((pause) => !skipped.includes(pause.after));
  const command = cutPauses(plan, chosen);
  const saved =
    chosen.reduce((sum, pause) => sum + (pause.endTicks - pause.startTicks), 0) / 90_000;
  return (
    <div className="edit-review" aria-label="Long pauses">
      <p className="review-footnote">
        Each pause keeps a quarter-second breath. Untick any to leave as it is.
      </p>
      <ul>
        {pauses.map((pause) => (
          <li key={pause.after}>
            <Checkbox
              aria-label={`Shorten the ${pause.seconds.toFixed(1)} s pause after “${words[pause.after]!.text}”`}
              checked={!skipped.includes(pause.after)}
              onCheckedChange={() =>
                setSkipped((old) =>
                  old.includes(pause.after)
                    ? old.filter((item) => item !== pause.after)
                    : [...old, pause.after],
                )
              }
            />
            <button type="button" onClick={() => onSeek(frameOfTicks(plan, pause.startTicks))}>
              after “{words[pause.after]!.text}”
            </button>
            <span className="mono">{pause.seconds.toFixed(1)} s</span>
          </li>
        ))}
      </ul>
      <div className="edit-inline">
        <Button
          size="sm"
          disabled={busy || !command}
          onClick={() => {
            if (command) onApply(command);
            onClose();
          }}
        >
          Shorten {chosen.length}
          {saved > 0 ? ` · saves ${saved.toFixed(1)} s` : ''}
        </Button>
        <span className="review-spacer" />
        <Button size="sm" variant="ghost" onClick={onClose}>
          Close
        </Button>
      </div>
    </div>
  );
}
