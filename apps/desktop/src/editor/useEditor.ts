/**
 * The editor's state: a document, a plan, and a history.
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
import { newest, oldestFirstNewest } from '../daemon/ordering.js';
import type { EditCommandJson, PreviewPlan } from '../daemon/client.js';

const PROXY_FILE = 'proxy.mp4';
const PROXY_KIND = 'media.proxy.v1';

export interface EditorState {
  readonly docId: string | null;
  readonly revision: number;
  readonly plan: PreviewPlan | null;
  readonly proxyUrl: string | null;
  readonly loading: boolean;
  readonly busy: boolean;
  readonly problem: string | null;
  readonly canUndo: boolean;
  readonly canRedo: boolean;
  readonly apply: (command: EditCommandJson) => Promise<void>;
  readonly undo: () => Promise<void>;
  readonly redo: () => Promise<void>;
}

export function useEditor(api: ShellApi = daemonApi): EditorState {
  const [projectId, setProjectId] = useState<string | null>(null);
  const [docId, setDocId] = useState<string | null>(null);
  const [revision, setRevision] = useState(0);
  const [plan, setPlan] = useState<PreviewPlan | null>(null);
  const [proxyUrl, setProxyUrl] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const [undoStack, setUndoStack] = useState<readonly EditCommandJson[]>([]);
  const [redoStack, setRedoStack] = useState<readonly EditCommandJson[]>([]);

  useEffect(() => {
    let live = true;
    void (async () => {
      try {
        const projects = await api.listProjects();
        const project = newest(projects);
        if (!project) {
          if (live) {
            setLoading(false);
          }
          return;
        }
        const docs = await api.listEditDocs(project.projectId);
        const latestDoc = oldestFirstNewest(docs);
        if (!latestDoc) {
          if (live) {
            setLoading(false);
          }
          return;
        }
        const [fetched, jobs] = await Promise.all([
          api.previewPlan(project.projectId, latestDoc.docId),
          api.listJobs(project.projectId).catch(() => []),
        ]);
        const proxy = jobs
          .flatMap((job) => job.tasks)
          .find((task) => task.outputKind === PROXY_KIND && task.outputArtifactId !== '');
        if (live) {
          setProjectId(project.projectId);
          setDocId(latestDoc.docId);
          setRevision(fetched.revision);
          setPlan(fetched);
          setProxyUrl(
            proxy ? api.mediaUrl(project.projectId, proxy.outputArtifactId, PROXY_FILE) : null,
          );
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
  }, [api]);

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
    docId,
    revision,
    plan,
    proxyUrl,
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
