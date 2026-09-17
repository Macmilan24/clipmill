/**
 * The editor's state: a document, a plan, and a history.
 *
 * Which document is not a question this hook answers. It is handed the clip —
 * project, document, source, and the run it came out of — and opens exactly
 * that. It used to open the newest document of the newest project, which is
 * right for one project with one approval and wrong the moment a second of
 * either exists: approving a clip in an older project opened another project's
 * edit, and a batch approval left no way to reach any but the last.
 *
 * The proxy and the face tracks come from the clip's own source. A job says
 * which recording it ran over, so the media for this clip is the media of the
 * run that produced it — named by the route when the opener knew it, otherwise
 * the newest run of that source — and never whichever proxy the project
 * published last.
 *
 * Undo is not a special path. Applying a command returns the command that
 * undoes it, so undoing is applying that — which means an undo is logged,
 * survives a restart, and can itself be undone by the inverse it returns. The
 * daemon keeps no stack on purpose; the two stacks here are the renderer's
 * memory of where it has been, not the record of what happened.
 *
 * Every apply re-fetches the plan. It would be cheaper to patch it, and the
 * plan names that as an optimization — but a player showing a patched plan that
 * drifted from the document would be exactly the divergence this workstream
 * exists to prevent, and correctness comes before the SLO.
 */
import { useCallback, useEffect, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import { newest } from '../daemon/ordering.js';
import type { EditCommandJson, Job, PreviewPlan } from '../daemon/client.js';
import { publishedArtifact } from '../library/model.js';
import type { ClipRef } from '../shell/route.js';

const PROXY_FILE = 'proxy.mp4';
const PROXY_KIND = 'media.proxy.v1';
const FACES_KIND = 'vision.face_track.v1';

/** Where a clip's media comes from: an artifact, and the project it is in. */
export interface ArtifactRef {
  readonly projectId: string;
  readonly artifactId: string;
}

export interface EditorState {
  readonly docId: string | null;
  readonly revision: number;
  readonly plan: PreviewPlan | null;
  readonly proxyUrl: string | null;
  /** The face tracks a crop path is re-solved from, when the run published any. */
  readonly faceTrack: ArtifactRef | null;
  readonly loading: boolean;
  readonly busy: boolean;
  readonly problem: string | null;
  readonly canUndo: boolean;
  readonly canRedo: boolean;
  readonly apply: (command: EditCommandJson) => Promise<void>;
  readonly undo: () => Promise<void>;
  readonly redo: () => Promise<void>;
}

/**
 * The run a clip's media is read from.
 *
 * The one the clip names, when it names one. Otherwise the newest job over the
 * clip's source that published a proxy — a job that does not say which source
 * it ran over predates jobs carrying that, and is admitted rather than
 * refused, because the alternative is an editor with no picture.
 */
export function mediaRun(jobs: readonly Job[], clip: ClipRef): Job | null {
  if (clip.jobId) {
    return jobs.find((job) => job.jobId === clip.jobId) ?? null;
  }
  return newest(
    jobs.filter(
      (job) =>
        publishedArtifact(job, PROXY_KIND) !== null &&
        (job.sourceId === '' || job.sourceId === clip.sourceId),
    ),
  );
}

export function useEditor(clip: ClipRef | null, api: ShellApi = daemonApi): EditorState {
  const [revision, setRevision] = useState(0);
  const [plan, setPlan] = useState<PreviewPlan | null>(null);
  const [proxyUrl, setProxyUrl] = useState<string | null>(null);
  const [faceTrack, setFaceTrack] = useState<ArtifactRef | null>(null);
  const [loading, setLoading] = useState(clip !== null);
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const [undoStack, setUndoStack] = useState<readonly EditCommandJson[]>([]);
  const [redoStack, setRedoStack] = useState<readonly EditCommandJson[]>([]);

  const projectId = clip?.projectId ?? null;
  const docId = clip?.docId ?? null;
  const sourceId = clip?.sourceId ?? null;
  const jobId = clip?.jobId ?? null;

  useEffect(() => {
    // A different document is a different history. The stacks belonged to the
    // last one, and an undo replayed onto this one would be an edit nobody
    // made here.
    setUndoStack([]);
    setRedoStack([]);
    setPlan(null);
    setProxyUrl(null);
    setFaceTrack(null);
    setProblem(null);
    if (!projectId || !docId || !sourceId) {
      setLoading(false);
      return undefined;
    }
    let live = true;
    setLoading(true);
    void (async () => {
      try {
        const [fetched, jobs] = await Promise.all([
          api.previewPlan(projectId, docId),
          api.listJobs(projectId).catch(() => []),
        ]);
        const run = mediaRun(jobs, {
          projectId,
          docId,
          sourceId,
          ...(jobId ? { jobId } : {}),
        });
        const proxy = publishedArtifact(run, PROXY_KIND);
        const faces = publishedArtifact(run, FACES_KIND);
        if (live) {
          setRevision(fetched.revision);
          setPlan(fetched);
          setProxyUrl(proxy ? api.mediaUrl(projectId, proxy, PROXY_FILE) : null);
          setFaceTrack(faces ? { projectId, artifactId: faces } : null);
          setLoading(false);
        }
      } catch (error) {
        if (live) {
          setProblem((error as Error).message);
          setLoading(false);
        }
      }
    })();
    return () => {
      live = false;
    };
  }, [api, projectId, docId, sourceId, jobId]);

  /** Send a command, take the inverse, and re-read the plan. */
  const send = useCallback(
    async (command: EditCommandJson): Promise<EditCommandJson | null> => {
      if (!docId || !projectId) {
        return null;
      }
      setBusy(true);
      setProblem(null);
      try {
        const applied = await api.applyEditCommand(docId, revision, command);
        const refreshed = await api.previewPlan(projectId, docId);
        setRevision(applied.revision);
        setPlan(refreshed);
        return JSON.parse(applied.inverseCommandJson) as EditCommandJson;
      } catch (error) {
        // A conflict means somebody else moved the document. Reporting it is
        // the honest answer; silently rebasing would lose an edit nobody
        // decided to discard.
        setProblem((error as Error).message);
        return null;
      } finally {
        setBusy(false);
      }
    },
    [api, docId, projectId, revision],
  );

  const apply = useCallback(
    async (command: EditCommandJson) => {
      const inverse = await send(command);
      if (inverse) {
        setUndoStack((stack) => [...stack, inverse]);
        // A new edit ends the future that redo was holding.
        setRedoStack([]);
      }
    },
    [send],
  );

  const undo = useCallback(async () => {
    const inverse = undoStack.at(-1);
    if (!inverse) {
      return;
    }
    const back = await send(inverse);
    if (back) {
      setUndoStack((stack) => stack.slice(0, -1));
      setRedoStack((stack) => [...stack, back]);
    }
  }, [send, undoStack]);

  const redo = useCallback(async () => {
    const command = redoStack.at(-1);
    if (!command) {
      return;
    }
    const inverse = await send(command);
    if (inverse) {
      setRedoStack((stack) => stack.slice(0, -1));
      setUndoStack((stack) => [...stack, inverse]);
    }
  }, [send, redoStack]);

  return {
    docId: plan ? docId : null,
    revision,
    plan,
    proxyUrl,
    faceTrack,
    loading,
    busy,
    problem,
    canUndo: undoStack.length > 0,
    canRedo: redoStack.length > 0,
    apply,
    undo,
    redo,
  };
}
