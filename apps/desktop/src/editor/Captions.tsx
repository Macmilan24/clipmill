/**
 * 09 Captions: the words, and the five things a person actually wants to do to
 * them.
 *
 * Correct a word, split, merge, re-break a line, drop a filler. Each is one IR
 * command, so each is undoable and each is replayable — and none of them
 * re-derives anything. The cue a person is editing came from the caption
 * engine and the render will read the same list back; nothing here re-wraps
 * text or re-times a word.
 *
 * A correction is addressed to the word, not to the cue it is shown in. The
 * cues on screen are one of two groupings of the same words — the burned-in
 * one — and the sidecars are written from the other; a correction that reached
 * only the cue on screen left a deaf viewer reading the wrong name. The other
 * four are about the grouping, and are addressed to the grouping on screen.
 *
 * Removing a filler is a **caption** edit and not a media edit. The word was
 * said; the caption may stop showing it. Rippling the audio to match would be a
 * different and much larger decision, and one nobody asked this button to make.
 *
 * Re-transcribing a selection is Phase 2 and is marked as such rather than
 * shipped as a button that quietly does nothing.
 */
import { Check, Scissors, Trash2 } from 'lucide-react';
import { useEffect, useState } from 'react';

import { Badge } from '../components/ui/badge.js';
import { Button } from '../components/ui/button.js';
import { Input } from '../components/ui/input.js';
import { ScrollArea } from '../components/ui/scroll-area.js';
import type { EditCommandJson, PreviewCue, PreviewPlan } from '../daemon/client.js';
import { correctWord, mergeCues, setCueLines, splitCue } from './commands.js';

/**
 * Words a caption may stop showing without anybody having to argue about it.
 * The same list the caption engine tags with, kept short for the same reason.
 */
const FILLERS = new Set(['ah', 'eh', 'er', 'erm', 'hmm', 'huh', 'mhm', 'uh', 'uhm', 'um', 'umm']);

export interface CaptionsProps {
  readonly plan: PreviewPlan;
  readonly frame: number;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
}

interface Selection {
  readonly cueId: string;
  readonly wordIndex: number;
}

export function Captions({ plan, frame, busy, onApply }: CaptionsProps) {
  const [selected, setSelected] = useState<Selection | null>(null);
  const cue = selected
    ? (plan.cues.find((candidate) => candidate.cueId === selected.cueId) ?? null)
    : null;

  return (
    <div className="flex h-full flex-col gap-3 p-4 text-sm">
      <ScrollArea className="h-[320px] pr-2">
        <ul className="flex flex-col gap-2">
          {plan.cues.map((candidate, position) => (
            <li key={candidate.cueId}>
              <Phrase
                cue={candidate}
                live={frame >= candidate.firstFrame && frame < candidate.endFrame}
                selected={selected?.cueId === candidate.cueId ? selected.wordIndex : -1}
                onSelectWord={(wordIndex) => setSelected({ cueId: candidate.cueId, wordIndex })}
                onMergeWithNext={
                  position + 1 < plan.cues.length
                    ? () =>
                        onApply(
                          mergeCues(
                            candidate.cueId,
                            plan.cues[position + 1]!.cueId,
                            plan.presentation,
                          ),
                        )
                    : null
                }
                busy={busy}
              />
            </li>
          ))}
        </ul>
      </ScrollArea>

      {cue && selected ? (
        <WordActions
          plan={plan}
          cue={cue}
          wordIndex={selected.wordIndex}
          busy={busy}
          onApply={onApply}
          onDone={() => setSelected(null)}
        />
      ) : (
        <p className="text-xs text-[var(--cm-ink-2)]">
          Select a word to correct it, split the cue there, re-break its lines, or drop it.
        </p>
      )}

      <p className="mt-auto text-xs text-[var(--cm-ink-3)]">
        <Badge variant="outline">Phase 2</Badge> Re-transcribing a selection with a stronger model
        is not built. Corrections made here are an overlay, so when it is built it will propose
        without erasing them.
      </p>
    </div>
  );
}

function Phrase({
  cue,
  live,
  selected,
  onSelectWord,
  onMergeWithNext,
  busy,
}: {
  readonly cue: PreviewCue;
  readonly live: boolean;
  readonly selected: number;
  readonly onSelectWord: (wordIndex: number) => void;
  readonly onMergeWithNext: (() => void) | null;
  readonly busy: boolean;
}) {
  let index = 0;
  return (
    <div
      className={`rounded-lg border p-2 ${
        live
          ? 'border-[var(--cm-accent)] bg-[var(--cm-surface-2)]'
          : 'border-[var(--cm-line-1)] bg-[var(--cm-surface-1)]'
      }`}
    >
      <div className="mb-1 flex items-center justify-between">
        <span className="font-mono text-[10px] text-[var(--cm-ink-3)]">{cue.cueId}</span>
        {onMergeWithNext && (
          <Button size="sm" variant="ghost" disabled={busy} onClick={onMergeWithNext}>
            Merge with next
          </Button>
        )}
      </div>
      {cue.lines.map((line, lineIndex) => (
        // eslint-disable-next-line react/no-array-index-key -- a line's position is its identity
        <p key={lineIndex} className="leading-relaxed">
          {line.map((word) => {
            const mine = index;
            index += 1;
            const filler = FILLERS.has(word.text.toLowerCase().replaceAll(/[^a-z']/g, ''));
            return (
              <button
                key={`${word.text}-${mine}`}
                type="button"
                onClick={() => onSelectWord(mine)}
                className={`rounded px-0.5 ${
                  mine === selected
                    ? 'bg-[var(--cm-accent)] text-white'
                    : filler
                      ? 'text-[var(--cm-ink-3)] italic'
                      : 'text-[var(--cm-ink-1)]'
                }`}
                title={filler ? 'tagged as a filler' : undefined}
              >
                {word.text}
              </button>
            );
          })}
        </p>
      ))}
    </div>
  );
}

function WordActions({
  plan,
  cue,
  wordIndex,
  busy,
  onApply,
  onDone,
}: {
  readonly plan: PreviewPlan;
  readonly cue: PreviewCue;
  readonly wordIndex: number;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onDone: () => void;
}) {
  const words = cue.lines.flat();
  const total = words.length;
  const word = words[wordIndex];
  const [draft, setDraft] = useState(word?.text ?? '');
  // A different word is a different draft.
  useEffect(() => {
    setDraft(word?.text ?? '');
  }, [word?.text, cue.cueId, wordIndex]);
  const corrected = draft.trim();
  const changed = word !== undefined && corrected !== '' && corrected !== word.text;

  const correct = (text: string) => {
    if (!word) {
      return;
    }
    onApply(correctWord(plan, cue, wordIndex, word, text));
    onDone();
  };

  return (
    <div className="flex flex-col gap-2 rounded-lg border border-[var(--cm-line-1)] bg-[var(--cm-surface-1)] p-2">
      <form
        className="flex items-center gap-2"
        onSubmit={(event) => {
          event.preventDefault();
          if (changed) {
            correct(corrected);
          }
        }}
      >
        <Input
          aria-label="Correct this word"
          value={draft}
          disabled={busy || !word}
          onChange={(event) => setDraft(event.target.value)}
          className="h-8 font-mono text-xs"
        />
        <Button type="submit" size="sm" disabled={busy || !changed}>
          <Check className="size-3" /> Correct
        </Button>
      </form>
      <p className="text-[11px] text-[var(--cm-ink-3)]">
        “{word?.text ?? ''}” in {cue.cueId}. A correction lands in the burned-in caption and in the
        sidecars alike; the timing is untouched.
      </p>
      <div className="flex flex-wrap items-center gap-2">
        <Button
          size="sm"
          variant="outline"
          disabled={busy || wordIndex === 0 || wordIndex >= total}
          onClick={() => {
            // A new cue needs a name replay can reproduce, so it is derived from
            // the cue it came out of rather than generated.
            onApply(splitCue(cue.cueId, wordIndex, `${cue.cueId}_b`, plan.presentation));
            onDone();
          }}
        >
          <Scissors className="size-3" /> Split here
        </Button>
        <Button
          size="sm"
          variant="outline"
          disabled={busy || wordIndex === 0 || wordIndex >= total}
          onClick={() => {
            onApply(setCueLines(cue.cueId, [wordIndex, total - wordIndex], plan.presentation));
            onDone();
          }}
        >
          Break line here
        </Button>
        <Button
          size="sm"
          variant="ghost"
          disabled={busy || !word}
          onClick={() => {
            // The word was said; the caption stops showing it. Emptying the
            // text is a caption edit — rippling the media to match would be a
            // different decision and a much larger one.
            correct('·');
          }}
          title="Replace this word in the caption. The audio is untouched."
        >
          <Trash2 className="size-3" /> Drop from caption
        </Button>
      </div>
    </div>
  );
}
