import { useEffect, useMemo, useRef, useState } from 'react';
import { Check, Search, Scissors } from 'lucide-react';

import { Button } from '../components/ui/button.js';
import { Input } from '../components/ui/input.js';
import type { EditCommandJson, PreviewPlan } from '../daemon/client.js';
import type { Transcript } from '../results/transcript.js';
import { frameAt } from './player.js';
import { extendWithCaptions } from './commands.js';
import {
  TICKS,
  cutPauses,
  cutWords,
  isFiller,
  longPauses,
  programWords,
  replaceCaptionWords,
} from './transcript.js';

export interface WordRange {
  readonly first: number;
  readonly last: number;
}

export function EditorTranscript({
  plan,
  transcript,
  frame,
  busy,
  selected,
  findSignal,
  onSelect,
  onSeek,
  onApply,
}: {
  readonly plan: PreviewPlan;
  readonly transcript: Transcript | null;
  readonly frame: number;
  readonly busy: boolean;
  readonly selected: WordRange | null;
  readonly findSignal: number;
  readonly onSelect: (range: WordRange) => void;
  readonly onSeek: (frame: number) => void;
  readonly onApply: (command: EditCommandJson) => void;
}) {
  const words = useMemo(() => programWords(plan, transcript), [plan, transcript]);
  const fillers = useMemo(
    () => words.map((word, at) => (isFiller(word.text) ? at : -1)).filter((at) => at >= 0),
    [words],
  );
  const pauses = useMemo(() => longPauses(words), [words]);
  const [excludedFillers, setExcludedFillers] = useState<readonly number[]>([]);
  const [excludedPauses, setExcludedPauses] = useState<readonly number[]>([]);
  const [find, setFind] = useState('');
  const [replacement, setReplacement] = useState('');
  const [showFind, setShowFind] = useState(false);
  const findInput = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (findSignal > 0) {
      setShowFind(true);
      requestAnimationFrame(() => findInput.current?.focus());
    }
  }, [findSignal]);
  const [review, setReview] = useState<'fillers' | 'pauses' | null>(null);
  const currentTick = (frame * plan.rateDen * TICKS) / plan.rateNum;
  const active = words.findIndex(
    (word) => currentTick >= word.startTicks && currentTick < word.endTicks,
  );
  const chosenFillers = fillers.filter((at) => !excludedFillers.includes(at));
  const chosenPauses = pauses.filter((pause) => !excludedPauses.includes(pause.after));
  const first = plan.segments[0];
  const last = plan.segments.at(-1);
  const preceding =
    first && transcript?.sentences.findLast((item) => item.startTicks < first.inTicks);
  const following = last && transcript?.sentences.find((item) => item.endTicks > last.outTicks);

  return (
    <aside className="editor-transcript" aria-label="Transcript editor">
      <div className="editor-transcript-heading">
        <div>
          <span className="editor-eyebrow">Source words</span>
          <h2>Transcript</h2>
        </div>
        <Button
          size="icon-sm"
          variant="ghost"
          aria-label="Find and replace in transcript"
          title="Find and replace (⌘F)"
          onClick={() => setShowFind((value) => !value)}
        >
          <Search className="size-4" />
        </Button>
      </div>
      {showFind && (
        <form
          className="editor-find"
          onSubmit={(event) => {
            event.preventDefault();
            const command = replaceCaptionWords(plan, find, replacement);
            if (command && !busy) onApply(command);
          }}
        >
          <Input
            ref={findInput}
            aria-label="Find caption word"
            placeholder="Find a word"
            value={find}
            onChange={(event) => setFind(event.target.value)}
          />
          <Input
            aria-label="Replace with"
            placeholder="Replace with"
            value={replacement}
            onChange={(event) => setReplacement(event.target.value)}
          />
          <Button
            size="sm"
            type="submit"
            disabled={busy || !replaceCaptionWords(plan, find, replacement)}
          >
            Replace all
          </Button>
          <p>Corrects every matching caption and subtitle word.</p>
        </form>
      )}
      <div className="editor-transcript-actions">
        <button
          type="button"
          disabled={busy || fillers.length === 0}
          onClick={() => setReview(review === 'fillers' ? null : 'fillers')}
        >
          <strong>{fillers.length}</strong> fillers <span>Review & cut</span>
        </button>
        <button
          type="button"
          disabled={busy || pauses.length === 0}
          onClick={() => setReview(review === 'pauses' ? null : 'pauses')}
        >
          <strong>{pauses.length}</strong> pauses over 0.8 s <span>Shorten</span>
        </button>
      </div>
      {review === 'fillers' && (
        <div className="editor-transcript-review" aria-label="Review filler words">
          <p>These cuts remove the words from video and audio. Untick any you want to keep.</p>
          {fillers.map((at) => (
            <label key={at}>
              <input
                type="checkbox"
                checked={!excludedFillers.includes(at)}
                onChange={() =>
                  setExcludedFillers((old) =>
                    old.includes(at) ? old.filter((item) => item !== at) : [...old, at],
                  )
                }
              />{' '}
              {words[at]!.text} <span>{(words[at]!.startTicks / TICKS).toFixed(1)}s</span>
            </label>
          ))}
          <Button
            size="sm"
            disabled={busy || chosenFillers.length === 0}
            onClick={() => {
              const command = cutWords(plan, words, chosenFillers);
              if (command) onApply(command);
              setReview(null);
            }}
          >
            <Scissors className="size-3.5" /> Cut {chosenFillers.length} words
          </Button>
        </div>
      )}
      {review === 'pauses' && (
        <div className="editor-transcript-review" aria-label="Review long pauses">
          <p>Keep a quarter-second breath in each selected pause.</p>
          {pauses.map((pause) => (
            <label key={pause.after}>
              <input
                type="checkbox"
                checked={!excludedPauses.includes(pause.after)}
                onChange={() =>
                  setExcludedPauses((old) =>
                    old.includes(pause.after)
                      ? old.filter((item) => item !== pause.after)
                      : [...old, pause.after],
                  )
                }
              />{' '}
              {pause.seconds.toFixed(1)} s after “{words[pause.after]!.text}”
            </label>
          ))}
          <Button
            size="sm"
            disabled={busy || chosenPauses.length === 0}
            onClick={() => {
              const command = cutPauses(plan, chosenPauses);
              if (command) onApply(command);
              setReview(null);
            }}
          >
            <Scissors className="size-3.5" /> Shorten {chosenPauses.length} pauses
          </Button>
        </div>
      )}
      {first && preceding && transcript && (
        <div className="editor-source-context">
          <span>Before this clip</span>
          <p>
            {transcript.words
              .slice(preceding.firstWord, preceding.firstWord + preceding.wordCount)
              .map((word) => word.text)
              .join(' ')}
          </p>
          <Button
            size="sm"
            variant="outline"
            disabled={busy}
            onClick={() =>
              onApply(extendWithCaptions(first.segmentId, preceding.startTicks, first.outTicks))
            }
          >
            Include before with captions
          </Button>
        </div>
      )}
      {!transcript ? (
        <p className="editor-transcript-empty">
          This analysis has no transcript. Analyze the recording again to edit by word.
        </p>
      ) : words.length === 0 ? (
        <p className="editor-transcript-empty">No spoken words remain in this edit.</p>
      ) : (
        <div className="editor-word-stream">
          {words.map((word, at) => (
            <button
              key={`${word.segmentId}-${word.sourceIndex}`}
              type="button"
              className="editor-word"
              data-live={at === active}
              data-selected={selected !== null && at >= selected.first && at <= selected.last}
              data-filler={isFiller(word.text)}
              aria-pressed={selected !== null && at >= selected.first && at <= selected.last}
              onClick={(event) => {
                onSeek(frameAt(plan, word.startTicks / TICKS));
                if (event.shiftKey && selected)
                  onSelect({
                    first: Math.min(selected.first, at),
                    last: Math.max(selected.last, at),
                  });
                else onSelect({ first: at, last: at });
              }}
              title={`${(word.startTicks / TICKS).toFixed(2)} s · Shift-click to select a range`}
            >
              {word.text}
            </button>
          ))}
        </div>
      )}
      {last && following && transcript && (
        <div className="editor-source-context">
          <span>After this clip</span>
          <p>
            {transcript.words
              .slice(following.firstWord, following.firstWord + following.wordCount)
              .map((word) => word.text)
              .join(' ')}
          </p>
          <Button
            size="sm"
            variant="outline"
            disabled={busy}
            onClick={() =>
              onApply(extendWithCaptions(last.segmentId, last.inTicks, following.endTicks))
            }
          >
            Include after with captions
          </Button>
        </div>
      )}
      {selected && words[selected.first] && (
        <div className="editor-transcript-selection">
          <span>
            {selected.first === selected.last
              ? '1 word'
              : `${selected.last - selected.first + 1} words`}{' '}
            selected
          </span>
          <Button
            size="sm"
            variant="outline"
            disabled={
              busy ||
              !cutWords(
                plan,
                words,
                Array.from(
                  { length: selected.last - selected.first + 1 },
                  (_, index) => selected.first + index,
                ),
              )
            }
            onClick={() => {
              const command = cutWords(
                plan,
                words,
                Array.from(
                  { length: selected.last - selected.first + 1 },
                  (_, index) => selected.first + index,
                ),
              );
              if (command) onApply(command);
            }}
          >
            <Scissors className="size-3.5" /> Cut from video
          </Button>
          <span className="editor-transcript-hint">
            <Check className="size-3" /> Undo with ⌘Z
          </span>
        </div>
      )}
    </aside>
  );
}
