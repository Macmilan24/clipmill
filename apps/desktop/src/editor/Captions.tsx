/**
 * Undoable caption commands: correct, split, merge, re-break, and remove fillers.
 * Word corrections apply across burned-in and sidecar groupings; grouping edits
 * target the displayed cues. Filler removal changes captions only, leaving media
 * unchanged. Caption timing and wrapping remain owned by the caption engine.
 */
import { Check, Scissors, Trash2, X } from 'lucide-react';
import { useEffect, useState } from 'react';

import { Button } from '../components/ui/button.js';
import { Input } from '../components/ui/input.js';
import type { EditCommandJson, PreviewCue, PreviewPlan } from '../daemon/client.js';
import { timecode } from './player.js';
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
  readonly cueId?: string;
  readonly onApply: (command: EditCommandJson) => void;
}

interface Selection {
  readonly cueId: string;
  readonly wordIndex: number;
}

export function Captions({ plan, frame, busy, cueId, onApply }: CaptionsProps) {
  const [selected, setSelected] = useState<Selection | null>(null);
  const visible = cueId ? plan.cues.filter((candidate) => candidate.cueId === cueId) : plan.cues;
  const cue = selected
    ? (plan.cues.find((candidate) => candidate.cueId === selected.cueId) ?? null)
    : null;

  return (
    <div className="editor-panel editor-captions-panel">
      <div className="editor-panel-heading">
        <h2 className="editor-panel-title">{cueId ? 'Caption words' : 'Transcript'}</h2>
        <span>
          {cueId ? timecode(plan, visible[0]?.firstFrame ?? frame) : `${plan.cues.length} captions`}
        </span>
      </div>
      <p className="editor-help">Select a word to edit.</p>
      {visible.length === 0 && (
        <p className="py-4 text-xs text-[var(--cm-text-muted)]">No captions in this clip.</p>
      )}
      <div className="editor-caption-list">
        <ul className="flex flex-col gap-2">
          {visible.map((candidate) => (
            <li key={candidate.cueId}>
              <Phrase
                cue={candidate}
                label={timecode(plan, candidate.firstFrame)}
                live={frame >= candidate.firstFrame && frame < candidate.endFrame}
                selected={selected?.cueId === candidate.cueId ? selected.wordIndex : -1}
                onSelectWord={(wordIndex) => setSelected({ cueId: candidate.cueId, wordIndex })}
                onMergeWithNext={
                  plan.cues.findIndex((item) => item.cueId === candidate.cueId) + 1 <
                  plan.cues.length
                    ? () =>
                        onApply(
                          mergeCues(
                            candidate.cueId,
                            plan.cues[
                              plan.cues.findIndex((item) => item.cueId === candidate.cueId) + 1
                            ]!.cueId,
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
      </div>

      {cue && selected ? (
        <WordActions
          plan={plan}
          cue={cue}
          wordIndex={selected.wordIndex}
          busy={busy}
          onApply={onApply}
          onDone={() => setSelected(null)}
        />
      ) : null}
    </div>
  );
}

function Phrase({
  cue,
  label,
  live,
  selected,
  onSelectWord,
  onMergeWithNext,
  busy,
}: {
  readonly cue: PreviewCue;
  readonly label: string;
  readonly live: boolean;
  readonly selected: number;
  readonly onSelectWord: (wordIndex: number) => void;
  readonly onMergeWithNext: (() => void) | null;
  readonly busy: boolean;
}) {
  let index = 0;
  return (
    <div
      className={`editor-phrase group border-l-2 px-3 py-2 ${
        live ? 'border-[var(--cm-accent)] bg-[var(--cm-accent-selected)]' : 'border-transparent'
      }`}
    >
      <div className="mb-1 flex items-center justify-between">
        <span className="font-mono text-[11px] text-[var(--cm-ink-3)]">{label}</span>
        {onMergeWithNext && (
          <Button
            size="xs"
            variant="ghost"
            disabled={busy}
            onClick={onMergeWithNext}
            className="editor-merge text-[11px] text-[var(--cm-text-secondary)]"
          >
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
                aria-pressed={mine === selected}
                className={`rounded px-0.5 ${
                  mine === selected
                    ? 'bg-[var(--cm-accent)] text-[var(--cm-accent-foreground)]'
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
    <div className="editor-word-actions">
      <div className="editor-panel-heading">
        <h3 className="editor-panel-title">Edit word</h3>
        <Button variant="ghost" size="icon-xs" onClick={onDone} aria-label="Close word editor">
          <X />
        </Button>
      </div>
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
          className="h-8 text-sm"
        />
        <Button type="submit" size="sm" disabled={busy || !changed}>
          <Check className="size-3" /> Correct
        </Button>
      </form>
      <p className="text-[11px] text-[var(--cm-ink-3)]">
        Updates captions and subtitle files. Timing stays the same.
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
