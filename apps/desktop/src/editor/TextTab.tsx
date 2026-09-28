/**
 * The Text tab: a hook title that opens the clip, labels anywhere else, and
 * emoji — picked one at a time, or put on the words that call for them.
 *
 * Each is one overlay the document keeps in program time. Adding one selects
 * it; everything about it — words or emoji, look, size, place and when it is
 * up — is changed here, and its place can also be dragged on the preview.
 */
import { Heading1, Plus, Sparkles, Trash2 } from 'lucide-react';
import { useEffect, useState } from 'react';

import { Button } from '../components/ui/button.js';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../components/ui/select.js';
import type { EditCommandJson, PreviewOverlay, PreviewPlan } from '../daemon/client.js';
import { clockTenths } from '../inspector/review.js';
import { EmojiPicture } from './EmojiPicture.js';
import { batch } from './commands.js';
import { CommitSlider, Field, SpanTiming, Swatch } from './controls.js';
import { EMOJI, EMOJI_SIZES, emojiOf, emojiOverlay, emojiOverlays, keywordEmoji } from './emoji.js';
import {
  type Overlay,
  TEXT_LOOKS,
  TEXT_PLACES,
  TEXT_SIZES,
  addOverlay,
  breakLines,
  charactersPerLine,
  freshOverlayId,
  hookOverlay,
  labelOverlay,
  overflows,
  removeOverlay,
  savedOverlay,
  setOverlay,
  withContent,
} from './overlays.js';
import type { EditorSelection } from './selection.js';
import { frameOfTicks, programTicks, ticksOfFrame } from './timeline.js';
import type { ProgramWord } from './transcript.js';

/** The emoji shown before the rest are asked for: one row. */
const FIRST_EMOJI = 8;

export interface TextTabProps {
  readonly plan: PreviewPlan;
  readonly frame: number;
  readonly selection: EditorSelection;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onSelect: (selection: EditorSelection) => void;
  readonly onSeek: (frame: number) => void;
  /** What a new hook title says until it is changed. */
  readonly hook: string;
  /** The clip's words in program time, for emoji on the ones that call for one. */
  readonly words?: readonly ProgramWord[];
  /** Where a pinned emoji's picture loads from; absent shows the character. */
  readonly emojiUrl?: ((code: string) => string) | null;
}

export function TextTab({
  plan,
  frame,
  selection,
  busy,
  onApply,
  onSelect,
  onSeek,
  hook,
  words = [],
  emojiUrl = null,
}: TextTabProps) {
  const overlays = plan.overlays ?? [];
  const selected =
    selection.kind === 'overlay'
      ? overlays.find((overlay) => overlay.overlayId === selection.overlayId)
      : undefined;
  const hasHook = overlays.some((overlay) => overlay.kind !== 'emoji' && overlay.role === 'hook');
  const ids = overlays.map((overlay) => ({ overlay_id: overlay.overlayId }));
  const add = (overlay: Overlay) => {
    onApply(addOverlay(overlay));
    onSelect({ kind: 'overlay', overlayId: overlay.overlay_id });
    onSeek(frameOfTicks(plan, overlay.start_ticks));
  };
  const [everyEmoji, setEveryEmoji] = useState(false);
  const [emojiNote, setEmojiNote] = useState<string | null>(null);
  const placed = emojiOverlays(plan);
  const onWords = () => {
    const made = keywordEmoji(words, plan);
    if (made.length === 0) {
      setEmojiNote(
        words.length === 0
          ? 'This clip has no transcript to find words in.'
          : 'No word in this clip calls for one that does not already have one near it.',
      );
      return;
    }
    onApply(batch(made.map((overlay) => addOverlay(overlay))));
    setEmojiNote(
      `Added ${made.length} where words call for one. Undo takes them all back together.`,
    );
  };

  return (
    <div className="review-panel-body">
      <section className="review-section">
        <h3 className="review-section-title">Text</h3>
        <div className="edit-inline">
          <Button
            size="sm"
            variant="outline"
            disabled={busy || hasHook}
            onClick={() => add(hookOverlay(freshOverlayId(ids), hook, plan))}
          >
            <Heading1 className="size-4" aria-hidden="true" />
            Add a hook title
          </Button>
          <Button
            size="sm"
            variant="outline"
            disabled={busy}
            onClick={() => add(labelOverlay(freshOverlayId(ids), ticksOfFrame(plan, frame), plan))}
          >
            <Plus className="size-4" aria-hidden="true" />
            Add text here
          </Button>
        </div>
        <p className="review-footnote">
          {hasHook
            ? 'The hook opens the clip. Add more text at the playhead, and drag any of it on the preview.'
            : 'A hook title says in a few words what the clip is about, over its opening seconds.'}
        </p>
      </section>

      <section className="review-section">
        <h3 className="review-section-title">Emoji</h3>
        <div className="edit-emoji-grid" role="group" aria-label="Emoji">
          {(everyEmoji ? EMOJI : EMOJI.slice(0, FIRST_EMOJI)).map((entry) => (
            <button
              key={entry.code}
              type="button"
              title={entry.label}
              aria-label={`Add ${entry.label}`}
              disabled={busy}
              onClick={() =>
                add(emojiOverlay(freshOverlayId(ids), entry.code, ticksOfFrame(plan, frame), plan))
              }
            >
              <EmojiPicture code={entry.code} url={emojiUrl} />
            </button>
          ))}
        </div>
        <div className="edit-inline">
          <Button size="sm" variant="ghost" onClick={() => setEveryEmoji(!everyEmoji)}>
            {everyEmoji ? 'Fewer' : `All ${EMOJI.length}`}
          </Button>
          <Button size="sm" variant="outline" disabled={busy} onClick={onWords}>
            <Sparkles className="size-4" aria-hidden="true" />
            On key words
          </Button>
          {placed.length > 0 && (
            <Button
              size="sm"
              variant="ghost"
              disabled={busy}
              onClick={() => {
                onApply(batch(placed.map((overlay) => removeOverlay(overlay.overlayId))));
                if (selected?.kind === 'emoji') onSelect({ kind: 'clip' });
                setEmojiNote(null);
              }}
            >
              <Trash2 className="size-4" aria-hidden="true" />
              Remove all {placed.length}
            </Button>
          )}
        </div>
        <p className="review-footnote">
          {emojiNote ??
            'Pick one to put it at the playhead for a moment. On key words puts one where a word calls for it — money, idea, mistake — at least three seconds apart.'}
        </p>
      </section>

      {overlays.length > 0 && (
        <section className="review-section">
          <h3 className="review-section-title">On the picture</h3>
          <ul className="edit-text-list" aria-label="Texts and emoji">
            {overlays.map((overlay) => (
              <li key={overlay.overlayId}>
                <button
                  type="button"
                  aria-pressed={selected?.overlayId === overlay.overlayId}
                  onClick={() => {
                    onSelect({ kind: 'overlay', overlayId: overlay.overlayId });
                    onSeek(overlay.firstFrame);
                  }}
                >
                  <span className="edit-text-role">
                    {overlay.kind === 'emoji' ? 'Emoji' : overlay.role === 'hook' ? 'Hook' : 'Text'}
                  </span>
                  {overlay.kind === 'emoji' ? (
                    <span className="edit-text-words">
                      <EmojiPicture
                        code={overlay.emoji ?? ''}
                        url={emojiUrl}
                        className="edit-emoji-inline"
                      />{' '}
                      {emojiOf(overlay.emoji)?.label ?? 'Emoji'}
                    </span>
                  ) : (
                    <span className="edit-text-words">{overlay.text.replace(/\n/g, ' ')}</span>
                  )}
                  <span className="edit-text-time mono">
                    {clockTenths(overlay.startTicks)}–{clockTenths(overlay.endTicks)}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        </section>
      )}
      {selected?.kind === 'emoji' ? (
        <EmojiControls
          key={selected.overlayId}
          plan={plan}
          frame={frame}
          overlay={selected}
          busy={busy}
          emojiUrl={emojiUrl}
          onApply={onApply}
          onRemoved={() => onSelect({ kind: 'clip' })}
        />
      ) : (
        selected && (
          <TextControls
            key={selected.overlayId}
            plan={plan}
            frame={frame}
            overlay={selected}
            busy={busy}
            onApply={onApply}
            onRemoved={() => onSelect({ kind: 'clip' })}
          />
        )
      )}
    </div>
  );
}

function EmojiControls({
  plan,
  frame,
  overlay,
  busy,
  emojiUrl,
  onApply,
  onRemoved,
}: {
  readonly plan: PreviewPlan;
  readonly frame: number;
  readonly overlay: PreviewOverlay;
  readonly busy: boolean;
  readonly emojiUrl: ((code: string) => string) | null;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onRemoved: () => void;
}) {
  const saved = savedOverlay(overlay);
  const change = (next: Overlay) => onApply(setOverlay(next));
  return (
    <>
      <section className="review-section">
        <h3 className="review-section-title">This emoji</h3>
        <Field label="Emoji">
          <Select
            value={overlay.emoji ?? ''}
            disabled={busy}
            onValueChange={(emoji) => change(withContent(saved, { emoji }))}
          >
            <SelectTrigger aria-label="Which emoji" className="h-8 w-[190px] text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {EMOJI.map((entry) => (
                <SelectItem key={entry.code} value={entry.code}>
                  <EmojiPicture code={entry.code} url={emojiUrl} className="edit-emoji-inline" />{' '}
                  {entry.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
        <Field label="Size">
          <CommitSlider
            label="Emoji size"
            min={EMOJI_SIZES.min / 10}
            max={EMOJI_SIZES.max / 10}
            value={Math.round(overlay.size / 10)}
            disabled={busy}
            format={(value) => `${value}%`}
            onCommit={(value) => change(withContent(saved, { size: value * 10 }))}
          />
        </Field>
        <Field label="Place">
          <div className="review-segmented" role="group" aria-label="Emoji place">
            {TEXT_PLACES.map((place) => (
              <button
                key={place.name}
                type="button"
                aria-pressed={overlay.x === 500 && overlay.y === place.y}
                disabled={busy}
                onClick={() => change(withContent(saved, { x: 500, y: place.y }))}
              >
                {place.name}
              </button>
            ))}
          </div>
        </Field>
        <p className="review-footnote">
          Its size is a share of the frame’s short side. Drag it on the preview to put it anywhere.
        </p>
      </section>

      <section className="review-section">
        <h3 className="review-section-title">When it shows</h3>
        <SpanTiming
          noun="Emoji"
          startTicks={overlay.startTicks}
          endTicks={overlay.endTicks}
          programEnd={programTicks(plan)}
          playhead={ticksOfFrame(plan, frame)}
          busy={busy}
          onCommit={(start, end) => change({ ...saved, start_ticks: start, end_ticks: end })}
        />
      </section>

      <Button
        size="sm"
        variant="outline"
        className="self-start"
        disabled={busy}
        onClick={() => {
          onApply(removeOverlay(overlay.overlayId));
          onRemoved();
        }}
      >
        <Trash2 className="size-4" aria-hidden="true" />
        Remove this emoji
      </Button>
    </>
  );
}

function TextControls({
  plan,
  frame,
  overlay,
  busy,
  onApply,
  onRemoved,
}: {
  readonly plan: PreviewPlan;
  readonly frame: number;
  readonly overlay: PreviewOverlay;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onRemoved: () => void;
}) {
  const saved = savedOverlay(overlay);
  const change = (next: Overlay) => onApply(setOverlay(next));
  const [words, setWords] = useState(overlay.text);
  useEffect(() => setWords(overlay.text), [overlay.text]);
  const commitWords = () => {
    const text = words.replace(/[{}\\]/g, '').replace(/\n{2,}/g, '\n');
    if (text.trim() && text !== overlay.text) change(withContent(saved, { text }));
    else setWords(overlay.text);
  };
  const wide = overflows(words, overlay.size, plan);
  const programEnd = programTicks(plan);
  const look = TEXT_LOOKS.find(
    (item) =>
      item.colour.toLowerCase() === overlay.colour.toLowerCase() &&
      (item.plate ?? '').toLowerCase() === (overlay.plate ?? '').toLowerCase(),
  );

  return (
    <>
      <section className="review-section">
        <h3 className="review-section-title">{overlay.role === 'hook' ? 'Hook title' : 'Text'}</h3>
        <label className="edit-field-stack">
          <span className="edit-field-label">Words</span>
          <textarea
            className="edit-text-input"
            aria-label="Text words"
            rows={4}
            value={words}
            disabled={busy}
            onChange={(event) => setWords(event.target.value)}
            onBlur={commitWords}
            onKeyDown={(event) => {
              if (event.key === 'Enter' && (event.metaKey || event.ctrlKey)) {
                event.preventDefault();
                commitWords();
              }
              if (event.key === 'Escape') setWords(overlay.text);
            }}
          />
        </label>
        {wide && (
          <p className="review-footnote text-[var(--cm-warning-ink)]">
            A line is longer than the frame holds at this size.{' '}
            <button
              type="button"
              className="edit-link"
              disabled={busy}
              onClick={() =>
                change(
                  withContent(saved, {
                    text: breakLines(words, charactersPerLine(overlay.size, plan)),
                  }),
                )
              }
            >
              Break it to fit
            </button>
          </p>
        )}
        <p className="review-footnote">
          Lines break where you press Return. Set in the caption font, drawn as the export burns it
          in.
        </p>
      </section>

      <section className="review-section">
        <h3 className="review-section-title">Look</h3>
        <div className="edit-text-looks" role="group" aria-label="Text look">
          {TEXT_LOOKS.map((item) => (
            <button
              key={item.name}
              type="button"
              title={item.name}
              aria-label={item.name}
              aria-pressed={look?.name === item.name}
              disabled={busy}
              style={{
                color: item.colour,
                background: item.plate ?? 'transparent',
                textShadow: item.plate ? undefined : '0 0 2px #000, 0 0 2px #000',
              }}
              onClick={() => change(withContent(saved, { colour: item.colour, plate: item.plate }))}
            >
              Aa
            </button>
          ))}
        </div>
        <Field label="Colour">
          <Swatch
            label="Text colour"
            value={overlay.colour}
            disabled={busy}
            onCommit={(colour) => change(withContent(saved, { colour }))}
          />
        </Field>
        <Field label="Plate">
          <div className="flex items-center gap-2">
            <div className="review-segmented" role="group" aria-label="Plate">
              <button
                type="button"
                aria-pressed={!overlay.plate}
                disabled={busy}
                onClick={() => change(withContent(saved, { plate: undefined }))}
              >
                Outline
              </button>
              <button
                type="button"
                aria-pressed={Boolean(overlay.plate)}
                disabled={busy}
                onClick={() => change(withContent(saved, { plate: overlay.plate ?? '#111111' }))}
              >
                Plate
              </button>
            </div>
            {overlay.plate && (
              <Swatch
                label="Plate colour"
                value={overlay.plate}
                disabled={busy}
                onCommit={(plate) => change(withContent(saved, { plate }))}
              />
            )}
          </div>
        </Field>
        <Field label="Size">
          <CommitSlider
            label="Text size"
            min={TEXT_SIZES.min}
            max={TEXT_SIZES.max}
            value={overlay.size}
            disabled={busy}
            format={(value) => `${value} px`}
            onCommit={(size) => change(withContent(saved, { size }))}
          />
        </Field>
        <Field label="Place">
          <div className="review-segmented" role="group" aria-label="Text place">
            {TEXT_PLACES.map((place) => (
              <button
                key={place.name}
                type="button"
                aria-pressed={overlay.x === 500 && overlay.y === place.y}
                disabled={busy}
                onClick={() => change(withContent(saved, { x: 500, y: place.y }))}
              >
                {place.name}
              </button>
            ))}
          </div>
        </Field>
        <p className="review-footnote">Drag it on the preview to put it anywhere.</p>
      </section>

      <section className="review-section">
        <h3 className="review-section-title">When it shows</h3>
        <SpanTiming
          startTicks={overlay.startTicks}
          endTicks={overlay.endTicks}
          programEnd={programEnd}
          playhead={ticksOfFrame(plan, frame)}
          busy={busy}
          onCommit={(start, end) => change({ ...saved, start_ticks: start, end_ticks: end })}
        />
      </section>

      <Button
        size="sm"
        variant="outline"
        className="self-start"
        disabled={busy}
        onClick={() => {
          onApply(removeOverlay(overlay.overlayId));
          onRemoved();
        }}
      >
        <Trash2 className="size-4" aria-hidden="true" />
        Remove this text
      </Button>
    </>
  );
}
