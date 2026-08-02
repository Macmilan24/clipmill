/**
 * The Results board's and the Inspector's state, in one place.
 *
 * Both screens read the same snapshot — the Inspector is the board with one row
 * opened — so they share a hook rather than each fetching. That also makes the
 * decision path obvious: deciding writes to the daemon and then reloads, so what
 * a screen shows is always what the store holds rather than what the screen
 * hoped it wrote.
 */
import { useCallback, useEffect, useMemo, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import type { ClipDecision, CropPath } from '../daemon/client.js';
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
  readonly decide: (candidateId: string, decision: ClipDecision) => Promise<void>;
  readonly solveFor: (candidateId: string) => void;
  /** Build the edit document from a named cut, without changing the decision. */
  readonly direct: (candidateId: string, cut: 'chosen' | 'alternative') => Promise<void>;
}

export function useResults(
  projectId: string | null,
  sourceId: string | null,
  api: ShellApi = daemonApi,
): ResultsState {
  const loader = useMemo(() => new ResultsLoader(api), [api]);
  const [snapshot, setSnapshot] = useState<ResultsSnapshot>(EMPTY_SNAPSHOT);
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [crop, setCrop] = useState<CropPath | null>(null);
  const [cues, setCues] = useState<readonly OverlayCue[]>([]);

  const reload = useCallback(() => {
    if (!projectId) {
      return;
    }
    setLoading(true);
    void loader
      .load(projectId, sourceId)
      .then(setSnapshot)
      .finally(() => setLoading(false));
  }, [loader, projectId, sourceId]);

  useEffect(reload, [reload]);

  const proxyUrl = useMemo(() => {
    if (!projectId || !snapshot.proxyArtifactId) {
      return null;
    }
    return api.mediaUrl(projectId, snapshot.proxyArtifactId, PROXY_FILE);
  }, [api, projectId, snapshot.proxyArtifactId]);

  /**
   * Ask where the camera should point over a clip.
   *
   * A proposal, so this is safe to call whenever the selection moves. A refusal
   * is not an error here: a fitted frame with a reason is a legitimate answer
   * and the preview says so.
   */
  const solveFor = useCallback(
    (candidateId: string) => {
      const row = snapshot.rows.find((candidate) => candidate.candidateId === candidateId);
      const faceTrack = snapshot.faceTrackArtifactId;
      if (!projectId || !row || !faceTrack) {
        setCrop(null);
        return;
      }
      void api
        .solveCropPath(projectId, faceTrack, row.startTicks, row.endTicks)
        .then(setCrop)
        .catch(() => setCrop(null));
    },
    [api, projectId, snapshot],
  );

  const direct = useCallback(
    async (candidateId: string, cut: 'chosen' | 'alternative') => {
      if (!projectId || !snapshot.source) {
        return;
      }
      setBusy(true);
      setNotice(null);
      try {
        const directed = await api.directClip({
          projectId,
          sourceId: snapshot.source.sourceId,
          candidateId,
          cut,
        });
        setCues(overlayCuesFromEdit(directed.documentJson, directed.startTicks));
        setNotice(
          directed.decisions.length > 0 ? directed.decisions.join(' ') : 'Sent to the editor.',
        );
      } catch (error) {
        setNotice((error as Error).message);
      } finally {
        setBusy(false);
      }
    },
    [api, projectId, snapshot.source],
  );

  const decide = useCallback(
    async (candidateId: string, decision: ClipDecision) => {
      if (!projectId || !snapshot.source) {
        return;
      }
      setBusy(true);
      setNotice(null);
      try {
        await api.setClipDecision(projectId, snapshot.source.sourceId, candidateId, decision);
        if (decision === 'approved') {
          // Approving is what creates the edit document. The other two record
          // an opinion and nothing else, which is why only this one directs.
          const directed = await api.directClip({
            projectId,
            sourceId: snapshot.source.sourceId,
            candidateId,
            cut: 'chosen',
          });
          // The overlay is the burned-in grouping of the document that was just
          // created, so what the preview draws is what the encoder will draw.
          setCues(overlayCuesFromEdit(directed.documentJson, directed.startTicks));
          setNotice(
            directed.decisions.length > 0 ? directed.decisions.join(' ') : 'Sent to the editor.',
          );
        } else {
          setNotice(decision === 'kept' ? 'Kept for later.' : 'Rejected.');
        }
        reload();
      } catch (error) {
        setNotice((error as Error).message);
      } finally {
        setBusy(false);
      }
    },
    [api, projectId, reload, snapshot.source],
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
    solveFor,
    direct,
  };
}
