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
import { ChevronLeft, ChevronRight, Pause, Play, Redo2, Undo2, Upload } from 'lucide-react';
import type { JSX, ReactNode } from 'react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { Badge } from '../components/ui/badge.js';
import { Button } from '../components/ui/button.js';
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from '../components/ui/empty.js';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../components/ui/tabs.js';
import { Audio } from '../editor/Audio.js';
import { Captions } from '../editor/Captions.js';
import { Reframe } from '../editor/Reframe.js';
import { snapToWord, trimEndAt, trimStartAt } from '../editor/commands.js';
import type { EditCommandJson } from '../daemon/client.js';
import type { PreviewPlan } from '../daemon/client.js';
import {
  cropAt,
  cueAt,
  cueLines,
  frameAtProxySeconds,
  gainAt,
  highlightedWord,
  lanePosition,
  proxySecondsAt,
  segmentAt,
  sourceOf,
  stageTransform,
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
  readonly onResolve: () => void;
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
  const [playing, setPlaying] = useState(false);
  // What is drawn is always a frame the program has. A trim can shorten the
  // program under a playhead that did not move, and between that render and
  // the effect below that moves it back, a frame past the end would find no
  // segment — and no segment would take the picture down with it.
  const frame = plan ? Math.max(0, Math.min(playhead, plan.frameCount - 1)) : playhead;

  // A different document is a different program; a playhead left where the
  // last one was would seek the new proxy to a frame it may not have.
  useEffect(() => {
    setFrame(0);
    setPlaying(false);
  }, [docId]);

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
      setFrame(next);
      const element = video.current;
      const seconds = proxySecondsAt(plan, next);
      if (element && seconds !== null) {
        // Set whether or not the media has loaded: before metadata a browser
        // keeps it as the position to start from, which is what is wanted.
        element.currentTime = seconds;
      }
    },
    [plan],
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
    seek(Math.min(playhead, plan.frameCount - 1));
    // eslint-disable-next-line react-hooks/exhaustive-deps -- the plan is the signal
  }, [plan]);

  const step = useCallback((by: number) => seek(frame + by), [frame, seek]);

  // Arrow keys step a frame at a time, which is the transport an editor
  // reaches for when a cut is one frame wrong.
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === 'ArrowLeft') {
        step(-1);
      } else if (event.key === 'ArrowRight') {
        step(1);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [step]);

  const cue = useMemo(() => (plan ? cueAt(plan, frame) : null), [plan, frame]);
  const highlighted = useMemo(
    () => (plan && cue ? highlightedWord(plan, cue, frame) : -1),
    [plan, cue, frame],
  );
  const crop = useMemo(() => (plan ? cropAt(plan, frame) : null), [plan, frame]);
  const segment = useMemo(() => (plan ? segmentAt(plan, frame) : null), [plan, frame]);

  /**
   * What the media element reports, read back as a program frame.
   *
   * The element plays the proxy, which runs on past the segment's end into
   * footage the clip does not include. When it gets there the next segment
   * begins — seeked to its own place in the proxy — or, on the last one, the
   * program is over and playback stops on its final frame rather than into the
   * rest of the recording.
   */
  const onProxyTime = useCallback(
    (seconds: number) => {
      if (!plan || !segment) {
        return;
      }
      const element = video.current;
      const proxyEnd = proxySecondsAt(plan, segment.endFrame - 1);
      const ended = proxyEnd !== null && seconds > proxyEnd + secondsPerFrame(plan);
      if (!ended) {
        setFrame(frameAtProxySeconds(plan, segment, seconds));
        return;
      }
      const following = plan.segments[plan.segments.indexOf(segment) + 1];
      if (following) {
        seek(following.firstFrame);
        return;
      }
      element?.pause();
      setPlaying(false);
      seek(plan.frameCount - 1);
    },
    [plan, segment, seek],
  );

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

  const source = segment ? sourceOf(plan, segment) : null;
  const proxyUrl = segment ? (proxyUrls.get(segment.sourceFingerprint) ?? null) : null;

  return (
    <div className="flex h-full min-h-0 flex-col gap-4 p-6">
      <div className="flex min-h-0 flex-1 items-start justify-center gap-6">
        <Stage
          proxyUrl={proxyUrl}
          videoRef={video}
          crop={crop}
          source={source}
          lines={cue ? cueLines(cue) : []}
          cue={cue}
          highlighted={highlighted}
          startSeconds={proxySecondsAt(plan, frame)}
          onProxyTime={onProxyTime}
          onEnded={() => setPlaying(false)}
        />
        <aside className="flex w-[340px] shrink-0 flex-col rounded-xl border border-[var(--cm-line-1)] bg-[var(--cm-surface-1)]">
          <div className="flex items-center justify-between border-b border-[var(--cm-line-1)] p-3">
            <span className="flex min-w-0 flex-col gap-0.5">
              {labels && (
                <span className="truncate text-xs text-[var(--cm-ink-1)]" data-testid="clip-name">
                  {[labels.project, labels.clip].filter(Boolean).join(' · ')}
                </span>
              )}
              <span className="flex items-center gap-2">
                <Badge variant="outline">r{plan.revision}</Badge>
                <span className="truncate font-mono text-[10px] text-[var(--cm-ink-3)]">
                  {docId}
                </span>
              </span>
            </span>
            <span className="flex gap-1">
              {onExport && (
                <Button size="sm" variant="ghost" onClick={onExport} aria-label="Export this clip">
                  <Upload className="size-4" />
                </Button>
              )}
              <Button
                size="sm"
                variant="ghost"
                disabled={!canUndo || busy}
                onClick={onUndo}
                aria-label="Undo"
              >
                <Undo2 className="size-4" />
              </Button>
              <Button
                size="sm"
                variant="ghost"
                disabled={!canRedo || busy}
                onClick={onRedo}
                aria-label="Redo"
              >
                <Redo2 className="size-4" />
              </Button>
            </span>
          </div>
          <Tabs defaultValue="reframe" className="min-h-0 flex-1">
            <TabsList className="m-3 w-[calc(100%-1.5rem)]">
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
                onResolve={onResolve}
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
                  <Row label="Layout" value={crop ? 'Speaker-follow' : 'Fit'} />
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
                {problem && <p className="text-xs text-[var(--cm-danger-ink)]">{problem}</p>}
              </div>
            </TabsContent>
          </Tabs>
        </aside>
      </div>

      <Transport
        plan={plan}
        frame={frame}
        playing={playing}
        onStep={step}
        onSeek={seek}
        onToggle={() => {
          const element = video.current;
          if (!element) {
            return;
          }
          if (playing) {
            element.pause();
          } else {
            // Playing from the program's end starts it over rather than
            // running into the recording past the clip.
            if (frame >= plan.frameCount - 1) {
              seek(0);
            }
            void element.play();
          }
          setPlaying(!playing);
        }}
      />

      <Lanes plan={plan} frame={frame} />
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

/** Seconds one frame lasts, the slack allowed before a segment counts as over. */
function secondsPerFrame(plan: PreviewPlan): number {
  return plan.rateNum > 0 ? plan.rateDen / plan.rateNum : 0;
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
  crop,
  source,
  lines,
  cue,
  highlighted,
  startSeconds,
  onProxyTime,
  onEnded,
}: {
  readonly proxyUrl: string | null;
  readonly videoRef: React.RefObject<HTMLVideoElement | null>;
  readonly crop: ReturnType<typeof cropAt>;
  /** The frame the crop is measured in. Null when the plan does not know it. */
  readonly source: { readonly displayWidth: number; readonly displayHeight: number } | null;
  readonly lines: readonly string[];
  readonly cue: ReturnType<typeof cueAt>;
  readonly highlighted: number;
  /** Where in the proxy the current frame is, to land on when the media loads. */
  readonly startSeconds: number | null;
  readonly onProxyTime: (seconds: number) => void;
  readonly onEnded: () => void;
}) {
  // The crop is expressed against the source frame, and the element on stage
  // is the source scaled to the stage's height — so the transform is built
  // against the source's dimensions, from the plan, not against the output's,
  // and it is applied to the video element itself: a percentage translate is
  // a share of the transformed element's own box, and the stage-sized wrapper
  // this used to sit on is narrower than a landscape source, so off-centre
  // crops moved too little.
  const transform = crop && source ? stageTransform(crop, source) : undefined;

  let index = 0;
  return (
    <div
      className="relative aspect-[9/16] h-full max-h-[520px] overflow-hidden rounded-xl bg-black"
      data-testid="stage"
    >
      {proxyUrl ? (
        <div className="absolute inset-0 flex items-center justify-center">
          {/* eslint-disable-next-line jsx-a11y/media-has-caption -- the cues are
              drawn below from the plan rather than as a text track. */}
          <video
            ref={videoRef}
            src={proxyUrl}
            className={crop ? 'h-full w-auto max-w-none' : 'max-h-full max-w-full'}
            style={{ transform }}
            playsInline
            data-testid="proxy"
            data-start-seconds={startSeconds ?? undefined}
            onLoadedMetadata={(event) => {
              // A proxy opens at its own zero, which is the recording's
              // opening; the clip begins minutes later.
              if (startSeconds !== null) {
                event.currentTarget.currentTime = startSeconds;
              }
            }}
            onTimeUpdate={(event) => onProxyTime(event.currentTarget.currentTime)}
            onEnded={onEnded}
          />
        </div>
      ) : (
        <p className="grid h-full place-items-center p-6 text-center text-sm text-[var(--cm-ink-2)]">
          This recording has no proxy, so there is nothing to play.
        </p>
      )}
      {lines.length > 0 && (
        <p
          className="pointer-events-none absolute inset-x-4 bottom-16 text-center text-lg leading-tight font-semibold drop-shadow-[0_2px_6px_rgba(0,0,0,0.95)]"
          data-testid="caption"
        >
          {cue?.lines.map((line, lineIndex) => (
            // eslint-disable-next-line react/no-array-index-key -- lines have no
            // identity of their own; their position is what they are.
            <span key={lineIndex} className="block">
              {line.map((word) => {
                const mine = index;
                index += 1;
                return (
                  <span
                    key={`${word.text}-${mine}`}
                    className={mine <= highlighted ? 'text-[var(--cm-accent)]' : 'text-white'}
                  >
                    {word.text}{' '}
                  </span>
                );
              })}
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
  onStep,
  onSeek,
  onToggle,
}: {
  readonly plan: PreviewPlan;
  readonly frame: number;
  readonly playing: boolean;
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
        className="w-full accent-[var(--cm-accent)]"
      />
      <div className="flex items-center justify-center gap-3">
        <Button variant="ghost" size="sm" onClick={() => onStep(-1)} aria-label="Previous frame">
          <ChevronLeft className="size-4" />
        </Button>
        <Button size="sm" onClick={onToggle} aria-label={playing ? 'Pause' : 'Play'}>
          {playing ? <Pause className="size-4" /> : <Play className="size-4" />}
        </Button>
        <Button variant="ghost" size="sm" onClick={() => onStep(1)} aria-label="Next frame">
          <ChevronRight className="size-4" />
        </Button>
        <span className="ml-3 font-mono text-xs text-[var(--cm-ink-2)]" data-testid="timecode">
          {timecode(plan, frame)} · frame {frame} of {plan.frameCount}
        </span>
      </div>
    </div>
  );
}

/** The four lanes an edit is made of, all drawn from the one plan. */
function Lanes({ plan, frame }: { readonly plan: PreviewPlan; readonly frame: number }) {
  const playhead = lanePosition(plan, frame);
  const lanes: readonly {
    readonly id: string;
    readonly label: string;
    readonly body: JSX.Element;
  }[] = [
    {
      id: 'video',
      label: 'V1',
      body: <div className="h-full rounded bg-[var(--cm-surface-2)]" />,
    },
    {
      id: 'reframe',
      label: 'R1',
      body: (
        <div className="flex h-full items-stretch gap-px">
          {plan.crops.map((crop, at) => (
            <span
              // eslint-disable-next-line react/no-array-index-key -- a frame's
              // index is its identity.
              key={at}
              className={
                crop ? 'flex-1 bg-[var(--cm-accent)]/50' : 'flex-1 bg-[var(--cm-surface-2)]'
              }
            />
          ))}
        </div>
      ),
    },
    {
      id: 'captions',
      label: 'C1',
      body: (
        <div className="relative h-full rounded bg-[var(--cm-surface-2)]">
          {plan.cues.map((cue) => (
            <span
              key={cue.cueId}
              title={cueLines(cue).join(' ')}
              className="absolute inset-y-0 rounded bg-[var(--cm-accent)]/60"
              style={{
                left: `${lanePosition(plan, cue.firstFrame)}%`,
                width: `${Math.max(0.4, lanePosition(plan, cue.endFrame) - lanePosition(plan, cue.firstFrame))}%`,
              }}
            />
          ))}
        </div>
      ),
    },
    {
      id: 'audio',
      label: 'A1',
      body: (
        <div className="relative h-full rounded bg-[var(--cm-surface-2)]">
          {plan.gain.map((point) => (
            <span
              key={point.frame}
              title={`${point.gainDb.toFixed(1)} dB`}
              className="absolute inset-y-0 w-0.5 bg-[var(--cm-ink-3)]"
              style={{ left: `${lanePosition(plan, point.frame)}%` }}
            />
          ))}
          <span className="absolute inset-x-0 top-1/2 h-px bg-[var(--cm-ink-3)]/40" />
        </div>
      ),
    },
  ];

  return (
    <section className="relative flex flex-col gap-2" aria-label="Timeline">
      {lanes.map((lane) => (
        <div key={lane.id} className="flex items-center gap-3">
          <span className="w-6 shrink-0 font-mono text-[10px] text-[var(--cm-ink-3)]">
            {lane.label}
          </span>
          <div className="h-8 flex-1 overflow-hidden rounded">{lane.body}</div>
        </div>
      ))}
      <span
        className="pointer-events-none absolute inset-y-0 w-px bg-[var(--cm-accent)]"
        style={{ left: `calc(2.25rem + ${playhead}% * 0.94)` }}
        data-testid="playhead"
      />
    </section>
  );
}
