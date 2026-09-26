/**
 * The Text tab: a hook title that opens the clip, and labels anywhere else.
 *
 * Each text is one overlay the document keeps in program time. Adding one
 * selects it; everything about it — words, look, size, place and when it is
 * up — is changed here, and its place can also be dragged on the preview.
 */
import { Heading1, Plus, Trash2 } from 'lucide-react';
import { useEffect, useState } from 'react';

import { Button } from '../components/ui/button.js';
import { Input } from '../components/ui/input.js';
import type { EditCommandJson, PreviewOverlay, PreviewPlan } from '../daemon/client.js';
import { clockTenths } from '../inspector/review.js';
import { CommitSlider, Field, Swatch } from './controls.js';
import {
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
import { frameOfTicks, ticksOfFrame } from './timeline.js';

const TICKS = 90_000;

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
}: TextTabProps) {
  const overlays = plan.overlays ?? [];
  const selected =
    selection.kind === 'overlay'
      ? overlays.find((overlay) => overlay.overlayId === selection.overlayId)
      : undefined;
  const hasHook = overlays.some((overlay) => overlay.role === 'hook');
  const ids = overlays.map((overlay) => ({ overlay_id: overlay.overlayId }));
  const add = (overlay: ReturnType<typeof hookOverlay>) => {
    onApply(addOverlay(overlay));
    onSelect({ kind: 'overlay', overlayId: overlay.overlay_id });
    onSeek(frameOfTicks(plan, overlay.start_ticks));
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
        {overlays.length > 0 && (
          <ul className="edit-text-list" aria-label="Texts">
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
                    {overlay.role === 'hook' ? 'Hook' : 'Text'}
                  </span>
                  <span className="edit-text-words">{overlay.text.replace(/\n/g, ' ')}</span>
                  <span className="edit-text-time mono">
                    {clockTenths(overlay.startTicks)}–{clockTenths(overlay.endTicks)}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>
      {selected && (
        <TextControls
          key={selected.overlayId}
          plan={plan}
          frame={frame}
          overlay={selected}
          busy={busy}
          onApply={onApply}
          onRemoved={() => onSelect({ kind: 'clip' })}
        />
      )}
    </div>
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
  const change = (next: typeof saved) => onApply(setOverlay(next));
  const [words, setWords] = useState(overlay.text);
  useEffect(() => setWords(overlay.text), [overlay.text]);
  const commitWords = () => {
    const text = words.replace(/[{}\\]/g, '').replace(/\n{2,}/g, '\n');
    if (text.trim() && text !== overlay.text) change(withContent(saved, { text }));
    else setWords(overlay.text);
  };
  const wide = overflows(words, overlay.size, plan);
  const programEnd = plan.segments.reduce((sum, part) => sum + part.outTicks - part.inTicks, 0);
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
        <TextTiming
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

/** Seconds typed exactly, or taken from the playhead. */
function TextTiming({
  startTicks,
  endTicks,
  programEnd,
  playhead,
  busy,
  onCommit,
}: {
  readonly startTicks: number;
  readonly endTicks: number;
  readonly programEnd: number;
  readonly playhead: number;
  readonly busy: boolean;
  readonly onCommit: (start: number, end: number) => void;
}) {
  const [start, setStart] = useState(String(startTicks / TICKS));
  const [end, setEnd] = useState(String(endTicks / TICKS));
  useEffect(() => {
    setStart(String(startTicks / TICKS));
    setEnd(String(endTicks / TICKS));
  }, [startTicks, endTicks]);
  const valid = (from: number, to: number) =>
    Number.isFinite(from) && Number.isFinite(to) && from >= 0 && to > from && to <= programEnd;
  const commit = (from: number, to: number) => {
    if (valid(from, to) && (from !== startTicks || to !== endTicks)) onCommit(from, to);
  };
  const typed = (value: string) => Math.round(Number(value) * TICKS);
  return (
    <div className="space-y-2">
      <div className="flex items-end gap-2">
        <label className="edit-field-stack">
          <span className="edit-field-label">From</span>
          <Input
            aria-label="Text starts at"
            className="h-8 w-24 font-mono"
            value={start}
            disabled={busy}
            inputMode="decimal"
            onChange={(event) => setStart(event.target.value)}
            onBlur={() => commit(typed(start), endTicks)}
          />
        </label>
        <label className="edit-field-stack">
          <span className="edit-field-label">To</span>
          <Input
            aria-label="Text ends at"
            className="h-8 w-24 font-mono"
            value={end}
            disabled={busy}
            inputMode="decimal"
            onChange={(event) => setEnd(event.target.value)}
            onBlur={() => commit(startTicks, typed(end))}
          />
        </label>
        <span className="pb-2 text-xs text-[var(--cm-text-muted)]">seconds</span>
      </div>
      <div className="edit-inline">
        <Button
          size="sm"
          variant="ghost"
          disabled={busy || !valid(playhead, endTicks)}
          onClick={() => commit(playhead, endTicks)}
        >
          Start at the playhead
        </Button>
        <Button
          size="sm"
          variant="ghost"
          disabled={busy || !valid(startTicks, playhead)}
          onClick={() => commit(startTicks, playhead)}
        >
          End at the playhead
        </Button>
      </div>
    </div>
  );
}
