/**
 * The Editor's properties, by kind: captions, framing, audio and details.
 * Selecting something on the monitor or the timeline opens its tab with its own
 * controls on top. Sliders and colours send one command when let go.
 */
import type { EditIr } from '@clipmill/contracts';
import {
  ArrowDownLeft,
  ArrowDownRight,
  ArrowUpLeft,
  ArrowUpRight,
  AudioLines,
  Captions as CaptionsIcon,
  Copy,
  Crop,
  Info,
  Minus,
  PanelTop,
  PictureInPicture2,
  Plus,
  RectangleHorizontal,
  Rows2,
  ScanFace,
  X,
} from 'lucide-react';
import { useEffect, useState } from 'react';

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
import type { CaptionFont, EditCommandJson, PreviewCue, PreviewPlan } from '../daemon/client.js';
import type { CaptionDraft } from '../screens/Editor.js';
import { clockTenths } from '../inspector/review.js';
import type { EditorFocus } from '../shell/route.js';
import { repairAll, shortCues } from './captionRepairs.js';
import { CaptionStyleControls, LOOKS } from './CaptionStyle.js';
import { CommitSlider, Field, Swatch } from './controls.js';
import {
  INSET_SIZES,
  type InsetCorner,
  type LayoutChoice,
  type LayoutStyle,
  SPLITS,
  ZOOMS,
  DEFAULT_INSET,
  layoutCommands,
  recordingSplit,
  refit,
  setLayoutStyle,
  splitCommands,
  styleOf,
  switchCommands,
  viewportHeights,
} from './layouts.js';
import { CueTiming } from './CueTiming.js';
import {
  batch,
  correctWord,
  mergeCues,
  removeCaptionWord,
  removeCropKeyframe,
  removeGainPoint,
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
  /** Draw a caption look before it is chosen; null goes back to the saved one. */
  readonly onTryLook?: ((look: CaptionDraft | null) => void) | null;
  /** Every caption typeface, with whether this installation has it. */
  readonly fonts?: readonly CaptionFont[];
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
  onTryLook = null,
  fonts = [],
}: EditorPropertiesProps) {
  const cue =
    selection.kind === 'cue' ? plan.cues.find((item) => item.cueId === selection.cueId) : undefined;
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

      <CaptionStyleControls
        plan={plan}
        document={document}
        busy={busy}
        fonts={fonts}
        onApply={onApply}
        onTryLook={onTryLook}
      />
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

  const layout = saved?.layout;
  const style = styleOf(layout);
  const output = { width: plan.width, height: plan.height };
  const screenSplit = source ? recordingSplit(source, output) : null;
  const choose = (choice: LayoutChoice) => {
    if (!source) {
      onApply(setLayout(choice === 'screen_and_face' ? 'two_up' : choice, part.segmentId));
      return;
    }
    onApply(batch(layoutCommands(part.segmentId, choice, layout, output, source)));
  };
  const restyle = (next: LayoutStyle) => onApply(setLayoutStyle(part.segmentId, next));
  const pressed = (choice: LayoutChoice) =>
    choice === 'screen_and_face'
      ? state === 'two_up' && style.split === screenSplit
      : choice === 'two_up'
        ? state === 'two_up' && style.split !== screenSplit
        : state === choice;
  const wholeMain = state === 'picture_in_picture' && (layout?.crop_path?.length ?? 0) === 0;
  const others = document?.video.segments?.filter((item) => item.segment_id !== part.segmentId);

  return (
    <div className="review-panel-body">
      <section className="review-section">
        <h3 className="review-section-title">
          {plan.segments.length > 1 ? `Section ${index + 1} of ${plan.segments.length}` : 'Framing'}
        </h3>
        <div className="edit-layouts" role="group" aria-label="Layout">
          {(
            [
              ['speaker_fill', 'Follow speaker', ScanFace],
              ['fit', 'Whole frame', RectangleHorizontal],
              ['two_up', 'Two speakers', Rows2],
              ['screen_and_face', 'Screen and face', PanelTop],
              ['picture_in_picture', 'Picture in picture', PictureInPicture2],
            ] as const
          ).map(([choice, label, Icon]) => (
            <button
              key={choice}
              type="button"
              aria-pressed={pressed(choice)}
              disabled={busy || (choice === 'two_up' && !part.hasTwoUpPaths)}
              onClick={() => choose(choice)}
            >
              <Icon className="size-4" aria-hidden="true" />
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
              onClick={() =>
                onApply(
                  source
                    ? batch(switchCommands(part.segmentId, layout, output, source))
                    : swapPortraits(part.segmentId),
                )
              }
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

      {state === 'two_up' && source && (
        <section className="review-section">
          <h3 className="review-section-title">Split</h3>
          <Field label="Top">
            <CommitSlider
              label="Top viewport share"
              min={SPLITS.min / 10}
              max={SPLITS.max / 10}
              value={Math.round((style.split ?? SPLITS.even) / 10)}
              disabled={busy}
              format={(value) => `${value}%`}
              onCommit={(value) =>
                onApply(
                  batch(
                    splitCommands(
                      part.segmentId,
                      { ...style, split: value * 10 },
                      layout,
                      output,
                      source,
                    ),
                  ),
                )
              }
            />
          </Field>
          <p className="review-footnote">
            {`Top ${viewportHeights(style.split ?? SPLITS.even, plan.height)[0]} px, bottom ${viewportHeights(style.split ?? SPLITS.even, plan.height)[1]} px. `}
            Both crops keep their centre as the split moves.
          </p>
        </section>
      )}

      {(state === 'fit' || wholeMain) && (
        <section className="review-section">
          <h3 className="review-section-title">
            {wholeMain ? 'Behind the picture' : 'Background'}
          </h3>
          <Field label="Fill">
            <div className="flex items-center gap-2">
              <div className="review-segmented" role="group" aria-label="Background">
                <button
                  type="button"
                  aria-pressed={style.background?.kind !== 'colour'}
                  disabled={busy}
                  onClick={() => restyle({ ...style, background: undefined })}
                >
                  Blur
                </button>
                <button
                  type="button"
                  aria-pressed={style.background?.kind === 'colour'}
                  disabled={busy}
                  onClick={() =>
                    restyle({ ...style, background: { kind: 'colour', colour: '#000000' } })
                  }
                >
                  Colour
                </button>
              </div>
              {style.background?.kind === 'colour' && (
                <Swatch
                  label="Background colour"
                  value={style.background.colour}
                  disabled={busy}
                  onCommit={(colour) =>
                    restyle({ ...style, background: { kind: 'colour', colour } })
                  }
                />
              )}
            </div>
          </Field>
          <Field label="Zoom">
            <CommitSlider
              label="Picture zoom"
              min={ZOOMS.min}
              max={ZOOMS.max}
              value={style.zoom ?? ZOOMS.min}
              disabled={busy}
              format={(value) => `${value}%`}
              onCommit={(value) =>
                restyle({ ...style, zoom: value === ZOOMS.min ? undefined : value })
              }
            />
          </Field>
          <p className="review-footnote">
            Zooming grows the picture about its centre; its sides give way at the frame’s edge.
          </p>
        </section>
      )}

      {state === 'picture_in_picture' && (
        <section className="review-section">
          <h3 className="review-section-title">Inset</h3>
          <Field label="Corner">
            <div className="review-segmented" role="group" aria-label="Inset corner">
              {(
                [
                  ['top_left', 'Top left', ArrowUpLeft],
                  ['top_right', 'Top right', ArrowUpRight],
                  ['bottom_left', 'Bottom left', ArrowDownLeft],
                  ['bottom_right', 'Bottom right', ArrowDownRight],
                ] as const satisfies readonly (readonly [InsetCorner, string, unknown])[]
              ).map(([corner, label, Icon]) => (
                <button
                  key={corner}
                  type="button"
                  aria-label={label}
                  title={label}
                  aria-pressed={(style.inset ?? DEFAULT_INSET).corner === corner}
                  disabled={busy}
                  onClick={() =>
                    restyle({ ...style, inset: { ...(style.inset ?? DEFAULT_INSET), corner } })
                  }
                >
                  <Icon className="size-4" aria-hidden="true" />
                </button>
              ))}
            </div>
          </Field>
          <Field label="Size">
            <CommitSlider
              label="Inset size"
              min={INSET_SIZES.min / 10}
              max={INSET_SIZES.max / 10}
              value={Math.round((style.inset ?? DEFAULT_INSET).size / 10)}
              disabled={busy}
              format={(value) => `${value}%`}
              onCommit={(value) =>
                restyle({
                  ...style,
                  inset: { ...(style.inset ?? DEFAULT_INSET), size: value * 10 },
                })
              }
            />
          </Field>
          <Field label="Main picture">
            <div className="review-segmented" role="group" aria-label="Main picture">
              <button
                type="button"
                aria-pressed={wholeMain}
                disabled={busy}
                onClick={() =>
                  onApply({ op: 'replace_crop_path', segment_id: part.segmentId, path: [] })
                }
              >
                Whole frame
              </button>
              <button
                type="button"
                aria-pressed={!wholeMain}
                disabled={busy || !source}
                onClick={() => {
                  if (!source) return;
                  onApply({
                    op: 'replace_crop_path',
                    segment_id: part.segmentId,
                    path: refit(
                      [
                        {
                          t_ticks: 0,
                          rect: {
                            x: 0,
                            y: 0,
                            width: source.displayWidth,
                            height: source.displayHeight,
                          },
                        },
                      ],
                      output,
                      source,
                    ),
                  });
                }}
              >
                Cropped
              </button>
            </div>
          </Field>
          <p className="review-footnote">
            Drag the inset on the preview to choose who it shows; drag elsewhere to move the main
            picture.
          </p>
        </section>
      )}

      {others && others.length > 0 && state !== 'speaker_fill' && (
        <Button
          size="sm"
          variant="outline"
          className="self-start"
          disabled={busy}
          onClick={() =>
            onApply(
              batch(
                others.flatMap((item) =>
                  item.layout.state === 'two_up' && source
                    ? splitCommands(item.segment_id, style, item.layout, output, source)
                    : [setLayoutStyle(item.segment_id, style)],
                ),
              ),
            )
          }
        >
          <Copy className="size-4" aria-hidden="true" />
          Use this style in every section
        </Button>
      )}

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
  const look = LOOKS.find((preset) =>
    (document?.captions.style_ref ?? plan.captionStyle?.styleRef ?? '').includes(
      `.${preset.look}.`,
    ),
  );
  return (
    <div className="review-panel-body">
      <dl className="review-facts">
        <Fact label="Length" value={clockTenths(programTicks(plan))} mono />
        {first && last && (
          <Fact
            label="From the recording"
            value={`${clockTenths(first.inTicks)} – ${clockTenths(last.outTicks)}`}
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
