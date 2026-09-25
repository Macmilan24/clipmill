/**
 * The Editor's properties, by kind: captions, framing, audio and details.
 * Selecting something on the monitor or the timeline opens its tab with its own
 * controls on top. Sliders and colours send one command when let go.
 */
import type { EditIr } from '@clipmill/contracts';
import { AudioLines, Captions as CaptionsIcon, Crop, Info, Minus, Plus, X } from 'lucide-react';
import { type ReactNode, useEffect, useRef, useState } from 'react';

import { Button } from '../components/ui/button.js';
import { Input } from '../components/ui/input.js';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../components/ui/select.js';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../components/ui/tabs.js';
import type { EditCommandJson, PreviewCue, PreviewPlan } from '../daemon/client.js';
import { clockTenths, timecode } from '../inspector/review.js';
import type { EditorFocus } from '../shell/route.js';
import { repairAll, shortCues } from './captionRepairs.js';
import { CueTiming } from './CueTiming.js';
import {
  batch,
  correctWord,
  mergeCues,
  removeCaptionWord,
  removeCropKeyframe,
  removeGainPoint,
  setCaptionOptions,
  setCaptionStyle,
  setCropKeyframe,
  setCueLines,
  setCueRegion,
  setCueTiming,
  setGain,
  setLayout,
  splitCue,
  swapPortraits,
} from './commands.js';
import { cropAt, gainAt, segmentAt, sourceOf } from './player.js';
import type { EditorSelection, PropertiesTab } from './selection.js';
import { freshCueId, programTicks, ticksOfFrame } from './timeline.js';
import { shownCues } from './transcript.js';
import { CaptionLookSample } from '../results/CaptionLookSample.js';

const PRESETS = [
  { label: 'Clean', ref: 'clipmill.captions.clean.v1' },
  { label: 'Minimal', ref: 'clipmill.captions.minimal.v1' },
  { label: 'Boxed', ref: 'clipmill.captions.boxed.v1' },
] as const;

const REGIONS = [
  ['upper_safe', 'Top'],
  ['center', 'Middle'],
  ['lower_safe', 'Bottom'],
] as const;

type Region = (typeof REGIONS)[number][0];
type Easing = 'linear' | 'ease_in' | 'ease_out' | 'ease_in_out';

export interface EditorPropertiesProps {
  readonly plan: PreviewPlan;
  readonly focus?: EditorFocus | null;
  readonly document: EditIr | null;
  readonly frame: number;
  readonly selection: EditorSelection;
  readonly tab: PropertiesTab;
  readonly onTab: (tab: PropertiesTab) => void;
  readonly busy: boolean;
  readonly resolving: boolean;
  readonly resolveRefusal: string | null;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onResolve: () => void;
  readonly onSelect: (selection: EditorSelection) => void;
  readonly onSeek: (frame: number) => void;
}

export function EditorProperties(props: EditorPropertiesProps) {
  const { tab, onTab } = props;
  const [captionTrack, setCaptionTrack] = useState<'on-screen' | 'reading'>(
    props.focus?.track ?? 'on-screen',
  );
  useEffect(() => {
    if (props.focus?.panel === 'captions') setCaptionTrack(props.focus.track);
  }, [props.focus]);
  const captionPlan: PreviewPlan =
    captionTrack === 'reading' && props.plan.readingCues
      ? { ...props.plan, cues: props.plan.readingCues, presentation: 'reading' }
      : props.plan;
  return (
    <aside className="review-side edit-side" aria-label="Properties">
      <Tabs
        value={tab}
        onValueChange={(next) => onTab(next as PropertiesTab)}
        className="review-tabs"
      >
        <TabsList className="review-tab-list">
          <TabsTrigger value="captions">
            <CaptionsIcon aria-hidden="true" />
            Captions
          </TabsTrigger>
          <TabsTrigger value="framing">
            <Crop aria-hidden="true" />
            Framing
          </TabsTrigger>
          <TabsTrigger value="audio">
            <AudioLines aria-hidden="true" />
            Audio
          </TabsTrigger>
          <TabsTrigger value="details">
            <Info aria-hidden="true" />
            Details
          </TabsTrigger>
        </TabsList>
        <TabsContent value="captions" className="review-tab-panel">
          <div
            className="review-segmented edit-wide mx-3 mt-3"
            role="group"
            aria-label="Caption track"
          >
            <button
              type="button"
              aria-pressed={captionTrack === 'on-screen'}
              onClick={() => setCaptionTrack('on-screen')}
            >
              On-screen
            </button>
            <button
              type="button"
              aria-pressed={captionTrack === 'reading'}
              disabled={!props.plan.readingCues}
              onClick={() => setCaptionTrack('reading')}
            >
              Subtitle file
            </button>
          </div>
          <CaptionsTab {...props} plan={captionPlan} />
        </TabsContent>
        <TabsContent value="framing" className="review-tab-panel">
          <FramingTab {...props} />
        </TabsContent>
        <TabsContent value="audio" className="review-tab-panel">
          <AudioTab {...props} />
        </TabsContent>
        <TabsContent value="details" className="review-tab-panel">
          <DetailsTab {...props} />
        </TabsContent>
      </Tabs>
    </aside>
  );
}

/* Captions ------------------------------------------------------------------ */

function CaptionsTab({
  plan,
  document,
  selection,
  busy,
  onApply,
  onSelect,
  onSeek,
}: EditorPropertiesProps) {
  const cue =
    selection.kind === 'cue' ? plan.cues.find((item) => item.cueId === selection.cueId) : undefined;
  const styleRef = document?.captions.style_ref ?? plan.captionStyle?.styleRef ?? PRESETS[0].ref;
  const options = document?.captions.options ?? {};
  const highlightEnabled = options.highlight_spoken_word ?? !styleRef.includes('.minimal.');
  const update = (change: Partial<typeof options>) =>
    onApply(setCaptionOptions({ ...options, ...change }));
  const chooseLook = (ref: string) =>
    onApply(
      options.highlight_spoken_word === undefined
        ? batch([
            setCaptionOptions({ ...options, highlight_spoken_word: highlightEnabled }),
            setCaptionStyle(ref),
          ])
        : setCaptionStyle(ref),
    );
  const style = plan.captionStyle;
  const regionOfAll = plan.cues.every((item) => item.region === plan.cues[0]?.region)
    ? plan.cues[0]?.region
    : null;
  const [perLine, setPerLine] = useState(4);
  const problems = plan.presentation === 'reading' ? shortCues(plan) : [];

  if (plan.cues.length === 0) {
    return <p className="review-empty-note">This clip has no captions. Nothing was said in it.</p>;
  }

  return (
    <div className="review-panel-body">
      {problems.length > 0 && (
        <section className="review-section" aria-label="Caption problems">
          <h3 className="review-section-title">Subtitle timing · {problems.length} to review</h3>
          <p className="review-footnote">
            Captions shorter than the reading guideline appear here. Only flashes under a third of a
            second block export.
          </p>
          {problems.map((problem) => (
            <div
              key={problem.cue.cueId}
              className="flex items-center justify-between gap-2 py-1 text-xs"
            >
              <button
                type="button"
                className="min-w-0 truncate text-left underline"
                onClick={() => {
                  onSelect({ kind: 'cue', cueId: problem.cue.cueId });
                  onSeek(problem.cue.firstFrame);
                }}
              >
                {problem.cue.lines
                  .flat()
                  .map((word) => word.text)
                  .join(' ')}{' '}
                · {((problem.endTicks - problem.startTicks) / 90_000).toFixed(2)}s
              </button>
              {problem.repair && (
                <Button
                  size="xs"
                  variant="outline"
                  disabled={busy}
                  onClick={() => onApply(problem.repair!.command)}
                >
                  Fix
                </Button>
              )}
            </div>
          ))}
          {problems.length > 1 && (
            <Button
              size="sm"
              variant="outline"
              disabled={busy}
              onClick={() => {
                const command = repairAll(plan);
                if (command) onApply(command);
              }}
            >
              Fix all
            </Button>
          )}
        </section>
      )}
      {cue && (
        <SelectedCaption
          key={`${cue.cueId}:${plan.revision}`}
          plan={plan}
          document={document}
          cue={cue}
          busy={busy}
          onApply={onApply}
          onSelect={onSelect}
          onSeek={onSeek}
        />
      )}

      <section className="review-section">
        <h3 className="review-section-title">Look</h3>
        <div className="review-segmented edit-wide" role="group" aria-label="Caption look">
          {PRESETS.map((preset) => (
            <button
              key={preset.ref}
              type="button"
              className="edit-look-button"
              aria-pressed={styleRef.includes(`.${preset.label.toLowerCase()}.`)}
              disabled={busy}
              onClick={() => chooseLook(preset.ref)}
            >
              <CaptionLookSample
                look={preset.label.toLowerCase() as 'clean' | 'minimal' | 'boxed'}
                highlight={highlightEnabled}
              />
              {preset.label}
            </button>
          ))}
        </div>
        <label className="mt-3 flex items-center gap-2 text-xs">
          <input
            type="checkbox"
            checked={highlightEnabled}
            disabled={busy}
            onChange={(event) => update({ highlight_spoken_word: event.target.checked })}
          />
          Highlight the spoken word
        </label>
        <Field label="Size">
          <CommitSlider
            label="Caption size"
            min={40}
            max={140}
            value={options.font_size ?? style?.fontSize ?? 84}
            disabled={busy}
            format={(value) => `${value}`}
            onCommit={(value) => update({ font_size: value })}
          />
        </Field>
        <Field label="Colours">
          <div className="edit-swatches">
            {highlightEnabled && (
              <Swatch
                label="Spoken word"
                value={options.spoken ?? style?.spoken ?? '#ffd65c'}
                disabled={busy}
                onCommit={(value) => update({ spoken: value })}
              />
            )}
            <Swatch
              label="Words"
              value={options.unspoken ?? style?.unspoken ?? '#ffffff'}
              disabled={busy}
              onCommit={(value) => update({ unspoken: value })}
            />
            <Swatch
              label="Outline"
              value={options.outline ?? style?.outline ?? '#000000'}
              disabled={busy}
              onCommit={(value) => update({ outline: value })}
            />
          </div>
        </Field>
        <Field label="Case">
          <div className="review-segmented" role="group" aria-label="Letter case">
            {(
              [
                ['original', 'As said'],
                ['upper', 'AA'],
                ['lower', 'aa'],
              ] as const
            ).map(([value, label]) => (
              <button
                key={value}
                type="button"
                aria-pressed={(options.text_case ?? 'original') === value}
                disabled={busy}
                onClick={() => update({ text_case: value })}
              >
                {label}
              </button>
            ))}
          </div>
        </Field>
      </section>

      <section className="review-section">
        <h3 className="review-section-title">All captions</h3>
        <Field label="Position">
          <RegionPicker
            label="Where every caption sits"
            value={regionOfAll ?? null}
            disabled={busy}
            onPick={(region) => {
              const moving = plan.cues.filter((item) => item.region !== region);
              if (moving.length > 0)
                onApply(
                  batch(moving.map((item) => setCueRegion(item.cueId, region, plan.presentation))),
                );
            }}
          />
        </Field>
        <Field label="Words per line">
          <div className="edit-inline">
            <Stepper label="Words per line" value={perLine} min={1} max={8} onChange={setPerLine} />
            <Button
              size="sm"
              variant="outline"
              disabled={busy}
              onClick={() =>
                onApply(
                  batch(
                    plan.cues.map((item) =>
                      setCueLines(
                        item.cueId,
                        wordCounts(item.lines.flat().length, perLine),
                        plan.presentation,
                      ),
                    ),
                  ),
                )
              }
            >
              Apply
            </Button>
          </div>
        </Field>
      </section>
    </div>
  );
}

function SelectedCaption({
  plan,
  document,
  cue,
  busy,
  onApply,
  onSelect,
  onSeek,
}: {
  readonly plan: PreviewPlan;
  readonly document: EditIr | null;
  readonly cue: PreviewCue;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onSelect: (selection: EditorSelection) => void;
  readonly onSeek: (frame: number) => void;
}) {
  const words = cue.lines.flat();
  const [editing, setEditing] = useState<number | null>(null);
  const cues = shownCues(plan, document);
  const position = cues.findIndex((item) => item.cue_id === cue.cueId);
  const saved = cues[position];
  const next = plan.cues[plan.cues.findIndex((item) => item.cueId === cue.cueId) + 1];
  const duration = programTicks(plan);

  const nudge = (edge: 'start' | 'end', direction: 1 | -1) => {
    if (!saved) return;
    const inner = saved.lines.flatMap((line) => line.words);
    const low =
      edge === 'start'
        ? (cues[position - 1]?.end_ticks ?? 0)
        : (inner.at(-1)?.end_ticks ?? saved.end_ticks);
    const high =
      edge === 'start'
        ? (inner[0]?.start_ticks ?? saved.start_ticks)
        : (cues[position + 1]?.start_ticks ?? duration);
    const current = edge === 'start' ? saved.start_ticks : saved.end_ticks;
    const moved = Math.max(low, Math.min(high, current + direction * ticksOfFrame(plan, 1)));
    if (moved === current) return;
    onApply(
      edge === 'start'
        ? setCueTiming(cue.cueId, moved, saved.end_ticks, plan.presentation)
        : setCueTiming(cue.cueId, saved.start_ticks, moved, plan.presentation),
    );
  };

  return (
    <section className="review-section edit-selected" aria-label="Selected caption">
      <div className="edit-selected-head">
        <h3 className="review-section-title">
          Caption at {clockTenths(ticksOfFrame(plan, cue.firstFrame))}
        </h3>
        <button
          type="button"
          className="edit-icon-button"
          aria-label="Deselect the caption"
          onClick={() => onSelect({ kind: 'clip' })}
        >
          <X className="size-3.5" aria-hidden="true" />
        </button>
      </div>
      <p className="edit-words" aria-label="Words in this caption">
        {words.map((word, index) => (
          <button
            // eslint-disable-next-line react/no-array-index-key -- a word's place is its identity here
            key={index}
            type="button"
            className="edit-word-chip"
            aria-pressed={editing === index}
            onClick={() => {
              setEditing(editing === index ? null : index);
              onSeek(cue.firstFrame);
            }}
          >
            {word.text}
          </button>
        ))}
      </p>
      {editing !== null && words[editing] && (
        <WordEditor
          key={`${cue.cueId}:${editing}`}
          plan={plan}
          cue={cue}
          index={editing}
          busy={busy}
          existing={plan.cues.map((item) => item.cueId)}
          onApply={onApply}
          onDone={() => setEditing(null)}
        />
      )}
      {saved && (
        <>
          <Field label="Starts">
            <TimeNudge
              label="start"
              ticks={saved.start_ticks}
              disabled={busy}
              onNudge={(direction) => nudge('start', direction)}
            />
          </Field>
          <Field label="Ends">
            <TimeNudge
              label="end"
              ticks={saved.end_ticks}
              disabled={busy}
              onNudge={(direction) => nudge('end', direction)}
            />
          </Field>
        </>
      )}
      <CueTiming plan={plan} cue={cue} busy={busy} onApply={onApply} />
      <Field label="Position">
        <RegionPicker
          label="Where this caption sits"
          value={cue.region as Region}
          disabled={busy}
          onPick={(region) => onApply(setCueRegion(cue.cueId, region, plan.presentation))}
        />
      </Field>
      {next && (
        <Button
          size="sm"
          variant="outline"
          className="self-start"
          disabled={busy}
          onClick={() => onApply(mergeCues(cue.cueId, next.cueId, plan.presentation))}
        >
          Join with the next caption
        </Button>
      )}
    </section>
  );
}

function WordEditor({
  plan,
  cue,
  index,
  busy,
  existing,
  onApply,
  onDone,
}: {
  readonly plan: PreviewPlan;
  readonly cue: PreviewCue;
  readonly index: number;
  readonly busy: boolean;
  readonly existing: readonly string[];
  readonly onApply: (command: EditCommandJson) => void;
  readonly onDone: () => void;
}) {
  const words = cue.lines.flat();
  const word = words[index]!;
  const [draft, setDraft] = useState(word.text);
  const corrected = draft.trim();
  const changed = corrected !== '' && corrected !== word.text;
  const inside = index > 0 && index < words.length;
  return (
    <div className="edit-word-editor">
      <form
        className="edit-inline"
        onSubmit={(event) => {
          event.preventDefault();
          if (!changed || busy) return;
          onApply(correctWord(plan, cue, index, word, corrected));
          onDone();
        }}
      >
        <Input
          aria-label="Correct this word"
          value={draft}
          autoFocus
          disabled={busy}
          onChange={(event) => setDraft(event.target.value)}
          className="h-8"
        />
        <Button type="submit" size="sm" disabled={busy || !changed}>
          Correct
        </Button>
      </form>
      <div className="edit-inline edit-word-actions">
        <Button
          size="xs"
          variant="ghost"
          disabled={busy || !inside}
          onClick={() => {
            onApply(splitCue(cue.cueId, index, freshCueId(existing, cue.cueId), plan.presentation));
            onDone();
          }}
        >
          Split caption here
        </Button>
        <Button
          size="xs"
          variant="ghost"
          disabled={busy || !inside}
          onClick={() => {
            onApply(setCueLines(cue.cueId, [index, words.length - index], plan.presentation));
            onDone();
          }}
        >
          New line here
        </Button>
        <Button
          size="xs"
          variant="ghost"
          disabled={busy}
          onClick={() => {
            onApply(removeCaptionWord(cue.cueId, index, plan.presentation));
            onDone();
          }}
        >
          Hide word
        </Button>
      </div>
      <p className="review-footnote">
        A correction changes the captions and subtitle files, not the sound.
      </p>
    </div>
  );
}

/* Framing ------------------------------------------------------------------- */

function FramingTab({
  plan,
  document,
  frame,
  selection,
  busy,
  resolving,
  resolveRefusal,
  onApply,
  onResolve,
  onSelect,
}: EditorPropertiesProps) {
  const part =
    selection.kind === 'section' || selection.kind === 'keyframe'
      ? plan.segments.find((item) => item.segmentId === selection.segmentId)
      : segmentAt(plan, frame);
  if (!part) return <p className="review-empty-note">Nothing to frame.</p>;
  const saved = document?.video.segments?.find((item) => item.segment_id === part.segmentId);
  const state =
    saved?.layout.state ??
    (part.hasTwoUpPaths ? 'two_up' : plan.crops[part.firstFrame] ? 'speaker_fill' : 'fit');
  const index = plan.segments.indexOf(part);
  const keyframe =
    selection.kind === 'keyframe'
      ? saved?.layout[selection.secondary ? 'secondary_crop_path' : 'crop_path']?.find(
          (item) => item.t_ticks === selection.tTicks,
        )
      : undefined;
  const source = sourceOf(plan, part);
  const localTicks = Math.max(0, ticksOfFrame(plan, frame) - part.programStartTicks);
  const here = cropAt(plan, frame);

  return (
    <div className="review-panel-body">
      <section className="review-section">
        <h3 className="review-section-title">
          {plan.segments.length > 1 ? `Section ${index + 1} of ${plan.segments.length}` : 'Framing'}
        </h3>
        <div className="review-segmented edit-wide" role="group" aria-label="Framing">
          {(
            [
              ['speaker_fill', 'Follow speaker'],
              ['fit', 'Whole frame'],
              ['two_up', 'Two speakers'],
            ] as const
          ).map(([mode, label]) => (
            <button
              key={mode}
              type="button"
              aria-pressed={state === mode}
              disabled={busy || (mode === 'two_up' && !part.hasTwoUpPaths)}
              onClick={() => onApply(setLayout(mode, part.segmentId))}
            >
              {label}
            </button>
          ))}
        </div>
        {!part.hasTwoUpPaths && (
          <p className="review-footnote">Two speakers needs two faces tracked in this section.</p>
        )}
        <div className="edit-inline">
          {state === 'two_up' && (
            <Button
              size="sm"
              variant="outline"
              disabled={busy}
              onClick={() => onApply(swapPortraits(part.segmentId))}
            >
              Switch speakers
            </Button>
          )}
          <Button
            size="sm"
            variant="outline"
            disabled={busy || resolving || resolveRefusal !== null}
            onClick={onResolve}
          >
            {resolving ? 'Recalculating…' : 'Recalculate framing'}
          </Button>
        </div>
        {resolveRefusal && <p className="review-footnote">{resolveRefusal}</p>}
        <p className="review-footnote">
          Drag the picture to reframe, and pinch or ⌘-scroll over it to zoom. Each change is a
          keyframe on the timeline.
        </p>
      </section>

      {selection.kind === 'keyframe' && keyframe && source ? (
        <KeyframeControls
          key={`${selection.segmentId}:${selection.tTicks}:${selection.secondary}:${plan.revision}`}
          plan={plan}
          rect={keyframe.rect}
          easing={keyframe.easing ?? 'linear'}
          source={source}
          segmentId={selection.segmentId}
          tTicks={selection.tTicks}
          secondary={selection.secondary}
          busy={busy}
          onApply={onApply}
          onRemoved={() => onSelect({ kind: 'section', segmentId: selection.segmentId })}
        />
      ) : (
        state !== 'fit' &&
        here && (
          <section className="review-section">
            <h3 className="review-section-title">Keyframes</h3>
            <p className="review-footnote">
              Select a diamond on the timeline to fine-tune it, or add one at the playhead.
            </p>
            <Button
              size="sm"
              variant="outline"
              className="self-start"
              disabled={busy}
              onClick={() => {
                onApply(setCropKeyframe(localTicks, here, part.segmentId));
                onSelect({
                  kind: 'keyframe',
                  segmentId: part.segmentId,
                  tTicks: localTicks,
                  secondary: false,
                });
              }}
            >
              Add a keyframe here
            </Button>
          </section>
        )
      )}
    </div>
  );
}

function KeyframeControls({
  plan,
  rect,
  easing,
  source,
  segmentId,
  tTicks,
  secondary,
  busy,
  onApply,
  onRemoved,
}: {
  readonly plan: PreviewPlan;
  readonly rect: {
    readonly x: number;
    readonly y: number;
    readonly width: number;
    readonly height: number;
  };
  readonly easing: Easing;
  readonly source: { readonly displayWidth: number; readonly displayHeight: number };
  readonly segmentId: string;
  readonly tTicks: number;
  readonly secondary: boolean;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onRemoved: () => void;
}) {
  const write = (change: Partial<typeof rect>, nextEasing: Easing = easing) =>
    onApply(setCropKeyframe(tTicks, { ...rect, ...change }, segmentId, secondary, nextEasing));
  const zoom = Math.round((source.displayHeight / rect.height) * 100);
  return (
    <section className="review-section">
      <h3 className="review-section-title">
        {secondary ? 'Lower speaker' : 'Keyframe'} at{' '}
        {clockTenths(
          tTicks +
            (plan.segments.find((item) => item.segmentId === segmentId)?.programStartTicks ?? 0),
        )}
      </h3>
      <Field label="Across">
        <CommitSlider
          label="Horizontal position"
          min={0}
          max={Math.max(0, source.displayWidth - rect.width)}
          value={rect.x}
          disabled={busy || source.displayWidth <= rect.width}
          format={(value) =>
            `${Math.round((value / Math.max(1, source.displayWidth - rect.width)) * 100)}%`
          }
          onCommit={(value) => write({ x: value })}
        />
      </Field>
      <Field label="Up / down">
        <CommitSlider
          label="Vertical position"
          min={0}
          max={Math.max(0, source.displayHeight - rect.height)}
          value={rect.y}
          disabled={busy || source.displayHeight <= rect.height}
          format={(value) =>
            `${Math.round((value / Math.max(1, source.displayHeight - rect.height)) * 100)}%`
          }
          onCommit={(value) => write({ y: value })}
        />
      </Field>
      <Field label="Zoom">
        <CommitSlider
          label="Zoom"
          min={100}
          max={300}
          value={zoom}
          disabled={busy}
          format={(value) => `${value}%`}
          onCommit={(value) => {
            const height = Math.max(2, Math.round(source.displayHeight / (value / 100) / 2) * 2);
            const width = Math.min(
              source.displayWidth,
              Math.max(2, Math.round((height * rect.width) / rect.height / 2) * 2),
            );
            write({
              width,
              height,
              x: Math.max(
                0,
                Math.min(
                  source.displayWidth - width,
                  Math.round(rect.x + (rect.width - width) / 2),
                ),
              ),
              y: Math.max(
                0,
                Math.min(
                  source.displayHeight - height,
                  Math.round(rect.y + (rect.height - height) / 2),
                ),
              ),
            });
          }}
        />
      </Field>
      <Field label="Easing">
        <Select
          value={easing}
          disabled={busy}
          onValueChange={(value) => write({}, value as Easing)}
        >
          <SelectTrigger aria-label="Easing into this keyframe" className="edit-select">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="linear">Steady</SelectItem>
            <SelectItem value="ease_in">Ease in</SelectItem>
            <SelectItem value="ease_out">Ease out</SelectItem>
            <SelectItem value="ease_in_out">Ease in and out</SelectItem>
          </SelectContent>
        </Select>
      </Field>
      <Button
        size="sm"
        variant="outline"
        className="self-start"
        disabled={busy}
        onClick={() => {
          onApply(removeCropKeyframe(tTicks, segmentId, secondary));
          onRemoved();
        }}
      >
        Remove keyframe
      </Button>
    </section>
  );
}

/* Audio --------------------------------------------------------------------- */

function AudioTab({
  plan,
  document,
  frame,
  selection,
  busy,
  onApply,
  onSelect,
}: EditorPropertiesProps) {
  const playhead = ticksOfFrame(plan, frame);
  const point =
    selection.kind === 'gain'
      ? plan.gain.find(
          (item, index) =>
            (document?.audio.gain_curve?.[index]?.t_ticks ?? ticksOfFrame(plan, item.frame)) ===
            selection.tTicks,
        )
      : undefined;
  const target = document?.audio.target_lufs;
  return (
    <div className="review-panel-body">
      {selection.kind === 'gain' && point && (
        <section className="review-section edit-selected" aria-label="Selected volume point">
          <div className="edit-selected-head">
            <h3 className="review-section-title">
              Volume point at {clockTenths(selection.tTicks)}
            </h3>
            <button
              type="button"
              className="edit-icon-button"
              aria-label="Deselect the volume point"
              onClick={() => onSelect({ kind: 'clip' })}
            >
              <X className="size-3.5" aria-hidden="true" />
            </button>
          </div>
          <Field label="Level">
            <CommitSlider
              key={`${selection.tTicks}:${plan.revision}`}
              label="Level at this point"
              min={-12}
              max={12}
              value={Math.round(point.gainDb)}
              disabled={busy}
              format={decibels}
              onCommit={(value) => onApply(setGain(selection.tTicks, value))}
            />
          </Field>
          <Button
            size="sm"
            variant="outline"
            className="self-start"
            disabled={busy}
            onClick={() => {
              onApply(removeGainPoint(selection.tTicks));
              onSelect({ kind: 'clip' });
            }}
          >
            Remove point
          </Button>
        </section>
      )}
      <section className="review-section">
        <h3 className="review-section-title">Volume</h3>
        <Field label="At playhead">
          <CommitSlider
            key={`${frame}:${plan.revision}`}
            label="Level at the playhead"
            min={-12}
            max={12}
            value={Math.round(gainAt(plan, frame))}
            disabled={busy}
            format={decibels}
            onCommit={(value) => {
              onApply(setGain(playhead, value));
              onSelect({ kind: 'gain', tTicks: playhead });
            }}
          />
        </Field>
        <p className="review-footnote">
          Double-click the audio lane to add a point, and drag points to shape the level.
        </p>
      </section>
      <section className="review-section">
        <h3 className="review-section-title">Loudness</h3>
        <p className="edit-fact">
          {target === undefined
            ? 'Evened out when the clip is exported.'
            : `Evened out to ${target} LUFS when the clip is exported, the level short-form apps expect.`}
        </p>
        <p className="review-footnote">
          The preview plays the proxy’s sound with your level changes.
        </p>
      </section>
    </div>
  );
}

/* Details ------------------------------------------------------------------- */

function DetailsTab({ plan, document }: EditorPropertiesProps) {
  const first = plan.segments[0];
  const last = plan.segments.at(-1);
  const look = PRESETS.find((preset) =>
    (document?.captions.style_ref ?? plan.captionStyle?.styleRef ?? '').includes(
      `.${preset.label.toLowerCase()}.`,
    ),
  );
  return (
    <div className="review-panel-body">
      <dl className="review-facts">
        <Fact label="Length" value={clockTenths(programTicks(plan))} mono />
        {first && last && (
          <Fact
            label="From the recording"
            value={`${timecode(first.inTicks)} – ${timecode(last.outTicks)}`}
            mono
          />
        )}
        <Fact label="Sections" value={String(plan.segments.length)} />
        <Fact label="Captions" value={`${plan.cues.length} · ${look?.label ?? 'Custom'}`} />
        <Fact label="Frame" value={`${plan.width} × ${plan.height}`} mono />
        <Fact label="Rate" value={`${(plan.rateNum / plan.rateDen).toFixed(2)} fps`} mono />
      </dl>
      <details className="review-disclosure">
        <summary>Diagnostics</summary>
        <dl className="review-facts review-facts-technical">
          <Fact label="Revision" value={String(plan.revision)} mono />
          <Fact
            label="Captions from"
            value={plan.presentation === 'burn_in' ? 'Burned-in grouping' : 'Reading grouping'}
          />
          <Fact label="Preview" value="Drawn from the render plan" />
          {plan.proxies.map((proxy) => (
            <Fact
              key={proxy.artifactId}
              label="Proxy"
              value={`${proxy.width} × ${proxy.height}`}
              mono
            />
          ))}
        </dl>
      </details>
    </div>
  );
}

/* Controls ------------------------------------------------------------------ */

function Field({ label, children }: { readonly label: string; readonly children: ReactNode }) {
  return (
    <div className="edit-field">
      <span className="edit-field-label">{label}</span>
      <div className="edit-field-control">{children}</div>
    </div>
  );
}

function Fact({
  label,
  value,
  mono = false,
}: {
  readonly label: string;
  readonly value: string;
  readonly mono?: boolean;
}) {
  return (
    <div>
      <dt>{label}</dt>
      <dd className={mono ? 'mono' : undefined}>{value}</dd>
    </div>
  );
}

function RegionPicker({
  label,
  value,
  disabled,
  onPick,
}: {
  readonly label: string;
  readonly value: string | null;
  readonly disabled: boolean;
  readonly onPick: (region: Region) => void;
}) {
  return (
    <div className="review-segmented" role="group" aria-label={label}>
      {REGIONS.map(([region, name]) => (
        <button
          key={region}
          type="button"
          aria-pressed={value === region}
          disabled={disabled}
          onClick={() => value !== region && onPick(region)}
        >
          {name}
        </button>
      ))}
    </div>
  );
}

/** A slider that moves freely and sends its value once, when let go. */
function CommitSlider({
  label,
  min,
  max,
  value,
  disabled,
  format,
  onCommit,
}: {
  readonly label: string;
  readonly min: number;
  readonly max: number;
  readonly value: number;
  readonly disabled: boolean;
  readonly format: (value: number) => string;
  readonly onCommit: (value: number) => void;
}) {
  const [draft, setDraft] = useState(value);
  useEffect(() => setDraft(value), [value]);
  const commit = () => {
    if (draft !== value) onCommit(draft);
  };
  return (
    <div className="edit-slider">
      <input
        type="range"
        aria-label={label}
        min={min}
        max={max}
        step={1}
        value={draft}
        disabled={disabled}
        onChange={(event) => setDraft(Number(event.target.value))}
        onPointerUp={commit}
        onKeyUp={commit}
        onBlur={commit}
      />
      <span className="mono">{format(draft)}</span>
    </div>
  );
}

/** A colour well that sends its colour once the picker closes. */
function Swatch({
  label,
  value,
  disabled,
  onCommit,
}: {
  readonly label: string;
  readonly value: string;
  readonly disabled: boolean;
  readonly onCommit: (value: string) => void;
}) {
  const shown = value.slice(0, 7).toLowerCase();
  const [draft, setDraft] = useState(shown);
  const input = useRef<HTMLInputElement>(null);
  const latest = useRef({ shown, onCommit });
  latest.current = { shown, onCommit };
  useEffect(() => setDraft(shown), [shown]);
  // React's onChange is the input event; the picker's own change is its close.
  useEffect(() => {
    const element = input.current;
    if (!element) return;
    const closed = () => {
      if (element.value !== latest.current.shown) latest.current.onCommit(element.value);
    };
    element.addEventListener('change', closed);
    return () => element.removeEventListener('change', closed);
  }, []);
  return (
    <label className="edit-swatch" title={label}>
      <input
        ref={input}
        type="color"
        aria-label={label}
        value={draft}
        disabled={disabled}
        onChange={(event) => setDraft(event.target.value)}
      />
      <span style={{ background: draft }} aria-hidden="true" />
      <span className="edit-swatch-label">{label}</span>
    </label>
  );
}

function Stepper({
  label,
  value,
  min,
  max,
  onChange,
}: {
  readonly label: string;
  readonly value: number;
  readonly min: number;
  readonly max: number;
  readonly onChange: (value: number) => void;
}) {
  return (
    <div className="edit-stepper" role="group" aria-label={label}>
      <button
        type="button"
        aria-label="Fewer"
        disabled={value <= min}
        onClick={() => onChange(value - 1)}
      >
        <Minus className="size-3" aria-hidden="true" />
      </button>
      <span className="mono" aria-live="polite">
        {value}
      </span>
      <button
        type="button"
        aria-label="More"
        disabled={value >= max}
        onClick={() => onChange(value + 1)}
      >
        <Plus className="size-3" aria-hidden="true" />
      </button>
    </div>
  );
}

function TimeNudge({
  label,
  ticks,
  disabled,
  onNudge,
}: {
  readonly label: string;
  readonly ticks: number;
  readonly disabled: boolean;
  readonly onNudge: (direction: 1 | -1) => void;
}) {
  return (
    <span className="edit-stepper" role="group" aria-label={`Caption ${label}`}>
      <button
        type="button"
        aria-label={`Move the ${label} a frame earlier`}
        disabled={disabled}
        onClick={() => onNudge(-1)}
      >
        <Minus className="size-3" aria-hidden="true" />
      </button>
      <span className="mono">{clockHundredths(ticks)}</span>
      <button
        type="button"
        aria-label={`Move the ${label} a frame later`}
        disabled={disabled}
        onClick={() => onNudge(1)}
      >
        <Plus className="size-3" aria-hidden="true" />
      </button>
    </span>
  );
}

function decibels(value: number): string {
  return value === 0 ? '0 dB' : `${value > 0 ? '+' : '−'}${Math.abs(value)} dB`;
}

function clockHundredths(ticks: number): string {
  const hundredths = Math.max(0, Math.round(ticks / 900));
  const seconds = Math.floor(hundredths / 100);
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}.${String(hundredths % 100).padStart(2, '0')}`;
}

function wordCounts(total: number, limit: number): number[] {
  const counts: number[] = [];
  for (let left = total; left > 0; left -= limit) counts.push(Math.min(left, limit));
  return counts;
}
