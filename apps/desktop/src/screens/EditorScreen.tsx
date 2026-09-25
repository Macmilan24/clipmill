/**
 * Open the route's edit document, or show a document picker when none is named.
 * `useEditor` owns document state and resolves face tracks from the clip's run
 * for re-solving.
 */
import { useCallback, useEffect, useRef, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import { batch, setLayout, solvedKeyframe } from '../editor/commands.js';
import { DocumentPicker } from '../editor/DocumentPicker.js';
import { useEditDocuments } from '../editor/documents.js';
import { segmentAt, sourceOf } from '../editor/player.js';
import { useEditor } from '../editor/useEditor.js';
import { historySteps } from '../editor/history.js';
import type { ClipRef, EditorFocus } from '../shell/route.js';
import { Editor } from './Editor.js';

export interface EditorScreenProps {
  /** The clip to open, or null when the row was reached with none named. */
  readonly clip: ClipRef | null;
  /** What to land on inside the clip, when the opener said. */
  readonly focus?: EditorFocus | null;
  readonly onOpenResults: () => void;
  /** Open a different clip here — from the list this screen offers. */
  readonly onOpen: (clip: ClipRef) => void;
  /** Take the open clip to the export screen. */
  readonly onExport: (clip: ClipRef) => void;
  readonly api?: ShellApi;
}

export function EditorScreen({
  clip,
  focus = null,
  onOpenResults,
  onOpen,
  onExport,
  api = daemonApi,
}: EditorScreenProps) {
  const [sourceRefresh, setSourceRefresh] = useState(0);
  const [relinking, setRelinking] = useState(false);
  const [relinkProblem, setRelinkProblem] = useState<string | null>(null);
  const editor = useEditor(clip, api, sourceRefresh);
  const [resolving, setResolving] = useState(false);
  const [resolveProblem, setResolveProblem] = useState<string | null>(null);
  const resolveVersion = useRef(0);
  useEffect(() => {
    resolveVersion.current += 1;
    setResolveProblem(null);
    setResolving(false);
  }, [clip?.docId, editor.plan?.revision]);

  // Whether the recording is still where it was registered. The editor runs
  // from preview copies either way; only an export needs the original, so a
  // moved file is said once, with the way to fix it, rather than discovered
  // when an export fails.
  const [sourceMissing, setSourceMissing] = useState(false);
  const sourceId = clip?.sourceId ?? null;
  useEffect(() => {
    setSourceMissing(false);
    if (!sourceId) return undefined;
    let live = true;
    // A shell without source details answers nothing, rather than stopping
    // the screen: this is information, not a precondition.
    Promise.resolve()
      .then(() => api.getSource(sourceId))
      .then((details) => {
        if (live) setSourceMissing(details.source.missing === true);
      })
      .catch(() => undefined);
    return () => {
      live = false;
    };
  }, [api, sourceId, sourceRefresh]);

  const onRelink = useCallback(async () => {
    if (!clip || relinking) return;
    setRelinking(true);
    setRelinkProblem(null);
    try {
      const chosen = await api.chooseSourceFile();
      if (chosen === null) return;
      await api.relinkSource(clip.projectId, clip.sourceId, chosen);
      setSourceRefresh((current) => current + 1);
    } catch (error) {
      setRelinkProblem(error instanceof Error ? error.message : String(error));
    } finally {
      setRelinking(false);
    }
  }, [api, clip, relinking]);

  /**
   * Why the solver cannot be asked, or `null` when it can.
   *
   * A returned-early callback behind an enabled button is a control that lies:
   * the click does nothing and the screen says nothing about why. The reason
   * lives here because this is the only layer that knows it, and it reaches the
   * interface rather than staying a comment (R25).
   */
  const resolveRefusal = editor.faceTrack
    ? null
    : 'This analysis has no face evidence. Reanalyze the recording to add automatic framing.';

  /**
   * Ask the solver again and write what it says as one undoable step.
   *
   * The solve itself writes nothing — it is a proposal — so turning it into
   * keyframes is the editor's decision and is recorded as such.
   */
  const onResolve = useCallback(
    async (frame: number) => {
      const plan = editor.plan;
      const faceTrack = editor.faceTrack;
      // The span the solver is asked about is the segment's own window into the
      // source — the face tracks are in source time — and the answer comes back
      // in shares of the source frame, which the plan names. Asked from zero to
      // the clip's duration and converted against the output's dimensions, as
      // this was, the path followed faces from the recording's opening.
      const segment = plan ? segmentAt(plan, frame) : null;
      const source = plan && segment ? sourceOf(plan, segment) : null;
      if (!faceTrack || !plan || !segment || !source) {
        return;
      }
      const version = ++resolveVersion.current;
      setResolving(true);
      setResolveProblem(null);
      try {
        const solved = await api.solveCropPath(
          faceTrack.projectId,
          faceTrack.artifactId,
          segment.inTicks,
          segment.outTicks,
        );
        if (version !== resolveVersion.current) return;
        if (solved.fit || solved.keyframes.length === 0) {
          await editor.apply(setLayout('fit', segment.segmentId));
          return;
        }
        const aspect = { width: plan.width, height: plan.height };
        await editor.apply(
          batch([
            setLayout('speaker_fill', segment.segmentId),
            {
              op: 'replace_crop_path',
              segment_id: segment.segmentId,
              path: solved.keyframes.map((keyframe) => {
                const converted = solvedKeyframe(keyframe, segment, source, aspect);
                return { t_ticks: converted.tTicks, rect: converted.rect };
              }),
            },
          ]),
        );
      } catch (error) {
        if (version !== resolveVersion.current) return;
        setResolveProblem(
          error instanceof Error ? error.message : 'Framing could not be recalculated. Try again.',
        );
      } finally {
        if (version === resolveVersion.current) setResolving(false);
      }
    },
    [api, editor],
  );

  return (
    <Editor
      plan={editor.plan}
      document={editor.document}
      transcript={editor.transcript}
      filmstrip={editor.filmstrip}
      peaks={editor.peaks}
      filmstripUrl={(file) =>
        editor.filmstrip && clip
          ? api.mediaUrl(clip.projectId, editor.filmstrip.artifactId, file)
          : ''
      }
      proxyUrls={editor.proxyUrls}
      docId={editor.docId}
      labels={clip?.labels ?? null}
      focus={focus}
      loading={editor.loading}
      problem={relinkProblem ?? editor.problem ?? resolveProblem}
      busy={editor.busy}
      canUndo={editor.canUndo}
      canRedo={editor.canRedo}
      resolveRefusal={resolveRefusal}
      resolving={resolving}
      picker={clip === null ? <ClipList onOpen={onOpen} api={api} /> : null}
      onOpenResults={onOpenResults}
      onExport={clip === null ? null : () => onExport(clip)}
      onRelink={clip === null || !sourceMissing ? null : () => void onRelink()}
      relinking={relinking}
      onApply={(command) => {
        void editor.apply(command);
      }}
      onUndo={() => {
        void editor.undo();
      }}
      onRedo={() => {
        void editor.redo();
      }}
      onResolve={(frame) => {
        void onResolve(frame);
      }}
      onLoadHistory={
        api.listEditHistory && clip
          ? async () => historySteps(await api.listEditHistory!(clip.docId))
          : null
      }
      fontUrl={api.captionFontUrl ?? null}
      previewCaptions={
        api.previewCaptions && clip
          ? async (draft) => {
              const current = editor.document?.captions;
              try {
                const preview = await api.previewCaptions!(
                  clip.docId,
                  draft.styleRef ?? '',
                  JSON.stringify(draft.options ?? current?.options ?? {}),
                );
                return preview.ass;
              } catch {
                // A look that cannot be drawn yet is simply not drawn; the
                // saved captions stay on the preview.
                return null;
              }
            }
          : null
      }
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
  return <DocumentPicker documents={documents} verb="Edit" onOpen={onOpen} />;
}
