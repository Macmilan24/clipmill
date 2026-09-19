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
 * The proxy comes with the plan: the daemon names, for every source the
 * document draws from, the proxy it is previewed on and how its clock relates
 * to the source's. The face tracks come from the clip's own run — a job says
 * which recording it ran over, so the run is the one the route named or the
 * newest over that source — and never from whichever pass the project ran
 * last.
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
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import { newest } from '../daemon/ordering.js';
import type { EditCommandJson, Job, PreviewPlan } from '../daemon/client.js';
import { publishedArtifact } from '../library/model.js';
import type { ClipRef } from '../shell/route.js';

const PROXY_KIND = 'media.proxy.v1';
const FACES_KIND = 'vision.face_track.v1';

/** Tauri rejects with strings; browser adapters may reject with Error objects. */
function failureMessage(error: unknown, fallback: string): string {
  const message = error instanceof Error ? error.message : typeof error === 'string' ? error : '';
  return message.trim() || fallback;
}

/** Where a clip's media comes from: an artifact, and the project it is in. */
export interface ArtifactRef {
  readonly projectId: string;
  readonly artifactId: string;
}

export interface EditorState {
  readonly docId: string | null;
  readonly revision: number;
  readonly plan: PreviewPlan | null;
  /** The proxy for each source the plan names, by fingerprint, as a URL. */
  readonly proxyUrls: ReadonlyMap<string, string>;
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
  const [faceTrack, setFaceTrack] = useState<ArtifactRef | null>(null);
  const [loading, setLoading] = useState(clip !== null);
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const [undoStack, setUndoStack] = useState<readonly EditCommandJson[]>([]);
  const [redoStack, setRedoStack] = useState<readonly EditCommandJson[]>([]);
  /**
   * Which document this hook is on, as a number that changes when it does.
   *
   * A mutation is in flight when the person opens another clip. Its answer —
   * a new revision, a plan, an inverse for the undo stack — is about the
   * document that was open when it was sent, and applied to the one that is
   * open when it lands it would show B with A's footage and undo A's edit on
   * B. So every mutation captures the session it started in and, after each
   * await, touches nothing unless that session is still current.
   */
  const session = useRef(0);
  /**
   * Which plan request is the current one, within a session.
   *
   * Every apply re-fetches the plan, and two applies in flight can answer out
   * of order. A plan that arrives after a newer request was made describes a
   * revision the document has already left, and drawing it would show an edit
   * being undone that nobody undid. So each request takes a number, and an
   * answer is kept only if it is still the latest.
   */
  const latest = useRef(0);

  const projectId = clip?.projectId ?? null;
  const docId = clip?.docId ?? null;
  const sourceId = clip?.sourceId ?? null;
  const jobId = clip?.jobId ?? null;

  useEffect(() => {
    // A different document is a different history. The stacks belonged to the
    // last one, and an undo replayed onto this one would be an edit nobody
    // made here.
    session.current += 1;
    setUndoStack([]);
    setRedoStack([]);
    setPlan(null);
    setFaceTrack(null);
    setProblem(null);
    setBusy(false);
    if (!projectId || !docId || !sourceId) {
      setLoading(false);
      return undefined;
    }
    let live = true;
    setLoading(true);
    latest.current += 1;
    const request = latest.current;
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
        const faces = publishedArtifact(run, FACES_KIND);
        if (live && request === latest.current) {
          setRevision(fetched.revision);
          setPlan(fetched);
          setFaceTrack(faces ? { projectId, artifactId: faces } : null);
          setLoading(false);
        }
      } catch (error) {
        if (live && request === latest.current) {
          setProblem(failureMessage(error, 'The clip could not be opened. Try opening it again.'));
          setLoading(false);
        }
      }
    })();
    return () => {
      live = false;
    };
  }, [api, projectId, docId, sourceId, jobId]);

  /**
   * Send a command, take the inverse, and re-read the plan.
   *
   * Null when nothing should follow: the command was refused, or the document
   * it was sent to is no longer the one open, in which case its answer is
   * dropped whole — plan, revision, and the inverse that would have gone on
   * another document's undo stack.
   */
  const send = useCallback(
    async (command: EditCommandJson): Promise<EditCommandJson | null> => {
      if (!docId || !projectId) {
        return null;
      }
      const mine = session.current;
      const current = () => mine === session.current;
      setBusy(true);
      setProblem(null);
      try {
        const applied = await api.applyEditCommand(docId, revision, command);
        if (!current()) {
          return null;
        }
        latest.current += 1;
        const request = latest.current;
        const refreshed = await api.previewPlan(projectId, docId);
        if (!current()) {
          return null;
        }
        if (request === latest.current) {
          // The plan is bound to the revision it describes. One that describes
          // an older revision than the apply reached is a stale answer, and
          // the one describing the newer revision is on its way.
          if (refreshed.revision >= applied.revision) {
            setRevision(refreshed.revision);
            setPlan(refreshed);
          } else {
            setRevision(applied.revision);
          }
        }
        return JSON.parse(applied.inverseCommandJson) as EditCommandJson;
      } catch (error) {
        // A conflict means somebody else moved the document. Reporting it is
        // the honest answer; silently rebasing would lose an edit nobody
        // decided to discard.
        if (current()) {
          setProblem(failureMessage(error, 'The edit could not be saved. Try again.'));
        }
        return null;
      } finally {
        if (current()) {
          setBusy(false);
        }
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

  // Built from the plan rather than looked up: the daemon already matched
  // each source to its proxy, and a URL is the one thing it cannot know.
  const proxyUrls = useMemo(() => {
    const urls = new Map<string, string>();
    if (projectId && plan) {
      for (const proxy of plan.proxies) {
        urls.set(proxy.sourceFingerprint, api.mediaUrl(projectId, proxy.artifactId, proxy.file));
      }
    }
    return urls;
  }, [api, plan, projectId]);

  return {
    docId: plan ? docId : null,
    revision,
    plan,
    proxyUrls,
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
