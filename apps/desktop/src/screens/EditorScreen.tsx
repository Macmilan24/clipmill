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
import { useCallback, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import { batch, setCropKeyframe, setLayout } from '../editor/commands.js';
import { DocumentPicker } from '../editor/DocumentPicker.js';
import { useEditDocuments } from '../editor/documents.js';
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
    : 'Nothing has detected faces in this recording, so there is no track to follow. ' +
      'An analysis does not schedule a face pass yet, which is why every clip is fitted.';

  /**
   * Ask the solver again and write what it says as one undoable step.
   *
   * The solve itself writes nothing — it is a proposal — so turning it into
   * keyframes is the editor's decision and is recorded as such.
   */
  const onResolve = useCallback(async () => {
    const plan = editor.plan;
    const faceTrack = editor.faceTrack;
    if (!faceTrack || !plan) {
      return;
    }
    setResolving(true);
    try {
      const solved = await api.solveCropPath(
        faceTrack.projectId,
        faceTrack.artifactId,
        0,
        Math.round((plan.frameCount * plan.rateDen * 90_000) / plan.rateNum),
      );
      if (solved.fit || solved.keyframes.length === 0) {
        await editor.apply(setLayout('fit'));
        return;
      }
      await editor.apply(
        batch([
          setLayout('speaker_fill'),
          ...solved.keyframes.map((keyframe) =>
            setCropKeyframe(Number(keyframe.tTicks), {
              // The solver answers in shares of the frame; the document holds
              // pixels, and the output's own dimensions are what they are of.
              x: Math.round(
                (keyframe.centerX - (keyframe.scale * plan.width) / plan.height / 2) * plan.height,
              ),
              y: Math.round((keyframe.centerY - keyframe.scale / 2) * plan.height),
              width: Math.round((keyframe.scale * plan.height * plan.width) / plan.height),
              height: Math.round(keyframe.scale * plan.height),
            }),
          ),
        ]),
      );
    } finally {
      setResolving(false);
    }
  }, [api, editor]);

  return (
    <Editor
      plan={editor.plan}
      proxyUrl={editor.proxyUrl}
      docId={editor.docId}
      labels={clip?.labels ?? null}
      loading={editor.loading}
      problem={editor.problem}
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
      onResolve={() => {
        void onResolve();
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
