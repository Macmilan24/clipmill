/**
 * The Export screen's container: hold what the user typed, and ask the daemon
 * what it means.
 *
 * Every keystroke in the pattern or the destination re-plans, debounced. That
 * is deliberate churn: the alternative is resolving the pattern here, which
 * would be a second implementation of the naming rules and would eventually
 * disagree with the files that get written. A local socket can answer this far
 * faster than a person types.
 */
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import type { ExportPlan, ExportRequest } from '../daemon/client.js';
import { Export } from './Export.js';

/** Long enough that a typed word is one request, short enough to feel live. */
const PLAN_DEBOUNCE_MS = 250;
/** Past this, the rights confirmation applies. The daemon holds the same rule. */
const RIGHTS_GATE_SECONDS = 60;
const DURATION_GATE = 'duration_60s';
/**
 * What Phase 1 attests. One value, because the document is model-assisted and
 * hand-authored in exactly one way, and a picker offering positions nobody can
 * verify would be a picker that manufactures claims.
 */
const ATTESTATION = 'own_content';
/**
 * The model work that shaped every clip this pipeline produces: captions from
 * recognition, and a crop path from a face pass. Declared rather than inferred,
 * because a disclosure a renderer guessed is a disclosure nobody checked.
 */
const AI_ASSISTANCE = ['asr_captions', 'reframe'] as const;

export interface ExportScreenProps {
  readonly api?: ShellApi;
}

export function ExportScreen({ api = daemonApi }: ExportScreenProps) {
  const [projectId, setProjectId] = useState<string | null>(null);
  const [docId, setDocId] = useState<string | null>(null);
  const [durationTicks, setDurationTicks] = useState(0);
  const [title, setTitle] = useState('');
  const [destination, setDestination] = useState('');
  const [pattern, setPattern] = useState('{index}-{clip}');
  const [gatePassed, setGatePassed] = useState(false);
  const [plan, setPlan] = useState<ExportPlan | null>(null);
  const [planning, setPlanning] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [queued, setQueued] = useState<string | null>(null);
  const [archive, setArchive] = useState<{ path: string; entryCount: number } | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  // The newest document of the newest project, which is the clip somebody just
  // approved. Phase 1 has no project picker, and inventing one here would be a
  // control with nothing behind it.
  useEffect(() => {
    let live = true;
    void (async () => {
      try {
        const projects = await api.listProjects();
        const project = projects.at(-1);
        if (!project || !live) {
          return;
        }
        setProjectId(project.projectId);
        const docs = await api.listEditDocs(project.projectId);
        const newest = docs.at(-1);
        if (!newest || !live) {
          return;
        }
        setDocId(newest.docId);
        const preview = await api.previewPlan(project.projectId, newest.docId);
        if (!live) {
          return;
        }
        // Duration decides whether the rights gate applies, and the plan is
        // where a duration can be read without parsing the document.
        const seconds = (preview.frameCount * preview.rateDen) / preview.rateNum;
        setDurationTicks(Math.round(seconds * 90_000));
        setTitle(firstWords(preview));
      } catch (cause) {
        if (live) {
          setError(cause instanceof Error ? cause.message : String(cause));
        }
      }
    })();
    return () => {
      live = false;
    };
  }, [api]);

  const rightsGateNeeded = durationTicks / 90_000 > RIGHTS_GATE_SECONDS;

  const request = useMemo<ExportRequest | null>(() => {
    if (docId === null) {
      return null;
    }
    return {
      docId,
      destinationDir: destination,
      namingPattern: pattern,
      sourceAttestation: ATTESTATION,
      gatesPassed: gatePassed ? [DURATION_GATE] : [],
      aiAssistance: [...AI_ASSISTANCE],
      index: 1,
      date: today(),
      title,
    };
  }, [docId, destination, pattern, gatePassed, title]);

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
  }, [api, request, destination]);

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
    if (request === null) {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      setQueued(await api.exportClip(request));
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }, [api, request]);

  const onArchive = useCallback(async () => {
    if (projectId === null || destination.trim() === '') {
      setError('An archive needs a folder to go in.');
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const written = await api.exportArchive(projectId, destination);
      setArchive({ path: written.path, entryCount: written.entryCount });
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }, [api, projectId, destination]);

  return (
    <Export
      docId={docId}
      destination={destination}
      pattern={pattern}
      title={title}
      attestation={ATTESTATION}
      rightsGateNeeded={rightsGateNeeded}
      rightsGatePassed={gatePassed}
      plan={plan}
      planning={planning}
      busy={busy}
      error={error}
      queued={queued}
      archive={archive}
      onDestinationChange={setDestination}
      onPatternChange={setPattern}
      onChooseFolder={() => void onChooseFolder()}
      onRightsGateChange={setGatePassed}
      onExport={() => void onExport()}
      onArchive={() => void onArchive()}
    />
  );
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
