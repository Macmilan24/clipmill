/**
 * The Framing tab's B-roll: a picture or other footage over a moment of the
 * clip while the voice carries on. Added at the playhead, listed in program
 * order, and each one's fit, motion, recording and timing changed here.
 */
import { Clapperboard, ImagePlus, Trash2 } from 'lucide-react';
import { useEffect, useState } from 'react';

import type { EditIr } from '@clipmill/contracts';

import { Button } from '../components/ui/button.js';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../components/ui/select.js';
import type {
  Asset,
  AssetLicense,
  EditCommandJson,
  PreviewCutaway,
  PreviewPlan,
} from '../daemon/client.js';
import { clockTenths } from '../inspector/review.js';
import { type AssetAccess, LICENSES, withAsset } from './brand.js';
import { CommitSlider, Field, SpanTiming } from './controls.js';
import {
  type Cutaway,
  SHORTEST_CUTAWAY,
  footageCutaway,
  pictureCutaway,
  recordingTicks,
  savedCutaway,
  savedCutaways,
  setCutaways,
  withCutaway,
} from './cutaways.js';
import type { EditorSelection } from './selection.js';
import { frameOfTicks, programTicks, ticksOfFrame } from './timeline.js';

const SECOND = 90_000;

/** A recording of the project that footage can be cut from. */
export interface Recording {
  readonly fingerprint: string;
  readonly name: string;
  /** The file is not where it was registered; the export needs it relinked. */
  readonly missing?: boolean;
}

/** How the editor reaches the project's recordings. */
export interface RecordingAccess {
  readonly list: () => Promise<readonly Recording[]>;
}

export function CutawaysSection({
  plan,
  document,
  frame,
  selection,
  busy,
  onApply,
  onSelect,
  onSeek,
  assets,
  recordings,
}: {
  readonly plan: PreviewPlan;
  readonly document: EditIr | null;
  readonly frame: number;
  readonly selection: EditorSelection;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onSelect: (selection: EditorSelection) => void;
  readonly onSeek: (frame: number) => void;
  readonly assets: AssetAccess | null;
  readonly recordings: RecordingAccess | null;
}) {
  const shown = plan.cutaways ?? [];
  const list = savedCutaways(plan);
  const [picking, setPicking] = useState<'picture' | 'footage' | null>(null);
  const [note, setNote] = useState<string | null>(null);
  const [recorded, setRecorded] = useState<readonly Recording[]>([]);
  useEffect(() => {
    if (!recordings) return undefined;
    let current = true;
    void recordings
      .list()
      .then((found) => {
        if (current) setRecorded(found);
      })
      .catch(() => {});
    return () => {
      current = false;
    };
  }, [recordings]);
  const selected =
    selection.kind === 'cutaway'
      ? shown.find((cutaway) => cutaway.cutawayId === selection.cutawayId)
      : undefined;
  const place = (cutaway: Cutaway | null, withAssets?: NonNullable<EditIr['assets']>) => {
    if (!cutaway) {
      setNote(
        'There is no room at the playhead: another cutaway covers it, or the clip ends too soon after it.',
      );
      return;
    }
    setNote(null);
    setPicking(null);
    onApply(setCutaways(withCutaway(list, cutaway), withAssets));
    onSelect({ kind: 'cutaway', cutawayId: cutaway.cutaway_id });
    onSeek(frameOfTicks(plan, cutaway.start_ticks));
  };
  const playhead = ticksOfFrame(plan, frame);
  const nameOf = (cutaway: PreviewCutaway) =>
    cutaway.kind === 'footage'
      ? (recorded.find((item) => item.fingerprint === cutaway.sourceFingerprint)?.name ?? 'Footage')
      : 'Picture';

  return (
    <>
      <section className="review-section">
        <h3 className="review-section-title">B-roll</h3>
        <div className="edit-inline">
          {/* A picture joins the clip's asset list, which is read from the
              document, so it waits for the document to load. */}
          <Button
            size="sm"
            variant="outline"
            aria-pressed={picking === 'picture'}
            disabled={busy || !document}
            onClick={() => setPicking(picking === 'picture' ? null : 'picture')}
          >
            <ImagePlus className="size-4" aria-hidden="true" />
            Add a picture
          </Button>
          <Button
            size="sm"
            variant="outline"
            aria-pressed={picking === 'footage'}
            disabled={busy}
            onClick={() => setPicking(picking === 'footage' ? null : 'footage')}
          >
            <Clapperboard className="size-4" aria-hidden="true" />
            Add footage
          </Button>
        </div>
        {picking === 'picture' && (
          <PicturePicker
            assets={assets}
            busy={busy}
            onPick={(picture) =>
              place(
                pictureCutaway(plan, list, picture.hash, playhead),
                withAsset(document?.assets, picture),
              )
            }
          />
        )}
        {picking === 'footage' && (
          <RecordingPicker
            recordings={recordings ? recorded : null}
            busy={busy}
            onPick={(recording) =>
              place(footageCutaway(plan, list, recording.fingerprint, 0, playhead))
            }
          />
        )}
        <p className="review-footnote">
          {note ??
            'Cover a moment with a picture or other footage while the voice carries on. It goes in at the playhead, for up to two and a half seconds.'}
        </p>
        {shown.length > 0 && (
          <ul className="edit-text-list" aria-label="Cutaways">
            {shown.map((cutaway) => (
              <li key={cutaway.cutawayId}>
                <button
                  type="button"
                  aria-pressed={selected?.cutawayId === cutaway.cutawayId}
                  onClick={() => {
                    onSelect({ kind: 'cutaway', cutawayId: cutaway.cutawayId });
                    onSeek(cutaway.firstFrame);
                  }}
                >
                  <span className="edit-text-role">
                    {cutaway.kind === 'footage' ? 'Footage' : 'Picture'}
                  </span>
                  <span className="edit-text-words">{nameOf(cutaway)}</span>
                  <span className="edit-text-time mono">
                    {clockTenths(cutaway.startTicks)}–{clockTenths(cutaway.endTicks)}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>
      {selected && (
        <CutawayControls
          key={selected.cutawayId}
          plan={plan}
          cutaway={selected}
          list={list}
          playhead={playhead}
          busy={busy}
          onApply={onApply}
          onRemoved={() => onSelect({ kind: 'clip' })}
        />
      )}
    </>
  );
}

function PicturePicker({
  assets,
  busy,
  onPick,
}: {
  readonly assets: AssetAccess | null;
  readonly busy: boolean;
  readonly onPick: (picture: Asset) => void;
}) {
  const [pictures, setPictures] = useState<readonly Asset[]>([]);
  const [license, setLicense] = useState<AssetLicense>('own_content');
  const [bringing, setBringing] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  useEffect(() => {
    if (!assets) return undefined;
    let current = true;
    void assets
      .list('image')
      .then((found) => {
        if (current) setPictures(found);
      })
      .catch(() => {});
    return () => {
      current = false;
    };
  }, [assets]);
  if (!assets) {
    return <p className="review-footnote">Pictures can be brought in in the ClipMill app.</p>;
  }
  const bring = async () => {
    setBringing(true);
    setProblem(null);
    try {
      const picture = await assets.bring('image', license);
      if (picture) onPick(picture);
    } catch (error) {
      setProblem(error instanceof Error ? error.message : String(error));
    } finally {
      setBringing(false);
    }
  };
  return (
    <>
      {pictures.length > 0 && (
        <div className="edit-brand-pictures" role="group" aria-label="Your pictures">
          {pictures.slice(0, 8).map((picture) => (
            <button
              key={picture.hash}
              type="button"
              title={picture.name}
              aria-label={`Show ${picture.name}`}
              disabled={busy}
              onClick={() => onPick(picture)}
            >
              <img src={assets.url(picture.hash)} alt="" />
            </button>
          ))}
        </div>
      )}
      <div className="flex flex-wrap items-center gap-2">
        <Select value={license} onValueChange={(value) => setLicense(value as AssetLicense)}>
          <SelectTrigger aria-label="Whose picture it is" className="h-8 w-[190px] text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {LICENSES.map((item) => (
              <SelectItem key={item.value} value={item.value}>
                {item.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Button size="sm" variant="ghost" disabled={busy || bringing} onClick={() => void bring()}>
          {bringing ? 'Bringing it in…' : 'Bring one in'}
        </Button>
      </div>
      {problem && <p className="review-footnote text-[var(--cm-warning-ink)]">{problem}</p>}
    </>
  );
}

function RecordingPicker({
  recordings,
  busy,
  onPick,
}: {
  readonly recordings: readonly Recording[] | null;
  readonly busy: boolean;
  readonly onPick: (recording: Recording) => void;
}) {
  if (!recordings) {
    return <p className="review-footnote">Footage can be added in the ClipMill app.</p>;
  }
  if (recordings.length === 0) {
    return <p className="review-footnote">This project has no recordings yet.</p>;
  }
  return (
    <ul className="edit-sounds" aria-label="Recordings">
      {recordings.map((recording) => (
        <li key={recording.fingerprint}>
          <button type="button" disabled={busy} onClick={() => onPick(recording)}>
            <Clapperboard className="size-3.5" aria-hidden="true" />
            <span className="edit-sound-name">{recording.name}</span>
            {recording.missing && <span className="edit-sound-length">moved</span>}
          </button>
        </li>
      ))}
    </ul>
  );
}

function CutawayControls({
  plan,
  cutaway,
  list,
  playhead,
  busy,
  onApply,
  onRemoved,
}: {
  readonly plan: PreviewPlan;
  readonly cutaway: PreviewCutaway;
  readonly list: readonly Cutaway[];
  readonly playhead: number;
  readonly busy: boolean;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onRemoved: () => void;
}) {
  const saved = savedCutaway(cutaway);
  const change = (next: Cutaway) => onApply(setCutaways(withCutaway(list, next)));
  const end = programTicks(plan);
  // Its neighbours bound where it can be moved to.
  const others = list.filter((item) => item.cutaway_id !== cutaway.cutawayId);
  const after = others
    .filter((item) => item.start_ticks >= cutaway.endTicks)
    .reduce((first, item) => Math.min(first, item.start_ticks), end);
  const before = others
    .filter((item) => item.end_ticks <= cutaway.startTicks)
    .reduce((last, item) => Math.max(last, item.end_ticks), 0);
  const recording = recordingTicks(plan, cutaway.sourceFingerprint);
  const span = cutaway.endTicks - cutaway.startTicks;
  const latestIn = recording === null ? null : Math.max(0, recording - span);

  return (
    <>
      <section className="review-section">
        <h3 className="review-section-title">This cutaway</h3>
        <Field label="Fit">
          <div className="review-segmented" role="group" aria-label="Cutaway fit">
            {(
              [
                ['fill', 'Fill'],
                ['fit', 'Whole'],
              ] as const
            ).map(([fit, label]) => (
              <button
                key={fit}
                type="button"
                aria-pressed={cutaway.fit === fit}
                disabled={busy}
                onClick={() => {
                  const { fit: _fit, ...rest } = saved;
                  change(fit === 'fit' ? { ...rest, fit } : rest);
                }}
              >
                {label}
              </button>
            ))}
          </div>
        </Field>
        {saved.content.kind === 'picture' && (
          <Field label="Motion">
            <div className="review-segmented" role="group" aria-label="Picture motion">
              {(
                [
                  [false, 'Still'],
                  [true, 'Push in'],
                ] as const
              ).map(([push, label]) => (
                <button
                  key={label}
                  type="button"
                  aria-pressed={cutaway.pushIn === push}
                  disabled={busy}
                  onClick={() => {
                    if (saved.content.kind !== 'picture') return;
                    const { push_in: _push, ...content } = saved.content;
                    change({ ...saved, content: push ? { ...content, push_in: true } : content });
                  }}
                >
                  {label}
                </button>
              ))}
            </div>
          </Field>
        )}
        {saved.content.kind === 'footage' && latestIn !== null && latestIn > 0 && (
          <Field label="From">
            <CommitSlider
              label="Where in the recording it starts"
              min={0}
              max={Math.floor(latestIn / SECOND)}
              value={Math.round(cutaway.inTicks / SECOND)}
              disabled={busy}
              format={(seconds) => clockTenths(seconds * SECOND)}
              onCommit={(seconds) => {
                if (saved.content.kind !== 'footage') return;
                change({
                  ...saved,
                  content: { ...saved.content, in_ticks: Math.min(seconds * SECOND, latestIn) },
                });
              }}
            />
          </Field>
        )}
        <p className="review-footnote">
          {saved.content.kind === 'footage'
            ? 'Its own sound is left out; the clip’s voice carries on over it.'
            : 'Fill covers the frame and crops what does not fit; Whole shows all of it over a blurred copy.'}
        </p>
      </section>

      <section className="review-section">
        <h3 className="review-section-title">When it shows</h3>
        <SpanTiming
          noun="Cutaway"
          startTicks={cutaway.startTicks}
          endTicks={cutaway.endTicks}
          programEnd={end}
          playhead={playhead}
          busy={busy}
          onCommit={(start, stop) => {
            // Never over its neighbours, and never shorter than a cutaway can be.
            const from = Math.max(start, before);
            const to = Math.min(stop, after);
            if (to - from < SHORTEST_CUTAWAY) return;
            const moved = { ...saved, start_ticks: from, end_ticks: to };
            // Footage whose start moves keeps each moment where it was: it
            // starts that much earlier or later in its recording.
            if (moved.content.kind === 'footage' && from !== cutaway.startTicks) {
              moved.content = {
                ...moved.content,
                in_ticks: Math.max(0, moved.content.in_ticks + from - cutaway.startTicks),
              };
            }
            change(moved);
          }}
        />
      </section>

      <Button
        size="sm"
        variant="outline"
        className="self-start"
        disabled={busy}
        onClick={() => {
          onApply(setCutaways(others));
          onRemoved();
        }}
      >
        <Trash2 className="size-4" aria-hidden="true" />
        Remove this cutaway
      </Button>
    </>
  );
}
