/**
 * The Results board's and the Inspector's state, in one place.
 *
 * Both screens read the same snapshot — the Inspector is the board with one row
 * opened — so they share a hook rather than each fetching. That also makes the
 * decision path obvious: deciding writes to the daemon and then reloads, so what
 * a screen shows is always what the store holds rather than what the screen
 * hoped it wrote.
 */
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import type { ClipCut, ClipDecision, CropPath, DirectedClip } from '../daemon/client.js';
import type { OverlayCue } from '../inspector/Preview.js';
import { EMPTY_SNAPSHOT, type ResultsSnapshot, ResultsLoader } from './loader.js';
import { overlayCuesFromEdit } from './model.js';

/** The proxy file the media protocol serves for a proxy artifact. */
const PROXY_FILE = 'proxy.mp4';

export interface ResultsState {
  readonly loading: boolean;
  readonly snapshot: ResultsSnapshot;
  readonly proxyUrl: string | null;
  readonly crop: CropPath | null;
  readonly cues: readonly OverlayCue[];
  readonly busy: boolean;
  readonly notice: string | null;
  readonly reload: () => void;
  /**
   * Record a decision.
   *
   * Approving is the one that directs: it answers with the document, which is
   * the one the clip already had if it had one. Keeping and rejecting answer
   * with nothing, because they create nothing.
   */
  readonly decide: (candidateId: string, decision: ClipDecision) => Promise<DirectedClip | null>;
  /**
   * Approve several at once, each through the same path a single approval takes.
   *
   * One reload at the end rather than one per clip: the board would otherwise
   * redraw after each, and the decisions are already durable after each write.
   */
  readonly approveMany: (candidateIds: readonly string[]) => Promise<void>;
  /**
   * The filmstrip frame nearest a moment, as a URL the media protocol serves.
   *
   * Null when the run published no filmstrip. Never a placeholder: a still that
   * is not from this recording is a picture of something else.
   */
  readonly tileUrl: (atTicks: number) => string | null;
  readonly solveFor: (candidateId: string) => void;
  /**
   * Build the edit document from a named cut, without changing the decision.
   *
   * An `exact` cut carries the window it wants. The daemon snaps that to the
   * lattice and answers with where it actually landed, which is why the notice
   * quotes the director rather than echoing what was asked for.
   */
  readonly manual: (startTicks: number, endTicks: number) => Promise<DirectedClip | null>;
  readonly direct: (
    candidateId: string,
    cut: ClipCut,
    window?: { readonly startTicks: number; readonly endTicks: number },
  ) => Promise<DirectedClip | null>;
}

export function useResults(
  projectId: string | null,
  sourceId: string | null,
  jobId: string | null = null,
  api: ShellApi = daemonApi,
): ResultsState {
  const loader = useMemo(() => new ResultsLoader(api), [api]);
  const [snapshot, setSnapshot] = useState<ResultsSnapshot>(EMPTY_SNAPSHOT);
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [crop, setCrop] = useState<CropPath | null>(null);
  const [cues, setCues] = useState<readonly OverlayCue[]>([]);

  const loadSequence = useRef(0);
  const cropSequence = useRef(0);
  const contextSequence = useRef(0);
  const reload = useCallback(() => {
    const sequence = ++loadSequence.current;
    if (!projectId) {
      setSnapshot(EMPTY_SNAPSHOT);
      setLoading(false);
      return;
    }
    setLoading(true);
    void loader
      .load(projectId, sourceId, jobId)
      .then((next) => {
        if (sequence === loadSequence.current) setSnapshot(next);
      })
      .catch((cause: unknown) => {
        if (sequence === loadSequence.current)
          setSnapshot({
            ...EMPTY_SNAPSHOT,
            problem: {
              kind: 'unreadable',
              detail: cause instanceof Error ? cause.message : String(cause),
            },
          });
      })
      .finally(() => {
        if (sequence === loadSequence.current) setLoading(false);
      });
  }, [loader, projectId, sourceId, jobId]);

  useEffect(() => {
    setSnapshot(EMPTY_SNAPSHOT);
    setCrop(null);
    setCues([]);
    setNotice(null);
    setBusy(false);
    reload();
    return () => {
      loadSequence.current++;
      cropSequence.current++;
      contextSequence.current++;
    };
  }, [reload]);

  const proxyUrl = useMemo(() => {
    if (!projectId || !snapshot.proxyArtifactId) {
      return null;
    }
    return api.mediaUrl(projectId, snapshot.proxyArtifactId, PROXY_FILE);
  }, [api, projectId, snapshot.proxyArtifactId]);

  const tileUrl = useCallback(
    (atTicks: number): string | null => {
      const strip = snapshot.filmstrip;
      if (!projectId || !strip || strip.tiles.length === 0) {
        return null;
      }
      let nearest = strip.tiles[0]!;
      for (const tile of strip.tiles) {
        if (Math.abs(tile.tTicks - atTicks) < Math.abs(nearest.tTicks - atTicks)) {
          nearest = tile;
        }
      }
      return api.mediaUrl(projectId, strip.artifactId, nearest.file);
    },
    [api, projectId, snapshot.filmstrip],
  );

  /**
   * Ask where the camera should point over a clip.
   *
   * A proposal, so this is safe to call whenever the selection moves. A refusal
   * is not an error here: a fitted frame with a reason is a legitimate answer
   * and the preview says so.
   */
  const solveFor = useCallback(
    (candidateId: string) => {
      const sequence = ++cropSequence.current;
      setCrop(null);
      setCues([]);
      const row = snapshot.rows.find((candidate) => candidate.candidateId === candidateId);
      const faceTrack = snapshot.faceTrackArtifactId;
      if (!projectId || !row || !faceTrack) {
        setCrop(null);
        return;
      }
      void api
        .solveCropPath(projectId, faceTrack, row.startTicks, row.endTicks)
        .then((next) => {
          if (sequence === cropSequence.current) setCrop(next);
        })
        .catch(() => {
          if (sequence === cropSequence.current) setCrop(null);
        });
    },
    [api, projectId, snapshot],
  );

  /** What a directed reply means on screen: the overlay, and a sentence. */
  const took = useCallback((directed: DirectedClip) => {
    // The overlay is the burned-in grouping of the document that came back —
    // reopened or built — so what the preview draws is what the encoder will.
    setCues(overlayCuesFromEdit(directed.documentJson, directed.startTicks));
    setNotice(
      directed.reopened
        ? 'This clip already has an edit; it was reopened as it stands.'
        : directed.decisions.length > 0
          ? directed.decisions.join(' ')
          : 'Sent to the editor.',
    );
  }, []);

  const direct = useCallback(
    async (
      candidateId: string,
      cut: ClipCut,
      window?: { readonly startTicks: number; readonly endTicks: number },
    ): Promise<DirectedClip | null> => {
      if (!projectId || !snapshot.source) {
        return null;
      }
      const context = contextSequence.current;
      setBusy(true);
      setNotice(null);
      try {
        const directed = await api.directClip({
          projectId,
          sourceId: snapshot.source.sourceId,
          candidateId,
          cut,
          ...(snapshot.rows.some(
            (row) => row.candidateId === candidateId && row.review?.status === 'rejected',
          )
            ? { allowDeclined: true }
            : {}),
          ...(window ? { startTicks: window.startTicks, endTicks: window.endTicks } : {}),
          // A different cut is a different edit, asked for on purpose. Without
          // this the daemon would hand back the existing document — with the
          // boundary this call was trying to replace.
          variation: true,
          ...(snapshot.run ? { jobId: snapshot.run.jobId } : {}),
        });
        if (context !== contextSequence.current) return null;
        took(directed);
        reload();
        return directed;
      } catch (error) {
        if (context === contextSequence.current) setNotice((error as Error).message);
        return null;
      } finally {
        if (context === contextSequence.current) setBusy(false);
      }
    },
    [api, projectId, reload, snapshot.source, snapshot.run, snapshot.rows, took],
  );

  const decide = useCallback(
    async (candidateId: string, decision: ClipDecision): Promise<DirectedClip | null> => {
      if (!projectId || !snapshot.source) {
        return null;
      }
      const context = contextSequence.current;
      setBusy(true);
      setNotice(null);
      try {
        let directed: DirectedClip | null = null;
        if (decision === 'approved') {
          // Approving is what creates the edit document, and the daemon
          // records the decision in the same write — so there is no moment
          // where the board says approved and the editor has nothing to
          // open. A clip that already has an edit gets it back, not a second.
          directed = await api.directClip({
            projectId,
            sourceId: snapshot.source.sourceId,
            candidateId,
            cut: 'chosen',
            approve: true,
            ...(snapshot.rows.some(
              (row) => row.candidateId === candidateId && row.review?.status === 'rejected',
            )
              ? { allowDeclined: true }
              : {}),
            // The run the board is showing: its candidate, its boundaries,
            // its transcript — not whichever run published each stage last.
            ...(snapshot.run ? { jobId: snapshot.run.jobId } : {}),
          });
          if (context !== contextSequence.current) return null;
          took(directed);
        } else {
          await api.setClipDecision(projectId, snapshot.source.sourceId, candidateId, decision);
          if (context !== contextSequence.current) return null;
          setNotice(decision === 'kept' ? 'Kept for later.' : 'Rejected.');
        }
        reload();
        return directed;
      } catch (error) {
        if (context === contextSequence.current) setNotice((error as Error).message);
        return null;
      } finally {
        if (context === contextSequence.current) setBusy(false);
      }
    },
    [api, projectId, reload, snapshot.source, snapshot.run, snapshot.rows, took],
  );

  const manual = useCallback(
    async (startTicks: number, endTicks: number): Promise<DirectedClip | null> => {
      if (!projectId || !snapshot.source || !snapshot.run) return null;
      const context = contextSequence.current;
      setBusy(true);
      setNotice(null);
      try {
        const directed = await api.directClip({
          projectId,
          sourceId: snapshot.source.sourceId,
          jobId: snapshot.run.jobId,
          candidateId: '',
          cut: 'exact',
          startTicks,
          endTicks,
          manualSpan: true,
          approve: false,
        });
        if (context !== contextSequence.current) return null;
        took(directed);
        reload();
        return directed;
      } catch (error) {
        if (context === contextSequence.current) setNotice((error as Error).message);
        return null;
      } finally {
        if (context === contextSequence.current) setBusy(false);
      }
    },
    [api, projectId, reload, snapshot.source, snapshot.run, took],
  );

  const approveMany = useCallback(
    async (candidateIds: readonly string[]) => {
      if (!projectId || !snapshot.source || candidateIds.length === 0) {
        return;
      }
      const context = contextSequence.current;
      setBusy(true);
      setNotice(null);
      const failures: string[] = [];
      try {
        // Sequential on purpose. The daemon is a single writer, so parallel
        // requests would only queue behind one another there — and a failure
        // needs to be attributable to the clip that caused it, which a
        // `Promise.all` cannot say.
        for (const candidateId of candidateIds) {
          try {
            if (
              snapshot.rows.some(
                (row) => row.candidateId === candidateId && row.review?.status === 'rejected',
              )
            ) {
              failures.push('Inspect declined moments individually before choosing to edit.');
              continue;
            }
            // eslint-disable-next-line no-await-in-loop -- see above
            await api.directClip({
              projectId,
              sourceId: snapshot.source.sourceId,
              candidateId,
              cut: 'chosen',
              approve: true,
              ...(snapshot.run ? { jobId: snapshot.run.jobId } : {}),
            });
          } catch (error) {
            failures.push((error as Error).message);
          }
        }
        // The requested batch may complete in the background, but its result
        // belongs to that recording and must not reload a different project.
        if (context !== contextSequence.current) return;
        const done = candidateIds.length - failures.length;
        setNotice(
          failures.length === 0
            ? `Approved ${done} ${done === 1 ? 'clip' : 'clips'} and sent them to the editor.`
            : `Approved ${done} of ${candidateIds.length}; ${failures[0]}`,
        );
        reload();
      } finally {
        if (context === contextSequence.current) setBusy(false);
      }
    },
    [api, projectId, reload, snapshot.source, snapshot.run, snapshot.rows],
  );

  return {
    loading,
    snapshot,
    proxyUrl,
    crop,
    // Empty until a clip is approved, because the burned-in grouping lives in
    // the document approving creates. Showing cues before then would mean
    // showing captions the render has not been asked to draw.
    cues,
    busy,
    notice,
    reload,
    decide,
    approveMany,
    tileUrl,
    solveFor,
    direct,
    manual,
  };
}
