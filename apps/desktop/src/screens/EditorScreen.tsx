/**
 * The editor's container: hold the document, hand the player its plan.
 *
 * The clip arrives named in full from the route — project, document, source,
 * and the run it came out of — and this opens exactly that. Nothing here looks
 * for "the newest" anything. With no clip named, the screen lists every edit
 * there is and lets a person choose, which is what a sidebar row has to do
 * when it cannot know which clip is meant.
 *
 * The state lives in `useEditor` so the screen stays a view. What this adds is
 * re-solving, which needs the face tracks the clip's own run published — the
 * hook resolves those beside the proxy, from the same job, for the same reason
 * the proxy is: the media for a clip is the media of the run that produced it.
 */
import { useCallback, useEffect, useRef, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import { batch, setLayout, solvedKeyframe } from '../editor/commands.js';
import { DocumentPicker } from '../editor/DocumentPicker.js';
import { useEditDocuments } from '../editor/documents.js';
import { segmentAt, sourceOf } from '../editor/player.js';
import { useEditor } from '../editor/useEditor.js';
import type { ClipRef } from '../shell/route.js';
import { Editor } from './Editor.js';

export interface EditorScreenProps {
  /** The clip to open, or null when the row was reached with none named. */
  readonly clip: ClipRef | null;
  readonly onOpenResults: () => void;
  /** Open a different clip here — from the list this screen offers. */
  readonly onOpen: (clip: ClipRef) => void;
  /** Take the open clip to the export screen. */
  readonly onExport: (clip: ClipRef) => void;
  readonly api?: ShellApi;
}

export function EditorScreen({
  clip,
  onOpenResults,
  onOpen,
  onExport,
  api = daemonApi,
}: EditorScreenProps) {
  const editor = useEditor(clip, api);
  const [resolving, setResolving] = useState(false);
  const [resolveProblem, setResolveProblem] = useState<string | null>(null);
  const resolveVersion = useRef(0);
  useEffect(() => {
    resolveVersion.current += 1;
    setResolveProblem(null);
    setResolving(false);
  }, [clip?.docId, editor.plan?.revision]);

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
      proxyUrls={editor.proxyUrls}
      docId={editor.docId}
      labels={clip?.labels ?? null}
      loading={editor.loading}
      problem={editor.problem ?? resolveProblem}
      busy={editor.busy}
      canUndo={editor.canUndo}
      canRedo={editor.canRedo}
      resolveRefusal={resolveRefusal}
      resolving={resolving}
      picker={clip === null ? <ClipList onOpen={onOpen} api={api} /> : null}
      onOpenResults={onOpenResults}
      onExport={clip === null ? null : () => onExport(clip)}
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
