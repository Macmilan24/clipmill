/**
 * Program monitor: the clip as it will render, drawn from the preview plan.
 * Drag the picture to reframe, pinch or ⌘-scroll to zoom, and click or drag a
 * caption to place it. A gesture draws a draft; one command is sent on release.
 */
import {
  ChevronFirst,
  ChevronLast,
  Pause,
  Play,
  Repeat,
  StepBack,
  StepForward,
  Volume2,
  VolumeX,
} from 'lucide-react';
import {
  type CSSProperties,
  type PointerEvent as ReactPointerEvent,
  type RefObject,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../components/ui/select.js';
import type { EditIr } from '@clipmill/contracts';

import type {
  EditCommandJson,
  FaceSighting,
  PreviewOverlay,
  PreviewPlan,
} from '../daemon/client.js';
import { SPEEDS } from '../inspector/playback.js';
import { clockTenths } from '../inspector/review.js';
import { formatTime, useTimeFormat } from '../shell/timeFormat.js';
import { TipButton } from '../inspector/TipButton.js';
import {
  batch,
  setCaptionOptions,
  setCropKeyframe,
  setCuePosition,
  setLayout,
  ticksAt,
} from './commands.js';
import { BrandLayer } from './BrandLayer.js';
import { CaptionCanvas } from './CaptionCanvas.js';
import type { ExactCaptions } from './exactCaptions.js';
import { CompositionCanvas } from './CompositionCanvas.js';
import { type FaceNow, byTrack, facesAt, fittedFrame } from './faces.js';
import { pressOrDrag } from './gesture.js';
import { cropAt, cueAt, highlightedWord, segmentAt, sourceOf, sourceTicksAt } from './player.js';
import { FRAME_SHAPES, shapeOfFrame } from './layouts.js';
import { overlaysAt, savedOverlay, setOverlay, withContent } from './overlays.js';
import type { EditorSelection } from './selection.js';

export type MonitorView = 'edit' | 'original';
export type SafePlatform = 'off' | 'all' | 'tiktok' | 'reels' | 'shorts';

/**
 * Where each app draws over a 1080×1920 frame, as shares of it: its top bar,
 * the caption and sound block along the bottom, and the button column on the
 * right. "All" is the most cautious of the three. A guide, never a rule.
 */
const SAFE_ZONES: Record<
  Exclude<SafePlatform, 'off'>,
  { top: number; bottom: number; left: number; right: number }
> = {
  all: { top: 150 / 1920, bottom: 480 / 1920, left: 60 / 1080, right: 130 / 1080 },
  tiktok: { top: 150 / 1920, bottom: 420 / 1920, left: 60 / 1080, right: 120 / 1080 },
  reels: { top: 110 / 1920, bottom: 480 / 1920, left: 60 / 1080, right: 130 / 1080 },
  shorts: { top: 120 / 1920, bottom: 360 / 1920, left: 60 / 1080, right: 110 / 1080 },
};

const SAFE_LABELS: Record<SafePlatform, string> = {
  off: 'Safe area off',
  all: 'All apps',
  tiktok: 'TikTok',
  reels: 'Reels',
  shorts: 'Shorts',
};

export interface MonitorPlayback {
  readonly frame: number;
  /**
   * The frame whose sound is being heard, which the captions follow: behind
   * `frame` by the output latency while a boost plays through Web Audio, the
   * same frame otherwise.
   */
  readonly heardFrame?: number;
  readonly playing: boolean;
  readonly speed: number;
  readonly loop: boolean;
  readonly muted: boolean;
  readonly preparing: boolean;
  readonly onToggle: () => void;
  readonly onStep: (frames: number) => void;
  readonly onSeek: (frame: number) => void;
  readonly onSpeed: (speed: number) => void;
  readonly onLoop: (loop: boolean) => void;
  readonly onMuted: (muted: boolean) => void;
  /** Wired to the media element; see `Editor`. */
  readonly onProxyTime: (seconds: number, atMediaEnd?: boolean) => number | null;
  readonly onMetadata: () => void;
  readonly onPlaying: (playing: boolean) => void;
  readonly onBuffering: (waiting: boolean) => void;
  readonly onError: () => void;
}

export interface EditorMonitorProps {
  readonly plan: PreviewPlan;
  readonly docId: string;
  readonly videoRef: RefObject<HTMLVideoElement | null>;
  readonly proxyUrl: string | null;
  readonly proxyUrls: ReadonlyMap<string, string>;
  readonly startSeconds: number | null;
  readonly playback: MonitorPlayback;
  /** The in and out marks, when both are set, for the loop and the clock. */
  readonly range: { readonly first: number; readonly last: number } | null;
  readonly busy: boolean;
  readonly selection: EditorSelection;
  readonly onSelect: (selection: EditorSelection) => void;
  readonly onApply: (command: EditCommandJson) => void;
  /**
   * The captions as the export burns them in, for libass to draw: the plan's
   * script or a look still being tried, and the faces it may use. Null keeps
   * the CSS approximation.
   */
  readonly captions?: ExactCaptions | null;
  /** The clip-wide caption options, so a drag can move every caption. */
  readonly captionOptions?: CaptionOptions;
  /**
   * The faces seen over a span of the source, so the Original view can offer
   * each one to follow. Absent shows the frame without them.
   */
  readonly loadFaces?:
    ((startTicks: number, endTicks: number) => Promise<readonly FaceSighting[]>) | null;
  /** Follow this face through the section at the playhead. */
  readonly onFollow?: ((trackId: number) => void) | null;
  /** Where the logo loads from; absent shows none. */
  readonly assetUrl?: ((hash: string) => string) | null;
}

type CaptionOptions = NonNullable<EditIr['captions']['options']>;

export type { ExactCaptions } from './exactCaptions.js';

export function EditorMonitor({
  plan,
  docId,
  videoRef,
  proxyUrl,
  proxyUrls,
  startSeconds,
  playback,
  range,
  busy,
  selection,
  onSelect,
  onApply,
  captions = null,
  captionOptions = {},
  loadFaces = null,
  onFollow = null,
  assetUrl = null,
}: EditorMonitorProps) {
  const [view, setView] = useState<MonitorView>('edit');
  const [safe, setSafe] = useState<SafePlatform>('off');
  const [grid, setGrid] = useState(false);
  const segment = segmentAt(plan, playback.frame);
  // The apps' own buttons and captions are mapped over a 9:16 frame only.
  const vertical = shapeOfFrame(plan) === 'vertical';
  const twoUp = cropAt(plan, playback.frame, true) !== null;
  const fitted = cropAt(plan, playback.frame) === null;
  const inset = segment?.layout === 'picture_in_picture';

  // The faces of the section at the playhead, fetched when the Original view
  // is showing: that is where a person points at the one to follow.
  const [faces, setFaces] = useState<{
    readonly key: string;
    readonly tracks: ReadonlyMap<number, readonly FaceSighting[]>;
  } | null>(null);
  const spanIn = segment?.inTicks ?? 0;
  const spanOut = segment?.outTicks ?? 0;
  const faceKey = segment ? `${segment.sourceFingerprint}:${spanIn}:${spanOut}` : null;
  const picking = view === 'original' && loadFaces !== null && onFollow !== null;
  useEffect(() => {
    if (!picking || !loadFaces || faceKey === null) return undefined;
    let live = true;
    loadFaces(spanIn, spanOut)
      .then((sightings) => {
        if (live) setFaces({ key: faceKey, tracks: byTrack(sightings) });
      })
      // No faces to offer is the frame without them, not a broken preview.
      .catch(() => {
        if (live) setFaces({ key: faceKey, tracks: new Map() });
      });
    return () => {
      live = false;
    };
  }, [picking, loadFaces, faceKey, spanIn, spanOut]);
  const ticksNow = sourceTicksAt(plan, playback.frame);
  const shown: readonly FaceNow[] =
    picking && faces && faces.key === faceKey && ticksNow !== null
      ? facesAt(faces.tracks, ticksNow)
      : [];

  return (
    <section
      className="review-viewer edit-viewer"
      aria-label="Clip preview"
      data-coach="edit-preview"
    >
      <div className="review-viewer-bar">
        <div className="review-segmented" role="group" aria-label="What the preview shows">
          <button type="button" aria-pressed={view === 'edit'} onClick={() => setView('edit')}>
            Edit
          </button>
          <button
            type="button"
            aria-pressed={view === 'original'}
            onClick={() => setView('original')}
          >
            Original
          </button>
        </div>
        <span className="review-viewer-note">
          {view === 'original'
            ? shown.length > 0
              ? 'Click a face to follow that person'
              : 'The whole frame, before framing and captions'
            : inset
              ? 'Picture in picture'
              : twoUp
                ? 'Two speakers'
                : fitted
                  ? 'Whole frame'
                  : 'Following the speaker'}
          {segment && plan.segments.length > 1
            ? ` · Section ${plan.segments.indexOf(segment) + 1} of ${plan.segments.length}`
            : ''}
        </span>
        <span className="review-spacer" />
        {vertical && (
          <Select value={safe} onValueChange={(value) => setSafe(value as SafePlatform)}>
            <SelectTrigger
              aria-label="Safe area"
              className="edit-viewer-select"
              data-active={safe !== 'off' ? 'true' : undefined}
              disabled={view !== 'edit'}
            >
              <SelectValue>{safe === 'off' ? 'Safe area' : SAFE_LABELS[safe]}</SelectValue>
            </SelectTrigger>
            <SelectContent align="end">
              {(Object.keys(SAFE_LABELS) as SafePlatform[]).map((platform) => (
                <SelectItem key={platform} value={platform}>
                  {platform === 'off' ? 'Off' : SAFE_LABELS[platform]}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
        <button
          type="button"
          className="review-viewer-toggle"
          aria-pressed={grid}
          disabled={view !== 'edit'}
          onClick={() => setGrid(!grid)}
        >
          Grid
        </button>
        <span className="review-viewer-note mono">
          {FRAME_SHAPES.find((item) => item.shape === shapeOfFrame(plan))?.ratio ?? '9:16'}
        </span>
      </div>

      <div className="review-stage-wrap">
        <Stage
          key={docId}
          plan={plan}
          videoRef={videoRef}
          proxyUrl={proxyUrl}
          proxyUrls={proxyUrls}
          startSeconds={startSeconds}
          playback={playback}
          view={view}
          safe={view === 'edit' && vertical ? safe : 'off'}
          grid={grid && view === 'edit'}
          busy={busy}
          selection={selection}
          onSelect={onSelect}
          onApply={onApply}
          captions={view === 'edit' ? captions : null}
          captionOptions={captionOptions}
          assetUrl={assetUrl}
          faces={shown}
          onFollow={
            onFollow
              ? (trackId) => {
                  onFollow(trackId);
                  setView('edit');
                }
              : null
          }
        />
      </div>

      <Transport plan={plan} playback={playback} range={range} disabled={!proxyUrl} />
    </section>
  );
}

const percent = (value: number) => `${value * 100}%`;

/** The faces on the whole frame, each a button that follows that person. */
function FacePicker({
  faces,
  frame,
  disabled,
  onFollow,
}: {
  readonly faces: readonly FaceNow[];
  readonly frame: ReturnType<typeof fittedFrame>;
  readonly disabled: boolean;
  readonly onFollow: (trackId: number) => void;
}) {
  return (
    <div className="edit-faces" role="group" aria-label="People in the frame">
      {faces.map((face, order) => (
        <button
          key={face.trackId}
          type="button"
          className="edit-face"
          disabled={disabled}
          aria-label={`Follow person ${order + 1}`}
          title="Follow this person"
          style={{
            left: percent(frame.left + face.x * frame.width),
            top: percent(frame.top + face.y * frame.height),
            width: percent(face.width * frame.width),
            height: percent(face.height * frame.height),
          }}
          onClick={() => onFollow(face.trackId)}
        >
          <span>Follow</span>
        </button>
      ))}
    </div>
  );
}

type Rect = {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
};

function Stage({
  plan,
  videoRef,
  proxyUrl,
  proxyUrls,
  startSeconds,
  playback,
  view,
  safe,
  grid,
  busy,
  selection,
  onSelect,
  onApply,
  captions,
  captionOptions,
  faces,
  onFollow,
  assetUrl,
}: {
  readonly assetUrl: ((hash: string) => string) | null;
  readonly captions: ExactCaptions | null;
  readonly captionOptions: CaptionOptions;
  readonly faces: readonly FaceNow[];
  readonly onFollow: ((trackId: number) => void) | null;
  readonly plan: PreviewPlan;
  readonly videoRef: RefObject<HTMLVideoElement | null>;
  readonly proxyUrl: string | null;
  readonly proxyUrls: ReadonlyMap<string, string>;
  readonly startSeconds: number | null;
  readonly playback: MonitorPlayback;
  readonly view: MonitorView;
  readonly safe: SafePlatform;
  readonly grid: boolean;
  readonly busy: boolean;
  readonly selection: EditorSelection;
  readonly onSelect: (selection: EditorSelection) => void;
  readonly onApply: (command: EditCommandJson) => void;
}) {
  const frame = playback.frame;
  const part = segmentAt(plan, frame);
  const source = part ? sourceOf(plan, part) : null;
  const localTicks = part ? ticksAt(plan, frame) - part.programStartTicks : 0;
  // A reframe in progress: which viewport, and the rectangle it would get.
  const [draft, setDraft] = useState<{ secondary: boolean; rect: Rect } | null>(null);
  // Where a caption being dragged would sit, as shares of the frame.
  const [captionDrag, setCaptionDrag] = useState<{ x: number; y: number } | null>(null);
  const [overlayDrag, setOverlayDrag] = useState<{
    overlayId: string;
    x: number;
    y: number;
  } | null>(null);
  // Whether libass is drawing the captions; the CSS ones then only take clicks.
  const [exact, setExact] = useState(false);
  const zoomTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const stage = useRef<HTMLDivElement>(null);

  const drawn = useMemo(() => {
    if (view === 'original') {
      // The recording as it was: no crop, no fill, no zoom.
      return {
        ...plan,
        crops: plan.crops.map(() => null),
        secondaryCrops: plan.secondaryCrops?.map(() => null),
        segments: plan.segments.map((segment) =>
          Object.assign({}, segment, { layout: 'fit', backgroundColour: null, zoomPercent: 100 }),
        ),
      } as PreviewPlan;
    }
    if (!draft) return plan;
    const key = draft.secondary ? 'secondaryCrops' : 'crops';
    const crops = [...(plan[key] ?? plan.crops.map(() => null))];
    crops[frame] = [draft.rect.x, draft.rect.y, draft.rect.width, draft.rect.height];
    return { ...plan, [key]: crops } as PreviewPlan;
  }, [plan, view, draft, frame]);

  useEffect(
    () => () => {
      if (zoomTimer.current) clearTimeout(zoomTimer.current);
    },
    [],
  );

  /**
   * Where a viewport sits on the output frame, in output pixels: the whole
   * frame, one of two viewports at the section's split — stacked, or side by
   * side in a landscape frame — or the inset.
   */
  const viewportOf = (secondary: boolean): Rect => {
    const whole = { x: 0, y: 0, width: plan.width, height: plan.height };
    if (part?.layout === 'picture_in_picture') {
      const inset = part.inset;
      return secondary && inset
        ? { x: inset[0], y: inset[1], width: inset[2], height: inset[2] }
        : whole;
    }
    if (cropAt(plan, frame, true) === null) return whole;
    if (plan.width > plan.height) {
      const left = part?.upperHeight || plan.width / 2;
      return secondary
        ? { x: left, y: 0, width: plan.width - left, height: plan.height }
        : { x: 0, y: 0, width: left, height: plan.height };
    }
    const upper = part?.upperHeight || plan.height / 2;
    return secondary
      ? { x: 0, y: upper, width: plan.width, height: plan.height - upper }
      : { x: 0, y: 0, width: plan.width, height: upper };
  };

  /** Which viewport a point on the stage is in: the inset or lower one, or not. */
  const secondaryAt = (clientX: number, clientY: number, box: DOMRect): boolean => {
    if (cropAt(plan, frame, true) === null) return false;
    const inner = viewportOf(true);
    const x = ((clientX - box.left) / Math.max(1, box.width)) * plan.width;
    const y = ((clientY - box.top) / Math.max(1, box.height)) * plan.height;
    return x >= inner.x && x < inner.x + inner.width && y >= inner.y && y < inner.y + inner.height;
  };

  /** The rectangle the camera takes now, or the centred full-height one a fit clip would get. */
  const baseRect = (secondary: boolean): Rect | null => {
    if (!source) return null;
    const current = cropAt(plan, frame, secondary);
    if (current) return current;
    if (secondary) return null;
    const height = source.displayHeight - (source.displayHeight % 2);
    const width = Math.min(
      source.displayWidth,
      2 * Math.round((height * plan.width) / plan.height / 2),
    );
    return { x: Math.round((source.displayWidth - width) / 2), y: 0, width, height };
  };

  const commit = (secondary: boolean, rect: Rect) => {
    if (!part) return;
    const keyframe = setCropKeyframe(localTicks, rect, part.segmentId, secondary);
    // Reframing a fitted picture follows it from here; a picture-in-picture
    // stays one, its full picture now a followed crop.
    const keeps = cropAt(plan, frame, secondary) !== null || part.layout === 'picture_in_picture';
    onApply(keeps ? keyframe : batch([setLayout('speaker_fill', part.segmentId), keyframe]));
    onSelect({ kind: 'keyframe', segmentId: part.segmentId, tTicks: localTicks, secondary });
  };

  const clampRect = (rect: Rect): Rect => {
    if (!source) return rect;
    return {
      width: rect.width,
      height: rect.height,
      x: Math.round(Math.max(0, Math.min(source.displayWidth - rect.width, rect.x))),
      y: Math.round(Math.max(0, Math.min(source.displayHeight - rect.height, rect.y))),
    };
  };

  const grabFrame = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (busy || view !== 'edit' || !part || !source) return;
    const box = event.currentTarget.getBoundingClientRect();
    const secondary = secondaryAt(event.clientX, event.clientY, box);
    const start = baseRect(secondary);
    if (!start) return;
    // One screen pixel moves the camera by the share of the crop its viewport
    // covers on screen: the whole stage, a split share of it, or the inset.
    const viewport = viewportOf(secondary);
    const perX = start.width / Math.max(1, (viewport.width / plan.width) * box.width);
    const perY = start.height / Math.max(1, (viewport.height / plan.height) * box.height);
    const moved = (dx: number, dy: number) =>
      clampRect({ ...start, x: start.x - dx * perX, y: start.y - dy * perY });
    pressOrDrag(event, {
      onDrag: (dx, dy) => setDraft({ secondary, rect: moved(dx, dy) }),
      onDrop: (dx, dy) => {
        setDraft(null);
        const rect = moved(dx, dy);
        if (rect.x !== start.x || rect.y !== start.y) commit(secondary, rect);
      },
      onClick: () => {
        if (selection.kind === 'cue') onSelect({ kind: 'clip' });
      },
      onCancel: () => setDraft(null),
    });
  };

  // Registered by hand: React's wheel listener is passive, and a pinch the
  // stage cannot cancel would zoom the whole window instead.
  const wheelState = useRef({
    busy,
    view,
    part,
    source,
    draft,
    baseRect,
    clampRect,
    commit,
    viewportOf,
    secondaryAt,
  });
  wheelState.current = {
    busy,
    view,
    part,
    source,
    draft,
    baseRect,
    clampRect,
    commit,
    viewportOf,
    secondaryAt,
  };
  useEffect(() => {
    const element = stage.current;
    if (!element) return;
    const listener = (event: WheelEvent) => {
      const current = wheelState.current;
      if (!(event.ctrlKey || event.metaKey)) return;
      event.preventDefault();
      if (current.busy || current.view !== 'edit' || !current.part || !current.source) return;
      const secondary =
        current.draft?.secondary ??
        current.secondaryAt(event.clientX, event.clientY, element.getBoundingClientRect());
      const start = current.draft?.rect ?? current.baseRect(secondary);
      if (!start) return;
      // Each viewport keeps its own shape: a crop of another shape is one the
      // render refuses rather than stretches.
      const viewport = current.viewportOf(secondary);
      const factor = Math.exp(event.deltaY * 0.01);
      const height = Math.max(
        Math.round(current.source.displayHeight * 0.2),
        Math.min(current.source.displayHeight, Math.round((start.height * factor) / 2) * 2),
      );
      const width = Math.min(
        current.source.displayWidth,
        Math.max(2, Math.round((height * viewport.width) / viewport.height / 2) * 2),
      );
      const rect = current.clampRect({
        width,
        height,
        x: start.x + (start.width - width) / 2,
        y: start.y + (start.height - height) / 2,
      });
      setDraft({ secondary, rect });
      if (zoomTimer.current) clearTimeout(zoomTimer.current);
      // One edit per gesture, not one per wheel tick.
      zoomTimer.current = setTimeout(() => {
        setDraft(null);
        wheelState.current.commit(secondary, rect);
      }, 280);
    };
    element.addEventListener('wheel', listener, { passive: false });
    return () => element.removeEventListener('wheel', listener);
  }, []);

  const heard = playback.heardFrame ?? frame;
  const cue = cueAt(plan, heard);
  const style = plan.captionStyle;
  const highlighted = cue ? highlightedWord(plan, cue, heard) : -1;
  const grabCaption = (event: ReactPointerEvent<HTMLParagraphElement>) => {
    if (!cue) return;
    const box = stage.current?.getBoundingClientRect();
    const caption = event.currentTarget.getBoundingClientRect();
    // Where the caption's centre was when the drag began, so it follows the
    // pointer from wherever on the text it was picked up.
    const origin = box
      ? {
          x: (caption.left + caption.width / 2 - box.left) / Math.max(1, box.width),
          y: (caption.top + caption.height / 2 - box.top) / Math.max(1, box.height),
        }
      : null;
    const placed = (dx: number, dy: number) =>
      box && origin
        ? {
            x: snapCentre(clampShare(origin.x + dx / Math.max(1, box.width))),
            y: clampShare(origin.y + dy / Math.max(1, box.height)),
          }
        : null;
    pressOrDrag(event, {
      onClick: () => onSelect({ kind: 'cue', cueId: cue.cueId }),
      onDrag: (dx, dy) => {
        if (busy) return;
        setCaptionDrag(placed(dx, dy));
      },
      onDrop: (dx, dy, next) => {
        setCaptionDrag(null);
        const at = placed(dx, dy);
        if (!at || busy) return;
        const position = { x: Math.round(at.x * 1000), y: Math.round(at.y * 1000) };
        onSelect({ kind: 'cue', cueId: cue.cueId });
        // Alt places this caption alone; otherwise every caption moves, as
        // a creator placing captions over a face means all of them.
        if (next.altKey) {
          onApply(setCuePosition(cue.cueId, position, plan.presentation));
          return;
        }
        // A cue placed on its own is one whose position is not the clip's;
        // moving every caption brings those along too.
        const track = captionOptions.position;
        const alone = plan.cues.filter(
          (item) =>
            item.position &&
            (!track || item.position[0] !== track.x || item.position[1] !== track.y),
        );
        onApply(
          batch([
            setCaptionOptions({ ...captionOptions, position }),
            ...alone.map((item) => setCuePosition(item.cueId, null, plan.presentation)),
          ]),
        );
      },
      onCancel: () => setCaptionDrag(null),
    });
  };

  // Texts over the picture: shown, picked and moved here; drawn exactly by
  // libass when it is drawing, when these only take the pointer.
  const shownOverlays = overlaysAt(plan, frame);
  const grabOverlay = (event: ReactPointerEvent<HTMLParagraphElement>, overlay: PreviewOverlay) => {
    const box = stage.current?.getBoundingClientRect();
    const placed = (dx: number, dy: number) =>
      box
        ? {
            x: snapCentre(clampShare(overlay.x / 1000 + dx / Math.max(1, box.width))),
            y: clampShare(overlay.y / 1000 + dy / Math.max(1, box.height)),
          }
        : null;
    pressOrDrag(event, {
      onClick: () => onSelect({ kind: 'overlay', overlayId: overlay.overlayId }),
      onDrag: (dx, dy) => {
        if (busy) return;
        const at = placed(dx, dy);
        setOverlayDrag(at ? { overlayId: overlay.overlayId, ...at } : null);
      },
      onDrop: (dx, dy) => {
        setOverlayDrag(null);
        const at = placed(dx, dy);
        if (!at || busy) return;
        onSelect({ kind: 'overlay', overlayId: overlay.overlayId });
        onApply(
          setOverlay(
            withContent(savedOverlay(overlay), {
              x: Math.round(at.x * 1000),
              y: Math.round(at.y * 1000),
            }),
          ),
        );
      },
      onCancel: () => setOverlayDrag(null),
    });
  };

  // Sizes are stated at the 1920-pixel design height, whatever the shape.
  const relative = (pixels: number) =>
    `${((pixels * plan.height) / 1920 / Math.max(1, plan.width)) * 100}cqw`;
  const verticalMargin = ((style?.marginVertical ?? 260) / 1920) * 100;
  const centred =
    captionDrag ??
    (cue?.position ? { x: cue.position[0] / 1000, y: cue.position[1] / 1000 } : null);
  const position = centred
    ? {
        left: `${centred.x * 100}%`,
        top: `${centred.y * 100}%`,
        transform: 'translate(-50%, -50%)',
      }
    : cue?.region === 'upper_safe'
      ? { top: `${verticalMargin}%` }
      : cue?.region === 'center'
        ? { top: '50%', transform: 'translateY(-50%)' }
        : { bottom: `${verticalMargin}%` };
  let index = 0;
  const zone = safe === 'off' ? null : SAFE_ZONES[safe];
  const reframable = view === 'edit' && !busy && proxyUrl !== null;

  return (
    <div
      ref={stage}
      className="review-stage edit-stage"
      data-view="result"
      data-testid="stage"
      style={
        {
          containerType: 'inline-size',
          '--output-aspect': String(plan.width / Math.max(1, plan.height)),
        } as CSSProperties
      }
    >
      {proxyUrl ? (
        <>
          {/* eslint-disable-next-line jsx-a11y/media-has-caption -- the cues are
              drawn below from the plan rather than as a text track. */}
          <video
            ref={videoRef}
            src={proxyUrl}
            className="review-decoder"
            crossOrigin="anonymous"
            playsInline
            data-testid="proxy"
            data-start-seconds={startSeconds ?? undefined}
            muted={playback.muted}
            onLoadedMetadata={playback.onMetadata}
            onPlay={() => playback.onPlaying(true)}
            onPause={() => playback.onPlaying(false)}
            onEnded={(event) => {
              playback.onPlaying(false);
              // The last decoded PTS can precede the last program frame.
              // EOF itself must advance or finish the edit, not restart the proxy.
              playback.onProxyTime(event.currentTarget.currentTime, true);
            }}
            onError={playback.onError}
          />
          <CompositionCanvas
            video={videoRef}
            mediaKey={proxyUrl}
            plan={drawn}
            frame={frame}
            onFrame={playback.onProxyTime}
            proxyUrls={proxyUrls}
            onError={playback.onError}
            onBuffering={playback.onBuffering}
          />
          <div
            className="edit-frame-grab"
            data-enabled={reframable ? 'true' : undefined}
            data-dragging={draft ? 'true' : undefined}
            aria-hidden="true"
            onPointerDown={grabFrame}
          />
          {view === 'edit' && <BrandLayer plan={plan} frame={frame} assetUrl={assetUrl} />}
          {captions && (
            <CaptionCanvas
              ass={captions.ass}
              faces={captions.faces}
              family={captions.family}
              seconds={(heard * plan.rateDen) / Math.max(1, plan.rateNum)}
              frameWidth={plan.width}
              frameHeight={plan.height}
              onDrawing={setExact}
            />
          )}
        </>
      ) : (
        <div className="review-unavailable" role="note">
          <p className="review-unavailable-title">Preview unavailable</p>
          <p>This recording has no proxy, so there is nothing to play.</p>
        </div>
      )}
      {grid && <div className="edit-grid" aria-hidden="true" />}
      {view === 'original' && source && onFollow && faces.length > 0 && (
        <FacePicker
          faces={faces}
          frame={fittedFrame(source, plan)}
          disabled={busy}
          onFollow={onFollow}
        />
      )}
      {zone && (
        <div className="review-safe-area" aria-hidden="true">
          <span style={{ top: 0, left: 0, right: 0, height: share(zone.top) }} />
          <span style={{ bottom: 0, left: 0, right: 0, height: share(zone.bottom) }} />
          <span
            style={{
              top: share(zone.top),
              bottom: share(zone.bottom),
              left: 0,
              width: share(zone.left),
            }}
          />
          <span
            style={{
              top: share(zone.top),
              bottom: share(zone.bottom),
              right: 0,
              width: share(zone.right),
            }}
          />
          <i
            style={{
              top: share(zone.top),
              bottom: share(zone.bottom),
              left: share(zone.left),
              right: share(zone.right),
            }}
          />
        </div>
      )}
      {playback.preparing && (
        <div role="status" className="edit-stage-status">
          <span>Preparing preview…</span>
        </div>
      )}
      {proxyUrl && view === 'edit' && cue && (
        <p
          className="edit-caption"
          data-selected={selection.kind === 'cue' && selection.cueId === cue.cueId}
          data-dragging={captionDrag !== null ? 'true' : undefined}
          onPointerDown={grabCaption}
          data-exact={exact ? 'true' : undefined}
          style={{
            ...(centred
              ? { width: 'max-content', maxWidth: '100%' }
              : {
                  left: `${(((style?.marginHorizontal ?? 90) * plan.height) / 1920 / plan.width) * 100}%`,
                  right: `${(((style?.marginHorizontal ?? 90) * plan.height) / 1920 / plan.width) * 100}%`,
                }),
            ...position,
            fontFamily:
              !style || style.fontFamily === 'Inter' ? 'ClipMill Caption Inter' : style.fontFamily,
            fontSize: relative(style?.fontSize ?? 84),
            fontWeight: style?.bold === false ? 400 : 700,
            WebkitTextStroke: style?.boxed
              ? undefined
              : `${relative(style?.outlineWidth ?? 5)} ${style?.outline ?? '#000000'}`,
            paintOrder: 'stroke fill',
            textShadow: style?.boxed
              ? undefined
              : `${relative(style?.shadowDepth ?? 2)} ${relative(style?.shadowDepth ?? 2)} 0 ${style?.shadow ?? '#000000'}`,
          }}
          data-testid="caption"
        >
          {cue.lines.map((line, lineIndex) => (
            // eslint-disable-next-line react/no-array-index-key -- lines have no
            // identity of their own; their position is what they are.
            <span key={lineIndex} className="block whitespace-nowrap">
              <span
                style={
                  style?.boxed
                    ? {
                        background: style.outline,
                        padding: `${relative(style.outlineWidth)} ${relative(style.outlineWidth * 2)}`,
                        boxDecorationBreak: 'clone',
                      }
                    : undefined
                }
              >
                {line.map((word) => {
                  const mine = index;
                  index += 1;
                  return (
                    <span
                      key={`${word.text}-${mine}`}
                      style={{
                        // Without a sweep every word is in the words colour,
                        // as the export draws it.
                        color:
                          cue.karaoke && mine <= highlighted
                            ? (style?.spoken ?? '#ffd65c')
                            : (style?.unspoken ?? '#ffffff'),
                      }}
                    >
                      {word.text}{' '}
                    </span>
                  );
                })}
              </span>
            </span>
          ))}
        </p>
      )}
      {proxyUrl &&
        view === 'edit' &&
        shownOverlays.map((overlay) => {
          const moved = overlayDrag?.overlayId === overlay.overlayId ? overlayDrag : null;
          return (
            <p
              key={overlay.overlayId}
              className="edit-overlay-text"
              data-testid="overlay-text"
              data-selected={
                selection.kind === 'overlay' && selection.overlayId === overlay.overlayId
                  ? 'true'
                  : undefined
              }
              data-dragging={moved ? 'true' : undefined}
              data-exact={exact ? 'true' : undefined}
              onPointerDown={(event) => grabOverlay(event, overlay)}
              style={{
                left: `${(moved?.x ?? overlay.x / 1000) * 100}%`,
                top: `${(moved?.y ?? overlay.y / 1000) * 100}%`,
                fontFamily:
                  !style || style.fontFamily === 'Inter'
                    ? 'ClipMill Caption Inter'
                    : style.fontFamily,
                fontSize: relative(overlay.size),
                color: overlay.colour,
                background: overlay.plate ?? undefined,
                padding: overlay.plate ? relative(Math.max(8, overlay.size / 5)) : undefined,
                WebkitTextStroke: overlay.plate
                  ? undefined
                  : `${relative(Math.max(2, overlay.size / 16))} #000000`,
              }}
            >
              {overlay.text}
            </p>
          );
        })}
    </div>
  );
}

function share(value: number): string {
  return `${(value * 100).toFixed(2)}%`;
}

function Transport({
  plan,
  playback,
  range,
  disabled,
}: {
  readonly plan: PreviewPlan;
  readonly playback: MonitorPlayback;
  readonly range: { readonly first: number; readonly last: number } | null;
  readonly disabled: boolean;
}) {
  const { frame, playing, speed, loop, muted } = playback;
  const ticks = ticksAt(plan, frame);
  const length = ticksAt(plan, plan.frameCount);
  const format = useTimeFormat();
  const fps = plan.rateNum / Math.max(1, plan.rateDen);
  const recording = sourceTicksAt(plan, frame);
  return (
    <div className="review-transport" aria-label="Transport">
      <span
        className="review-timecode mono"
        data-testid="timecode"
        title={recording === null ? undefined : `${clockTenths(recording)} in the recording`}
      >
        {formatTime(ticks, format, fps)}
        <span className="edit-length"> / {formatTime(length, format, fps)}</span>
        <span className="sr-only">
          {' '}
          · frame {frame} of {plan.frameCount}
        </span>
      </span>
      <div className="review-transport-buttons">
        <TipButton label="Go to the start" disabled={disabled} onClick={() => playback.onSeek(0)}>
          <ChevronFirst className="size-4" />
        </TipButton>
        <TipButton
          label="Previous frame"
          disabled={disabled || frame === 0}
          onClick={() => playback.onStep(-1)}
        >
          <StepBack className="size-4" />
        </TipButton>
        <TipButton
          label={playing ? 'Pause' : 'Play'}
          size="icon"
          disabled={disabled}
          className="review-play"
          onClick={playback.onToggle}
        >
          {playing ? <Pause className="size-4" /> : <Play className="size-4" />}
        </TipButton>
        <TipButton
          label="Next frame"
          disabled={disabled || frame >= plan.frameCount - 1}
          onClick={() => playback.onStep(1)}
        >
          <StepForward className="size-4" />
        </TipButton>
        <TipButton
          label="Go to the end"
          disabled={disabled}
          onClick={() => playback.onSeek(plan.frameCount - 1)}
        >
          <ChevronLast className="size-4" />
        </TipButton>
      </div>
      <div className="review-transport-tools">
        <TipButton
          label="Playback speed"
          disabled={disabled}
          className="review-speed mono"
          onClick={() => {
            const at = SPEEDS.indexOf(speed as (typeof SPEEDS)[number]);
            playback.onSpeed(SPEEDS[(at + 1) % SPEEDS.length]!);
          }}
        >
          {`${speed}×`}
        </TipButton>
        <TipButton
          label={range ? 'Loop the marked range' : 'Loop the clip'}
          pressed={loop}
          disabled={disabled}
          onClick={() => playback.onLoop(!loop)}
        >
          <Repeat className="size-4" />
        </TipButton>
        <TipButton
          label={muted ? 'Unmute' : 'Mute'}
          pressed={muted}
          disabled={disabled}
          onClick={() => playback.onMuted(!muted)}
        >
          {muted ? <VolumeX className="size-4" /> : <Volume2 className="size-4" />}
        </TipButton>
      </div>
    </div>
  );
}

/** A share of the frame, kept on it. */
function clampShare(value: number): number {
  return Math.max(0.02, Math.min(0.98, value));
}

/** The horizontal centre pulls a caption dragged close to it. */
function snapCentre(value: number): number {
  return Math.abs(value - 0.5) < 0.025 ? 0.5 : value;
}
