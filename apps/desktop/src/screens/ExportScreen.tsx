/**
 * The Export screen's container: hold what the user typed, and ask the daemon
 * what it means.
 *
 * Which document is exported is not decided here. The clip arrives named in
 * full from the route — the document a person approved, edited, and chose to
 * deliver — and the request names that document and nothing else. It used to
 * be the newest document of the newest project, which delivered another
 * project's clip the moment someone approved one in an older project. With no
 * clip named, the screen lists every edit there is and lets a person choose.
 *
 * Which *revision* is exported is not decided here either, but it is held.
 * The plan says which revision it checked, the request carries that revision
 * as the one reviewed, and the daemon refuses any other: an edit that landed
 * between the review and the click is a conflict, re-planned and shown, not
 * a clip nobody looked at. What was queued is then followed to the files.
 *
 * Every keystroke in the pattern or the destination re-plans, debounced. That
 * is deliberate churn: the alternative is resolving the pattern here, which
 * would be a second implementation of the naming rules and would eventually
 * disagree with the files that get written. A local socket can answer this far
 * faster than a person types.
 */
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import type { ExportPlan, ExportRequest, QueuedExport } from '../daemon/client.js';
import { DocumentPicker } from '../editor/DocumentPicker.js';
import { useEditDocuments } from '../editor/documents.js';
import { latestExportOf, useDelivery } from '../export/delivery.js';
import type { ClipRef } from '../shell/route.js';
import { Export } from './Export.js';
import { BatchExportScreen } from './BatchExportScreen.js';
import { UploadPanel } from '../youtube/UploadPanel.js';

/** Long enough that a typed word is one request, short enough to feel live. */
const PLAN_DEBOUNCE_MS = 250;
/** Past this, the rights confirmation applies. The daemon holds the same rule. */
const RIGHTS_GATE_SECONDS = 60;
const DURATION_GATE = 'duration_60s';
/** The confirmation that ships a subtitle file faster than the reading profile. */
const READING_RATE_GATE = 'captions_reading_rate';
const HOT_CAPTION_CODE = 'captions.reading_rate';
/** The tokens that make each clip's name its own; the daemon insists on one. */
const UNIQUE_TOKENS = ['{index}', '{clip}', '{address}'] as const;

/**
 * The pattern the daemon is asked to resolve.
 *
 * A plain name is what most people type, and the daemon is right that a
 * plain name would give every clip in an export the same file. So the name
 * is kept and the clip's number is added to it — `reacher` becomes
 * `reacher-{index}`, which is `reacher-01` — rather than refusing what was
 * typed. Nothing is added to a pattern that already names each clip, and
 * an empty field takes the default.
 */
export function effectivePattern(typed: string): string {
  const pattern = typed.trim();
  if (pattern === '') {
    return DEFAULT_PATTERN;
  }
  return UNIQUE_TOKENS.some((token) => pattern.includes(token)) ? pattern : `${pattern}-{index}`;
}
const DEFAULT_PATTERN = '{index}-{clip}';
/**
 * The model work that shaped every clip this pipeline produces: captions from
 * recognition, and a crop path from a face pass. Declared rather than inferred,
 * because a disclosure a renderer guessed is a disclosure nobody checked.
 */
const AI_ASSISTANCE = ['asr_captions', 'reframe'] as const;

export interface ExportScreenProps {
  /** The clip to deliver, or null when the row was reached with none named. */
  readonly clip: ClipRef | null;
  /** Open a different clip here — from the list this screen offers. */
  readonly onOpen: (clip: ClipRef) => void;
  readonly onEdit?: (clip: ClipRef) => void;
  readonly onOpenChannelSettings?: () => void;
  readonly api?: ShellApi;
}

export function ExportScreen({
  clip,
  onOpen,
  onEdit,
  onOpenChannelSettings,
  api = daemonApi,
}: ExportScreenProps) {
  const [batchMode, setBatchMode] = useState(false);
  const projectId = clip?.projectId ?? null;
  const docId = clip?.docId ?? null;
  const [durationTicks, setDurationTicks] = useState(0);
  const [title, setTitle] = useState('');
  const [destination, setDestination] = useState('');
  const [pattern, setPattern] = useState('{index}-{clip}');
  const [attestation, setAttestation] = useState('');
  const [rightsApproval, setRightsApproval] = useState<string | null>(null);
  const [captionApproval, setCaptionApproval] = useState<string | null>(null);
  const [plan, setPlan] = useState<ExportPlan | null>(null);
  const [planning, setPlanning] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [queued, setQueued] = useState<QueuedExport | null>(null);
  const [archive, setArchive] = useState<{ path: string; entryCount: number } | null>(null);
  /** Bumped to plan again over the document as it is now, after a conflict. */
  const [replan, setReplan] = useState(0);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const selectionGeneration = useRef(0);
  const delivery = useDelivery(projectId, queued, api);
  const approvalKey = plan
    ? JSON.stringify([
        projectId,
        docId,
        plan.revision,
        plan.findings
          .filter((finding) => finding.code === HOT_CAPTION_CODE)
          .map((finding) => finding.detail.replace(/ — confirmed as read\.$/, ''))
          .sort(),
      ])
    : null;
  const gatePassed = approvalKey !== null && rightsApproval === approvalKey;
  const hotCaptionsConfirmed = approvalKey !== null && captionApproval === approvalKey;
  const [audition, setAudition] = useState<{
    projectId: string;
    renderId: string;
    url: string;
  } | null>(null);
  const [auditionProblem, setAuditionProblem] = useState<string | null>(null);
  const renderId = delivery?.renderArtifactId;
  useEffect(() => {
    setAudition(null);
    setAuditionProblem(null);
    if (!projectId || !renderId) return;
    let live = true;
    void api
      .resolveMedia(projectId, renderId)
      .then((media) => {
        const file = media.files.find((file) => file.mediaType === 'video/mp4');
        if (live && file)
          setAudition({ projectId, renderId, url: api.mediaUrl(projectId, renderId, file.path) });
        else if (live)
          setAuditionProblem(
            'The rendered artifact contains no playable MP4. Open the delivered file to review it.',
          );
      })
      .catch(() => {
        if (live)
          setAuditionProblem(
            'The rendered preview could not be loaded. Open the delivered MP4 to review it.',
          );
      });
    return () => {
      live = false;
    };
  }, [api, projectId, renderId]);

  // The named document's plan: its duration decides whether the rights gate
  // applies, and its opening words are the title's default. A different clip
  // is a different answer to both, and a stale plan or a queued id from the
  // last one must not be shown under this one's name.
  useEffect(() => {
    selectionGeneration.current += 1;
    setBusy(false);
    setAttestation('');
    setRightsApproval(null);
    setCaptionApproval(null);
    setDurationTicks(0);
    setTitle('');
    setPlan(null);
    setQueued(null);
    setArchive(null);
    setError(null);
    if (!projectId || !docId) {
      return undefined;
    }
    let live = true;
    void (async () => {
      try {
        const preview = await api.previewPlan(projectId, docId);
        if (!live) {
          return;
        }
        const seconds = (preview.frameCount * preview.rateDen) / preview.rateNum;
        setDurationTicks(Math.round(seconds * 90_000));
        setTitle(firstWords(preview));
      } catch (cause) {
        if (live) {
          setError(cause instanceof Error ? cause.message : String(cause));
        }
      }
    })();
    // The document's export, if it has one, is the daemon's to remember:
    // an export queued before this screen was left — or before the
    // application was relaunched — is still running or already delivered,
    // and it is followed from where it is rather than forgotten.
    void (async () => {
      try {
        const jobs = await api.listJobs(projectId);
        const found = latestExportOf(jobs, docId);
        if (live && found !== null) {
          setQueued((current) => current ?? found);
        }
      } catch {
        // No jobs to read is no export to restore; the screen is what it
        // was without one.
      }
    })();
    return () => {
      live = false;
    };
  }, [api, projectId, docId]);

  const rightsGateNeeded = durationTicks / 90_000 > RIGHTS_GATE_SECONDS;

  const request = useMemo<ExportRequest | null>(() => {
    if (docId === null) {
      return null;
    }
    return {
      docId,
      destinationDir: destination,
      namingPattern: effectivePattern(pattern),
      sourceAttestation: attestation,
      gatesPassed: [
        ...(gatePassed ? [DURATION_GATE] : []),
        ...(hotCaptionsConfirmed ? [READING_RATE_GATE] : []),
      ],
      aiAssistance: [...AI_ASSISTANCE],
      index: 1,
      date: today(),
      title,
    };
  }, [docId, destination, pattern, gatePassed, hotCaptionsConfirmed, title, attestation]);

  // The captions the strip named as too fast to read. Once confirmed they
  // come back as advisories under the same code, so the confirmation stays
  // on screen with its count rather than vanishing the moment it is given.
  const hotCaptions = useMemo(
    () => (plan?.findings ?? []).filter((finding) => finding.code === HOT_CAPTION_CODE),
    [plan],
  );

  useEffect(() => {
    if (request === null || destination.trim() === '') {
      setPlan(null);
      return undefined;
    }
    if (timer.current !== null) {
      clearTimeout(timer.current);
    }
    setPlanning(true);
    let live = true;
    timer.current = setTimeout(() => {
      void (async () => {
        try {
          const answer = await api.planExport(request);
          if (live) {
            setPlan(answer);
            setError(null);
          }
        } catch (cause) {
          if (live) {
            setPlan(null);
            setError(cause instanceof Error ? cause.message : String(cause));
          }
        } finally {
          if (live) {
            setPlanning(false);
          }
        }
      })();
    }, PLAN_DEBOUNCE_MS);
    return () => {
      live = false;
      if (timer.current !== null) {
        clearTimeout(timer.current);
      }
    };
  }, [api, request, destination, replan]);

  const onChooseFolder = useCallback(async () => {
    try {
      const chosen = await api.chooseExportFolder();
      if (chosen !== null) {
        setDestination(chosen);
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }, [api]);

  const onExport = useCallback(async () => {
    if (request === null || plan === null || !attestation) {
      return;
    }
    const generation = selectionGeneration.current;
    setBusy(true);
    setError(null);
    try {
      // The revision the plan checked is the revision that may leave.
      const exported = await api.exportClip({ ...request, expectedRevision: plan.revision });
      if (selectionGeneration.current === generation) setQueued(exported);
    } catch (cause) {
      if (selectionGeneration.current !== generation) return;
      const message = cause instanceof Error ? cause.message : String(cause);
      setError(message);
      if (message.includes('moved since it was reviewed')) {
        // The document is not what was reviewed. Plan again over what it is
        // now, so the findings and the names on screen are of that, and let
        // the person look before asking again.
        setRightsApproval(null);
        setCaptionApproval(null);
        setPlan(null);
        setReplan((count) => count + 1);
      }
    } finally {
      if (selectionGeneration.current === generation) setBusy(false);
    }
  }, [api, request, plan, attestation]);

  const onReveal = useCallback(
    async (path: string) => {
      try {
        await api.revealPath(path);
      } catch (cause) {
        setError(cause instanceof Error ? cause.message : String(cause));
      }
    },
    [api],
  );

  const onArchive = useCallback(async () => {
    if (projectId === null || destination.trim() === '') {
      setError('An archive needs a folder to go in.');
      return;
    }
    const generation = selectionGeneration.current;
    setBusy(true);
    setError(null);
    try {
      const written = await api.exportArchive(projectId, destination);
      if (selectionGeneration.current === generation)
        setArchive({ path: written.path, entryCount: written.entryCount });
    } catch (cause) {
      if (selectionGeneration.current === generation)
        setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      if (selectionGeneration.current === generation) setBusy(false);
    }
  }, [api, projectId, destination]);

  if (batchMode)
    return (
      <BatchExportScreen
        api={api}
        onBack={() => setBatchMode(false)}
        {...(onEdit ? { onEdit } : {})}
      />
    );
  return (
    <Export
      publishing={
        projectId && docId ? (
          <UploadPanel
            key={`${projectId}:${docId}`}
            api={api}
            projectId={projectId}
            docId={docId}
            exportJobId={queued?.jobId ?? null}
            revision={queued?.revision ?? null}
            renderArtifactId={delivery?.renderArtifactId ?? null}
            irArtifactId={queued?.irArtifactId ?? null}
            currentRevision={plan?.revision ?? null}
            delivered={Boolean(delivery?.settled && delivery.files)}
            {...(onOpenChannelSettings ? { onSetup: onOpenChannelSettings } : {})}
          />
        ) : null
      }
      onBatch={() => setBatchMode(true)}
      onEdit={clip && onEdit ? () => onEdit(clip) : undefined}
      docId={docId}
      labels={clip?.labels ?? null}
      picker={clip === null ? <ClipList onOpen={onOpen} api={api} /> : null}
      destination={destination}
      pattern={pattern}
      title={title}
      attestation={attestation}
      onAttestationChange={(value) => {
        setAttestation(value);
        setRightsApproval(null);
      }}
      audition={
        audition?.projectId === projectId && audition.renderId === renderId ? audition.url : null
      }
      auditionProblem={auditionProblem}
      onAuditionError={() =>
        setAuditionProblem(
          'The rendered file could not be played here. Open the delivered MP4 to review it.',
        )
      }
      rightsGateNeeded={rightsGateNeeded}
      rightsGatePassed={gatePassed}
      hotCaptions={hotCaptions}
      hotCaptionsConfirmed={hotCaptionsConfirmed}
      onHotCaptionsChange={(checked) => setCaptionApproval(checked ? approvalKey : null)}
      plan={plan}
      planning={planning}
      busy={busy}
      error={error}
      delivery={delivery}
      archive={archive}
      onDestinationChange={setDestination}
      onPatternChange={setPattern}
      onChooseFolder={() => void onChooseFolder()}
      onRightsGateChange={(checked) => setRightsApproval(checked ? approvalKey : null)}
      onExport={() => void onExport()}
      onArchive={() => void onArchive()}
      onReveal={(path) => void onReveal(path)}
    />
  );
}

/** The list, mounted only when there is no clip so it fetches only then. */
function ClipList({
  onOpen,
  api,
}: {
  readonly onOpen: (clip: ClipRef) => void;
  readonly api: ShellApi;
}) {
  const documents = useEditDocuments(api);
  return <DocumentPicker documents={documents} verb="Export" onOpen={onOpen} />;
}

/**
 * The clip's opening words, for the `{clip}` token.
 *
 * Taken from the caption plan rather than invented: a clip's title, when it has
 * one, is what it opens by saying.
 */
function firstWords(plan: {
  readonly cues: readonly { readonly lines: readonly (readonly { readonly text: string }[])[] }[];
}): string {
  const words = plan.cues
    .flatMap((cue) => cue.lines.flat())
    .slice(0, 6)
    .map((word) => word.text)
    .filter((text) => text.trim() !== '');
  return words.join(' ');
}

/** Today, as `YYYY-MM-DD`. The daemon reads no clock, so this side must. */
function today(): string {
  const now = new Date();
  const month = String(now.getMonth() + 1).padStart(2, '0');
  const day = String(now.getDate()).padStart(2, '0');
  return `${now.getFullYear()}-${month}-${day}`;
}
