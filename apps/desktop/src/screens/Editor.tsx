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
import { ChevronLeft, ChevronRight, Pause, Play, Redo2, Undo2 } from 'lucide-react';
import type { JSX } from 'react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { Badge } from '../components/ui/badge.js';
import { Button } from '../components/ui/button.js';
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from '../components/ui/empty.js';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../components/ui/tabs.js';
import { Audio } from '../editor/Audio.js';
import { Captions } from '../editor/Captions.js';
import { Reframe } from '../editor/Reframe.js';
import { snapToWord, ticksAt, trim } from '../editor/commands.js';
import type { EditCommandJson } from '../daemon/client.js';
import type { PreviewPlan } from '../daemon/client.js';
import {
  cropAt,
  cueAt,
  cueLines,
  frameAt,
  gainAt,
  highlightedWord,
  lanePosition,
  secondsAt,
  timecode,
} from '../editor/player.js';

export interface EditorProps {
  /** Null until a clip has been approved and a document exists. */
  readonly plan: PreviewPlan | null;
  readonly proxyUrl: string | null;
  readonly docId: string | null;
  readonly loading: boolean;
  readonly problem: string | null;
  readonly busy: boolean;
  readonly canUndo: boolean;
  readonly canRedo: boolean;
  readonly resolving: boolean;
  readonly onOpenResults: () => void;
  readonly onApply: (command: EditCommandJson) => void;
  readonly onUndo: () => void;
  readonly onRedo: () => void;
  readonly onResolve: () => void;
}

export function Editor({
  plan,
  proxyUrl,
  docId,
  loading,
  problem,
  busy,
  canUndo,
  canRedo,
  resolving,
  onOpenResults,
  onApply,
  onUndo,
  onRedo,
  onResolve,
}: EditorProps) {
  const video = useRef<HTMLVideoElement>(null);
  const [frame, setFrame] = useState(0);
  const [playing, setPlaying] = useState(false);

  const step = useCallback(
    (by: number) => {
      const element = video.current;
      if (!element || !plan) {
        return;
      }
      const next = Math.max(0, Math.min(plan.frameCount - 1, frame + by));
      element.currentTime = secondsAt(plan, next);
      setFrame(next);
    },
    [frame, plan],
  );

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

  if (loading) {
    return <div className="p-8 text-sm text-[var(--cm-ink-2)]">Fetching the preview plan…</div>;
  }

  if (!plan || !docId) {
    return (
      <div className="p-8">
        <Empty>
          <EmptyHeader>
            <EmptyTitle>No clip is open in the editor</EmptyTitle>
            <EmptyDescription>
              {problem ??
                'Approving a clip in the Inspector creates its edit document; the editor opens the newest one.'}
            </EmptyDescription>
          </EmptyHeader>
          <Button variant="outline" onClick={onOpenResults}>
            Go to Results
          </Button>
        </Empty>
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col gap-4 p-6">
      <div className="flex min-h-0 flex-1 items-start justify-center gap-6">
        <Stage
          plan={plan}
          proxyUrl={proxyUrl}
          videoRef={video}
          crop={crop}
          lines={cue ? cueLines(cue) : []}
          cue={cue}
          highlighted={highlighted}
          onFrame={setFrame}
          onEnded={() => setPlaying(false)}
        />
        <aside className="flex w-[340px] shrink-0 flex-col rounded-xl border border-[var(--cm-line-1)] bg-[var(--cm-surface-1)]">
          <div className="flex items-center justify-between border-b border-[var(--cm-line-1)] p-3">
            <span className="flex items-center gap-2">
              <Badge variant="outline">r{plan.revision}</Badge>
              <span className="truncate font-mono text-[10px] text-[var(--cm-ink-3)]">{docId}</span>
            </span>
            <span className="flex gap-1">
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
                </dl>
                <p className="text-xs text-[var(--cm-ink-2)]">
                  Trimming snaps to a caption boundary rather than to the pointer: a cut inside a
                  word is a cut a viewer hears.
                </p>
                <div className="flex gap-2">
                  <Button
                    size="sm"
                    variant="outline"
                    disabled={busy}
                    onClick={() =>
                      onApply(
                        trim(
                          ticksAt(plan, snapToWord(plan, frame)),
                          ticksAt(plan, plan.frameCount),
                        ),
                      )
                    }
                  >
                    Trim start here
                  </Button>
                  <Button
                    size="sm"
                    variant="outline"
                    disabled={busy}
                    onClick={() => onApply(trim(0, ticksAt(plan, snapToWord(plan, frame))))}
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
        onToggle={() => {
          const element = video.current;
          if (!element) {
            return;
          }
          if (playing) {
            element.pause();
          } else {
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

/** The 9:16 stage: the proxy, cropped by the plan, with the plan's captions. */
function Stage({
  plan,
  proxyUrl,
  videoRef,
  crop,
  lines,
  cue,
  highlighted,
  onFrame,
  onEnded,
}: {
  readonly plan: PreviewPlan;
  readonly proxyUrl: string | null;
  readonly videoRef: React.RefObject<HTMLVideoElement | null>;
  readonly crop: ReturnType<typeof cropAt>;
  readonly lines: readonly string[];
  readonly cue: ReturnType<typeof cueAt>;
  readonly highlighted: number;
  readonly onFrame: (frame: number) => void;
  readonly onEnded: () => void;
}) {
  // The crop is expressed against the source frame, so the transform is a scale
  // by how much of the width it takes and a translate to its centre. Both come
  // out of the plan; neither is a guess about what the encoder will do.
  const transform = crop
    ? `scale(${plan.width / crop.width}) translate(${
        (0.5 - (crop.x + crop.width / 2) / plan.width) * 100
      }%, ${(0.5 - (crop.y + crop.height / 2) / plan.height) * 100}%)`
    : undefined;

  let index = 0;
  return (
    <div
      className="relative aspect-[9/16] h-full max-h-[520px] overflow-hidden rounded-xl bg-black"
      data-testid="stage"
    >
      {proxyUrl ? (
        <div className="absolute inset-0 flex items-center justify-center" style={{ transform }}>
          {/* eslint-disable-next-line jsx-a11y/media-has-caption -- the cues are
              drawn below from the plan rather than as a text track. */}
          <video
            ref={videoRef}
            src={proxyUrl}
            className={crop ? 'h-full w-auto max-w-none' : 'max-h-full max-w-full'}
            playsInline
            onTimeUpdate={(event) => onFrame(frameAt(plan, event.currentTarget.currentTime))}
            onEnded={onEnded}
          />
        </div>
      ) : (
        <p className="grid h-full place-items-center p-6 text-center text-sm text-[var(--cm-ink-2)]">
          This project published no proxy, so there is nothing to play.
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
  onToggle,
}: {
  readonly plan: PreviewPlan;
  readonly frame: number;
  readonly playing: boolean;
  readonly onStep: (by: number) => void;
  readonly onToggle: () => void;
}) {
  return (
    <div className="flex items-center justify-center gap-3" aria-label="Transport">
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
