/**
 * Review monitor: the vertical result or the source frame with its crop outlined.
 * Frames are composed with the editor's drawing code, so the preview matches the
 * edit an approval creates; the video element only decodes.
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
import { type CSSProperties, useEffect, useRef, useState, useSyncExternalStore } from 'react';

import type { CropPath, PreviewPlan } from '../daemon/client.js';
import { CaptionCanvas } from '../editor/CaptionCanvas.js';
import { drawComposition, observeVideoFrames } from '../editor/CompositionCanvas.js';
import type { ExactCaptions } from '../editor/exactCaptions.js';
import { cropAt, sourceOf } from '../editor/player.js';
import { framingNote, programAt, scaledSegment } from './dryRun.js';
import { TICKS_PER_SECOND } from '../results/model.js';
import { type PlaybackController, SPEEDS } from './playback.js';
import { formatTime, useTimeFormat } from '../shell/timeFormat.js';
import { type Cut, SAFE_AREA, clockTenths, cropRect } from './review.js';
import { TipButton } from './TipButton.js';

export type MonitorView = 'result' | 'source';

export interface MonitorProps {
  /** The proxy, or null when this run published none. */
  readonly src: string | null;
  /** The solver's path. Null while it is being asked, and for a fitted frame. */
  readonly crop: CropPath | null;
  /**
   * The clip an approval of the cut on screen would build. When present the
   * result is drawn from it — its sections, framing and captions — and the
   * solver's path is only the stand-in while it is asked for.
   */
  readonly plan?: PreviewPlan | null;
  /** Its captions for libass, when they can be drawn exactly. */
  readonly captions?: ExactCaptions | null;
  readonly controller: PlaybackController;
  /** The cut on screen, for the clip-relative clock. */
  readonly cut: Cut;
  readonly view: MonitorView;
  readonly onView: (view: MonitorView) => void;
  readonly safeArea: boolean;
  readonly onSafeArea: (shown: boolean) => void;
  /** Present when the search published a runner-up to compare against. */
  readonly alternative: {
    readonly shown: boolean;
    readonly onShow: (shown: boolean) => void;
  } | null;
}

export function Monitor({
  src,
  crop,
  plan = null,
  captions = null,
  controller,
  cut,
  view,
  onView,
  safeArea,
  onSafeArea,
  alternative,
}: MonitorProps) {
  const fitted = !crop || crop.fit || crop.keyframes.length === 0;
  return (
    <section className="review-viewer" aria-label="Preview">
      <div className="review-viewer-bar">
        <div className="review-segmented" role="group" aria-label="What the preview shows">
          <button type="button" aria-pressed={view === 'result'} onClick={() => onView('result')}>
            Result
          </button>
          <button type="button" aria-pressed={view === 'source'} onClick={() => onView('source')}>
            Source
          </button>
        </div>
        {plan ? (
          <PlanNote plan={plan} controller={controller} />
        ) : (
          <span className="review-viewer-note">
            {fitted
              ? crop?.fitReason
                ? `Whole frame · ${crop.fitReason}`
                : 'Whole frame'
              : 'Following the speaker'}
          </span>
        )}
        <span className="review-spacer" />
        {alternative && (
          <button
            type="button"
            className="review-viewer-toggle"
            aria-pressed={alternative.shown}
            onClick={() => alternative.onShow(!alternative.shown)}
          >
            Alternative cut
          </button>
        )}
        <button
          type="button"
          className="review-viewer-toggle"
          aria-pressed={safeArea}
          disabled={view !== 'result'}
          onClick={() => onSafeArea(!safeArea)}
        >
          Safe area
        </button>
        <span className="review-viewer-note mono">9:16</span>
      </div>

      <div className="review-stage-wrap">
        {src ? (
          <Stage
            src={src}
            crop={crop}
            plan={plan}
            captions={captions}
            controller={controller}
            view={view}
            safeArea={safeArea}
          />
        ) : (
          <div className="review-unavailable" role="note">
            <p className="review-unavailable-title">Preview unavailable</p>
            <p>This analysis published no proxy video, so there is nothing to play.</p>
          </div>
        )}
      </div>

      <Transport controller={controller} cut={cut} disabled={!src} />
    </section>
  );
}

/** The framing at the playhead, read from the clip an approval would build. */
function PlanNote({
  plan,
  controller,
}: {
  readonly plan: PreviewPlan;
  readonly controller: PlaybackController;
}) {
  const ticks = useSyncExternalStore(controller.subscribe, () => controller.getState().ticks);
  const decisions = plan.decisions ?? [];
  return (
    <span className="review-viewer-note" title={decisions.join('\n') || undefined}>
      {framingNote(plan, programAt(plan, ticks))}
    </span>
  );
}

/** The plan's captions at the playhead, drawn by libass; none outside the cut. */
function PlanCaptions({
  plan,
  captions,
  controller,
}: {
  readonly plan: PreviewPlan;
  readonly captions: ExactCaptions;
  readonly controller: PlaybackController;
}) {
  const ticks = useSyncExternalStore(controller.subscribe, () => controller.getState().ticks);
  const moment = programAt(plan, ticks);
  if (!moment?.inside) return null;
  return (
    <CaptionCanvas
      ass={captions.ass}
      faces={captions.faces}
      family={captions.family}
      seconds={(moment.frame * plan.rateDen) / Math.max(1, plan.rateNum)}
      frameWidth={plan.width}
      frameHeight={plan.height}
    />
  );
}

function Stage({
  src,
  crop,
  plan,
  captions,
  controller,
  view,
  safeArea,
}: {
  readonly src: string;
  readonly crop: CropPath | null;
  readonly plan: PreviewPlan | null;
  readonly captions: ExactCaptions | null;
  readonly controller: PlaybackController;
  readonly view: MonitorView;
  readonly safeArea: boolean;
}) {
  const video = useRef<HTMLVideoElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const stage = useRef<HTMLDivElement>(null);
  const [aspect, setAspect] = useState(16 / 9);
  const problem = useSyncExternalStore(controller.subscribe, () => controller.getState().problem);
  const latest = useRef({ crop, view, plan });
  latest.current = { crop, view, plan };
  const observer = useRef<ReturnType<typeof observeVideoFrames> | null>(null);

  useEffect(() => {
    const media = video.current;
    controller.attach(media);
    return () => controller.attach(null);
  }, [controller, src]);

  // The canvas is as sharp as the space it has, never more: a full 1080×1920
  // composition per frame is the render's job, not a reviewer's preview.
  useEffect(() => {
    const box = stage.current;
    const output = canvas.current;
    if (!box || !output) return;
    const size = () => {
      const scale = Math.min(2, window.devicePixelRatio || 1);
      output.width = Math.max(1, Math.round(box.clientWidth * scale));
      output.height = Math.max(1, Math.round(box.clientHeight * scale));
      observer.current?.redraw();
    };
    size();
    if (typeof ResizeObserver === 'undefined') return;
    const watcher = new ResizeObserver(size);
    watcher.observe(box);
    return () => watcher.disconnect();
  }, [view, aspect]);

  useEffect(() => {
    const media = video.current;
    const output = canvas.current;
    if (!media || !output) return;
    const draw = (seconds: number, picture: HTMLCanvasElement) => {
      controller.presented(seconds);
      const context = output.getContext('2d');
      if (!context || media.seeking) return;
      const frame = { width: picture.width, height: picture.height };
      const drawing = latest.current.plan;
      const moment = drawing ? programAt(drawing, seconds * TICKS_PER_SECOND) : null;
      // The approval's own crop where there is a plan; the solver's otherwise.
      const planned = drawing && moment ? cropAt(drawing, moment.frame) : null;
      const rect =
        drawing && moment
          ? planned && sourceOf(drawing, moment.segment)
            ? scaledRect(planned, sourceOf(drawing, moment.segment)!, frame)
            : null
          : cropRect(latest.current.crop, seconds * TICKS_PER_SECOND, frame);
      try {
        if (latest.current.view === 'result' && drawing && moment) {
          const scale = output.width / Math.max(1, drawing.width);
          drawComposition(context, picture, {
            crop: planned,
            secondary: cropAt(drawing, moment.frame, true),
            source: sourceOf(drawing, moment.segment),
            width: output.width,
            height: output.height,
            segment: scaledSegment(moment.segment, scale),
          });
          return;
        }
        if (latest.current.view === 'result') {
          drawComposition(context, picture, {
            crop: rect,
            secondary: null,
            source: null,
            width: output.width,
            height: output.height,
          });
          return;
        }
        context.globalCompositeOperation = 'copy';
        context.drawImage(picture, 0, 0, output.width, output.height);
        context.globalCompositeOperation = 'source-over';
        if (rect) {
          // What the camera leaves out, dimmed; what it keeps, outlined.
          const sx = output.width / frame.width;
          const sy = output.height / frame.height;
          const x = rect.x * sx;
          const y = rect.y * sy;
          const w = rect.width * sx;
          const h = rect.height * sy;
          context.fillStyle = 'rgb(0 0 0 / 0.5)';
          context.fillRect(0, 0, x, output.height);
          context.fillRect(x + w, 0, output.width - x - w, output.height);
          context.fillRect(x, 0, w, y);
          context.fillRect(x, y + h, w, output.height - y - h);
          context.strokeStyle = 'rgb(255 255 255 / 0.9)';
          context.lineWidth = Math.max(1, output.width / 480);
          context.strokeRect(x, y, w, h);
        }
      } catch (error) {
        // A decoder can drop its frame mid-transition; anything else is a bug.
        if (!(error instanceof DOMException && error.name === 'InvalidStateError')) throw error;
      }
    };
    const active = observeVideoFrames(media, draw);
    observer.current = active;
    return () => {
      active.dispose();
      observer.current = null;
    };
  }, [controller, src]);

  // A new path, plan or view has no new frame to trigger a draw.
  useEffect(() => {
    observer.current?.redraw();
  }, [crop, plan, view]);

  return (
    <div
      ref={stage}
      className="review-stage"
      data-view={view}
      style={{ '--review-source-aspect': String(aspect) } as CSSProperties}
    >
      {/* eslint-disable-next-line jsx-a11y/media-has-caption -- the transcript
          beside the preview is the text of what is said. */}
      <video
        ref={video}
        src={src}
        className="review-decoder"
        crossOrigin="anonymous"
        playsInline
        preload="auto"
        data-testid="proxy"
        aria-hidden="true"
        tabIndex={-1}
        onLoadedMetadata={(event) => {
          const media = event.currentTarget;
          if (media.videoWidth > 0 && media.videoHeight > 0) {
            setAspect(media.videoWidth / media.videoHeight);
          }
        }}
      />
      <canvas
        ref={canvas}
        className="review-canvas"
        role="img"
        aria-label={
          view === 'result'
            ? 'The clip, framed for vertical video'
            : 'The source frame, with the crop outlined'
        }
      />
      {plan && captions && view === 'result' && (
        <PlanCaptions plan={plan} captions={captions} controller={controller} />
      )}
      {safeArea && view === 'result' && <SafeAreaGuide />}
      {problem && (
        <div role="alert" className="review-unavailable review-stage-problem">
          <p className="review-unavailable-title">Preview needs attention</p>
          <p>{problem}</p>
        </div>
      )}
    </div>
  );
}

/** A crop in the source's display pixels, as a rectangle of the decoded picture. */
function scaledRect(
  rect: { readonly x: number; readonly y: number; readonly width: number; readonly height: number },
  source: { readonly displayWidth: number; readonly displayHeight: number },
  frame: { readonly width: number; readonly height: number },
) {
  const sx = frame.width / Math.max(1, source.displayWidth);
  const sy = frame.height / Math.max(1, source.displayHeight);
  return { x: rect.x * sx, y: rect.y * sy, width: rect.width * sx, height: rect.height * sy };
}

/** Where the apps' own buttons and captions sit over the clip. */
function SafeAreaGuide() {
  const percent = (share: number) => `${(share * 100).toFixed(2)}%`;
  return (
    <div className="review-safe-area" aria-hidden="true">
      <span style={{ top: 0, left: 0, right: 0, height: percent(SAFE_AREA.top) }} />
      <span style={{ bottom: 0, left: 0, right: 0, height: percent(SAFE_AREA.bottom) }} />
      <span
        style={{
          top: percent(SAFE_AREA.top),
          bottom: percent(SAFE_AREA.bottom),
          left: 0,
          width: percent(SAFE_AREA.left),
        }}
      />
      <span
        style={{
          top: percent(SAFE_AREA.top),
          bottom: percent(SAFE_AREA.bottom),
          right: 0,
          width: percent(SAFE_AREA.right),
        }}
      />
      <i
        style={{
          top: percent(SAFE_AREA.top),
          bottom: percent(SAFE_AREA.bottom),
          left: percent(SAFE_AREA.left),
          right: percent(SAFE_AREA.right),
        }}
      />
    </div>
  );
}

function Transport({
  controller,
  cut,
  disabled,
  fps = 30_000 / 1_001,
}: {
  readonly controller: PlaybackController;
  readonly cut: Cut;
  readonly disabled: boolean;
  /** The recording's frame rate, for a frame count. */
  readonly fps?: number;
}) {
  const ticks = useSyncExternalStore(controller.subscribe, () => controller.getState().ticks);
  const playing = useSyncExternalStore(controller.subscribe, () => controller.getState().playing);
  const speed = useSyncExternalStore(controller.subscribe, () => controller.getState().speed);
  const loop = useSyncExternalStore(controller.subscribe, () => controller.getState().loop);
  const muted = useSyncExternalStore(controller.subscribe, () => controller.getState().muted);
  const into = ticks - cut.startTicks;
  const length = cut.endTicks - cut.startTicks;
  const format = useTimeFormat();
  // Clip time, as the Editor and the export count it; where it is in the
  // recording is a hover away rather than a second clock beside it.
  return (
    <div className="review-transport" aria-label="Transport">
      <span
        className="review-timecode mono"
        data-testid="timecode"
        title={`${clockTenths(ticks)} in the recording`}
      >
        {into < 0 ? `−${formatTime(-into, format, fps)}` : formatTime(into, format, fps)}
        <span className="edit-length"> / {formatTime(length, format, fps)}</span>
      </span>
      <div className="review-transport-buttons">
        <TipButton
          label="Go to the start of the cut"
          disabled={disabled}
          onClick={() => controller.seek(cut.startTicks)}
        >
          <ChevronFirst className="size-4" />
        </TipButton>
        <TipButton label="Back one frame" disabled={disabled} onClick={() => controller.step(-1)}>
          <StepBack className="size-4" />
        </TipButton>
        <TipButton
          label={playing ? 'Pause' : 'Play'}
          size="icon"
          disabled={disabled}
          className="review-play"
          onClick={() => controller.toggle()}
        >
          {playing ? <Pause className="size-4" /> : <Play className="size-4" />}
        </TipButton>
        <TipButton label="Forward one frame" disabled={disabled} onClick={() => controller.step(1)}>
          <StepForward className="size-4" />
        </TipButton>
        <TipButton
          label="Go to the end of the cut"
          disabled={disabled}
          onClick={() => controller.seek(cut.endTicks)}
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
            controller.setSpeed(SPEEDS[(at + 1) % SPEEDS.length]!);
          }}
        >
          {`${speed}×`}
        </TipButton>
        <TipButton
          label="Loop the cut"
          pressed={loop}
          disabled={disabled}
          onClick={() => controller.setLoop(!loop)}
        >
          <Repeat className="size-4" />
        </TipButton>
        <TipButton
          label={muted ? 'Unmute' : 'Mute'}
          pressed={muted}
          disabled={disabled}
          onClick={() => controller.setMuted(!muted)}
        >
          {muted ? <VolumeX className="size-4" /> : <Volume2 className="size-4" />}
        </TipButton>
      </div>
    </div>
  );
}
