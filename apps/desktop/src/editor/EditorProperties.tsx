import { useState } from 'react';
import type { EditIr } from '@clipmill/contracts';
import { Scan, Volume2 } from 'lucide-react';

import { Button } from '../components/ui/button.js';
import { Input } from '../components/ui/input.js';
import type { EditCommandJson, PreviewPlan } from '../daemon/client.js';
import type { Transcript } from '../results/transcript.js';
import { Audio } from './Audio.js';
import { Captions } from './Captions.js';
import { Reframe } from './Reframe.js';
import {
  batch,
  removeCaptionWord,
  setCaptionOptions,
  setCaptionStyle,
  setCueLines,
  setCueRegion,
  setCueTiming,
  setCropKeyframe,
  setLayout,
  swapPortraits,
  trimStartAt,
  trimEndAt,
  snapToWord,
} from './commands.js';
import { cropAt, cueLines, segmentAt, sourceOf } from './player.js';
import { type ProgramWord, programWords, cutWords, TICKS } from './transcript.js';
import type { WordRange } from './EditorTranscript.js';

export type EditorSelection =
  | { readonly kind: 'clip' }
  | { readonly kind: 'cue'; readonly cueId: string }
  | { readonly kind: 'section'; readonly segmentId: string }
  | {
      readonly kind: 'keyframe';
      readonly segmentId: string;
      readonly tTicks: number;
      readonly secondary: boolean;
    }
  | { readonly kind: 'gain'; readonly frame: number; readonly tTicks: number }
  | { readonly kind: 'words'; readonly range: WordRange };

const PRESETS = [
  { label: 'Clean', ref: 'clipmill.captions.clean.v1' },
  { label: 'Minimal', ref: 'clipmill.captions.minimal.v1' },
  { label: 'Boxed', ref: 'clipmill.captions.boxed.v1' },
] as const;

function wordCounts(total: number, limit: number): number[] {
  const counts: number[] = [];
  for (let left = total; left > 0; left -= limit) counts.push(Math.min(left, limit));
  return counts;
}

function captionWordsOnly(
  plan: PreviewPlan,
  doc: EditIr | null,
  words: readonly ProgramWord[],
  range: WordRange,
): EditCommandJson | null {
  if (!doc) return null;
  const source =
    plan.presentation === 'burn_in' && doc.captions.burn_in?.length
      ? doc.captions.burn_in
      : doc.captions.cues;
  if (!source) return null;
  const selected = words.slice(range.first, range.last + 1);
  const commands: EditCommandJson[] = [];
  for (const cue of source) {
    const flat = cue.lines.flatMap((line) => line.words);
    for (let at = flat.length - 1; at >= 0; at -= 1) {
      const word = flat[at]!;
      const middle = (word.start_ticks + word.end_ticks) / 2;
      if (
        selected.some((candidate) => middle >= candidate.startTicks && middle < candidate.endTicks)
      ) {
        commands.push(removeCaptionWord(cue.cue_id, at, plan.presentation));
      }
    }
  }
  return commands.length ? batch(commands) : null;
}

export function EditorProperties({
  plan,
  document,
  transcript,
  frame,
  selection,
  busy,
  resolving,
  resolveRefusal,
  onApply,
  onResolve,
}: {
  readonly plan: PreviewPlan;
  readonly document: EditIr | null;
  readonly transcript: Transcript | null;
  readonly frame: number;
  readonly selection: EditorSelection;
  readonly busy: boolean;
  readonly resolving: boolean;
  readonly resolveRefusal: string | null;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onResolve: () => void;
}) {
  const cue =
    selection.kind === 'cue' ? plan.cues.find((item) => item.cueId === selection.cueId) : null;
  const part =
    selection.kind === 'section' || selection.kind === 'keyframe'
      ? plan.segments.find((item) => item.segmentId === selection.segmentId)
      : segmentAt(plan, frame);
  const savedPart = document?.video.segments?.find((item) => item.segment_id === part?.segmentId);
  const savedCue =
    cue &&
    (plan.presentation === 'burn_in' && document?.captions.burn_in?.length
      ? document.captions.burn_in
      : document?.captions.cues
    )?.find((item) => item.cue_id === cue.cueId);
  const words = programWords(plan, transcript);
  const range = selection.kind === 'words' ? selection.range : null;
  const chosenWords = range
    ? Array.from({ length: range.last - range.first + 1 }, (_, index) => range.first + index)
    : [];
  const cut = range ? cutWords(plan, words, chosenWords) : null;
  const hide = range ? captionWordsOnly(plan, document, words, range) : null;
  const [wordsPerLine, setWordsPerLine] = useState(4);
  const styleRef = document?.captions.style_ref ?? plan.captionStyle?.styleRef ?? PRESETS[0].ref;
  const options = document?.captions.options ?? {};
  const updateOptions = (change: Partial<typeof options>) =>
    onApply(setCaptionOptions({ ...options, ...change }));
  const applyLines = (all: boolean) => {
    const cues = all ? plan.cues : cue ? [cue] : [];
    const commands = cues.map((item) =>
      setCueLines(
        item.cueId,
        wordCounts(item.lines.flat().length, wordsPerLine),
        plan.presentation,
      ),
    );
    if (commands.length) onApply(batch(commands));
  };

  return (
    <aside className="editor-properties" aria-label="Properties">
      <div className="editor-properties-heading">
        <span className="editor-eyebrow">Selection</span>
        <h2>
          {selection.kind === 'clip'
            ? 'Clip'
            : selection.kind === 'cue'
              ? 'Caption'
              : selection.kind === 'section'
                ? 'Section'
                : selection.kind === 'keyframe'
                  ? 'Framing keyframe'
                  : selection.kind === 'gain'
                    ? 'Audio point'
                    : 'Words'}
        </h2>
      </div>
      <div className="editor-properties-scroll">
        {selection.kind === 'words' && (
          <section className="editor-prop-section">
            <p>
              {chosenWords.length} spoken {chosenWords.length === 1 ? 'word' : 'words'} selected.
            </p>
            <Button
              size="sm"
              disabled={busy || !cut}
              onClick={() => {
                if (cut) onApply(cut);
              }}
            >
              Cut from video
            </Button>
            <Button
              size="sm"
              variant="outline"
              disabled={busy || !hide}
              onClick={() => {
                if (hide) onApply(hide);
              }}
            >
              Hide in captions only
            </Button>
          </section>
        )}

        {selection.kind === 'cue' && cue && (
          <>
            <section className="editor-prop-section">
              <span className="editor-eyebrow">Text</span>
              <p className="editor-selected-copy">{cueLines(cue).join(' ')}</p>
            </section>
            <section className="editor-prop-section">
              <span className="editor-eyebrow">Position</span>
              <div className="editor-prop-segmented">
                {(['upper_safe', 'center', 'lower_safe'] as const).map((region) => (
                  <button
                    type="button"
                    key={region}
                    aria-pressed={cue.region === region}
                    disabled={busy}
                    onClick={() => onApply(setCueRegion(cue.cueId, region, plan.presentation))}
                  >
                    {region === 'upper_safe' ? 'Top' : region === 'center' ? 'Middle' : 'Bottom'}
                  </button>
                ))}
              </div>
            </section>
          </>
        )}

        {(selection.kind === 'clip' || selection.kind === 'cue') && (
          <>
            <section className="editor-prop-section">
              <span className="editor-eyebrow">Caption look</span>
              <div className="editor-preset-grid">
                {PRESETS.map((preset) => (
                  <button
                    type="button"
                    key={preset.ref}
                    aria-pressed={styleRef.includes(`.${preset.label.toLowerCase()}.`)}
                    disabled={busy}
                    onClick={() => onApply(setCaptionStyle(preset.ref))}
                  >
                    <span>Aa</span>
                    {preset.label}
                  </button>
                ))}
              </div>
            </section>
            {selection.kind === 'cue' && cue && savedCue && (
              <CaptionTiming
                key={`${cue.cueId}:${plan.revision}`}
                cue={savedCue}
                plan={plan}
                busy={busy}
                onApply={onApply}
              />
            )}
            <section className="editor-prop-section editor-appearance">
              <span className="editor-eyebrow">Type & colour</span>
              <label>
                Size{' '}
                <Input
                  type="number"
                  min={24}
                  max={160}
                  value={options.font_size ?? plan.captionStyle?.fontSize ?? 84}
                  disabled={busy}
                  onChange={(event) => {
                    const value = Number(event.target.value);
                    if (value >= 24 && value <= 160) updateOptions({ font_size: value });
                  }}
                />
              </label>
              <label>
                Highlight{' '}
                <input
                  type="color"
                  value={options.spoken ?? plan.captionStyle?.spoken ?? '#ffd65c'}
                  disabled={busy}
                  onChange={(event) => updateOptions({ spoken: event.target.value })}
                />
              </label>
              <label>
                Waiting{' '}
                <input
                  type="color"
                  value={options.unspoken ?? plan.captionStyle?.unspoken ?? '#ffffff'}
                  disabled={busy}
                  onChange={(event) => updateOptions({ unspoken: event.target.value })}
                />
              </label>
              <label>
                Outline{' '}
                <input
                  type="color"
                  value={options.outline ?? plan.captionStyle?.outline ?? '#000000'}
                  disabled={busy}
                  onChange={(event) => updateOptions({ outline: event.target.value })}
                />
              </label>
              <label>
                Case{' '}
                <select
                  value={options.text_case ?? 'original'}
                  disabled={busy}
                  onChange={(event) =>
                    updateOptions({
                      text_case: event.target.value as 'original' | 'upper' | 'lower',
                    })
                  }
                >
                  <option value="original">As spoken</option>
                  <option value="upper">UPPERCASE</option>
                  <option value="lower">lowercase</option>
                </select>
              </label>
              <label>
                Words per line{' '}
                <Input
                  type="number"
                  min={1}
                  max={8}
                  value={wordsPerLine}
                  onChange={(event) =>
                    setWordsPerLine(Math.max(1, Math.min(8, Number(event.target.value))))
                  }
                />
              </label>
              <div className="editor-prop-actions">
                <Button
                  size="sm"
                  variant="outline"
                  disabled={busy || !cue}
                  onClick={() => applyLines(false)}
                >
                  Apply to caption
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  disabled={busy || plan.cues.length === 0}
                  onClick={() => applyLines(true)}
                >
                  Apply to all
                </Button>
              </div>
            </section>
          </>
        )}

        {selection.kind === 'cue' && cue && (
          <details className="editor-prop-details">
            <summary>Words & line breaks</summary>
            <Captions plan={plan} frame={frame} busy={busy} cueId={cue.cueId} onApply={onApply} />
          </details>
        )}

        {(selection.kind === 'section' || selection.kind === 'keyframe') && part && (
          <>
            <section className="editor-prop-section">
              <span className="editor-eyebrow">Layout</span>
              <div className="editor-prop-segmented">
                {(
                  [
                    ['speaker_fill', 'Fill'],
                    ['fit', 'Fit'],
                    ['two_up', 'Two-up'],
                  ] as const
                ).map(([mode, label]) => (
                  <button
                    key={mode}
                    type="button"
                    aria-pressed={savedPart?.layout.state === mode}
                    disabled={busy || (mode === 'two_up' && !part.hasTwoUpPaths)}
                    onClick={() => onApply(setLayout(mode, part.segmentId))}
                  >
                    {label}
                  </button>
                ))}
              </div>
              {savedPart?.layout.state === 'two_up' && (
                <Button
                  size="sm"
                  variant="outline"
                  disabled={busy}
                  onClick={() => onApply(swapPortraits(part.segmentId))}
                >
                  Switch speakers
                </Button>
              )}
            </section>
            {selection.kind === 'keyframe' && (
              <KeyframeControls
                plan={plan}
                document={document}
                frame={frame}
                selection={selection}
                busy={busy}
                onApply={onApply}
              />
            )}
            <details className="editor-prop-details">
              <summary>
                <Scan className="size-3.5" /> Framing tools
              </summary>
              <Reframe
                plan={plan}
                frame={frame}
                busy={busy}
                onApply={onApply}
                onResolve={onResolve}
                resolving={resolving}
                resolveRefusal={resolveRefusal}
              />
            </details>
          </>
        )}

        {selection.kind === 'gain' && (
          <Audio plan={plan} frame={frame} busy={busy} onApply={onApply} />
        )}
        {selection.kind === 'clip' && (
          <>
            <section className="editor-prop-section">
              <span className="editor-eyebrow">Clip edges</span>
              <div className="editor-prop-actions">
                <Button
                  size="sm"
                  variant="outline"
                  disabled={busy || !trimStartAt(plan, snapToWord(plan, frame))}
                  onClick={() => {
                    const command = trimStartAt(plan, snapToWord(plan, frame));
                    if (command) onApply(command);
                  }}
                >
                  Trim start here
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  disabled={busy || !trimEndAt(plan, snapToWord(plan, frame))}
                  onClick={() => {
                    const command = trimEndAt(plan, snapToWord(plan, frame));
                    if (command) onApply(command);
                  }}
                >
                  Trim end here
                </Button>
              </div>
            </section>
            <section className="editor-prop-section">
              <span className="editor-eyebrow">Clip details</span>
              <p>
                {plan.segments.length} {plan.segments.length === 1 ? 'section' : 'sections'} ·{' '}
                {plan.cues.length} captions ·{' '}
                {((plan.frameCount * plan.rateDen) / plan.rateNum).toFixed(1)} s
              </p>
            </section>
            <details className="editor-prop-details">
              <summary>
                <Scan className="size-3.5" /> Reframe
              </summary>
              <Reframe
                plan={plan}
                frame={frame}
                busy={busy}
                onApply={onApply}
                onResolve={onResolve}
                resolving={resolving}
                resolveRefusal={resolveRefusal}
              />
            </details>
            <details className="editor-prop-details">
              <summary>
                <Volume2 className="size-3.5" /> Audio
              </summary>
              <Audio plan={plan} frame={frame} busy={busy} onApply={onApply} />
            </details>
          </>
        )}
      </div>
    </aside>
  );
}

function CaptionTiming({
  cue,
  plan,
  busy,
  onApply,
}: {
  readonly cue: NonNullable<EditIr['captions']['cues']>[number];
  readonly plan: PreviewPlan;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
}) {
  const [start, setStart] = useState((cue.start_ticks / TICKS).toFixed(2));
  const [end, setEnd] = useState((cue.end_ticks / TICKS).toFixed(2));
  const from = Math.round(Number(start) * TICKS);
  const to = Math.round(Number(end) * TICKS);
  const valid = Number.isSafeInteger(from) && Number.isSafeInteger(to) && from >= 0 && to > from;
  return (
    <form
      className="editor-prop-section editor-timing"
      onSubmit={(event) => {
        event.preventDefault();
        if (valid && !busy) onApply(setCueTiming(cue.cue_id, from, to, plan.presentation));
      }}
    >
      <span className="editor-eyebrow">Display timing</span>
      <div>
        <label>
          Start{' '}
          <Input
            type="number"
            step="0.01"
            min="0"
            value={start}
            onChange={(event) => setStart(event.target.value)}
          />
        </label>
        <label>
          End{' '}
          <Input
            type="number"
            step="0.01"
            min="0"
            value={end}
            onChange={(event) => setEnd(event.target.value)}
          />
        </label>
      </div>
      <Button
        type="submit"
        size="sm"
        variant="outline"
        disabled={busy || !valid || (from === cue.start_ticks && to === cue.end_ticks)}
      >
        Save timing
      </Button>
    </form>
  );
}

function KeyframeControls({
  plan,
  document,
  frame,
  selection,
  busy,
  onApply,
}: {
  readonly plan: PreviewPlan;
  readonly document: EditIr | null;
  readonly frame: number;
  readonly selection: Extract<EditorSelection, { kind: 'keyframe' }>;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
}) {
  const segment = segmentAt(plan, frame);
  const source = segment ? sourceOf(plan, segment) : null;
  const crop = cropAt(plan, frame, selection.secondary);
  const keyframe = document?.video.segments
    ?.find((item) => item.segment_id === selection.segmentId)
    ?.layout[selection.secondary ? 'secondary_crop_path' : 'crop_path']?.find(
      (item) => item.t_ticks === selection.tTicks,
    );
  if (!source || !crop)
    return <p className="editor-help">Choose Fill or Two-up to place a framing keyframe.</p>;
  const move = (change: Partial<typeof crop>) =>
    onApply(
      setCropKeyframe(
        selection.tTicks,
        { ...crop, ...change },
        selection.segmentId,
        selection.secondary,
      ),
    );
  return (
    <section className="editor-prop-section editor-keyframe-controls">
      <span className="editor-eyebrow">Framing at this point</span>
      <label>
        Easing{' '}
        <select
          aria-label="Framing easing"
          value={keyframe?.easing ?? 'linear'}
          disabled={busy || !keyframe}
          onChange={(event) =>
            onApply(
              setCropKeyframe(
                selection.tTicks,
                keyframe!.rect,
                selection.segmentId,
                selection.secondary,
                event.target.value as 'linear' | 'ease_in' | 'ease_out' | 'ease_in_out',
              ),
            )
          }
        >
          <option value="linear">Linear</option>
          <option value="ease_in">Ease in</option>
          <option value="ease_out">Ease out</option>
          <option value="ease_in_out">Ease in & out</option>
        </select>
      </label>
      <label>
        Horizontal{' '}
        <input
          type="range"
          min={0}
          max={Math.max(0, source.displayWidth - crop.width)}
          value={crop.x}
          disabled={busy}
          onChange={(event) => move({ x: Number(event.target.value) })}
        />
      </label>
      <label>
        Vertical{' '}
        <input
          type="range"
          min={0}
          max={Math.max(0, source.displayHeight - crop.height)}
          value={crop.y}
          disabled={busy}
          onChange={(event) => move({ y: Number(event.target.value) })}
        />
      </label>
      <label>
        Zoom{' '}
        <input
          type="range"
          min={20}
          max={100}
          value={Math.round((crop.height / source.displayHeight) * 100)}
          disabled={busy}
          onChange={(event) => {
            const height = Math.max(
              2,
              Math.round((source.displayHeight * Number(event.target.value)) / 200) * 2,
            );
            const width = Math.max(2, Math.round((height * plan.width) / plan.height / 2) * 2);
            move({
              width,
              height,
              x: Math.max(
                0,
                Math.min(
                  source.displayWidth - width,
                  crop.x + Math.round((crop.width - width) / 2),
                ),
              ),
              y: Math.max(
                0,
                Math.min(
                  source.displayHeight - height,
                  crop.y + Math.round((crop.height - height) / 2),
                ),
              ),
            });
          }}
        />
      </label>
    </section>
  );
}
