/**
 * How every caption in the clip looks, reads and sits.
 *
 * Trying is not choosing. Hovering a look or a highlight, dragging a slider or
 * moving through a colour picker redraws the preview with the export's own
 * captions and saves nothing; letting go, or clicking, is one edit. That is
 * what makes comparing looks cheap: before, every look was a saved command and
 * a full plan round trip, and comparing two meant toggling between them.
 */
import type { EditIr } from '@clipmill/contracts';
import { Bookmark, Sparkles, Trash2, X } from 'lucide-react';
import { type JSX, useState } from 'react';

import { Button } from '../components/ui/button.js';
import { Input } from '../components/ui/input.js';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../components/ui/select.js';
import { Switch } from '../components/ui/switch.js';
import type { CaptionFont, EditCommandJson, PreviewPlan } from '../daemon/client.js';
import { CaptionLookSample } from '../results/CaptionLookSample.js';
import type { CaptionDraft } from '../screens/Editor.js';
import {
  type CaptionOptions,
  type SavedStyle,
  applyStyle,
  clearKeyWords,
  forgetStyle,
  keyWords,
  markKeyWords,
  nonSpeechCount,
  saveStyle,
  savedStyles,
} from './captionStyles.js';
import {
  batch,
  dropNonSpeechWords,
  regroupOnScreen,
  setCaptionOptions,
  setCaptionStyle,
  setCuePosition,
  setCueRegion,
} from './commands.js';
import { CommitSlider, Field, Swatch } from './controls.js';

export const LOOKS = [
  { label: 'Clean', ref: 'clipmill.captions.clean.v1', look: 'clean' },
  { label: 'Minimal', ref: 'clipmill.captions.minimal.v1', look: 'minimal' },
  { label: 'Boxed', ref: 'clipmill.captions.boxed.v1', look: 'boxed' },
] as const;

type HighlightStyle = NonNullable<CaptionOptions['highlight_style']>;

const HIGHLIGHTS: readonly (readonly [HighlightStyle, string])[] = [
  ['fill', 'Sweep'],
  ['word', 'Word'],
  ['box', 'Box'],
  ['pop', 'Pop'],
  ['underline', 'Underline'],
];

const REGIONS = [
  ['upper_safe', 'Top'],
  ['center', 'Middle'],
  ['lower_safe', 'Bottom'],
] as const;

export interface CaptionStyleControlsProps {
  readonly plan: PreviewPlan;
  readonly document: EditIr | null;
  readonly busy: boolean;
  readonly fonts: readonly CaptionFont[];
  readonly onApply: (command: EditCommandJson) => void;
  /** Draw a look before it is chosen; null goes back to the saved one. */
  readonly onTryLook: ((look: CaptionDraft | null) => void) | null;
}

export function CaptionStyleControls({
  plan,
  document,
  busy,
  fonts,
  onApply,
  onTryLook,
}: CaptionStyleControlsProps): JSX.Element {
  const style = plan.captionStyle;
  const styleRef = document?.captions.style_ref ?? style?.styleRef ?? LOOKS[0].ref;
  const options: CaptionOptions = document?.captions.options ?? {};
  const boxed = style?.boxed ?? styleRef.includes('.boxed.');
  const highlightOn = options.highlight_spoken_word ?? !styleRef.includes('.minimal.');
  const highlight = options.highlight_style ?? 'fill';
  const tryOptions = (change: Partial<CaptionOptions>) =>
    onTryLook?.({ options: { ...options, ...change } });
  const letGo = () => onTryLook?.(null);
  const update = (change: Partial<CaptionOptions>) => {
    letGo();
    onApply(setCaptionOptions({ ...options, ...change }));
  };
  const chooseLook = (ref: string) => {
    letGo();
    onApply(
      options.highlight_spoken_word === undefined
        ? batch([
            setCaptionOptions({ ...options, highlight_spoken_word: highlightOn }),
            setCaptionStyle(ref),
          ])
        : setCaptionStyle(ref),
    );
  };
  const installed = fonts.filter((font) => font.installed);
  const family = options.font_family ?? style?.fontFamily ?? 'Inter';
  const marked = keyWords(document).length;
  const noise = nonSpeechCount(document);
  const placed = Boolean(options.position) || plan.cues.some((cue) => cue.position);
  const regionOfAll =
    !placed && plan.cues.every((cue) => cue.region === plan.cues[0]?.region)
      ? plan.cues[0]?.region
      : null;

  return (
    <>
      <section className="review-section" aria-label="Caption look">
        <h3 className="review-section-title">Look</h3>
        <div
          className="review-segmented edit-wide"
          role="group"
          aria-label="Caption look"
          onPointerLeave={letGo}
        >
          {LOOKS.map((look) => (
            <button
              key={look.ref}
              type="button"
              className="edit-look-button"
              aria-pressed={styleRef.includes(`.${look.look}.`)}
              disabled={busy}
              onPointerEnter={() => onTryLook?.({ styleRef: look.ref, options })}
              onFocus={() => onTryLook?.({ styleRef: look.ref, options })}
              onBlur={letGo}
              onClick={() => chooseLook(look.ref)}
            >
              <CaptionLookSample look={look.look} highlight={highlightOn} />
              {look.label}
            </button>
          ))}
        </div>
        <SavedStyles
          styleRef={styleRef}
          options={options}
          busy={busy}
          onApply={(command) => {
            letGo();
            onApply(command);
          }}
          onTry={(saved) => onTryLook?.({ styleRef: saved.styleRef, options: saved.options })}
          onLetGo={letGo}
        />
      </section>

      <section className="review-section" aria-label="Spoken word">
        <div className="edit-section-head">
          <h3 className="review-section-title">Highlight the spoken word</h3>
          <Switch
            aria-label="Highlight the spoken word"
            checked={highlightOn}
            disabled={busy}
            onCheckedChange={(checked) => update({ highlight_spoken_word: checked })}
          />
        </div>
        {highlightOn && (
          <>
            <div
              className="review-segmented edit-wide edit-highlights"
              role="group"
              aria-label="How the spoken word is marked"
              onPointerLeave={letGo}
            >
              {HIGHLIGHTS.map(([value, label]) => (
                <button
                  key={value}
                  type="button"
                  aria-pressed={highlight === value}
                  disabled={busy}
                  onPointerEnter={() => tryOptions({ highlight_style: value })}
                  onClick={() => update({ highlight_style: value })}
                >
                  {label}
                </button>
              ))}
            </div>
            <Field label="Colour">
              <div className="edit-swatches">
                <Swatch
                  label={highlight === 'box' ? 'Box' : 'Spoken word'}
                  value={options.spoken ?? style?.spoken ?? '#ffd65c'}
                  disabled={busy}
                  onDraft={(value) => tryOptions({ spoken: value })}
                  onCommit={(value) => update({ spoken: value })}
                />
              </div>
            </Field>
          </>
        )}
      </section>

      <section className="review-section" aria-label="Type">
        <h3 className="review-section-title">Type</h3>
        <Field label="Font">
          <Select
            value={family}
            disabled={busy || installed.length < 2}
            onValueChange={(value) =>
              update({ font_family: value as NonNullable<CaptionOptions['font_family']> })
            }
          >
            <SelectTrigger aria-label="Caption font" className="w-full">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {(installed.length > 0
                ? installed
                : [{ family, label: family, file: '', installed: true }]
              ).map((font) => (
                <SelectItem key={font.family} value={font.family}>
                  {font.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
        <Field label="Size">
          <CommitSlider
            label="Caption size"
            min={40}
            max={160}
            value={options.font_size ?? style?.fontSize ?? 84}
            disabled={busy}
            format={(value) => `${value}`}
            onDraft={(value) => tryOptions({ font_size: value })}
            onCommit={(value) => update({ font_size: value })}
          />
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
        <Field label="Colours">
          <div className="edit-swatches">
            <Swatch
              label="Words"
              value={options.unspoken ?? style?.unspoken ?? '#ffffff'}
              disabled={busy}
              onDraft={(value) => tryOptions({ unspoken: value })}
              onCommit={(value) => update({ unspoken: value })}
            />
            <Swatch
              label={boxed ? 'Plate' : 'Outline'}
              value={options.outline ?? style?.outline ?? '#000000'}
              disabled={busy}
              onDraft={(value) => tryOptions({ outline: value })}
              onCommit={(value) => update({ outline: value })}
            />
          </div>
        </Field>
        <Field label={boxed ? 'Padding' : 'Outline'}>
          <CommitSlider
            label={boxed ? 'Plate padding' : 'Outline thickness'}
            min={0}
            max={16}
            value={options.outline_width ?? style?.outlineWidth ?? 5}
            disabled={busy}
            format={(value) => `${value}`}
            onDraft={(value) => tryOptions({ outline_width: value })}
            onCommit={(value) => update({ outline_width: value })}
          />
        </Field>
        {boxed ? (
          <Field label="Opacity">
            <CommitSlider
              label="Plate opacity"
              min={0}
              max={100}
              value={options.plate_opacity ?? 85}
              disabled={busy}
              format={(value) => `${value}%`}
              onDraft={(value) => tryOptions({ plate_opacity: value })}
              onCommit={(value) => update({ plate_opacity: value })}
            />
          </Field>
        ) : (
          <Field label="Shadow">
            <CommitSlider
              label="Shadow depth"
              min={0}
              max={12}
              value={options.shadow_depth ?? style?.shadowDepth ?? 2}
              disabled={busy}
              format={(value) => `${value}`}
              onDraft={(value) => tryOptions({ shadow_depth: value })}
              onCommit={(value) => update({ shadow_depth: value })}
            />
          </Field>
        )}
      </section>

      <section className="review-section" aria-label="Words">
        <h3 className="review-section-title">Words</h3>
        <Field label="On screen">
          <div className="review-segmented" role="group" aria-label="Words on screen at once">
            {[1, 2, 3, 4, 5].map((count) => (
              <button
                key={count}
                type="button"
                aria-pressed={options.words_on_screen === count}
                disabled={busy}
                onClick={() => onApply(regroupOnScreen(count))}
              >
                {count}
              </button>
            ))}
          </div>
        </Field>
        <p className="review-footnote">
          How many words the burned-in captions show at once. The subtitle files keep their reading
          groups.
        </p>
        <Field label="Key words">
          <div className="edit-inline">
            <Button
              size="sm"
              variant="outline"
              disabled={busy}
              onClick={() => {
                const command = markKeyWords(plan.cues, document);
                if (command) onApply(command);
              }}
            >
              <Sparkles className="size-3.5" aria-hidden="true" />
              Mark key words
            </Button>
            {marked > 0 && (
              <Button
                size="sm"
                variant="ghost"
                disabled={busy}
                aria-label="Clear the key words"
                onClick={() => {
                  const command = clearKeyWords(document);
                  if (command) onApply(command);
                }}
              >
                <X className="size-3.5" aria-hidden="true" />
                {marked}
              </Button>
            )}
          </div>
        </Field>
        <Field label="Accent">
          <div className="edit-swatches">
            <Swatch
              label="Key words"
              value={options.accent ?? style?.accent ?? '#4ade80'}
              disabled={busy}
              onDraft={(value) => tryOptions({ accent: value })}
              onCommit={(value) => update({ accent: value })}
            />
          </div>
        </Field>
        <p className="review-footnote">
          Right-click a word in the transcript to mark or unmark it on its own.
        </p>
        {noise > 0 && (
          <div className="edit-notice" role="status">
            <span>
              {noise === 1 ? 'One mark' : `${noise} marks`} the recognizer wrote are not speech,
              such as “-” or [BLANK_AUDIO].
            </span>
            <Button
              size="sm"
              variant="outline"
              disabled={busy}
              onClick={() => onApply(dropNonSpeechWords())}
            >
              Remove
            </Button>
          </div>
        )}
      </section>

      <section className="review-section" aria-label="Position">
        <h3 className="review-section-title">Position</h3>
        <div
          className="review-segmented edit-wide"
          role="group"
          aria-label="Where every caption sits"
        >
          {REGIONS.map(([region, name]) => (
            <button
              key={region}
              type="button"
              aria-pressed={regionOfAll === region}
              disabled={busy}
              onClick={() => {
                const { position: _drop, ...rest } = options;
                onApply(
                  batch([
                    ...(options.position ? [setCaptionOptions(rest)] : []),
                    ...plan.cues
                      .filter((cue) => cue.position)
                      .map((cue) => setCuePosition(cue.cueId, null, plan.presentation)),
                    ...plan.cues
                      .filter((cue) => cue.region !== region)
                      .map((cue) => setCueRegion(cue.cueId, region, plan.presentation)),
                  ]),
                );
              }}
            >
              {name}
            </button>
          ))}
        </div>
        <p className="review-footnote">
          {placed
            ? 'Placed by hand. Choose Top, Middle or Bottom to line them up again.'
            : 'Drag a caption on the preview to place every caption. Hold Option while dragging to move only that one.'}
        </p>
      </section>
    </>
  );
}

/** Styles kept under a name: applied as one edit, saved from the clip's own. */
function SavedStyles({
  styleRef,
  options,
  busy,
  onApply,
  onTry,
  onLetGo,
}: {
  readonly styleRef: string;
  readonly options: CaptionOptions;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onTry: (style: SavedStyle) => void;
  readonly onLetGo: () => void;
}): JSX.Element {
  const [styles, setStyles] = useState(() => savedStyles());
  const [naming, setNaming] = useState<string | null>(null);
  const save = () => {
    const name = naming?.trim() ?? '';
    if (!name) return;
    setStyles(saveStyle({ name, styleRef, options }));
    setNaming(null);
  };
  return (
    <div className="edit-saved-styles" onPointerLeave={onLetGo}>
      {styles.map((saved) => (
        <span key={saved.name} className="edit-saved-style">
          <button
            type="button"
            disabled={busy}
            onPointerEnter={() => onTry(saved)}
            onClick={() => onApply(applyStyle(saved, options))}
          >
            <Bookmark className="size-3" aria-hidden="true" />
            {saved.name}
          </button>
          <button
            type="button"
            aria-label={`Forget ${saved.name}`}
            onClick={() => setStyles(forgetStyle(saved.name))}
          >
            <Trash2 className="size-3" aria-hidden="true" />
          </button>
        </span>
      ))}
      {naming === null ? (
        <Button size="xs" variant="ghost" onClick={() => setNaming('')}>
          Save this style…
        </Button>
      ) : (
        <form
          className="edit-inline"
          onSubmit={(event) => {
            event.preventDefault();
            save();
          }}
        >
          <Input
            autoFocus
            aria-label="Style name"
            placeholder="Style name"
            value={naming}
            maxLength={40}
            onChange={(event) => setNaming(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Escape') setNaming(null);
            }}
          />
          <Button size="xs" type="submit" disabled={!naming.trim()}>
            Save
          </Button>
        </form>
      )}
    </div>
  );
}
