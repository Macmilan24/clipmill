/**
 * Clip Inspector: review one candidate at a time with its words, framing,
 * surroundings and reasons, then approve, keep or reject. The cut on screen is
 * what an approval builds; a moved cut of a clip with an edit becomes a second
 * edit beside the first.
 */
import '../inspector/review.css';

import {
  ArrowLeft,
  ArrowLeftToLine,
  ArrowRightToLine,
  Check,
  ChevronLeft,
  ChevronRight,
  FileText,
  Info,
  ListChecks,
  Maximize2,
  PanelLeft,
  RotateCcw,
  Undo2,
  ZoomIn,
  ZoomOut,
} from 'lucide-react';
import { useEffect, useMemo, useRef, useState, useSyncExternalStore } from 'react';

import { Button } from '../components/ui/button.js';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../components/ui/tabs.js';
import type { CropPath, PreviewPlan } from '../daemon/client.js';
import type { ExactCaptions } from '../editor/exactCaptions.js';
import { DetailsPanel } from '../inspector/DetailsPanel.js';
import { Monitor, type MonitorView } from '../inspector/Monitor.js';
import { PlaybackController } from '../inspector/playback.js';
import { Queue, type QueueFilter } from '../inspector/Queue.js';
import { type Cut, clockTenths, lengthLabel, overlapsOf, sameCut } from '../inspector/review.js';
import { BoundaryStrip, Overview, initialView, zoomView } from '../inspector/Timeline.js';
import { TipButton } from '../inspector/TipButton.js';
import { TranscriptPanel } from '../inspector/TranscriptPanel.js';
import { useReviewKeys } from '../inspector/useReviewKeys.js';
import { WhyPanel } from '../inspector/WhyPanel.js';
import { type CoachMark, CoachMarks } from '../onboarding/CoachMarks.js';
import type { Peaks } from '../results/loader.js';
import { type ClipRow, TICKS_PER_SECOND } from '../results/model.js';
import { TONE_INK, stateOf, wash } from '../results/parts/state.js';
import { snapEnd, snapStart } from '../results/transcript.js';
import type { TranscriptState } from '../results/useResults.js';

export interface ClipInspectorProps {
  readonly rows: readonly ClipRow[];
  readonly candidateId: string;
  readonly proxyUrl: string | null;
  readonly crop: CropPath | null;
  /**
   * The clip an approval of the cut on screen would build, and its captions.
   * Absent on a shell that cannot build one; the solver's crop stands in.
   */
  readonly preview?: PreviewPlan | null;
  readonly previewCaptions?: ExactCaptions | null;
  /** Ask for that clip for a cut: `null` is the search's own. */
  readonly onPreview?: ((cut: Cut | null) => void) | null;
  /** Where each clip's camera would point, by candidate, for the queue. */
  readonly framing?: ReadonlyMap<string, number>;
  readonly peaks: Peaks | null;
  readonly tileUrl: (atTicks: number) => string | null;
  readonly transcript: TranscriptState;
  /** How long the recording is, when the run measured it. */
  readonly sourceDurationTicks: number | null;
  readonly durationTarget: { readonly minTicks: number; readonly maxTicks: number } | null;
  /** True while a decision is being written. */
  readonly busy: boolean;
  /** What the last action said, when it said something. */
  readonly notice: string | null;
  readonly autoAdvance: boolean;
  readonly onAutoAdvance: (on: boolean) => void;
  readonly onSelect: (candidateId: string) => void;
  readonly onBack: () => void;
  /**
   * Approve with the cut on screen — `null` when it is still the search's —
   * and with `open`, go on to edit it.
   */
  readonly onApprove: (cut: Cut | null, open: boolean) => void;
  /** Keep, reject, or take a decision back with `null`. */
  readonly onDecide: (decision: 'kept' | 'rejected' | null) => void;
  /** Take back the last decision made here, or null when there is none. */
  readonly onUndo: (() => void) | null;
  /** Open the clip's existing edit, or null when it has none yet. */
  readonly onOpenEdit: (() => void) | null;
}

type Tab = 'transcript' | 'why' | 'details';

/** The Inspector's tips, the first time it opens. */
const INSPECTOR_TIPS: readonly CoachMark[] = [
  {
    target: 'preview',
    title: 'The clip as it will be built',
    body: 'This is what approving makes: the framing and the captions in your look. Source shows the whole recording, with the crop outlined.',
  },
  {
    target: 'cut',
    title: 'Where it starts and ends',
    body: 'Drag the handles on the strip, or press I and O at the playhead. Play the start and Play the end let you hear past the edges first.',
  },
  {
    target: 'decide',
    title: 'Decide',
    body: 'Approve builds the edit; Keep for later sets it aside; Reject drops it. A, H and X do the same, and a decision can be taken back.',
  },
  {
    target: 'transcript',
    title: 'What is said',
    body: 'Every word of the clip and the sentences either side. Click a word to go there.',
  },
];

/** A remembered preference, read defensively: storage can be absent or refuse. */
function remembered<T extends string>(key: string, allowed: readonly T[], fallback: T): T {
  try {
    const value = localStorage.getItem(key);
    return allowed.includes(value as T) ? (value as T) : fallback;
  } catch {
    return fallback;
  }
}

function remember(key: string, value: string): void {
  try {
    localStorage.setItem(key, value);
  } catch {
    /* A preference that cannot be saved is still in effect for this session. */
  }
}

export function ClipInspector(props: ClipInspectorProps) {
  const row = props.rows.find((candidate) => candidate.candidateId === props.candidateId);
  if (!row) {
    return (
      <div className="review-missing">
        <p>That clip is not in the current ranking.</p>
        <Button variant="outline" onClick={props.onBack}>
          Back to Results
        </Button>
      </div>
    );
  }
  // Keyed by clip: a different clip is a different cut, view and audition,
  // and none of the last one's should leak into it.
  return <Review key={row.candidateId} row={row} {...props} />;
}

function Review({
  row,
  rows,
  proxyUrl,
  crop,
  preview = null,
  previewCaptions = null,
  onPreview = null,
  framing,
  peaks,
  tileUrl,
  transcript,
  sourceDurationTicks,
  durationTarget,
  busy,
  notice,
  autoAdvance,
  onAutoAdvance,
  onSelect,
  onBack,
  onApprove,
  onDecide,
  onUndo,
  onOpenEdit,
}: ClipInspectorProps & { readonly row: ClipRow }) {
  // One clock per clip, starting on its first frame.
  // eslint-disable-next-line react-hooks/exhaustive-deps -- keyed by clip upstream
  const controller = useMemo(() => new PlaybackController(row.startTicks), []);
  useEffect(() => () => controller.dispose(), [controller]);

  const chosen: Cut = { startTicks: row.startTicks, endTicks: row.endTicks };
  const alternative = row.boundary?.alternative ?? null;
  const [draft, setDraft] = useState<Cut | null>(null);
  const [auditioning, setAuditioning] = useState(false);
  const shown: Cut = auditioning && alternative ? alternative : (draft ?? chosen);
  const moved = !sameCut(shown, chosen);

  const durationTicks = Math.max(
    sourceDurationTicks ?? 0,
    ...rows.map((candidate) => candidate.endTicks + 10 * TICKS_PER_SECOND),
  );
  const [view, setView] = useState<Cut>(() => initialView(chosen, durationTicks));
  const [tab, setTab] = useState<Tab>(() =>
    remembered('clipmill.review.tab', ['transcript', 'why', 'details'], 'transcript'),
  );
  const [monitorView, setMonitorView] = useState<MonitorView>(() =>
    remembered('clipmill.review.view', ['result', 'source'], 'result'),
  );
  const [safeArea, setSafeArea] = useState(
    () => remembered('clipmill.review.safe', ['on', 'off'], 'off') === 'on',
  );
  const [queueOpen, setQueueOpen] = useState(
    () => remembered('clipmill.review.queue', ['open', 'closed'], 'open') === 'open',
  );
  const [filter, setFilter] = useState<QueueFilter>(() =>
    remembered('clipmill.review.filter', ['all', 'undecided'], 'all'),
  );

  // Playing through the cut on screen stops at its end.
  useEffect(() => {
    controller.setCut(shown);
  }, [controller, shown.startTicks, shown.endTicks]);

  // Ask for the clip an approval of the cut on screen would build, once the
  // cut has stopped moving: a drag is many cuts, and only the last is judged.
  const askPreview = useRef(onPreview);
  askPreview.current = onPreview;
  useEffect(() => {
    const timer = setTimeout(() => askPreview.current?.(moved ? shown : null), 250);
    return () => clearTimeout(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps -- the cut is the signal
  }, [moved, shown.startTicks, shown.endTicks]);

  const words = transcript.status === 'ready' ? transcript.transcript : null;
  const index = rows.indexOf(row);
  const queue = rows.filter(
    (candidate) =>
      filter === 'all' || candidate.decision === null || candidate.candidateId === row.candidateId,
  );
  const inQueue = queue.indexOf(row);
  const overlaps = useMemo(() => overlapsOf(rows, row), [rows, row]);
  const approved = row.decision === 'approved';
  const hasEdit = row.docId !== null;
  const declined = row.review?.status === 'rejected';
  const state = stateOf(row);

  const setCut = (next: Cut) => {
    setAuditioning(false);
    setDraft(sameCut(next, chosen) ? null : next);
    // An edge moved out of sight — from the transcript, say — brings the
    // strip with it, so the cut being judged is the cut being shown.
    if (next.startTicks < view.startTicks || next.endTicks > view.endTicks) {
      setView(initialView(next, durationTicks));
    }
  };
  /** Move one edge from outside the strip, and park the playhead on it. */
  const moveEdge = (edge: 'in' | 'out', ticks: number) => {
    setCut(
      edge === 'in'
        ? { startTicks: ticks, endTicks: shown.endTicks }
        : { startTicks: shown.startTicks, endTicks: ticks },
    );
    controller.pause();
    controller.seek(ticks);
  };
  const approve = (open: boolean) => {
    if (busy) return;
    // Already approved, with its edit: there is nothing to record, only to open.
    if (approved && !moved) {
      if (open) onOpenEdit?.();
      return;
    }
    onApprove(moved ? shown : null, open);
  };
  const decide = (decision: 'kept' | 'rejected') => {
    if (busy) return;
    onDecide(row.decision === decision ? null : decision);
  };
  const markAt = (edge: 'in' | 'out') => {
    const at = controller.getState().ticks;
    const landed = words ? (edge === 'in' ? snapStart(words, at) : snapEnd(words, at)) : at;
    if (edge === 'in' && landed < shown.endTicks - TICKS_PER_SECOND) moveEdge('in', landed);
    else if (edge === 'out' && landed > shown.startTicks + TICKS_PER_SECOND)
      moveEdge('out', landed);
  };

  useReviewKeys({
    toggle: () => controller.toggle(),
    pause: () => controller.pause(),
    shuttle: (direction) => controller.shuttle(direction),
    step: (frames) => controller.step(frames),
    skip: (seconds) => controller.seek(controller.getState().ticks + seconds * TICKS_PER_SECOND),
    goToStart: () => controller.seek(shown.startTicks),
    goToEnd: () => controller.seek(shown.endTicks),
    playAroundStart: () => controller.playAround(shown.startTicks),
    playAroundEnd: () => controller.playAround(shown.endTicks),
    markStart: () => markAt('in'),
    markEnd: () => markAt('out'),
    previousClip: () => {
      const previous = queue[inQueue - 1];
      if (previous) onSelect(previous.candidateId);
    },
    nextClip: () => {
      const next = queue[inQueue + 1];
      if (next) onSelect(next.candidateId);
    },
    approve: () => approve(false),
    approveAndEdit: () => approve(true),
    keep: () => decide('kept'),
    reject: () => decide('rejected'),
    clear: () => {
      if (!busy && row.decision !== null) onDecide(null);
    },
    undo: () => onUndo?.(),
  });

  const lengthDelta = shown.endTicks - shown.startTicks - (chosen.endTicks - chosen.startTicks);

  return (
    <div className="review-workspace">
      <CoachMarks place="inspector" marks={INSPECTOR_TIPS} />
      <header className="review-heading">
        <div className="review-identity">
          <TipButton label="Back to results" onClick={onBack}>
            <ArrowLeft />
          </TipButton>
          <TipButton
            label={queueOpen ? 'Hide the clip list' : 'Show the clip list'}
            pressed={queueOpen}
            onClick={() => {
              setQueueOpen(!queueOpen);
              remember('clipmill.review.queue', queueOpen ? 'closed' : 'open');
            }}
          >
            <PanelLeft />
          </TipButton>
          <div className="review-title">
            <h1 title={row.headline}>{row.headline || 'Untitled clip'}</h1>
            <p>
              <span className="mono">
                Clip {String(row.rank).padStart(2, '0')} of {rows.length}
              </span>
              <span aria-hidden="true">·</span>
              <span className="mono">
                {clockTenths(chosen.startTicks)}–{clockTenths(chosen.endTicks)}
              </span>
              <span
                className="review-state"
                style={{ color: TONE_INK[state.tone], background: wash(state.tone) }}
              >
                {state.label}
              </span>
            </p>
          </div>
        </div>

        <div className="review-heading-actions">
          {notice && (
            <p className="review-notice" role="status" title={notice}>
              {notice}
            </p>
          )}
          {onUndo && (
            <TipButton label="Undo the last decision" onClick={onUndo} disabled={busy}>
              <Undo2 />
            </TipButton>
          )}
          <div className="review-nav">
            <TipButton
              label="Previous clip"
              disabled={inQueue <= 0}
              onClick={() => onSelect(queue[inQueue - 1]!.candidateId)}
            >
              <ChevronLeft />
            </TipButton>
            <span className="mono">
              {index + 1}/{rows.length}
            </span>
            <TipButton
              label="Next clip"
              disabled={inQueue < 0 || inQueue >= queue.length - 1}
              onClick={() => onSelect(queue[inQueue + 1]!.candidateId)}
            >
              <ChevronRight />
            </TipButton>
          </div>
          <div
            className="review-decisions"
            role="group"
            aria-label="Decide about this clip"
            data-coach="decide"
          >
            <Button
              variant="ghost"
              size="sm"
              className="review-reject"
              aria-pressed={row.decision === 'rejected'}
              disabled={busy}
              onClick={() => decide('rejected')}
            >
              Reject
            </Button>
            <Button
              variant="outline"
              size="sm"
              aria-pressed={row.decision === 'kept'}
              disabled={busy}
              onClick={() => decide('kept')}
            >
              Keep for later
            </Button>
            {hasEdit && onOpenEdit ? (
              <Button variant="outline" size="sm" disabled={busy} onClick={onOpenEdit}>
                Open edit
              </Button>
            ) : (
              <Button variant="outline" size="sm" disabled={busy} onClick={() => approve(true)}>
                {declined ? 'Edit anyway' : 'Approve & edit'}
              </Button>
            )}
            {approved && !moved ? (
              <span className="review-approved" role="status">
                <Check aria-hidden="true" />
                Approved
              </span>
            ) : (
              <Button size="sm" disabled={busy} onClick={() => approve(false)}>
                {busy
                  ? 'Working…'
                  : hasEdit && moved
                    ? 'Approve as new edit'
                    : declined
                      ? 'Approve anyway'
                      : 'Approve'}
              </Button>
            )}
          </div>
        </div>
      </header>

      <div className="review-body" data-queue={queueOpen ? 'open' : 'closed'}>
        {queueOpen && (
          <Queue
            rows={rows}
            framing={framing}
            candidateId={row.candidateId}
            filter={filter}
            onFilter={(next) => {
              setFilter(next);
              remember('clipmill.review.filter', next);
            }}
            tileUrl={tileUrl}
            busy={busy}
            onSelect={onSelect}
            autoAdvance={autoAdvance}
            onAutoAdvance={onAutoAdvance}
          />
        )}

        <Monitor
          src={proxyUrl}
          crop={crop}
          plan={preview}
          captions={previewCaptions}
          controller={controller}
          cut={shown}
          view={monitorView}
          onView={(next) => {
            setMonitorView(next);
            remember('clipmill.review.view', next);
          }}
          safeArea={safeArea}
          onSafeArea={(next) => {
            setSafeArea(next);
            remember('clipmill.review.safe', next ? 'on' : 'off');
          }}
          alternative={
            alternative
              ? {
                  shown: auditioning,
                  onShow: (on) => {
                    setAuditioning(on);
                    controller.pause();
                    controller.seek(on ? alternative.startTicks : (draft ?? chosen).startTicks);
                  },
                }
              : null
          }
        />

        <aside className="review-side" aria-label="About this clip">
          <Tabs
            value={tab}
            onValueChange={(next) => {
              setTab(next as Tab);
              remember('clipmill.review.tab', next);
            }}
            className="review-tabs"
          >
            <TabsList className="review-tab-list">
              <TabsTrigger value="transcript" data-coach="transcript">
                <FileText aria-hidden="true" />
                Transcript
              </TabsTrigger>
              <TabsTrigger value="why">
                <ListChecks aria-hidden="true" />
                Why
              </TabsTrigger>
              <TabsTrigger value="details">
                <Info aria-hidden="true" />
                Details
              </TabsTrigger>
            </TabsList>
            <TabsContent value="transcript" className="review-tab-panel">
              <TranscriptPanel
                state={transcript}
                cut={shown}
                controller={controller}
                onStart={(ticks) => moveEdge('in', ticks)}
                onEnd={(ticks) => moveEdge('out', ticks)}
              />
            </TabsContent>
            <TabsContent value="why" className="review-tab-panel">
              <WhyPanel row={row} onJump={(ticks) => controller.seek(ticks)} />
            </TabsContent>
            <TabsContent value="details" className="review-tab-panel">
              <DetailsPanel
                row={row}
                total={rows.length}
                cut={shown}
                overlaps={overlaps}
                durationTarget={durationTarget}
                onSelect={onSelect}
              />
            </TabsContent>
          </Tabs>
        </aside>
      </div>

      <section className="review-timeline" aria-label="Timeline">
        <Overview
          rows={rows}
          candidateId={row.candidateId}
          durationTicks={durationTicks}
          view={view}
          controller={controller}
          onSelect={onSelect}
          onSeek={(ticks) => controller.seek(ticks)}
        />
        <div className="review-strip-bar" data-coach="cut">
          <span className="review-cut-summary">
            <span className="review-lane-label">
              {auditioning ? 'Alternative' : moved ? 'Your cut' : 'Cut'}
            </span>
            <span className="mono">
              {clockTenths(shown.startTicks)} – {clockTenths(shown.endTicks)}
            </span>
            <span className="mono review-cut-length">
              {lengthLabel(shown.endTicks - shown.startTicks)}
              {moved && lengthDelta !== 0 && (
                <span>
                  {' '}
                  ({lengthDelta > 0 ? '+' : '−'}
                  {lengthLabel(Math.abs(lengthDelta))})
                </span>
              )}
            </span>
          </span>
          {auditioning && alternative && (
            <Button
              size="xs"
              variant="outline"
              onClick={() => {
                setDraft(sameCut(alternative, chosen) ? null : alternative);
                setAuditioning(false);
              }}
            >
              Use this cut
            </Button>
          )}
          {moved && !auditioning && (
            <Button size="xs" variant="ghost" onClick={() => setDraft(null)}>
              <RotateCcw aria-hidden="true" />
              Back to the suggested cut
            </Button>
          )}
          <span className="review-spacer" />
          <Button
            size="xs"
            variant="ghost"
            disabled={!proxyUrl}
            onClick={() => controller.playAround(shown.startTicks)}
          >
            <ArrowRightToLine aria-hidden="true" />
            Play the start
          </Button>
          <Button
            size="xs"
            variant="ghost"
            disabled={!proxyUrl}
            onClick={() => controller.playAround(shown.endTicks)}
          >
            <ArrowLeftToLine aria-hidden="true" />
            Play the end
          </Button>
          <span className="review-divider" aria-hidden="true" />
          <TipButton
            label="Zoom out"
            size="icon-xs"
            onClick={() =>
              setView(zoomView(view, 1.6, (view.startTicks + view.endTicks) / 2, durationTicks))
            }
          >
            <ZoomOut />
          </TipButton>
          <TipButton
            label="Zoom in"
            size="icon-xs"
            onClick={() =>
              setView(zoomView(view, 1 / 1.6, controller.getState().ticks, durationTicks))
            }
          >
            <ZoomIn />
          </TipButton>
          <TipButton
            label="Fit the cut"
            size="icon-xs"
            onClick={() => setView(initialView(shown, durationTicks))}
          >
            <Maximize2 />
          </TipButton>
        </div>
        <BoundaryStrip
          cut={shown}
          chosen={chosen}
          alternative={auditioning ? null : alternative}
          suggestedStarts={row.latticeStarts}
          suggestedEnds={row.latticeEnds}
          transcript={words}
          peaks={peaks}
          tileUrl={tileUrl}
          durationTicks={durationTicks}
          view={view}
          onView={setView}
          controller={controller}
          onCut={auditioning ? null : setCut}
        />
      </section>
      <PlayingAnnouncer controller={controller} />
    </div>
  );
}

/** Says play and pause out loud for screen readers, which cannot see the icon. */
function PlayingAnnouncer({ controller }: { readonly controller: PlaybackController }) {
  const playing = useSyncExternalStore(controller.subscribe, () => controller.getState().playing);
  return (
    <span className="sr-only" role="status" aria-live="polite">
      {playing ? 'Playing' : 'Paused'}
    </span>
  );
}
