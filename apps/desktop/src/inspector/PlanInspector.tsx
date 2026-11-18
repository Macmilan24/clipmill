import { useEffect, useMemo, useRef, useState } from 'react';

import { Button } from '../components/ui/button.js';
import { type ShellApi } from '../daemon/api.js';
import { type PreviewPlan } from '../daemon/client.js';
import { type RecordingRun } from '../daemon/mind.js';
import { useDraftAudio } from '../editor/useDraftAudio.js';
import { gainAt } from '../editor/player.js';
import { BrandLayer } from '../editor/BrandLayer.js';
import { CaptionCanvas } from '../editor/CaptionCanvas.js';
import { CompositionCanvas } from '../editor/CompositionCanvas.js';
import { exactCaptionsOf } from '../editor/exactCaptions.js';
import { proxySecondsAt, resolvePlaybackFrame, secondsAt } from '../editor/player.js';
import { clock, duration } from '../results/model.js';
import { type Transcript } from '../results/transcript.js';
import { type PlannedClip } from '../results/plans.js';

function PlanPlayer({
  plan,
  projectId,
  api,
}: {
  readonly plan: PreviewPlan;
  readonly projectId: string;
  readonly api: ShellApi;
}) {
  const media = useRef<HTMLVideoElement>(null);
  const position = useRef(0);
  const [frame, setFrame] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [error, setError] = useState('');
  const urls = useMemo(
    () =>
      new Map(
        plan.proxies.map((proxy) => [
          proxy.sourceFingerprint,
          api.mediaUrl(projectId, proxy.artifactId, proxy.file),
        ]),
      ),
    [plan, api, projectId],
  );
  const audioUrls = useMemo(
    () =>
      new Map(
        plan.proxies.flatMap((proxy) =>
          proxy.audioArtifactId && proxy.audioFile
            ? [
                [
                  proxy.sourceFingerprint,
                  api.mediaUrl(projectId, proxy.audioArtifactId, proxy.audioFile),
                ] as const,
              ]
            : [],
        ),
      ),
    [api, plan, projectId],
  );
  const draft = useDraftAudio(
    media,
    gainAt(plan, frame),
    plan.gain.some((point) => point.gainDb > 0),
    { plan, frame, audioUrls },
  );
  const first = plan.segments[0];
  const url = first && urls.get(first.sourceFingerprint);
  const captions = useMemo(() => exactCaptionsOf(plan, api.captionFontUrl ?? null), [plan, api]);
  if (!url || plan.frameCount < 1) return <p>The recording has no playable preview.</p>;
  function seek(next: number) {
    const seconds = proxySecondsAt(plan, next);
    if (!media.current || seconds === null) return;
    position.current = next;
    setFrame(next);
    media.current.currentTime = Math.max(0, seconds);
  }
  function play() {
    draft.connect();
    void media.current?.play().catch(() => setError('Playback could not start. Try again.'));
  }
  function advance(seconds: number, ended = false) {
    if (!media.current || media.current.seeking) return null;
    if (media.current.paused && !ended) return position.current;
    const next = resolvePlaybackFrame(plan, position.current, seconds);
    if (next.hold) return null;
    if (next.seek || next.ended) {
      if (next.ended) media.current.pause();
      seek(next.frame);
      if (ended && next.seek) play();
      return null;
    }
    position.current = next.frame;
    setFrame(next.frame);
    return next.frame;
  }
  return (
    <div className="mx-auto w-full max-w-80 space-y-3">
      <div
        className="relative mx-auto max-h-[48vh] overflow-hidden rounded-md bg-black"
        style={{
          aspectRatio: `${plan.width}/${plan.height}`,
          width: `min(100%, ${(48 * plan.width) / plan.height}vh)`,
        }}
      >
        {/* Captions are drawn from the edit's programme-timed track. */}
        <video
          ref={media}
          src={url}
          preload="auto"
          playsInline
          crossOrigin="anonymous"
          aria-hidden
          className="pointer-events-none absolute size-full opacity-0"
          onLoadedMetadata={() => seek(0)}
          onPlay={() => setPlaying(true)}
          onPause={() => setPlaying(false)}
          onEnded={(event) => advance(event.currentTarget.duration, true)}
          onError={() => setError('The recording preview could not be loaded.')}
        />
        <CompositionCanvas
          video={media}
          mediaKey={url}
          plan={plan}
          frame={frame}
          onFrame={advance}
          proxyUrls={urls}
        />
        <BrandLayer plan={plan} frame={frame} assetUrl={api.assetUrl ?? null} />
        {captions && (
          <CaptionCanvas
            ass={captions.ass}
            faces={captions.faces}
            family={captions.family}
            seconds={secondsAt(plan, frame)}
            frameWidth={plan.width}
            frameHeight={plan.height}
          />
        )}
      </div>
      <div className="flex items-center gap-2">
        <Button
          size="sm"
          variant="outline"
          onClick={() => {
            if (playing) media.current?.pause();
            else {
              if (position.current >= plan.frameCount - 1) seek(0);
              play();
            }
          }}
        >
          {playing ? 'Pause' : 'Play'}
        </Button>
        <input
          type="range"
          aria-label="Clip playhead"
          className="min-w-0 flex-1"
          min={0}
          max={plan.frameCount - 1}
          value={frame}
          onChange={(event) => seek(Number(event.target.value))}
        />
        <span className="font-mono text-xs">{duration(secondsAt(plan, frame))}</span>
      </div>
      {(error || draft.problem) && <p role="alert">{error || draft.problem}</p>}
    </div>
  );
}

export function PlanInspector({
  clip,
  preview,
  transcript,
  api,
  run,
  busy,
  notice,
  onBack,
  onEdit,
  onStarted,
}: {
  readonly clip: PlannedClip;
  readonly preview: PreviewPlan | null;
  readonly transcript: Transcript | null;
  readonly api: ShellApi;
  readonly run: RecordingRun;
  readonly busy: boolean;
  readonly notice: string | null;
  readonly onBack: () => void;
  readonly onEdit: () => void;
  readonly onStarted: ((jobId: string) => void) | undefined;
}) {
  const [decision, setDecision] = useState<string | null>(null);
  const [reason, setReason] = useState('');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const { projectId, sourceId, jobId } = run;
  const writeVersion = useRef(0);
  useEffect(() => {
    let live = true;
    const version = writeVersion.current;
    void api
      .planDecision?.({ projectId, sourceId, jobId }, clip.planId)
      .then((value) => {
        if (live && version === writeVersion.current) {
          setDecision(value.decision);
          setReason(value.reason);
        }
      })
      .catch(() => undefined);
    return () => {
      live = false;
    };
  }, [api, projectId, sourceId, jobId, clip.planId]);
  async function decide(next: 'kept' | 'rejected') {
    if (!api.planDecision) return;
    ++writeVersion.current;
    setSaving(true);
    setError('');
    try {
      const saved = await api.planDecision(
        run,
        clip.planId,
        next === decision ? 'undecided' : next,
        next === decision ? undefined : reason.trim() || undefined,
      );
      setDecision(saved.decision);
      setReason(saved.reason);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setSaving(false);
    }
  }
  return (
    <section className="workspace-page overflow-y-auto" aria-label="Clip inspector">
      <header className="flex flex-wrap items-center gap-3">
        <Button size="sm" variant="outline" onClick={onBack}>
          Back to Results
        </Button>
        <div className="min-w-0 flex-1">
          <h1 className="workspace-title">{clip.title}</h1>
          <p className="text-xs text-[var(--cm-text-secondary)]">
            {clip.formLabel} · {duration(clip.durationSeconds)}
            {clip.assessment && ` · Keep score ${clip.assessment.keepScore}/100`}
          </p>
        </div>
        <Button disabled={busy || saving} onClick={onEdit}>
          Edit clip
        </Button>
      </header>
      <div className="grid min-h-0 gap-4 md:grid-cols-2">
        <div className="workspace-panel p-4">
          {preview ? (
            <PlanPlayer api={api} projectId={projectId} plan={preview} />
          ) : (
            <p role="status">{notice || 'Preparing clip preview…'}</p>
          )}
        </div>
        <div className="workspace-panel space-y-3 overflow-y-auto p-4 text-sm">
          <h2 className="font-semibold">Parts in playback order</h2>
          <ol className="space-y-3">
            {clip.parts.map((part, index) => {
              const spoken = transcript?.words
                .filter(
                  (word) =>
                    word.startTicks >= part.startTicks &&
                    word.endTicks <= part.endTicks &&
                    !part.removed.some(
                      (cut) => word.startTicks < cut.end_ticks && word.endTicks > cut.start_ticks,
                    ),
                )
                .map((word) => word.text)
                .join(' ');
              const join = index > 0 ? clip.joins[index - 1] : undefined;
              const opinion =
                join && clip.assessment?.joins.find((item) => item.after === join.after);
              return (
                <li key={part.id}>
                  {join && (
                    <p className="mb-1 text-xs text-[var(--cm-text-secondary)]">
                      {join.kind === 'distant'
                        ? 'Join across the recording'
                        : join.kind === 'reorder'
                          ? 'Reordered join'
                          : 'Adjacent join'}
                      {opinion?.note && ` · ${opinion.note}`}
                    </p>
                  )}
                  <strong>
                    {index + 1}. {part.roleLabel}
                  </strong>{' '}
                  · {clock(part.startTicks)}–{clock(part.endTicks)}
                  {spoken && <p className="my-1">“{spoken}”</p>}
                  {part.why && <p>{part.why}</p>}
                  {part.removed.map((cut) => (
                    <p className="text-xs" key={cut.start_ticks}>
                      Removed {clock(cut.start_ticks)}–{clock(cut.end_ticks)} ·{' '}
                      {cut.reason.replaceAll('_', ' ')}
                    </p>
                  ))}
                </li>
              );
            })}
          </ol>
          {clip.assessment?.chapter && (
            <p className="text-xs">Chapter · {clip.assessment.chapter}</p>
          )}
          {clip.assessment?.summary && <p>{clip.assessment.summary}</p>}
          {clip.assessment?.reasons.map((note) => (
            <p key={note}>{note}</p>
          ))}
          {clip.evidence.length > 0 && (
            <details>
              <summary>Source evidence</summary>
              <p className="break-words text-xs">{clip.evidence.join(' · ')}</p>
            </details>
          )}
          {[...clip.doubts, ...(clip.assessment?.issues ?? [])].map((issue, index) => (
            <p key={index} className="text-xs">
              Review note · {issue}
            </p>
          ))}
          {clip.assessment?.repaired && <p className="text-xs">Revised after the join review.</p>}
          {api.planDecision && (
            <label className="block text-xs">
              Decision reason (optional)
              <input
                className="mt-1 block w-full rounded-md border border-[var(--cm-border)] bg-transparent px-2 py-1.5 text-sm"
                maxLength={240}
                value={reason}
                onChange={(event) => setReason(event.target.value)}
              />
            </label>
          )}
          <div className="flex flex-wrap gap-2">
            {api.planDecision &&
              (['kept', 'rejected'] as const).map((value) => (
                <Button
                  size="sm"
                  key={value}
                  variant={decision === value ? 'default' : 'outline'}
                  aria-pressed={decision === value}
                  disabled={saving || busy}
                  onClick={() => void decide(value)}
                >
                  {value === 'kept' ? 'Keep' : 'Reject'}
                </Button>
              ))}
            {api.reprompt && onStarted && (
              <Button
                size="sm"
                variant="outline"
                disabled={saving || busy}
                onClick={() => {
                  setSaving(true);
                  setError('');
                  const maxSeconds = Math.floor(clip.durationSeconds * 0.8);
                  const limits =
                    maxSeconds > 5
                      ? { minSeconds: Math.max(5, Math.floor(maxSeconds / 2)), maxSeconds }
                      : undefined;
                  void api.reprompt!(
                    run,
                    `Make shorter clips like “${clip.title}”${clip.hookText ? `, with an opening like “${clip.hookText}”` : ''}.`.slice(
                      0,
                      1000,
                    ),
                    limits,
                  )
                    .then(onStarted)
                    .catch((cause) =>
                      setError(cause instanceof Error ? cause.message : String(cause)),
                    )
                    .finally(() => setSaving(false));
                }}
              >
                More like this, shorter
              </Button>
            )}
          </div>
          {(error || (notice && preview)) && <p role="status">{error || notice}</p>}
        </div>
      </div>
    </section>
  );
}
