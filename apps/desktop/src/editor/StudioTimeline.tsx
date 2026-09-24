import { useEffect, useMemo, useRef, useState } from 'react';
import type { PointerEvent as ReactPointerEvent } from 'react';
import type { EditIr } from '@clipmill/contracts';
import {
  Captions,
  Film,
  Magnet,
  MapPin,
  Scan,
  Scissors,
  Volume2,
  ZoomIn,
  ZoomOut,
} from 'lucide-react';

import { Button } from '../components/ui/button.js';
import type { EditCommandJson, PreviewPlan } from '../daemon/client.js';
import type { Filmstrip, Peaks } from '../results/loader.js';
import type { Transcript } from '../results/transcript.js';
import { waveformPath } from '../results/waveform.js';
import {
  batch,
  extendWithCaptions,
  removeCropKeyframe,
  removeGainPoint,
  setCropKeyframe,
  setCueTiming,
  setGain,
  splitSegment,
  ticksAt,
  trim,
  trimEndAt,
  trimStartAt,
} from './commands.js';
import type { EditorSelection } from './EditorProperties.js';
import { frameAt, segmentAt, sourceTicksAt, timecode } from './player.js';

const TICKS = 90_000;
const MIN_ZOOM = 1;
const MAX_ZOOM = 8;
type Tool = 'select' | 'blade';

function percent(frame: number, count: number) {
  return `${(frame / Math.max(1, count - 1)) * 100}%`;
}
function frameFromPoint(x: number, rect: DOMRect, count: number) {
  return Math.max(
    0,
    Math.min(count - 1, Math.round(((x - rect.left) / Math.max(1, rect.width)) * (count - 1))),
  );
}
function gainFromPoint(y: number, rect: DOMRect) {
  return Math.max(
    -12,
    Math.min(12, Math.round((1 - (y - rect.top) / Math.max(1, rect.height)) * 24 - 12)),
  );
}
function programFrame(plan: PreviewPlan, ticks: number) {
  return frameAt(plan, ticks / TICKS);
}

function Waveform({
  plan,
  peaks,
  transcript,
}: {
  readonly plan: PreviewPlan;
  readonly peaks: Peaks | null;
  readonly transcript: Transcript | null;
}) {
  const spans = useMemo(
    () =>
      peaks && peaks.bucketTicks > 0
        ? plan.segments.map((segment) => ({
            segment,
            path: waveformPath(peaks, segment.inTicks, segment.outTicks),
          }))
        : [],
    [plan, peaks],
  );
  const sentenceStarts = useMemo(
    () =>
      transcript?.sentences.flatMap((sentence) =>
        plan.segments.flatMap((segment) =>
          sentence.startTicks >= segment.inTicks && sentence.startTicks < segment.outTicks
            ? [segment.programStartTicks + sentence.startTicks - segment.inTicks]
            : [],
        ),
      ) ?? [],
    [plan, transcript],
  );
  if (!spans.some((span) => span.path))
    return <span className="studio-waveform-empty">Waveform unavailable</span>;
  const duration = plan.segments.reduce((sum, item) => sum + item.outTicks - item.inTicks, 0);
  return (
    <div className="studio-waveform" role="img" aria-label="Audio waveform">
      {spans.map(({ segment, path }) =>
        path ? (
          <svg
            key={segment.segmentId}
            viewBox="0 0 100 100"
            preserveAspectRatio="none"
            style={{
              left: `${(segment.programStartTicks / duration) * 100}%`,
              width: `${((segment.outTicks - segment.inTicks) / duration) * 100}%`,
            }}
          >
            <path d={path} />
          </svg>
        ) : null,
      )}
      {sentenceStarts.map((ticks) => (
        <span
          key={ticks}
          className="studio-sentence-tick"
          style={{ left: `${(ticks / duration) * 100}%` }}
        />
      ))}
    </div>
  );
}

export function StudioTimeline({
  plan,
  document,
  transcript,
  filmstrip,
  peaks,
  frame,
  docId,
  selection,
  busy,
  zoom,
  onZoom,
  snap,
  onSnap,
  tool,
  onTool,
  markers,
  onMarker,
  filmstripUrl,
  onSeek,
  onSelect,
  onApply,
}: {
  readonly plan: PreviewPlan;
  readonly document: EditIr | null;
  readonly transcript: Transcript | null;
  readonly filmstrip: Filmstrip | null;
  readonly peaks: Peaks | null;
  readonly frame: number;
  readonly docId: string;
  readonly selection: EditorSelection;
  readonly busy: boolean;
  readonly zoom: number;
  readonly onZoom: (value: number) => void;
  readonly snap: boolean;
  readonly onSnap: (value: boolean) => void;
  readonly tool: Tool;
  readonly onTool: (value: Tool) => void;
  readonly markers: readonly number[];
  readonly onMarker: (frame: number) => void;
  readonly filmstripUrl: (file: string) => string;
  readonly onSeek: (frame: number) => void;
  readonly onSelect: (value: EditorSelection) => void;
  readonly onApply: (command: EditCommandJson) => void;
}) {
  const scroller = useRef<HTMLDivElement>(null);
  const [ghost, setGhost] = useState<{
    kind: 'cue' | 'edge' | 'keyframe';
    first: number;
    end?: number;
  } | null>(null);
  const duration = plan.segments.reduce((sum, item) => sum + item.outTicks - item.inTicks, 0);
  const snapTargets = useMemo(
    () => [
      0,
      plan.frameCount - 1,
      ...plan.cues.flatMap((cue) => [cue.firstFrame, cue.endFrame]),
      ...plan.segments.flatMap((item) => [item.firstFrame, item.endFrame]),
      ...markers,
    ],
    [plan, markers],
  );
  const snapped = (value: number) => {
    if (!snap) return value;
    const nearest = snapTargets.reduce(
      (best, target) => (Math.abs(target - value) < Math.abs(best - value) ? target : best),
      value,
    );
    return Math.abs(nearest - value) <= Math.max(2, Math.round(plan.frameCount / (200 * zoom)))
      ? nearest
      : value;
  };
  const stretch = (
    event: ReactPointerEvent,
    onMove: (frame: number) => void,
    onEnd: (frame: number, outsideTicks: number) => void,
  ) => {
    event.preventDefault();
    event.stopPropagation();
    const canvas = event.currentTarget.closest<HTMLElement>('.studio-timeline-canvas');
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const rawTicks = (x: number) =>
      Math.round(((x - rect.left) / Math.max(1, rect.width)) * duration);
    const move = (next: PointerEvent) =>
      onMove(snapped(frameFromPoint(next.clientX, rect, plan.frameCount)));
    const finish = (next: PointerEvent) => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', finish);
      onEnd(snapped(frameFromPoint(next.clientX, rect, plan.frameCount)), rawTicks(next.clientX));
      setGhost(null);
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', finish);
  };
  const seekAt = (event: ReactPointerEvent) => {
    const rect = event.currentTarget.getBoundingClientRect();
    const at = snapped(frameFromPoint(event.clientX, rect, plan.frameCount));
    if (tool === 'blade' && !busy) {
      const section = segmentAt(plan, at);
      const ticks = sourceTicksAt(plan, at);
      if (section && ticks !== null && ticks > section.inTicks && ticks < section.outTicks) {
        const existing = new Set(document?.video.segments?.map((item) => item.segment_id));
        let id = `${section.segmentId}_cut_${ticks}`;
        let suffix = 2;
        while (existing.has(id)) id = `${section.segmentId}_cut_${ticks}_${suffix++}`;
        onApply(splitSegment(section.segmentId, ticks, id));
        onTool('select');
      }
    } else onSeek(at);
  };
  const rulerStep = zoom >= 5 ? 2 : zoom >= 2.5 ? 5 : 10;
  const marks = Array.from({ length: Math.ceil(duration / TICKS / rulerStep) + 1 }, (_, index) =>
    Math.min(plan.frameCount - 1, programFrame(plan, index * rulerStep * TICKS)),
  );
  const cropPoints =
    document?.video.segments?.flatMap((part) => {
      const preview = plan.segments.find((item) => item.segmentId === part.segment_id);
      if (!preview) return [];
      return [
        ...(part.layout.crop_path ?? []).map((point) => ({ point, preview, secondary: false })),
        ...(part.layout.secondary_crop_path ?? []).map((point) => ({
          point,
          preview,
          secondary: true,
        })),
      ];
    }) ?? [];

  useEffect(() => {
    const lane = scroller.current;
    if (!lane) return;
    const x = (frame / Math.max(1, plan.frameCount - 1)) * lane.scrollWidth;
    if (x < lane.scrollLeft || x > lane.scrollLeft + lane.clientWidth)
      lane.scrollLeft = Math.max(0, x - lane.clientWidth / 2);
  }, [frame, plan.frameCount]);

  return (
    <section className="studio-timeline" aria-label="Timeline">
      <div className="studio-timeline-toolbar">
        <div className="studio-tool-group" aria-label="Timeline tools">
          <button
            type="button"
            aria-pressed={tool === 'select'}
            onClick={() => onTool('select')}
            title="Select (V)"
          >
            Select <kbd>V</kbd>
          </button>
          <button
            type="button"
            aria-pressed={tool === 'blade'}
            onClick={() => onTool('blade')}
            title="Blade (B)"
          >
            <Scissors className="size-3.5" /> Blade <kbd>B</kbd>
          </button>
        </div>
        <div className="studio-tool-group">
          <button
            type="button"
            aria-pressed={snap}
            onClick={() => onSnap(!snap)}
            title="Snapping (S)"
          >
            <Magnet className="size-3.5" /> Snap
          </button>
          <button type="button" onClick={() => onMarker(frame)} title="Add marker (M)">
            <MapPin className="size-3.5" /> Marker
          </button>
        </div>
        <div className="studio-zoom">
          <Button
            size="icon-xs"
            variant="ghost"
            aria-label="Zoom out timeline"
            onClick={() => onZoom(Math.max(MIN_ZOOM, zoom / 1.5))}
          >
            <ZoomOut />
          </Button>
          <input
            aria-label="Timeline zoom"
            type="range"
            min={1}
            max={8}
            step={0.25}
            value={zoom}
            onChange={(event) => onZoom(Number(event.target.value))}
          />
          <Button
            size="icon-xs"
            variant="ghost"
            aria-label="Zoom in timeline"
            onClick={() => onZoom(Math.min(MAX_ZOOM, zoom * 1.5))}
          >
            <ZoomIn />
          </Button>
          <button type="button" onClick={() => onZoom(1)}>
            Fit
          </button>
        </div>
      </div>
      <div className="studio-timeline-grid">
        <div className="studio-timeline-labels">
          <span className="studio-ruler-label">{timecode(plan, frame)}</span>
          <span>
            <Film /> Video
          </span>
          <span>
            <Scan /> Framing
          </span>
          <span>
            <Captions /> Captions
          </span>
          <span>
            <Volume2 /> Audio
          </span>
        </div>
        <div className="studio-timeline-scroll" ref={scroller}>
          <div className="studio-timeline-canvas" style={{ width: `${zoom * 100}%` }}>
            <div
              className="studio-timeline-ruler"
              onPointerDown={(event) => stretch(event, onSeek, onSeek)}
            >
              {marks.map((mark) => (
                <span key={mark} style={{ left: percent(mark, plan.frameCount) }}>
                  {timecode(plan, mark)}
                </span>
              ))}
              {markers.map((at, index) => (
                <span
                  key={`marker-${at}-${index}`}
                  className="studio-marker"
                  style={{ left: percent(at, plan.frameCount) }}
                  title={`Marker ${index + 1} · ${timecode(plan, at)}`}
                />
              ))}
            </div>
            <div className="studio-track studio-video-track" onPointerDown={seekAt}>
              {plan.segments.map((part, index) => {
                const first = part.firstFrame;
                const end = part.endFrame;
                const tiles =
                  filmstrip?.tiles.filter(
                    (tile) => tile.tTicks >= part.inTicks && tile.tTicks < part.outTicks,
                  ) ?? [];
                const stride = Math.max(1, Math.ceil(tiles.length / 120));
                return (
                  <div
                    key={part.segmentId}
                    className="studio-video-segment"
                    data-selected={
                      selection.kind === 'section' && selection.segmentId === part.segmentId
                    }
                    style={{
                      left: percent(first, plan.frameCount),
                      width: `${((end - first) / Math.max(1, plan.frameCount - 1)) * 100}%`,
                    }}
                    onPointerDown={(event) => {
                      if (event.target === event.currentTarget) {
                        onSelect({ kind: 'section', segmentId: part.segmentId });
                        onSeek(first);
                      }
                    }}
                  >
                    {tiles
                      .filter((_item, at) => at % stride === 0)
                      .map((tile) => (
                        <img
                          key={tile.file}
                          src={filmstripUrl(tile.file)}
                          alt=""
                          draggable={false}
                          style={{
                            left: `${((tile.tTicks - part.inTicks) / (part.outTicks - part.inTicks)) * 100}%`,
                          }}
                        />
                      ))}
                    <span className="studio-segment-label">
                      {String(index + 1).padStart(2, '0')} ·{' '}
                      {part.hasTwoUpPaths ? 'Two-up' : 'Video'}
                    </span>
                    <button
                      type="button"
                      className="studio-trim-handle start"
                      aria-label={`Drag start of section ${index + 1}`}
                      disabled={busy}
                      onPointerDown={(event) => {
                        stretch(
                          event,
                          (at) => setGhost({ kind: 'edge', first: at }),
                          (at, rawTicks) => {
                            if (index === 0 && rawTicks < 0 && transcript) {
                              const wanted = Math.max(
                                transcript.words[0]?.startTicks ?? 0,
                                part.inTicks + rawTicks,
                              );
                              const boundary = transcript.words
                                .filter((word) => word.startTicks < part.inTicks)
                                .reduce(
                                  (best, word) =>
                                    Math.abs(word.startTicks - wanted) < Math.abs(best - wanted)
                                      ? word.startTicks
                                      : best,
                                  wanted,
                                );
                              if (boundary < part.inTicks)
                                onApply(
                                  extendWithCaptions(part.segmentId, boundary, part.outTicks),
                                );
                              return;
                            }
                            if (at <= part.firstFrame) return;
                            const candidate =
                              index === 0
                                ? trimStartAt(plan, at)
                                : (() => {
                                    const ticks =
                                      part.inTicks + ticksAt(plan, at) - part.programStartTicks;
                                    return ticks > part.inTicks && ticks < part.outTicks
                                      ? trim(ticks, part.outTicks, part.segmentId)
                                      : null;
                                  })();
                            if (candidate) onApply(candidate);
                          },
                        );
                      }}
                    />
                    <button
                      type="button"
                      className="studio-trim-handle end"
                      aria-label={`Drag end of section ${index + 1}`}
                      disabled={busy}
                      onPointerDown={(event) => {
                        stretch(
                          event,
                          (at) => setGhost({ kind: 'edge', first: at }),
                          (at, rawTicks) => {
                            if (
                              index === plan.segments.length - 1 &&
                              rawTicks > duration &&
                              transcript
                            ) {
                              const wanted = Math.min(
                                transcript.words.at(-1)?.endTicks ?? part.outTicks,
                                part.outTicks + rawTicks - duration,
                              );
                              const boundary = transcript.words
                                .filter((word) => word.endTicks > part.outTicks)
                                .reduce(
                                  (best, word) =>
                                    Math.abs(word.endTicks - wanted) < Math.abs(best - wanted)
                                      ? word.endTicks
                                      : best,
                                  wanted,
                                );
                              if (boundary > part.outTicks)
                                onApply(extendWithCaptions(part.segmentId, part.inTicks, boundary));
                              return;
                            }
                            if (at >= part.endFrame - 1) return;
                            const candidate =
                              index === plan.segments.length - 1
                                ? trimEndAt(plan, at)
                                : (() => {
                                    const ticks =
                                      part.inTicks + ticksAt(plan, at) - part.programStartTicks;
                                    return ticks > part.inTicks && ticks < part.outTicks
                                      ? trim(part.inTicks, ticks, part.segmentId)
                                      : null;
                                  })();
                            if (candidate) onApply(candidate);
                          },
                        );
                      }}
                    />
                  </div>
                );
              })}
            </div>
            <div className="studio-track studio-framing-track" onPointerDown={seekAt}>
              {plan.segments.map((part) => (
                <span
                  key={part.segmentId}
                  className="studio-framing-run"
                  style={{
                    left: percent(part.firstFrame, plan.frameCount),
                    width: `${((part.endFrame - part.firstFrame) / Math.max(1, plan.frameCount - 1)) * 100}%`,
                  }}
                >
                  {part.hasTwoUpPaths ? 'Two-up' : plan.crops[part.firstFrame] ? 'Fill' : 'Fit'}
                </span>
              ))}
              {cropPoints.map(({ point, preview, secondary }) => {
                const at = programFrame(plan, preview.programStartTicks + point.t_ticks);
                return (
                  <button
                    key={`${preview.segmentId}-${point.t_ticks}-${secondary}`}
                    type="button"
                    className="studio-keyframe"
                    style={{ left: percent(at, plan.frameCount) }}
                    aria-label={`${secondary ? 'Lower' : 'Upper'} framing keyframe at ${timecode(plan, at)}`}
                    onClick={(event) => {
                      event.stopPropagation();
                      onSelect({
                        kind: 'keyframe',
                        segmentId: preview.segmentId,
                        tTicks: point.t_ticks,
                        secondary,
                      });
                      onSeek(at);
                    }}
                    onPointerDown={(event) => {
                      stretch(
                        event,
                        (next) => setGhost({ kind: 'keyframe', first: next }),
                        (next) => {
                          const tick = ticksAt(plan, next) - preview.programStartTicks;
                          if (
                            tick === point.t_ticks ||
                            tick < 0 ||
                            tick > preview.outTicks - preview.inTicks
                          )
                            return;
                          onApply(
                            batch([
                              removeCropKeyframe(point.t_ticks, preview.segmentId, secondary),
                              setCropKeyframe(
                                tick,
                                point.rect,
                                preview.segmentId,
                                secondary,
                                point.easing,
                              ),
                            ]),
                          );
                        },
                      );
                    }}
                  />
                );
              })}
            </div>
            <div className="studio-track studio-caption-track" onPointerDown={seekAt}>
              {plan.cues.map((cue) => {
                const saved = (
                  plan.presentation === 'burn_in' && document?.captions.burn_in?.length
                    ? document.captions.burn_in
                    : document?.captions.cues
                )?.find((item) => item.cue_id === cue.cueId);
                const start = saved?.start_ticks ?? ticksAt(plan, cue.firstFrame);
                const end = saved?.end_ticks ?? ticksAt(plan, cue.endFrame);
                const drawFirst =
                  ghost?.kind === 'cue' && selection.kind === 'cue' && selection.cueId === cue.cueId
                    ? ghost.first
                    : cue.firstFrame;
                const drawEnd =
                  ghost?.kind === 'cue' && selection.kind === 'cue' && selection.cueId === cue.cueId
                    ? (ghost.end ?? cue.endFrame)
                    : cue.endFrame;
                return (
                  <div
                    key={cue.cueId}
                    className="studio-caption-block"
                    data-selected={selection.kind === 'cue' && selection.cueId === cue.cueId}
                    style={{
                      left: percent(drawFirst, plan.frameCount),
                      width: `${((drawEnd - drawFirst) / Math.max(1, plan.frameCount - 1)) * 100}%`,
                    }}
                    onPointerDown={(event) => {
                      onSelect({ kind: 'cue', cueId: cue.cueId });
                      onSeek(cue.firstFrame);
                      stretch(
                        event,
                        (next) =>
                          setGhost({
                            kind: 'cue',
                            first: next,
                            end: next + cue.endFrame - cue.firstFrame,
                          }),
                        (next) => {
                          if (next === cue.firstFrame) return;
                          const shifted = ticksAt(plan, next) - ticksAt(plan, cue.firstFrame);
                          onApply(
                            setCueTiming(
                              cue.cueId,
                              start + shifted,
                              end + shifted,
                              plan.presentation,
                            ),
                          );
                        },
                      );
                    }}
                  >
                    <button
                      type="button"
                      className="studio-caption-edge start"
                      aria-label={`Drag start of caption ${cue.cueId}`}
                      onPointerDown={(event) =>
                        stretch(
                          event,
                          (next) => setGhost({ kind: 'cue', first: next, end: cue.endFrame }),
                          (next) => {
                            if (next !== cue.firstFrame)
                              onApply(
                                setCueTiming(
                                  cue.cueId,
                                  ticksAt(plan, next),
                                  end,
                                  plan.presentation,
                                ),
                              );
                          },
                        )
                      }
                    />
                    <span>
                      {cue.lines
                        .flat()
                        .map((word) => word.text)
                        .join(' ')}
                    </span>
                    <button
                      type="button"
                      className="studio-caption-edge end"
                      aria-label={`Drag end of caption ${cue.cueId}`}
                      onPointerDown={(event) =>
                        stretch(
                          event,
                          (next) => setGhost({ kind: 'cue', first: cue.firstFrame, end: next }),
                          (next) => {
                            if (next !== cue.endFrame)
                              onApply(
                                setCueTiming(
                                  cue.cueId,
                                  start,
                                  ticksAt(plan, next),
                                  plan.presentation,
                                ),
                              );
                          },
                        )
                      }
                    />
                  </div>
                );
              })}
            </div>
            <div
              className="studio-track studio-audio-track"
              title="Double-click to add a volume point"
              onPointerDown={seekAt}
              onDoubleClick={(event) => {
                if (busy) return;
                const rect = event.currentTarget.getBoundingClientRect();
                const at = frameFromPoint(event.clientX, rect, plan.frameCount);
                const gain = gainFromPoint(event.clientY, rect);
                onApply(setGain(ticksAt(plan, at), gain));
                onSelect({ kind: 'gain', frame: at, tTicks: ticksAt(plan, at) });
              }}
            >
              <Waveform plan={plan} peaks={peaks} transcript={transcript} />
              {plan.gain.map((point, index) => (
                <button
                  type="button"
                  key={point.frame}
                  className="studio-gain-point"
                  aria-label={`Gain ${point.gainDb.toFixed(1)} dB at ${timecode(plan, point.frame)}`}
                  disabled={busy}
                  onPointerDown={(event) => {
                    event.preventDefault();
                    event.stopPropagation();
                    const button = event.currentTarget;
                    const track = button.closest<HTMLElement>('.studio-audio-track');
                    if (!track) return;
                    const rect = track.getBoundingClientRect();
                    const originalTicks =
                      document?.audio.gain_curve?.[index]?.t_ticks ?? ticksAt(plan, point.frame);
                    const atPoint = (next: PointerEvent) => ({
                      frame: snapped(frameFromPoint(next.clientX, rect, plan.frameCount)),
                      gainDb: gainFromPoint(next.clientY, rect),
                    });
                    const move = (next: PointerEvent) => {
                      const { frame: at, gainDb } = atPoint(next);
                      button.style.left = percent(at, plan.frameCount);
                      button.style.top = `${50 - (gainDb / 24) * 100}%`;
                    };
                    const finish = (next: PointerEvent) => {
                      window.removeEventListener('pointermove', move);
                      window.removeEventListener('pointerup', finish);
                      button.style.left = percent(
                        Math.min(point.frame, plan.frameCount - 1),
                        plan.frameCount,
                      );
                      button.style.top = `${50 - (point.gainDb / 24) * 100}%`;
                      const { frame: at, gainDb } = atPoint(next);
                      const targetTicks = ticksAt(plan, at);
                      if (targetTicks === originalTicks && gainDb === point.gainDb) return;
                      if (
                        targetTicks !== originalTicks &&
                        document?.audio.gain_curve?.some((item) => item.t_ticks === targetTicks)
                      )
                        return;
                      onApply(
                        targetTicks === originalTicks
                          ? setGain(targetTicks, gainDb)
                          : batch([removeGainPoint(originalTicks), setGain(targetTicks, gainDb)]),
                      );
                      onSelect({ kind: 'gain', frame: at, tTicks: targetTicks });
                      onSeek(at);
                    };
                    onSelect({
                      kind: 'gain',
                      frame: Math.min(point.frame, plan.frameCount - 1),
                      tTicks: originalTicks,
                    });
                    window.addEventListener('pointermove', move);
                    window.addEventListener('pointerup', finish);
                  }}
                  style={{
                    left: percent(Math.min(point.frame, plan.frameCount - 1), plan.frameCount),
                    top: `${50 - (point.gainDb / 24) * 100}%`,
                  }}
                />
              ))}
            </div>
            {ghost && (
              <span
                className="studio-drag-ghost"
                style={{ left: percent(ghost.first, plan.frameCount) }}
              />
            )}
            <span
              className="studio-playhead"
              style={{ left: percent(frame, plan.frameCount) }}
              data-testid="playhead"
            >
              <button
                type="button"
                aria-label="Drag playhead"
                onPointerDown={(event) => stretch(event, onSeek, onSeek)}
              />
            </span>
          </div>
        </div>
      </div>
      <input
        className="studio-scrub"
        type="range"
        aria-label="Scrub"
        min={0}
        max={Math.max(0, plan.frameCount - 1)}
        value={frame}
        onChange={(event) => onSeek(Number(event.target.value))}
      />
      <span className="sr-only">Timeline for document {docId}</span>
    </section>
  );
}
