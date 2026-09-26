/**
 * Shared Results and Inspector state.
 * Decisions are written to the daemon, applied to the row at once, then re-read
 * quietly so neither view blanks while the store confirms them.
 */
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import type {
  ClipCut,
  ClipDecision,
  CropPath,
  DirectClipInput,
  DirectedClip,
  PreviewPlan,
} from '../daemon/client.js';
import { kitRequest } from '../editor/brand.js';
import { highlightFor, lookFor, optionsFor } from './captionLook.js';
import { EMPTY_SNAPSHOT, type ResultsSnapshot, ResultsLoader } from './loader.js';
import { TICKS_PER_SECOND } from './model.js';
import type { Transcript } from './transcript.js';

/** The proxy file the media protocol serves for a proxy artifact. */
const PROXY_FILE = 'proxy.mp4';

/**
 * How far either side of a clip the camera path is solved.
 *
 * The Inspector plays past a clip's edges so a reviewer can hear what comes
 * before and after, and a path that stopped at the edge would freeze the frame
 * exactly where they are deciding whether to extend it.
 */
const SOLVE_MARGIN_TICKS = 30 * TICKS_PER_SECOND;

/** A window of the recording, in source ticks. */
export interface Window {
  readonly startTicks: number;
  readonly endTicks: number;
}

/** The recording's words, and whether they could be read. */
export type TranscriptState =
  | { readonly status: 'idle' | 'loading' | 'missing'; readonly transcript: null }
  | { readonly status: 'ready'; readonly transcript: Transcript };

export interface ResultsState {
  readonly loading: boolean;
  readonly snapshot: ResultsSnapshot;
  readonly proxyUrl: string | null;
  readonly crop: CropPath | null;
  readonly busy: boolean;
  readonly notice: string | null;
  /** Say something in the notice line — for an action the screen took itself. */
  readonly say: (notice: string | null) => void;
  readonly reload: () => void;
  /**
   * Keep, reject, or take a decision back with `null`.
   *
   * Approving is `approve`, because approving is the one decision that builds
   * something. Answers whether the decision was recorded.
   */
  readonly decide: (
    candidateId: string,
    decision: Exclude<ClipDecision, 'approved'> | null,
  ) => Promise<boolean>;
  /**
   * Approve a clip with the cut on screen, and answer with its edit.
   *
   * `null` is the cut the search chose. A clip with an edit gets that edit
   * back — unless the cut on screen differs from the search's, which asks for
   * a second edit beside the first, because a different cut is a different
   * edit and the first one is somebody's work.
   */
  readonly approve: (candidateId: string, window: Window | null) => Promise<DirectedClip | null>;
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
   * Where each clip's camera would point, as a share across the frame, by
   * candidate. Empty until asked and answered; a clip with no entry sits in
   * the middle.
   */
  readonly framing: ReadonlyMap<string, number>;
  /**
   * The clip an approval of the cut on screen would build, as a preview plan,
   * for the clip it was asked for. Null until it arrives, or on a shell that
   * cannot build one — the solver's crop stands in then.
   */
  readonly preview: {
    readonly candidateId: string;
    readonly key: string;
    readonly plan: PreviewPlan;
  } | null;
  /** Ask for that preview for this clip and this cut; `null` is the search's cut. */
  readonly previewFor: (candidateId: string, window: Window | null) => void;
  /** Build an edit from a span nobody proposed, without deciding anything. */
  readonly manual: (startTicks: number, endTicks: number) => Promise<DirectedClip | null>;
  /** The recording's words, read when the Inspector first asks for them. */
  readonly transcript: TranscriptState;
  readonly requestTranscript: () => void;
}

const same = (left: Window, right: Window) =>
  left.startTicks === right.startTicks && left.endTicks === right.endTicks;

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
  // The clip an approval would build, for the cut on screen.
  const [preview, setPreview] = useState<{
    readonly candidateId: string;
    readonly key: string;
    readonly plan: PreviewPlan;
  } | null>(null);
  const previewSequence = useRef(0);
  const previewKey = useRef<string | null>(null);
  // Where each clip's camera would point, for thumbnails framed as the clip.
  const [framing, setFraming] = useState<ReadonlyMap<string, number>>(() => new Map());
  const [transcript, setTranscript] = useState<TranscriptState>({
    status: 'idle',
    transcript: null,
  });

  const loadSequence = useRef(0);
  const cropSequence = useRef(0);
  const contextSequence = useRef(0);
  const solved = useRef<string | null>(null);
  const transcriptFor = useRef<string | null>(null);

  const load = useCallback(
    (quietly: boolean) => {
      const sequence = ++loadSequence.current;
      if (!projectId) {
        setSnapshot(EMPTY_SNAPSHOT);
        setLoading(false);
        return;
      }
      if (!quietly) setLoading(true);
      void loader
        .load(projectId, sourceId, jobId)
        .then((next) => {
          if (sequence === loadSequence.current) setSnapshot(next);
        })
        .catch((cause: unknown) => {
          // A quiet re-read that fails keeps what is on screen: the decision
          // it was checking on is already durable, and blanking the board over
          // a failed refresh would lose more than it reports.
          if (sequence === loadSequence.current && !quietly)
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
    },
    [loader, projectId, sourceId, jobId],
  );
  const reload = useCallback(() => load(false), [load]);
  const refresh = useCallback(() => load(true), [load]);

  useEffect(() => {
    setSnapshot(EMPTY_SNAPSHOT);
    setCrop(null);
    setNotice(null);
    setBusy(false);
    setTranscript({ status: 'idle', transcript: null });
    solved.current = null;
    transcriptFor.current = null;
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
   * Ask where the camera should point over a clip and the context around it.
   *
   * A proposal, so this is safe to call whenever the selection moves. Asked
   * again only when the clip or the face tracks changed — a quiet refresh after
   * a decision must not blank the picture the reviewer is looking at. A refusal
   * is not an error here: a fitted frame with a reason is a legitimate answer
   * and the preview says so.
   */
  const solveFor = useCallback(
    (candidateId: string) => {
      const row = snapshot.rows.find((candidate) => candidate.candidateId === candidateId);
      const faceTrack = snapshot.faceTrackArtifactId;
      const key = row && faceTrack ? `${candidateId}:${faceTrack}:${row.startTicks}` : null;
      if (key !== null && key === solved.current) return;
      solved.current = key;
      const sequence = ++cropSequence.current;
      setCrop(null);
      if (!projectId || !row || !faceTrack) {
        return;
      }
      const limit = snapshot.sourceDurationTicks ?? Number.MAX_SAFE_INTEGER;
      void api
        .solveCropPath(
          projectId,
          faceTrack,
          Math.max(0, row.startTicks - SOLVE_MARGIN_TICKS),
          Math.min(limit, row.endTicks + SOLVE_MARGIN_TICKS),
        )
        .then((next) => {
          if (sequence === cropSequence.current) setCrop(next);
        })
        .catch(() => {
          if (sequence === cropSequence.current) setCrop(null);
        });
    },
    [api, projectId, snapshot],
  );

  const requestTranscript = useCallback(() => {
    const wanted = snapshot.transcriptArtifactId ?? null;
    if (!projectId || !snapshot.source) return;
    if (wanted === transcriptFor.current) return;
    transcriptFor.current = wanted;
    if (!wanted) {
      setTranscript({ status: 'missing', transcript: null });
      return;
    }
    const context = contextSequence.current;
    setTranscript({ status: 'loading', transcript: null });
    void loader.loadTranscript(projectId, snapshot).then((read) => {
      if (context !== contextSequence.current || transcriptFor.current !== wanted) return;
      setTranscript(
        read ? { status: 'ready', transcript: read } : { status: 'missing', transcript: null },
      );
    });
  }, [loader, projectId, snapshot]);

  /** Put a decision on its row at once; the quiet refresh confirms it. */
  const mark = useCallback(
    (candidateId: string, decision: ClipDecision | null, directed: DirectedClip | null) => {
      setSnapshot((current) => ({
        ...current,
        rows: current.rows.map((row) =>
          row.candidateId === candidateId
            ? {
                ...row,
                decision,
                ...(directed ? { docId: directed.docId, docJobId: directed.jobId || null } : {}),
              }
            : row,
        ),
      }));
    },
    [],
  );

  const decide = useCallback(
    async (
      candidateId: string,
      decision: Exclude<ClipDecision, 'approved'> | null,
    ): Promise<boolean> => {
      if (!projectId || !snapshot.source) {
        return false;
      }
      const context = contextSequence.current;
      setBusy(true);
      setNotice(null);
      try {
        await api.setClipDecision(projectId, snapshot.source.sourceId, candidateId, decision);
        if (context !== contextSequence.current) return false;
        mark(candidateId, decision, null);
        setNotice(
          decision === 'kept'
            ? 'Kept for later.'
            : decision === 'rejected'
              ? 'Rejected.'
              : 'Decision cleared.',
        );
        refresh();
        return true;
      } catch (error) {
        if (context === contextSequence.current) setNotice((error as Error).message);
        return false;
      } finally {
        if (context === contextSequence.current) setBusy(false);
      }
    },
    [api, projectId, refresh, mark, snapshot.source],
  );

  // One question for the whole board: where would each clip's camera point
  // half a second in. A shell or a run without faces keeps the middle.
  const framingFor = useRef<string | null>(null);
  useEffect(() => {
    const faceTrack = snapshot.faceTrackArtifactId;
    const rows = snapshot.rows;
    const key =
      projectId && faceTrack && rows.length > 0 && api.thumbnailFraming
        ? `${faceTrack}:${rows.map((row) => `${row.candidateId}@${row.startTicks}`).join(',')}`
        : null;
    if (key === framingFor.current) return undefined;
    framingFor.current = key;
    if (!key || !projectId || !faceTrack) {
      setFraming(new Map());
      return undefined;
    }
    let live = true;
    Promise.resolve()
      .then(() =>
        api.thumbnailFraming!(
          projectId,
          faceTrack,
          rows.map((row) => row.startTicks + TICKS_PER_SECOND / 2),
        ),
      )
      .then((centres) => {
        if (!live) return;
        setFraming(new Map(rows.map((row, at) => [row.candidateId, centres[at] ?? 0.5])));
      })
      // No framing is the middle of the frame, not a broken board.
      .catch(() => undefined);
    return () => {
      live = false;
    };
  }, [api, projectId, snapshot.faceTrackArtifactId, snapshot.rows]);

  /**
   * The request that builds this clip with this cut: the one approving sends,
   * and the one the Inspector's preview sends, so the clip judged is the clip
   * made — same cut, same look, same run.
   */
  const clipRequest = useCallback(
    (candidateId: string, window: Window | null): DirectClipInput | null => {
      const row = snapshot.rows.find((candidate) => candidate.candidateId === candidateId);
      if (!projectId || !snapshot.source || !row) return null;
      const chosen = { startTicks: row.startTicks, endTicks: row.endTicks };
      const moved = window !== null && !same(window, chosen);
      const alternative = row.boundary?.alternative ?? null;
      // The runner-up is named as itself, so the edit's rationale says whose
      // cut it is; any other moved cut is the reviewer's own.
      const cut: ClipCut = !moved
        ? 'chosen'
        : alternative && same(window, alternative)
          ? 'alternative'
          : 'exact';
      const look = lookFor(projectId);
      const options = optionsFor(projectId);
      return {
        projectId,
        sourceId: snapshot.source.sourceId,
        candidateId,
        cut,
        ...(look ? { styleRef: look } : {}),
        ...(options ? { captionOptionsJson: options } : {}),
        // The saved brand kit, and the shape clips are framed for.
        ...kitRequest(),
        highlightSpokenWord: highlightFor(projectId),
        ...(cut === 'exact' && window
          ? { startTicks: window.startTicks, endTicks: window.endTicks }
          : {}),
        ...(row.review?.status === 'rejected' ? { allowDeclined: true } : {}),
        // The run the board is showing: its candidate, its boundaries, its
        // transcript — not whichever run published each stage last.
        ...(snapshot.run ? { jobId: snapshot.run.jobId } : {}),
      };
    },
    [projectId, snapshot.source, snapshot.run, snapshot.rows],
  );

  /** The request that builds a span nobody proposed: the manual clip's. */
  const manualRequest = useCallback(
    (startTicks: number, endTicks: number): DirectClipInput | null => {
      if (!projectId || !snapshot.source || !snapshot.run) return null;
      const look = lookFor(projectId);
      const options = optionsFor(projectId);
      return {
        projectId,
        sourceId: snapshot.source.sourceId,
        jobId: snapshot.run.jobId,
        candidateId: '',
        cut: 'exact',
        ...(look ? { styleRef: look } : {}),
        ...(options ? { captionOptionsJson: options } : {}),
        // The saved brand kit, and the shape clips are framed for.
        ...kitRequest(),
        highlightSpokenWord: highlightFor(projectId),
        startTicks,
        endTicks,
        manualSpan: true,
      };
    },
    [projectId, snapshot.source, snapshot.run],
  );

  /**
   * Ask for the clip an approval of this cut would build, to draw it — or,
   * for the candidate `''`, the manual clip this span would make. Nothing is
   * written. A newer ask supersedes an older one still in flight, and the
   * same ask twice is asked once.
   */
  const previewFor = useCallback(
    (candidateId: string, window: Window | null) => {
      const request = !api.previewDirect
        ? null
        : candidateId === ''
          ? window
            ? manualRequest(window.startTicks, window.endTicks)
            : null
          : clipRequest(candidateId, window);
      const key = request ? JSON.stringify(request) : null;
      if (key !== null && key === previewKey.current) return;
      previewKey.current = key;
      const sequence = ++previewSequence.current;
      if (!request || !key) {
        setPreview(null);
        return;
      }
      void Promise.resolve()
        .then(() => api.previewDirect!(request))
        .then((plan) => {
          if (sequence === previewSequence.current) setPreview({ candidateId, key, plan });
        })
        // A clip that cannot be previewed is judged from the solver's crop,
        // as before; the approval itself will say what is wrong.
        .catch(() => {
          if (sequence === previewSequence.current) setPreview(null);
        });
    },
    [api, clipRequest, manualRequest],
  );

  const approve = useCallback(
    async (candidateId: string, window: Window | null): Promise<DirectedClip | null> => {
      const row = snapshot.rows.find((candidate) => candidate.candidateId === candidateId);
      const request = clipRequest(candidateId, window);
      if (!projectId || !snapshot.source || !row || !request) {
        return null;
      }
      const chosen = { startTicks: row.startTicks, endTicks: row.endTicks };
      const moved = window !== null && !same(window, chosen);
      const context = contextSequence.current;
      setBusy(true);
      setNotice(null);
      try {
        // Approving is what creates the edit document, and the daemon records
        // the decision in the same write — so there is no moment where the
        // board says approved and the editor has nothing to open.
        const directed = await api.directClip({
          ...request,
          approve: true,
          // A different cut of a clip that already has an edit is a second
          // edit. Without asking for one by name, the daemon would hand back
          // the existing document — with the boundary this was replacing.
          ...(moved && row.docId ? { variation: true } : {}),
        });
        if (context !== contextSequence.current) return null;
        mark(candidateId, 'approved', directed);
        setNotice(
          directed.reopened
            ? 'Approved. Its edit is kept as it stands.'
            : moved && row.docId
              ? 'Approved with your cut as a new edit. The earlier edit is still in Edits.'
              : 'Approved. The edit is ready.',
        );
        refresh();
        return directed;
      } catch (error) {
        if (context === contextSequence.current) setNotice((error as Error).message);
        return null;
      } finally {
        if (context === contextSequence.current) setBusy(false);
      }
    },
    [api, projectId, refresh, mark, snapshot.source, snapshot.rows, clipRequest],
  );

  const manual = useCallback(
    async (startTicks: number, endTicks: number): Promise<DirectedClip | null> => {
      const request = manualRequest(startTicks, endTicks);
      if (!request) return null;
      const context = contextSequence.current;
      setBusy(true);
      setNotice(null);
      try {
        const directed = await api.directClip({ ...request, approve: false });
        if (context !== contextSequence.current) return null;
        setNotice('Sent to the editor.');
        reload();
        return directed;
      } catch (error) {
        if (context === contextSequence.current) setNotice((error as Error).message);
        return null;
      } finally {
        if (context === contextSequence.current) setBusy(false);
      }
    },
    [api, reload, manualRequest],
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
            // The same request one approval sends, look and all: a clip
            // approved in a batch is the clip approved on its own.
            const request = clipRequest(candidateId, null);
            if (!request) continue;
            // eslint-disable-next-line no-await-in-loop -- see above
            await api.directClip({ ...request, approve: true });
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
    [api, projectId, reload, snapshot.source, snapshot.rows, clipRequest],
  );

  return {
    loading,
    snapshot,
    proxyUrl,
    crop,
    busy,
    notice,
    say: setNotice,
    reload,
    decide,
    approve,
    approveMany,
    tileUrl,
    solveFor,
    framing,
    preview,
    previewFor,
    manual,
    transcript,
    requestTranscript,
  };
}
