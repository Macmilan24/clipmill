/**
 * The Editor's timeline, drawn like the Inspector's strip: a ruler, the
 * picture, captions, framing and sound on the program clock, with the source
 * beyond each end of the clip washed out and ready to pull in. Every drag shows
 * a ghost and sends one command on release; a click only selects or seeks.
 */
import {
  Bookmark,
  Magnet,
  Maximize2,
  MousePointer2,
  Scissors,
  SquareSplitHorizontal,
  Trash2,
  X,
  ZoomIn,
  ZoomOut,
} from 'lucide-react';
import {
  type PointerEvent as ReactPointerEvent,
  type RefObject,
  memo,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import type { EditIr } from '@clipmill/contracts';

import type { EditCommandJson, PreviewPlan, PreviewSegment } from '../daemon/client.js';
import { type Cut, clamp, clockTenths, rulerStep } from '../inspector/review.js';
import { TipButton } from '../inspector/TipButton.js';
import type { Filmstrip, Peaks } from '../results/loader.js';
import { type Transcript, snapEnd, snapStart } from '../results/transcript.js';
import { waveformPath } from '../results/waveform.js';
import {
  batch,
  extendWithCaptions,
  removeCropKeyframe,
  removeGainPoint,
  setCropKeyframe,
  setCueTiming,
  setGain,
  trim,
} from './commands.js';
import { pressOrDrag } from './gesture.js';
import { type MenuTarget, TimelineMenu } from './TimelineMenu.js';
import type { EditorSelection } from './selection.js';
import {
  TICKS,
  extentOf,
  frameOfTicks,
  panSpan,
  programTicks,
  reach,
  sourceAt,
  ticksOfFrame,
  zoomSpan,
} from './timeline.js';
import { shownCues, rippleRange } from './transcript.js';

export type Tool = 'select' | 'blade';

/** How close, in pixels, an edge must come to something to snap to it. */
const SNAP_PIXELS = 8;
/** What the framing lane calls each layout. */
const LAYOUT_NAMES: Record<string, string> = {
  fit: 'Whole frame',
  speaker_fill: 'Follow speaker',
  two_up: 'Two speakers',
  picture_in_picture: 'Picture in picture',
};
/** The volume line's range, in dB either side of unchanged. */
const GAIN_RANGE = 12;

export interface EditorTimelineProps {
  readonly plan: PreviewPlan;
  readonly document: EditIr | null;
  readonly transcript: Transcript | null;
  readonly filmstrip: Filmstrip | null;
  readonly peaks: Peaks | null;
  readonly filmstripUrl: (file: string) => string;
  readonly frame: number;
  readonly playing: boolean;
  readonly view: Cut;
  readonly onView: (view: Cut) => void;
  readonly selection: EditorSelection;
  readonly busy: boolean;
  readonly tool: Tool;
  readonly onTool: (tool: Tool) => void;
  readonly snap: boolean;
  readonly onSnap: (snap: boolean) => void;
  readonly marks: { readonly in: number | null; readonly out: number | null };
  readonly onClearMarks: () => void;
  readonly markers: readonly number[];
  readonly onMarker: () => void;
  readonly onSeek: (frame: number) => void;
  readonly onSelect: (selection: EditorSelection) => void;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onSplit: (frame: number) => void;
  readonly onDeleteRange: () => void;
}

type Ghost =
  | { readonly kind: 'edge'; readonly ticks: number }
  | { readonly kind: 'cue'; readonly cueId: string; readonly start: number; readonly end: number }
  | { readonly kind: 'keyframe'; readonly id: string; readonly ticks: number }
  | { readonly kind: 'gain'; readonly index: number; readonly ticks: number; readonly db: number };

export function EditorTimeline(props: EditorTimelineProps) {
  const {
    plan,
    frame,
    playing,
    view,
    onView,
    busy,
    tool,
    onTool,
    snap,
    onSnap,
    marks,
    onClearMarks,
    onMarker,
    onSeek,
    onSplit,
    onDeleteRange,
  } = props;
  const track = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(0);
  const [hover, setHover] = useState<number | null>(null);
  const [scrubbing, setScrubbing] = useState(false);
  const extent = useMemo(() => extentOf(plan), [plan]);
  const duration = programTicks(plan);
  const span = Math.max(1, view.endTicks - view.startTicks);
  const playheadTicks = ticksOfFrame(plan, frame);

  useEffect(() => {
    const element = track.current;
    if (!element) return;
    const measure = () => setWidth(element.clientWidth);
    measure();
    if (typeof ResizeObserver === 'undefined') return;
    const watcher = new ResizeObserver(measure);
    watcher.observe(element);
    return () => watcher.disconnect();
  }, []);

  // Playback turns the page when the playhead reaches the edge of the view.
  useEffect(() => {
    if (scrubbing) return;
    if (playheadTicks < view.startTicks || playheadTicks > view.endTicks) {
      const start = playing ? playheadTicks - span * 0.1 : playheadTicks - span / 2;
      onView(panSpan(view, start - view.startTicks, extent));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- follows the playhead only
  }, [playheadTicks]);

  const playheadRef = useRef(playheadTicks);
  playheadRef.current = playheadTicks;
  const ticksAtX = useCallback(
    (clientX: number) => {
      const box = track.current?.getBoundingClientRect();
      if (!box || box.width === 0) return view.startTicks;
      return view.startTicks + clamp((clientX - box.left) / box.width, 0, 1) * span;
    },
    [view.startTicks, span],
  );
  /** Pixels to ticks at the width the track has now, not when it was last measured. */
  const pixelsToTicks = useCallback(
    (pixels: number) =>
      (pixels / Math.max(1, track.current?.getBoundingClientRect().width || width)) * span,
    [span, width],
  );
  const frameAtX = useCallback(
    (clientX: number) => frameOfTicks(plan, clamp(ticksAtX(clientX), 0, Math.max(0, duration - 1))),
    [plan, ticksAtX, duration],
  );

  // Registered by hand: React's wheel listener is passive, and a pinch the
  // timeline cannot cancel would zoom the whole window instead.
  const wheel = useRef({ view, extent, width, span, onView, ticksAtX });
  wheel.current = { view, extent, width, span, onView, ticksAtX };
  useEffect(() => {
    const element = track.current;
    if (!element) return;
    const listener = (event: WheelEvent) => {
      const current = wheel.current;
      event.preventDefault();
      if (event.ctrlKey || event.metaKey) {
        current.onView(
          zoomSpan(
            current.view,
            Math.exp(event.deltaY * 0.01),
            current.ticksAtX(event.clientX),
            current.extent,
          ),
        );
        return;
      }
      const delta = Math.abs(event.deltaX) > Math.abs(event.deltaY) ? event.deltaX : event.deltaY;
      current.onView(
        panSpan(current.view, (delta / Math.max(1, current.width)) * current.span, current.extent),
      );
    };
    element.addEventListener('wheel', listener, { passive: false });
    return () => element.removeEventListener('wheel', listener);
  }, []);

  const { onSelect } = props;
  const scrub = useCallback(
    (event: ReactPointerEvent) => {
      if (event.button > 0) return;
      if (tool === 'blade') {
        event.preventDefault();
        if (!busy) onSplit(frameAtX(event.clientX));
        return;
      }
      onSeek(frameAtX(event.clientX));
      setScrubbing(true);
      pressOrDrag(
        event,
        {
          onDrag: (_dx, _dy, next) => onSeek(frameAtX(next.clientX)),
          onDrop: () => setScrubbing(false),
          onClick: () => {
            setScrubbing(false);
            onSelect({ kind: 'clip' });
          },
          onCancel: () => setScrubbing(false),
        },
        0,
      );
    },
    [tool, busy, onSplit, onSeek, onSelect, frameAtX],
  );

  const at = useCallback(
    (ticks: number) => `${((ticks - view.startTicks) / span) * 100}%`,
    [view.startTicks, span],
  );
  const wide = useCallback((from: number, to: number) => `${((to - from) / span) * 100}%`, [span]);
  const step = rulerStep(span, width, 64);
  const rulerTicks: number[] = [];
  for (
    let tick = Math.max(0, Math.ceil(view.startTicks / step) * step);
    tick <= Math.min(duration, view.endTicks);
    tick += step
  ) {
    rulerTicks.push(tick);
  }
  const fine = step < TICKS;
  const markIn = marks.in === null ? null : ticksOfFrame(plan, marks.in);
  const markOut = marks.out === null ? null : ticksOfFrame(plan, marks.out);
  const ranged = markIn !== null && markOut !== null && markOut > markIn;
  const { before, after } = reach(plan);

  return (
    <section className="review-timeline edit-timeline" aria-label="Timeline">
      <div className="review-strip-bar edit-timeline-bar">
        <div className="review-segmented" role="group" aria-label="Timeline tool">
          <button
            type="button"
            aria-pressed={tool === 'select'}
            onClick={() => onTool('select')}
            aria-label="Select"
          >
            <MousePointer2 className="size-3.5" aria-hidden="true" />
            Select
          </button>
          <button
            type="button"
            aria-pressed={tool === 'blade'}
            onClick={() => onTool('blade')}
            aria-label="Blade"
          >
            <Scissors className="size-3.5" aria-hidden="true" />
            Blade
          </button>
        </div>
        <TipButton label="Split at the playhead" disabled={busy} onClick={() => onSplit(frame)}>
          <SquareSplitHorizontal className="size-4" />
        </TipButton>
        <TipButton
          label="Delete the marked range"
          disabled={busy || !ranged}
          onClick={onDeleteRange}
        >
          <Trash2 className="size-4" />
        </TipButton>
        <span className="review-divider" aria-hidden="true" />
        <TipButton label="Snapping" pressed={snap} onClick={() => onSnap(!snap)}>
          <Magnet className="size-4" />
        </TipButton>
        <TipButton label="Add a marker at the playhead" onClick={onMarker}>
          <Bookmark className="size-4" />
        </TipButton>
        <div className="review-cut-summary edit-timeline-summary">
          {ranged ? (
            <>
              <span className="mono">
                {clockTenths(markIn)} – {clockTenths(markOut)}
              </span>
              <span className="review-cut-length">{clockTenths(markOut - markIn)} marked</span>
              <button
                type="button"
                className="edit-chip-clear"
                onClick={onClearMarks}
                aria-label="Clear the marked range"
              >
                <X className="size-3" aria-hidden="true" />
              </button>
            </>
          ) : (
            <>
              <span className="mono">{clockTenths(duration)}</span>
              <span className="review-cut-length">
                {plan.cues.length} {plan.cues.length === 1 ? 'caption' : 'captions'}
                {plan.segments.length > 1 ? ` · ${plan.segments.length} sections` : ''}
              </span>
            </>
          )}
        </div>
        <span className="review-spacer" />
        <TipButton
          label="Zoom out"
          onClick={() => onView(zoomSpan(view, 1.5, playheadTicks, extent))}
        >
          <ZoomOut className="size-4" />
        </TipButton>
        <TipButton
          label="Zoom in"
          onClick={() => onView(zoomSpan(view, 1 / 1.5, playheadTicks, extent))}
        >
          <ZoomIn className="size-4" />
        </TipButton>
        <TipButton label="Fit the clip" onClick={() => onView(extent)}>
          <Maximize2 className="size-4" />
        </TipButton>
      </div>

      <div className="edit-tracks">
        <div className="edit-track-labels" aria-hidden="true">
          <span />
          <span>Video</span>
          <span>Captions</span>
          <span>Text</span>
          <span>Framing</span>
          <span>Audio</span>
        </div>
        <div
          ref={track}
          className="review-strip-track review-strip edit-track"
          role="group"
          aria-label="Clip timeline"
          data-tool={tool}
          onPointerMove={(event) => setHover(ticksAtX(event.clientX))}
          onPointerLeave={() => setHover(null)}
        >
          <div className="review-ruler edit-ruler" onPointerDown={scrub}>
            {rulerTicks.map((tick) => (
              <span key={tick} style={{ left: at(tick) }}>
                {fine ? clockTenths(tick) : clockTenths(tick).replace(/\.\d$/, '')}
              </span>
            ))}
            {ranged && (
              <i
                className="edit-range"
                style={{ left: at(markIn), width: wide(markIn, markOut) }}
              />
            )}
            {props.markers.map((marker) => (
              <b
                key={marker}
                className="edit-marker"
                style={{ left: at(ticksOfFrame(plan, marker)) }}
                title={`Marker · ${clockTenths(ticksOfFrame(plan, marker))}`}
              />
            ))}
          </div>
          <Lanes
            plan={plan}
            document={props.document}
            transcript={props.transcript}
            filmstrip={props.filmstrip}
            peaks={props.peaks}
            filmstripUrl={props.filmstripUrl}
            view={view}
            selection={props.selection}
            busy={busy}
            snap={snap}
            markers={props.markers}
            playheadRef={playheadRef}
            onSeek={onSeek}
            onSelect={onSelect}
            onApply={props.onApply}
            onSplit={onSplit}
            at={at}
            wide={wide}
            span={span}
            width={width}
            scrub={scrub}
            ticksAtX={ticksAtX}
            pixelsToTicks={pixelsToTicks}
          />
          {before > 0 && (
            <span
              className="review-outside edit-reach"
              style={{ left: 0, width: at(0) }}
              aria-hidden="true"
            />
          )}
          {after > 0 && (
            <span
              className="review-outside edit-reach"
              style={{ left: at(duration), right: 0 }}
              aria-hidden="true"
            />
          )}
          {ranged && (
            <span
              className="edit-range-wash"
              style={{ left: at(markIn), width: wide(markIn, markOut) }}
              aria-hidden="true"
            />
          )}
          {hover !== null && !scrubbing && (
            <span className="review-hover" style={{ left: at(hover) }} aria-hidden="true">
              <span className="mono">
                {hover < 0 ? `−${clockTenths(-hover)}` : clockTenths(Math.min(hover, duration))}
              </span>
            </span>
          )}
          {playheadTicks >= view.startTicks && playheadTicks <= view.endTicks && (
            <span
              className="review-playhead"
              style={{ left: at(playheadTicks) }}
              data-testid="playhead"
            />
          )}
        </div>
      </div>
    </section>
  );
}

interface LaneProps extends Pick<
  EditorTimelineProps,
  | 'plan'
  | 'document'
  | 'transcript'
  | 'filmstrip'
  | 'peaks'
  | 'filmstripUrl'
  | 'view'
  | 'selection'
  | 'busy'
  | 'snap'
  | 'markers'
  | 'onSeek'
  | 'onSelect'
  | 'onApply'
  | 'onSplit'
> {
  /** The playhead, read when a drag needs it rather than redrawn with it. */
  readonly playheadRef: RefObject<number>;
  readonly at: (ticks: number) => string;
  readonly wide: (from: number, to: number) => string;
  readonly span: number;
  readonly width: number;
  readonly scrub: (event: ReactPointerEvent) => void;
  readonly ticksAtX: (clientX: number) => number;
  readonly pixelsToTicks: (pixels: number) => number;
}

/** The four lanes. Kept apart from the playhead so playback does not redraw them. */
const Lanes = memo(function Lanes({
  plan,
  document,
  transcript,
  filmstrip,
  peaks,
  filmstripUrl,
  view,
  selection,
  busy,
  snap,
  markers,
  playheadRef,
  onSeek,
  onSelect,
  onApply,
  onSplit,
  at,
  wide,
  span,
  width,
  scrub,
  ticksAtX,
  pixelsToTicks,
}: LaneProps) {
  const [ghost, setGhost] = useState<Ghost | null>(null);
  // What a right-click landed on, for the timeline's menu.
  const [menu, setMenu] = useState<MenuTarget | null>(null);
  const duration = programTicks(plan);
  const { before, after } = reach(plan);

  // Stills along the whole view, the reach included, one per slot.
  const tiles = useMemo(
    () => (filmstrip ? [...filmstrip.tiles].toSorted((a, b) => a.tTicks - b.tTicks) : []),
    [filmstrip],
  );
  const slots = Math.max(1, Math.floor(width / 54));
  const stills = Array.from({ length: slots }, (_, index) => {
    const ticks = view.startTicks + ((index + 0.5) / slots) * span;
    if (ticks < -before || ticks > duration + after || tiles.length === 0) return null;
    const source = sourceAt(plan, ticks);
    return source ? filmstripUrl(nearestTile(tiles, source.ticks).file) : null;
  });

  const waves = useMemo(() => {
    if (!peaks || peaks.bucketTicks <= 0) return [];
    const first = plan.segments[0];
    const last = plan.segments.at(-1);
    const spans = plan.segments.map((part) => ({
      key: part.segmentId,
      from: part.programStartTicks,
      to: part.programStartTicks + part.outTicks - part.inTicks,
      path: waveformPath(peaks, part.inTicks, part.outTicks),
      outside: false,
    }));
    if (first && before > 0)
      spans.unshift({
        key: 'before',
        from: -before,
        to: 0,
        path: waveformPath(peaks, first.inTicks - before, first.inTicks, 60),
        outside: true,
      });
    if (last && after > 0)
      spans.push({
        key: 'after',
        from: duration,
        to: duration + after,
        path: waveformPath(peaks, last.outTicks, last.outTicks + after, 60),
        outside: true,
      });
    return spans;
  }, [peaks, plan, before, after, duration]);

  const cues = shownCues(plan, document);

  /** Snap a program tick to the playhead or a marker when close enough. */
  const magnet = (ticks: number) => {
    if (!snap) return ticks;
    const targets = [playheadRef.current, ...markers.map((marker) => ticksOfFrame(plan, marker))];
    const reachTicks = pixelsToTicks(SNAP_PIXELS);
    const near = targets.find((target) => Math.abs(target - ticks) <= reachTicks);
    return near ?? ticks;
  };

  /** A new edge for the clip: between words when snapping, on the magnet otherwise. */
  const landing = (edge: 'in' | 'out', part: PreviewSegment, wanted: number) => {
    const attracted = magnet(wanted);
    let source = part.inTicks + attracted - part.programStartTicks;
    if (snap && attracted === wanted && transcript && transcript.words.length > 0) {
      source = edge === 'in' ? snapStart(transcript, source) : snapEnd(transcript, source);
    }
    return Math.round(source);
  };

  const grabEdge = (event: ReactPointerEvent, index: number, edge: 'in' | 'out') => {
    const part = plan.segments[index]!;
    const length = part.outTicks - part.inTicks;
    const outer =
      (edge === 'in' && index === 0) || (edge === 'out' && index === plan.segments.length - 1);
    const origin = edge === 'in' ? part.programStartTicks : part.programStartTicks + length;
    const lowest =
      edge === 'in'
        ? outer
          ? -before
          : part.programStartTicks
        : part.programStartTicks + TICKS / 2;
    const highest =
      edge === 'in'
        ? part.programStartTicks + length - TICKS / 2
        : outer
          ? duration + after
          : part.programStartTicks + length;
    const target = (clientX: number) => {
      const raw = clamp(origin + pixelsToTicks(clientX - startX), lowest, highest);
      const source = landing(edge, part, raw);
      return {
        source,
        program: part.programStartTicks + source - part.inTicks,
      };
    };
    const startX = event.clientX;
    pressOrDrag(event, {
      onClick: () => onSelect({ kind: 'section', segmentId: part.segmentId }),
      onDrag: (_dx, _dy, next) => setGhost({ kind: 'edge', ticks: target(next.clientX).program }),
      onCancel: () => setGhost(null),
      onDrop: (_dx, _dy, next) => {
        setGhost(null);
        if (busy) return;
        const { source } = target(next.clientX);
        const command =
          edge === 'in'
            ? source < part.inTicks
              ? outer
                ? extendWithCaptions(part.segmentId, source, part.outTicks)
                : null
              : source > part.inTicks
                ? trim(source, part.outTicks, part.segmentId)
                : null
            : source > part.outTicks
              ? outer
                ? extendWithCaptions(part.segmentId, part.inTicks, source)
                : null
              : source < part.outTicks
                ? trim(part.inTicks, source, part.segmentId)
                : null;
        if (command) onApply(command);
      },
    });
  };

  const grabCueEdge = (event: ReactPointerEvent, cueId: string, edge: 'start' | 'end') => {
    const position = cues.findIndex((cue) => cue.cue_id === cueId);
    const saved = cues[position];
    if (!saved) return;
    const words = saved.lines.flatMap((line) => line.words);
    const firstWord = words[0]?.start_ticks ?? saved.start_ticks;
    const lastWord = words.at(-1)?.end_ticks ?? saved.end_ticks;
    const previousEnd = cues[position - 1]?.end_ticks ?? 0;
    const nextStart = cues[position + 1]?.start_ticks ?? duration;
    const startX = event.clientX;
    const moved = (clientX: number) => {
      const delta = pixelsToTicks(clientX - startX);
      return edge === 'start'
        ? {
            start: Math.round(clamp(magnet(saved.start_ticks + delta), previousEnd, firstWord)),
            end: saved.end_ticks,
          }
        : {
            start: saved.start_ticks,
            end: Math.round(clamp(magnet(saved.end_ticks + delta), lastWord, nextStart)),
          };
    };
    pressOrDrag(event, {
      onClick: () => onSelect({ kind: 'cue', cueId }),
      onDrag: (_dx, _dy, next) => setGhost({ kind: 'cue', cueId, ...moved(next.clientX) }),
      onCancel: () => setGhost(null),
      onDrop: (_dx, _dy, next) => {
        setGhost(null);
        const { start, end } = moved(next.clientX);
        onSelect({ kind: 'cue', cueId });
        if (!busy && (start !== saved.start_ticks || end !== saved.end_ticks)) {
          onApply(setCueTiming(cueId, start, end, plan.presentation));
        }
      },
    });
  };

  const keyframes =
    document?.video.segments?.flatMap((part) => {
      const shown = plan.segments.find((item) => item.segmentId === part.segment_id);
      if (!shown) return [];
      return [
        ...(part.layout.crop_path ?? []).map((point) => ({ point, shown, secondary: false })),
        ...(part.layout.secondary_crop_path ?? []).map((point) => ({
          point,
          shown,
          secondary: true,
        })),
      ];
    }) ?? [];

  const grabKeyframe = (
    event: ReactPointerEvent,
    shown: PreviewSegment,
    point: (typeof keyframes)[number]['point'],
    secondary: boolean,
  ) => {
    const id = `${shown.segmentId}:${point.t_ticks}:${secondary}`;
    const origin = shown.programStartTicks + point.t_ticks;
    const length = shown.outTicks - shown.inTicks;
    const startX = event.clientX;
    const moved = (clientX: number) =>
      Math.round(
        clamp(
          magnet(origin + pixelsToTicks(clientX - startX)),
          shown.programStartTicks,
          shown.programStartTicks + length - 1,
        ),
      );
    const select = () =>
      onSelect({ kind: 'keyframe', segmentId: shown.segmentId, tTicks: point.t_ticks, secondary });
    pressOrDrag(event, {
      onClick: () => {
        select();
        onSeek(frameOfTicks(plan, origin));
      },
      onDrag: (_dx, _dy, next) => setGhost({ kind: 'keyframe', id, ticks: moved(next.clientX) }),
      onCancel: () => setGhost(null),
      onDrop: (_dx, _dy, next) => {
        setGhost(null);
        const ticks = moved(next.clientX) - shown.programStartTicks;
        if (busy || ticks === point.t_ticks) return;
        onApply(
          batch([
            removeCropKeyframe(point.t_ticks, shown.segmentId, secondary),
            setCropKeyframe(ticks, point.rect, shown.segmentId, secondary, point.easing),
          ]),
        );
        onSelect({ kind: 'keyframe', segmentId: shown.segmentId, tTicks: ticks, secondary });
      },
    });
  };

  const audioLane = useRef<HTMLDivElement>(null);
  const gainAtY = (clientY: number) => {
    const box = audioLane.current?.getBoundingClientRect();
    if (!box || box.height === 0) return 0;
    const share = clamp((clientY - box.top) / box.height, 0, 1);
    return Math.round((1 - share * 2) * GAIN_RANGE);
  };
  const gainY = (db: number) =>
    `${(0.5 - clamp(db, -GAIN_RANGE, GAIN_RANGE) / (2 * GAIN_RANGE)) * 100}%`;
  const gainPoints = plan.gain.map((point, index) => ({
    index,
    ticks: document?.audio.gain_curve?.[index]?.t_ticks ?? ticksOfFrame(plan, point.frame),
    db: point.gainDb,
  }));
  const drawnGain = gainPoints.map((point) =>
    ghost?.kind === 'gain' && ghost.index === point.index
      ? { ...point, ticks: ghost.ticks, db: ghost.db }
      : point,
  );

  const grabGain = (event: ReactPointerEvent, point: (typeof gainPoints)[number]) => {
    const startX = event.clientX;
    const moved = (next: PointerEvent) => ({
      ticks: Math.round(
        clamp(magnet(point.ticks + pixelsToTicks(next.clientX - startX)), 0, duration - 1),
      ),
      db: gainAtY(next.clientY),
    });
    pressOrDrag(event, {
      onClick: () => onSelect({ kind: 'gain', tTicks: point.ticks }),
      onDrag: (_dx, _dy, next) => setGhost({ kind: 'gain', index: point.index, ...moved(next) }),
      onCancel: () => setGhost(null),
      onDrop: (_dx, _dy, next) => {
        setGhost(null);
        const { ticks, db } = moved(next);
        if (busy || (ticks === point.ticks && db === point.db)) return;
        if (ticks !== point.ticks && gainPoints.some((other) => other.ticks === ticks)) return;
        onApply(
          ticks === point.ticks
            ? setGain(ticks, db)
            : batch([removeGainPoint(point.ticks), setGain(ticks, db)]),
        );
        onSelect({ kind: 'gain', tTicks: ticks });
      },
    });
  };

  const sectionSelected = (id: string) =>
    selection.kind === 'section' && selection.segmentId === id;

  return (
    <TimelineMenu
      plan={plan}
      cues={cues}
      target={menu}
      playhead={frameOfTicks(plan, playheadRef.current)}
      busy={busy}
      onApply={onApply}
      onSeek={onSeek}
      onSplit={onSplit}
      onClose={() => setMenu(null)}
    >
      <div className="edit-lanes">
        <div
          className="review-stills edit-lane edit-video"
          onPointerDown={scrub}
          data-menu="true"
          onContextMenu={(event) => {
            // Sections let the pointer through to the lane for scrubbing, so
            // the lane finds the one under the pointer itself.
            const ticks = ticksAtX(event.clientX);
            const part = plan.segments.find(
              (candidate) =>
                ticks >= candidate.programStartTicks &&
                ticks < candidate.programStartTicks + candidate.outTicks - candidate.inTicks,
            );
            if (!part) {
              event.preventDefault();
              return;
            }
            setMenu({ kind: 'section', segmentId: part.segmentId });
            onSelect({ kind: 'section', segmentId: part.segmentId });
          }}
        >
          {stills.map((still, index) =>
            still ? (
              // eslint-disable-next-line react/no-array-index-key -- slots are positions
              <img key={index} src={still} alt="" loading="lazy" draggable={false} />
            ) : (
              // eslint-disable-next-line react/no-array-index-key -- slots are positions
              <span key={index} />
            ),
          )}
          {plan.segments.map((part, index) => {
            const from = part.programStartTicks;
            const to = from + part.outTicks - part.inTicks;
            return (
              <div
                key={part.segmentId}
                className="edit-section"
                data-selected={sectionSelected(part.segmentId) ? 'true' : undefined}
                style={{ left: at(from), width: wide(from, to) }}
              >
                {plan.segments.length > 1 && (
                  <span className="edit-section-label mono">
                    {String(index + 1).padStart(2, '0')}
                  </span>
                )}
                {(['in', 'out'] as const).map((edge) => (
                  <button
                    key={edge}
                    type="button"
                    className="review-handle edit-handle"
                    data-edge={edge}
                    data-outer={
                      (edge === 'in' && index === 0) ||
                      (edge === 'out' && index === plan.segments.length - 1)
                        ? 'true'
                        : undefined
                    }
                    disabled={busy}
                    aria-label={`${edge === 'in' ? 'Start' : 'End'} of ${
                      plan.segments.length > 1 ? `section ${index + 1}` : 'the clip'
                    }`}
                    onPointerDown={(event) => grabEdge(event, index, edge)}
                  />
                ))}
              </div>
            );
          })}
          {ghost?.kind === 'edge' && (
            <span className="edit-edge-ghost" style={{ left: at(ghost.ticks) }} aria-hidden="true">
              <span className="mono">
                {ghost.ticks < 0 ? `−${clockTenths(-ghost.ticks)}` : clockTenths(ghost.ticks)}
              </span>
            </span>
          )}
        </div>

        <div className="edit-lane edit-captions" onPointerDown={scrub}>
          {plan.cues.map((cue) => {
            const saved = cues.find((item) => item.cue_id === cue.cueId);
            const drawn =
              ghost?.kind === 'cue' && ghost.cueId === cue.cueId
                ? { start: ghost.start, end: ghost.end }
                : {
                    start: saved?.start_ticks ?? ticksOfFrame(plan, cue.firstFrame),
                    end: saved?.end_ticks ?? ticksOfFrame(plan, cue.endFrame),
                  };
            const selected = selection.kind === 'cue' && selection.cueId === cue.cueId;
            const text = cue.lines
              .flat()
              .map((word) => word.text)
              .join(' ');
            return (
              <div
                key={cue.cueId}
                className="edit-cue"
                data-selected={selected ? 'true' : undefined}
                data-region={cue.region}
                style={{ left: at(drawn.start), width: wide(drawn.start, drawn.end) }}
                title={text}
                onPointerDown={(event) => {
                  if (event.button > 0) return;
                  event.stopPropagation();
                  onSelect({ kind: 'cue', cueId: cue.cueId });
                  onSeek(cue.firstFrame);
                }}
                data-menu="true"
                onContextMenu={() => {
                  setMenu({ kind: 'cue', cueId: cue.cueId });
                  onSelect({ kind: 'cue', cueId: cue.cueId });
                }}
              >
                {saved && (
                  <button
                    type="button"
                    className="edit-cue-edge"
                    data-edge="start"
                    aria-label={`Start of the caption “${text}”`}
                    disabled={busy}
                    onPointerDown={(event) => grabCueEdge(event, cue.cueId, 'start')}
                  />
                )}
                <span>{text}</span>
                {saved && (
                  <button
                    type="button"
                    className="edit-cue-edge"
                    data-edge="end"
                    aria-label={`End of the caption “${text}”`}
                    disabled={busy}
                    onPointerDown={(event) => grabCueEdge(event, cue.cueId, 'end')}
                  />
                )}
              </div>
            );
          })}
        </div>

        <div className="edit-lane edit-texts" onPointerDown={scrub}>
          {(plan.overlays ?? []).map((overlay) => {
            const selected =
              selection.kind === 'overlay' && selection.overlayId === overlay.overlayId;
            const text = overlay.text.replace(/\n/g, ' ');
            return (
              <div
                key={overlay.overlayId}
                className="edit-text-block"
                data-selected={selected ? 'true' : undefined}
                style={{
                  left: at(overlay.startTicks),
                  width: wide(overlay.startTicks, overlay.endTicks),
                }}
                title={text}
                onPointerDown={(event) => {
                  if (event.button > 0) return;
                  event.stopPropagation();
                  onSelect({ kind: 'overlay', overlayId: overlay.overlayId });
                  onSeek(overlay.firstFrame);
                }}
              >
                <span>{overlay.role === 'hook' ? `Hook · ${text}` : text}</span>
              </div>
            );
          })}
        </div>

        <div className="edit-lane edit-framing" onPointerDown={scrub}>
          {plan.segments.map((part) => {
            const from = part.programStartTicks;
            const to = from + part.outTicks - part.inTicks;
            // The section's own layout; hosts older than it are read from
            // the crops the first frame draws.
            const two = (plan.secondaryCrops?.[part.firstFrame] ?? null) !== null;
            const fit = !two && plan.crops[part.firstFrame] === null;
            const named =
              (part.layout && LAYOUT_NAMES[part.layout]) ??
              (two ? 'Two speakers' : fit ? 'Whole frame' : 'Follow speaker');
            return (
              <button
                key={part.segmentId}
                type="button"
                className="edit-run"
                data-selected={sectionSelected(part.segmentId) ? 'true' : undefined}
                style={{ left: at(from), width: wide(from, to) }}
                onPointerDown={(event) => event.stopPropagation()}
                onClick={() => onSelect({ kind: 'section', segmentId: part.segmentId })}
                data-menu="true"
                onContextMenu={() => {
                  setMenu({ kind: 'section', segmentId: part.segmentId });
                  onSelect({ kind: 'section', segmentId: part.segmentId });
                }}
              >
                {named}
              </button>
            );
          })}
          {keyframes.map(({ point, shown, secondary }) => {
            const id = `${shown.segmentId}:${point.t_ticks}:${secondary}`;
            const ticks =
              ghost?.kind === 'keyframe' && ghost.id === id
                ? ghost.ticks
                : shown.programStartTicks + point.t_ticks;
            const selected =
              selection.kind === 'keyframe' &&
              selection.segmentId === shown.segmentId &&
              selection.tTicks === point.t_ticks &&
              selection.secondary === secondary;
            return (
              <button
                key={id}
                type="button"
                className="edit-keyframe"
                data-secondary={secondary ? 'true' : undefined}
                data-selected={selected ? 'true' : undefined}
                style={{ left: at(ticks) }}
                aria-label={`${secondary ? 'Lower' : 'Framing'} keyframe at ${clockTenths(ticks)}`}
                disabled={busy}
                onPointerDown={(event) => grabKeyframe(event, shown, point, secondary)}
                data-menu="true"
                onContextMenu={() =>
                  setMenu({
                    kind: 'keyframe',
                    segmentId: shown.segmentId,
                    tTicks: point.t_ticks,
                    secondary,
                    rect: point.rect,
                    easing: point.easing ?? 'linear',
                  })
                }
              />
            );
          })}
        </div>

        <div
          ref={audioLane}
          className="review-sound edit-lane edit-audio"
          onPointerDown={scrub}
          onDoubleClick={(event) => {
            if (busy) return;
            const ticks = Math.round(clamp(ticksAtX(event.clientX), 0, duration - 1));
            const db = gainAtY(event.clientY);
            onApply(setGain(ticks, db));
            onSelect({ kind: 'gain', tTicks: ticks });
          }}
        >
          {waves.map((wave) =>
            wave.path ? (
              <svg
                key={wave.key}
                viewBox="0 0 100 100"
                preserveAspectRatio="none"
                data-outside={wave.outside ? 'true' : undefined}
                style={{ left: at(wave.from), width: wide(wave.from, wave.to) }}
                aria-hidden="true"
              >
                <path d={wave.path} />
              </svg>
            ) : null,
          )}
          <svg
            className="edit-volume"
            viewBox="0 0 100 100"
            preserveAspectRatio="none"
            aria-hidden="true"
          >
            <polyline
              vectorEffect="non-scaling-stroke"
              points={volumeLine(drawnGain, view, span, duration)}
            />
          </svg>
          {drawnGain.map((point) => (
            <button
              key={point.index}
              type="button"
              className="edit-gain"
              data-selected={
                selection.kind === 'gain' && selection.tTicks === gainPoints[point.index]!.ticks
                  ? 'true'
                  : undefined
              }
              style={{ left: at(point.ticks), top: gainY(point.db) }}
              aria-label={`Volume ${point.db > 0 ? '+' : ''}${point.db.toFixed(1)} dB at ${clockTenths(point.ticks)}`}
              disabled={busy}
              onPointerDown={(event) => grabGain(event, gainPoints[point.index]!)}
              data-menu="true"
              onContextMenu={() =>
                setMenu({ kind: 'gain', tTicks: gainPoints[point.index]!.ticks, db: point.db })
              }
            />
          ))}
        </div>
      </div>
    </TimelineMenu>
  );
});

/** The tile nearest a source tick, from tiles sorted by time. */
function nearestTile<T extends { readonly tTicks: number }>(tiles: readonly T[], ticks: number): T {
  let low = 0;
  let high = tiles.length - 1;
  while (low < high) {
    const middle = (low + high) >> 1;
    if (tiles[middle]!.tTicks < ticks) low = middle + 1;
    else high = middle;
  }
  const after = tiles[low]!;
  const before = tiles[Math.max(0, low - 1)]!;
  return Math.abs(before.tTicks - ticks) <= Math.abs(after.tTicks - ticks) ? before : after;
}

/** The volume line across the view, flat at unchanged where nothing was set. */
function volumeLine(
  points: readonly { readonly ticks: number; readonly db: number }[],
  view: Cut,
  span: number,
  duration: number,
): string {
  const x = (ticks: number) => (((ticks - view.startTicks) / span) * 100).toFixed(3);
  const y = (db: number) =>
    ((0.5 - clamp(db, -GAIN_RANGE, GAIN_RANGE) / (2 * GAIN_RANGE)) * 100).toFixed(2);
  const sorted = [...points].toSorted((a, b) => a.ticks - b.ticks);
  if (sorted.length === 0) return `${x(0)},50 ${x(duration)},50`;
  const first = sorted[0]!;
  const last = sorted.at(-1)!;
  return [
    `${x(0)},${y(first.db)}`,
    ...sorted.map((point) => `${x(point.ticks)},${y(point.db)}`),
    `${x(duration)},${y(last.db)}`,
  ].join(' ');
}

/** The command that deletes the marked range, or null when it cannot. */
export function deleteRange(
  plan: PreviewPlan,
  inFrame: number,
  outFrame: number,
): EditCommandJson | null {
  const from = ticksOfFrame(plan, Math.min(inFrame, outFrame));
  const to = ticksOfFrame(plan, Math.max(inFrame, outFrame));
  return rippleRange(plan, from, to);
}
