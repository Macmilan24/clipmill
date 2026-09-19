/**
 * The editor: a player that shows what the export will look like, and the
 * timeline that says what it is made of.
 *
 * The player **applies** a plan and computes nothing. The crop at a frame, the
 * caption on screen, which word carries the highlight — all of it was decided
 * by the code that renders and arrives ready. That is the workstream's binding
 * rule, and it is the reason this file has no arithmetic in it beyond turning a
 * media element's seconds into a frame index.
 *
 * Four lanes, because those are the four things an edit is made of: the
 * pictures, where the camera points, what is said, and how loud. Each one draws
 * from the same plan, so a playhead is in the same place on all four by
 * construction rather than by four pieces of code agreeing.
 */
import {
  ArrowLeft,
  Check,
  ChevronLeft,
  ChevronRight,
  Pause,
  Play,
  Redo2,
  Undo2,
  Upload,
} from 'lucide-react';
import type { ReactNode } from 'react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { Button } from '../components/ui/button.js';
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from '../components/ui/empty.js';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../components/ui/tabs.js';
import { Audio } from '../editor/Audio.js';
import { CompositionCanvas } from '../editor/CompositionCanvas.js';
import { useDraftAudio } from '../editor/useDraftAudio.js';
import { Captions } from '../editor/Captions.js';
import { Reframe } from '../editor/Reframe.js';
import { snapToWord, trimEndAt, trimStartAt } from '../editor/commands.js';
import type { EditCommandJson } from '../daemon/client.js';
import type { PreviewPlan } from '../daemon/client.js';
import {
  cropAt,
  cueAt,
  cueLines,
  gainAt,
  highlightedWord,
  lanePosition,
  proxySecondsAt,
  resolvePlaybackFrame,
  segmentAt,
  timecode,
} from '../editor/player.js';

export interface EditorProps {
  /** Null until a clip has been approved and a document exists. */
  readonly plan: PreviewPlan | null;
  /**
   * The proxy for each source the plan names, by fingerprint, as a URL the
   * media protocol serves. A source with no entry has nothing to play.
   */
  readonly proxyUrls: ReadonlyMap<string, string>;
  readonly docId: string | null;
  /** What the clip is called — the project and the clip — when the route knew. */
  readonly labels: { readonly project?: string; readonly clip?: string } | null;
  readonly loading: boolean;
  readonly problem: string | null;
  readonly busy: boolean;
  readonly canUndo: boolean;
  readonly canRedo: boolean;
  readonly resolving: boolean;
  /** Why the solver cannot be asked, or `null` when it can. */
  readonly resolveRefusal: string | null;
  /**
   * What to show instead of a clip when none is open: the list of edits there
   * are. Null when a clip is named, so nothing is fetched for a list that
   * would not be shown.
   */
  readonly picker: ReactNode;
  readonly onOpenResults: () => void;
  /** Take this clip to the export screen. Null when no clip is open. */
  readonly onExport: (() => void) | null;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onUndo: () => void;
  readonly onRedo: () => void;
  readonly onResolve: (frame: number) => void;
}

export function Editor({
  plan,
  proxyUrls,
  docId,
  labels,
  loading,
  problem,
  busy,
  canUndo,
  canRedo,
  resolving,
  resolveRefusal,
  picker,
  onOpenResults,
  onExport,
  onApply,
  onUndo,
  onRedo,
  onResolve,
}: EditorProps) {
  const video = useRef<HTMLVideoElement>(null);
  const [playhead, setFrame] = useState(0);
  const clockFrame = useRef(0);
  const resumeAfterLoad = useRef(false);
  const updateFrame = useCallback((next: number) => {
    clockFrame.current = next;
    setFrame(next);
  }, []);
  const [playing, setPlaying] = useState(false);
  const [playbackProblem, setPlaybackProblem] = useState<string | null>(null);
  // What is drawn is always a frame the program has. A trim can shorten the
  // program under a playhead that did not move, and between that render and
  // the effect below that moves it back, a frame past the end would find no
  // segment — and no segment would take the picture down with it.
  const frame = plan ? Math.max(0, Math.min(playhead, plan.frameCount - 1)) : playhead;
  const draftAudio = useDraftAudio(video, plan ? gainAt(plan, frame) : 0);

  // A different document is a different program; a playhead left where the
  // last one was would seek the new proxy to a frame it may not have.
  useEffect(() => {
    updateFrame(0);
    resumeAfterLoad.current = false;
    setPlaying(false);
    setPlaybackProblem(null);
  }, [docId, updateFrame]);

  /**
   * Put the playhead on a program frame, and the media element where that
   * frame is in the proxy. Every seek goes through here — the transport, the
   * scrubber, the arrow keys — so there is one place the two clocks meet.
   */
  const seek = useCallback(
    (target: number) => {
      if (!plan) {
        return;
      }
      const next = Math.max(0, Math.min(plan.frameCount - 1, target));
      updateFrame(next);
      const element = video.current;
      const seconds = proxySecondsAt(plan, next);
      if (element && seconds !== null) {
        const source = segmentAt(plan, next);
        const url = source ? proxyUrls.get(source.sourceFingerprint) : null;
        if (url !== element.getAttribute('src')) {
          // React will change src; seeking the previous recording would show
          // unrelated footage. Resume only after the new proxy is positioned.
          resumeAfterLoad.current = !element.paused;
          return;
        }
        // Set whether or not the media has loaded: before metadata a browser
        // keeps it as the position to start from, which is what is wanted.
        element.currentTime = seconds;
      }
    },
    [plan, proxyUrls, updateFrame],
  );

  // A new plan is a new mapping. A trim moved the segment's window, so the
  // frame the playhead is on now shows different footage and may not exist
  // at all; the media element is told where that frame is now, and a playhead
  // past the new end is brought back onto the program. Keyed on the plan
  // rather than on the document because a trim changes neither the document
  // id nor the proxy.
  useEffect(() => {
    if (!plan) {
      return;
    }
    seek(Math.min(clockFrame.current, plan.frameCount - 1));
    // eslint-disable-next-line react-hooks/exhaustive-deps -- the plan is the signal
  }, [plan, docId]);

  const step = useCallback((by: number) => seek(frame + by), [frame, seek]);

  const togglePlayback = useCallback(() => {
    const element = video.current;
    if (!element || !plan) return;
    setPlaybackProblem(null);
    if (!element.paused) {
      element.pause();
    } else {
      if (frame >= plan.frameCount - 1 || element.ended) seek(0);
      draftAudio.connect();
      void element.play().catch(() => {
        setPlaying(false);
        setPlaybackProblem('Playback could not start. Try playing the clip again.');
      });
    }
  }, [frame, plan, seek, draftAudio.connect]);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const target = event.target;
      // Text, range inputs and Radix controls own their keyboard interactions.
      if (
        target instanceof Element &&
        target.closest(
          'input, textarea, select, button, [role="tab"], [role="slider"], [contenteditable="true"]',
        )
      )
        return;
      if (event.defaultPrevented || event.altKey || !plan) return;
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'z') {
        event.preventDefault();
        if (!busy && (event.shiftKey ? canRedo : canUndo)) (event.shiftKey ? onRedo : onUndo)();
      } else if (
        !event.metaKey &&
        !event.ctrlKey &&
        ['ArrowLeft', 'ArrowRight', ' '].includes(event.key)
      ) {
        event.preventDefault();
        if (event.key === ' ') togglePlayback();
        else step((event.key === 'ArrowLeft' ? -1 : 1) * (event.shiftKey ? 10 : 1));
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [step, plan, togglePlayback, busy, canRedo, canUndo, onRedo, onUndo]);

  const cue = useMemo(() => (plan ? cueAt(plan, frame) : null), [plan, frame]);
  const highlighted = useMemo(
    () => (plan && cue ? highlightedWord(plan, cue, frame) : -1),
    [plan, cue, frame],
  );
  const crop = useMemo(() => (plan ? cropAt(plan, frame) : null), [plan, frame]);
  const secondaryCrop = useMemo(() => (plan ? cropAt(plan, frame, true) : null), [plan, frame]);
  const segment = useMemo(() => (plan ? segmentAt(plan, frame) : null), [plan, frame]);

  // This callback returns the frame synchronously: the canvas must use its
  // crop for these decoded pixels, without waiting for React to move the UI.
  const onProxyTime = useCallback(
    (seconds: number, atMediaEnd = false): number | null => {
      const element = video.current;
      if (!plan || !element || element.seeking) return null;
      const active = segmentAt(plan, clockFrame.current);
      if (!active || proxyUrls.get(active.sourceFingerprint) !== element.getAttribute('src'))
        return null;
      // A paused seek requests a program frame even when the lower-rate proxy
      // decodes an earlier frame. Do not move the scrubber back to that frame.
      if (element.paused && !atMediaEnd) return clockFrame.current;
      const next = resolvePlaybackFrame(plan, clockFrame.current, seconds);
      if (next.seek || next.ended) {
        if (next.ended) {
          element.pause();
          setPlaying(false);
        }
        seek(next.frame);
        if (atMediaEnd && next.seek) {
          // Native EOF is paused already, but an edit can continue in another
          // recording (or repeat this one). Preserve that playback intent.
          const following = segmentAt(plan, next.frame);
          if (
            following &&
            proxyUrls.get(following.sourceFingerprint) !== element.getAttribute('src')
          ) {
            resumeAfterLoad.current = true;
          } else {
            void element.play().catch(() => {
              setPlaying(false);
              setPlaybackProblem('Playback could not continue. Try playing the clip again.');
            });
          }
        }
        return null;
      }
      if (atMediaEnd) {
        setPlaying(false);
        setPlaybackProblem(
          'The preview ended before the clip. Reopen the clip to reload its proxy.',
        );
        return null;
      }
      updateFrame(next.frame);
      return next.frame;
    },
    [plan, proxyUrls, seek, updateFrame],
  );

  const onMetadata = useCallback(() => {
    const element = video.current;
    if (!element) return;
    seek(clockFrame.current);
    if (resumeAfterLoad.current) {
      resumeAfterLoad.current = false;
      void element.play().catch(() => {
        setPlaying(false);
        setPlaybackProblem('Playback could not continue. Try playing the clip again.');
      });
    }
  }, [seek]);

  if (loading) {
    return <div className="p-8 text-sm text-[var(--cm-ink-2)]">Fetching the preview plan…</div>;
  }

  if (!plan || !docId) {
    return (
      <div className="p-8">
        <Empty>
          <EmptyHeader>
            <EmptyTitle>
              {problem && labels
                ? 'This clip could not be opened'
                : 'No clip is open in the editor'}
            </EmptyTitle>
            <EmptyDescription>
              {problem ??
                'Approving a clip in the Inspector creates its edit document and opens it here. Any edit already made can be reopened below.'}
            </EmptyDescription>
          </EmptyHeader>
          {picker}
          <Button variant="outline" onClick={onOpenResults}>
            Go to Results
          </Button>
        </Empty>
      </div>
    );
  }

  const proxyUrl = segment ? (proxyUrls.get(segment.sourceFingerprint) ?? null) : null;

  return (
    <div className="editor-workspace">
      <header className="workspace-heading editor-heading">
        <div className="flex min-w-0 items-center gap-4">
          <Button
            variant="ghost"
            size="icon-sm"
            onClick={onOpenResults}
            aria-label="Back to results"
            title="Back to results"
          >
            <ArrowLeft />
          </Button>
          <div className="min-w-0">
            <h1 className="truncate text-[15px] font-semibold" data-testid="clip-name">
              {labels?.clip ?? 'Clip editor'}
            </h1>
            <p className="workspace-subtitle truncate">
              {labels?.project ?? 'Your edit'} <span aria-hidden>·</span>{' '}
              {timecode(plan, plan.frameCount)}
            </p>
          </div>
        </div>
        <div className="flex shrink-0 items-center gap-3">
          <span
            role="status"
            className="flex items-center gap-1.5 text-[11px] text-[var(--cm-text-secondary)]"
          >
            {!busy && <Check className="size-3.5" />}{' '}
            {busy ? 'Saving…' : `Saved · r${plan.revision}`}
          </span>
          <div className="flex border-x border-[var(--cm-glass-border)] px-2">
            <Button
              size="icon-sm"
              variant="ghost"
              disabled={!canUndo || busy}
              onClick={onUndo}
              aria-label="Undo"
              title="Undo (⌘/Ctrl Z)"
            >
              <Undo2 />
            </Button>
            <Button
              size="icon-sm"
              variant="ghost"
              disabled={!canRedo || busy}
              onClick={onRedo}
              aria-label="Redo"
              title="Redo (⌘/Ctrl Shift Z)"
            >
              <Redo2 />
            </Button>
          </div>
          {onExport && (
            <Button size="sm" onClick={onExport} disabled={busy} aria-label="Export this clip">
              <Upload className="size-4" />
              Export clip
            </Button>
          )}
        </div>
      </header>
      {(problem || playbackProblem || draftAudio.problem || segment?.framingWarning) && (
        <p
          role="alert"
          className="border-b border-[var(--cm-glass-border)] bg-[var(--cm-glass)] px-6 py-2 text-xs text-[var(--cm-danger-ink)]"
        >
          {problem || playbackProblem || draftAudio.problem || segment?.framingWarning}
        </p>
      )}
      <div className="editor-body">
        <section className="editor-viewer" aria-label="Clip preview">
          <div className="flex items-center justify-between text-[11px] text-[var(--cm-text-muted)]">
            <span>Draft preview · r{plan.revision}</span>
            <span className="mono">
              {plan.width} × {plan.height} · {(plan.rateNum / plan.rateDen).toFixed(2)} fps
            </span>
          </div>
          <div className="editor-stage-wrap">
            <Stage
              key={docId}
              proxyUrl={proxyUrl}
              videoRef={video}
              frame={frame}
              plan={plan}
              lines={cue ? cueLines(cue) : []}
              cue={cue}
              highlighted={highlighted}
              startSeconds={proxySecondsAt(plan, frame)}
              onProxyTime={onProxyTime}
              onMetadata={onMetadata}
              onPlaying={setPlaying}
              onError={() => {
                setPlaying(false);
                setPlaybackProblem(
                  'The preview could not be loaded. Reopen the clip to try again.',
                );
              }}
            />
          </div>
          <p className="text-center text-[11px] leading-relaxed text-[var(--cm-text-muted)]">
            Fast proxy preview with your gain edits.{' '}
            {onExport ? (
              <button
                type="button"
                onClick={onExport}
                className="text-[var(--cm-accent)] underline underline-offset-2"
              >
                Render r{plan.revision} for final picture and mastered audio.
              </button>
            ) : (
              'Review the rendered file for final picture and mastered audio.'
            )}
          </p>
          <Transport
            plan={plan}
            frame={frame}
            playing={playing}
            disabled={!proxyUrl}
            onStep={step}
            onSeek={seek}
            onToggle={togglePlayback}
          />
        </section>
        <aside className="editor-properties" aria-label="Edit controls">
          <Tabs defaultValue="captions" className="flex min-h-0 flex-1 flex-col">
            <TabsList className="m-3 w-[calc(100%-1.5rem)] shrink-0">
              <TabsTrigger value="reframe">Reframe</TabsTrigger>
              <TabsTrigger value="captions">Captions</TabsTrigger>
              <TabsTrigger value="audio">Audio</TabsTrigger>
              <TabsTrigger value="clip">Clip</TabsTrigger>
            </TabsList>
            <TabsContent value="reframe">
              <Reframe
                plan={plan}
                frame={frame}
                busy={busy}
                onApply={onApply}
                onResolve={() => onResolve(frame)}
                resolving={resolving}
                resolveRefusal={resolveRefusal}
              />
            </TabsContent>
            <TabsContent value="captions">
              <Captions plan={plan} frame={frame} busy={busy} onApply={onApply} />
            </TabsContent>
            <TabsContent value="audio">
              <Audio plan={plan} frame={frame} busy={busy} onApply={onApply} />
            </TabsContent>
            <TabsContent value="clip">
              <div className="flex flex-col gap-3 p-4 text-sm">
                <dl className="space-y-2 text-xs">
                  <Row label="Output" value={`${plan.width}×${plan.height}`} />
                  <Row label="Rate" value={`${(plan.rateNum / plan.rateDen).toFixed(3)} fps`} />
                  <Row label="Frames" value={String(plan.frameCount)} />
                  <Row
                    label="Layout"
                    value={secondaryCrop ? 'Two portraits' : crop ? 'Face crop' : 'Fit'}
                  />
                  <Row label="Gain here" value={`${gainAt(plan, frame).toFixed(1)} dB`} />
                  {segment && (
                    <Row
                      label="Source window"
                      value={`${sourceClock(segment.inTicks)} – ${sourceClock(segment.outTicks)}`}
                    />
                  )}
                </dl>
                <p className="text-xs text-[var(--cm-ink-2)]">
                  Trimming snaps to a caption boundary rather than to the pointer: a cut inside a
                  word is a cut a viewer hears.
                </p>
                <div className="flex gap-2">
                  <Button
                    size="sm"
                    variant="outline"
                    disabled={busy || trimStartAt(plan, snapToWord(plan, frame)) === null}
                    onClick={() => {
                      const command = trimStartAt(plan, snapToWord(plan, frame));
                      if (command) {
                        onApply(command);
                      }
                    }}
                  >
                    Trim start here
                  </Button>
                  <Button
                    size="sm"
                    variant="outline"
                    disabled={busy || trimEndAt(plan, snapToWord(plan, frame)) === null}
                    onClick={() => {
                      const command = trimEndAt(plan, snapToWord(plan, frame));
                      if (command) {
                        onApply(command);
                      }
                    }}
                  >
                    Trim end here
                  </Button>
                </div>
              </div>
            </TabsContent>
          </Tabs>
        </aside>
      </div>

      <div className="editor-timeline">
        <div className="mb-3 flex items-center justify-between">
          <h2 className="text-[12px] font-medium">Timeline</h2>
          <span className="text-[10px] text-[var(--cm-text-muted)]">
            Space to play · ← → step · Shift to step 10 frames
          </span>
        </div>
        <Lanes plan={plan} frame={frame} onSeek={seek} />
      </div>
    </div>
  );
}

function Row({ label, value }: { readonly label: string; readonly value: string }) {
  return (
    <div className="flex justify-between">
      <dt className="text-[var(--cm-ink-2)]">{label}</dt>
      <dd className="text-[var(--cm-ink-1)]">{value}</dd>
    </div>
  );
}

/** A source tick as `m:ss`, which is how a person reads where a clip sits. */
function sourceClock(ticks: number): string {
  const seconds = Math.floor(ticks / 90_000);
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`;
}

/** The 9:16 stage: the proxy, cropped by the plan, with the plan's captions. */
function Stage({
  proxyUrl,
  videoRef,
  frame,
  plan,
  lines,
  cue,
  highlighted,
  startSeconds,
  onProxyTime,
  onMetadata,
  onPlaying,
  onError,
}: {
  readonly proxyUrl: string | null;
  readonly videoRef: React.RefObject<HTMLVideoElement | null>;
  readonly frame: number;
  readonly plan: PreviewPlan;
  readonly lines: readonly string[];
  readonly cue: ReturnType<typeof cueAt>;
  readonly highlighted: number;
  /** Where in the proxy the current frame is, to land on when the media loads. */
  readonly startSeconds: number | null;
  readonly onProxyTime: (seconds: number, atMediaEnd?: boolean) => number | null;
  readonly onMetadata: () => void;
  readonly onPlaying: (playing: boolean) => void;
  readonly onError: () => void;
}) {
  const style = plan.captionStyle;
  const relative = (pixels: number) => `${(pixels / plan.width) * 100}cqw`;
  const verticalMargin = ((style?.marginVertical ?? 260) / plan.height) * 100;
  const position =
    cue?.region === 'upper_safe'
      ? { top: `${verticalMargin}%` }
      : cue?.region === 'center'
        ? { top: '50%', transform: 'translateY(-50%)' }
        : { bottom: `${verticalMargin}%` };
  let index = 0;
  return (
    <div className="video-stage" data-testid="stage" style={{ containerType: 'inline-size' }}>
      {proxyUrl ? (
        <div className="absolute inset-0 flex items-center justify-center">
          {/* eslint-disable-next-line jsx-a11y/media-has-caption -- the cues are
              drawn below from the plan rather than as a text track. */}
          <video
            ref={videoRef}
            src={proxyUrl}
            className="pointer-events-none absolute h-full w-full opacity-0"
            crossOrigin="anonymous"
            playsInline
            data-testid="proxy"
            data-start-seconds={startSeconds ?? undefined}
            onLoadedMetadata={onMetadata}
            onPlay={() => onPlaying(true)}
            onPause={() => onPlaying(false)}
            onEnded={(event) => {
              onPlaying(false);
              // The last decoded PTS can precede the last program frame.
              // EOF itself must advance/finish the edit, not restart the proxy.
              onProxyTime(event.currentTarget.currentTime, true);
            }}
            onError={onError}
          />
          <CompositionCanvas
            video={videoRef}
            mediaKey={proxyUrl}
            plan={plan}
            frame={frame}
            onFrame={onProxyTime}
          />
        </div>
      ) : (
        <p className="grid h-full place-items-center p-6 text-center text-sm text-[var(--cm-ink-2)]">
          This recording has no proxy, so there is nothing to play.
        </p>
      )}
      {proxyUrl && lines.length > 0 && (
        <p
          className="pointer-events-none absolute text-center leading-tight"
          style={{
            ...position,
            left: `${((style?.marginHorizontal ?? 90) / plan.width) * 100}%`,
            right: `${((style?.marginHorizontal ?? 90) / plan.width) * 100}%`,
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
          {cue?.lines.map((line, lineIndex) => (
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
                        color:
                          !cue.karaoke || mine <= highlighted
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
    </div>
  );
}

function Transport({
  plan,
  frame,
  playing,
  disabled,
  onStep,
  onSeek,
  onToggle,
}: {
  readonly plan: PreviewPlan;
  readonly frame: number;
  readonly playing: boolean;
  readonly disabled: boolean;
  readonly onStep: (by: number) => void;
  readonly onSeek: (frame: number) => void;
  readonly onToggle: () => void;
}) {
  return (
    <div className="flex flex-col gap-2" aria-label="Transport">
      <input
        type="range"
        aria-label="Scrub"
        min={0}
        max={Math.max(0, plan.frameCount - 1)}
        step={1}
        value={frame}
        onChange={(event) => onSeek(Number(event.target.value))}
        className="studio-range"
      />
      <div className="flex items-center justify-center gap-3">
        <Button
          variant="ghost"
          size="sm"
          onClick={() => onStep(-1)}
          aria-label="Previous frame"
          disabled={disabled || frame === 0}
        >
          <ChevronLeft className="size-4" />
        </Button>
        <Button
          size="icon-sm"
          disabled={disabled}
          onClick={onToggle}
          aria-label={playing ? 'Pause' : 'Play'}
        >
          {playing ? <Pause className="size-4" /> : <Play className="size-4" />}
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => onStep(1)}
          aria-label="Next frame"
          disabled={disabled || frame >= plan.frameCount - 1}
        >
          <ChevronRight className="size-4" />
        </Button>
        <span className="ml-3 font-mono text-xs text-[var(--cm-ink-2)]" data-testid="timecode">
          {timecode(plan, frame)}{' '}
          <span className="text-[var(--cm-text-muted)]">/ {timecode(plan, plan.frameCount)}</span>
          <span className="sr-only">
            {' '}
            · frame {frame} of {plan.frameCount}
          </span>
        </span>
      </div>
    </div>
  );
}

/** Each track uses the same width, so the playhead aligns at every window size. */
function Lanes({
  plan,
  frame,
  onSeek,
}: {
  readonly plan: PreviewPlan;
  readonly frame: number;
  readonly onSeek: (frame: number) => void;
}) {
  const cropRuns = useMemo(() => {
    const runs: { first: number; end: number; mode: 'Fit' | 'Face crop' | 'Two portraits' }[] = [];
    for (let at = 0; at < plan.crops.length; at++) {
      const mode =
        plan.secondaryCrops?.[at] != null
          ? 'Two portraits'
          : plan.crops[at] != null
            ? 'Face crop'
            : 'Fit';
      const previous = runs.at(-1);
      if (previous && previous.mode === mode) previous.end = at + 1;
      else runs.push({ first: at, end: at + 1, mode });
    }
    return runs;
  }, [plan.crops, plan.secondaryCrops]);
  const playhead = lanePosition(plan, frame);
  const span = (first: number, end: number) => ({
    left: `${lanePosition(plan, first)}%`,
    width: `${Math.max(0.1, lanePosition(plan, end) - lanePosition(plan, first))}%`,
  });
  const lanes = [
    {
      id: 'video',
      label: 'Video',
      body: (
        <span className="absolute inset-0 flex items-center rounded-sm bg-[var(--cm-accent-selected)] px-2 text-[10px] text-[var(--cm-text-secondary)]">
          Source footage
        </span>
      ),
    },
    {
      id: 'reframe',
      label: 'Framing',
      body: cropRuns.map((run) => (
        <span
          key={run.first}
          className="absolute inset-y-0 flex items-center overflow-hidden rounded-sm border border-[var(--cm-glass-border)] px-2 text-[10px] text-[var(--cm-text-secondary)]"
          style={span(run.first, run.end)}
        >
          {run.mode}
        </span>
      )),
    },
    {
      id: 'captions',
      label: 'Captions',
      body: plan.cues.map((cue) => (
        <span
          key={cue.cueId}
          title={cueLines(cue).join(' ')}
          className="absolute inset-y-0 truncate rounded-sm border-r border-[var(--cm-glass)] bg-[var(--cm-accent-selected)] px-1.5 py-1 text-[10px] text-[var(--cm-text-secondary)]"
          style={span(cue.firstFrame, cue.endFrame)}
        >
          {cueLines(cue).join(' ')}
        </span>
      )),
    },
    {
      id: 'audio',
      label: 'Audio',
      body: (
        <>
          <span className="absolute inset-x-0 top-1/2 h-px bg-[var(--cm-text-muted)]/40" />
          {plan.gain.map((point) => (
            <span
              key={point.frame}
              title={`${point.gainDb.toFixed(1)} dB`}
              className="absolute top-1/2 size-1.5 -translate-y-1/2 rounded-full bg-[var(--cm-text-muted)]"
              style={{ left: `${lanePosition(plan, point.frame)}%` }}
            />
          ))}
        </>
      ),
    },
  ];
  return (
    <section className="timeline-grid" aria-label="Timeline">
      <span />
      <div className="flex justify-between font-mono text-[10px] text-[var(--cm-text-muted)]">
        {[0, 0.25, 0.5, 0.75, 1].map((ratio) => (
          <span key={ratio}>{timecode(plan, Math.floor(plan.frameCount * ratio))}</span>
        ))}
      </div>
      {lanes.map((lane) => (
        <Track
          key={lane.id}
          label={lane.label}
          playhead={playhead}
          frame={frame}
          frameCount={plan.frameCount}
          onSeek={onSeek}
        >
          {lane.body}
        </Track>
      ))}
    </section>
  );
}

function Track({
  label,
  playhead,
  frame,
  frameCount,
  onSeek,
  children,
}: {
  readonly label: string;
  readonly playhead: number;
  readonly frame: number;
  readonly frameCount: number;
  readonly onSeek: (frame: number) => void;
  readonly children: ReactNode;
}) {
  return (
    <>
      <span className="timeline-label">{label}</span>
      <button
        type="button"
        className="timeline-track text-left"
        aria-label={`Seek in ${label.toLowerCase()} track`}
        onClick={(event) => {
          if (!event.detail) return;
          const rect = event.currentTarget.getBoundingClientRect();
          if (rect.width > 0)
            onSeek(Math.round(((event.clientX - rect.left) / rect.width) * (frameCount - 1)));
        }}
        onKeyDown={(event) => {
          if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
            event.preventDefault();
            onSeek(frame + (event.key === 'ArrowLeft' ? -1 : 1));
          }
        }}
      >
        {children}
        <span
          className="timeline-needle"
          style={{ left: `${playhead}%` }}
          data-testid={label === 'Video' ? 'playhead' : undefined}
        />
      </button>
    </>
  );
}
