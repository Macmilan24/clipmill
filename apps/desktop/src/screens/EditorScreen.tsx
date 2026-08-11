/**
 * The editor's container: hold the document, hand the player its plan.
 *
 * The state lives in `useEditor` so the screen stays a view. What this adds is
 * the one thing the hook cannot know: re-solving needs a face track, and that
 * comes off the analyze job rather than out of the edit document.
 */
import { useCallback, useEffect, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import { newest } from '../daemon/ordering.js';
import { batch, setCropKeyframe, setLayout } from '../editor/commands.js';
import { useEditor } from '../editor/useEditor.js';
import { Editor } from './Editor.js';

export interface EditorScreenProps {
  readonly onOpenResults: () => void;
  readonly api?: ShellApi;
}

const FACES_KIND = 'vision.face_track.v1';

export function EditorScreen({ onOpenResults, api = daemonApi }: EditorScreenProps) {
  const editor = useEditor(api);
  /**
   * The face track, or `null` for "looked and there is none", or `undefined`
   * for "still looking".
   *
   * Three states rather than two because the interface says different things
   * about each, and collapsing the first two would have the screen assert that
   * a recording carries no faces during the moment before anyone has checked.
   */
  const [faceTrack, setFaceTrack] = useState<
    { projectId: string; artifactId: string } | null | undefined
  >(undefined);
  const [resolving, setResolving] = useState(false);

  useEffect(() => {
    let live = true;
    void (async () => {
      // Every path out of here settles the question, including the failures:
      // an unsettled lookup leaves the button disabled forever with nothing
      // said, which is the state this whole prop exists to stop.
      let found: { projectId: string; artifactId: string } | null = null;
      try {
        const projects = await api.listProjects();
        const project = newest(projects);
        if (project) {
          const jobs = await api.listJobs(project.projectId);
          const pass = jobs
            .flatMap((job) => job.tasks)
            .find((task) => task.outputKind === FACES_KIND && task.outputArtifactId !== '');
          if (pass) {
            found = { projectId: project.projectId, artifactId: pass.outputArtifactId };
          }
        }
      } catch {
        // Nothing to re-solve from is a disabled button, not an error banner.
      }
      if (live) {
        setFaceTrack(found);
      }
    })();
    return () => {
      live = false;
    };
  }, [api]);

  /**
   * Ask the solver again and write what it says as one undoable step.
   *
   * The solve itself writes nothing — it is a proposal — so turning it into
   * keyframes is the editor's decision and is recorded as such.
   */
  /**
   * Why the solver cannot be asked, or `null` when it can.
   *
   * A returned-early callback behind an enabled button is a control that lies:
   * the click does nothing and the screen says nothing about why. The reason
   * lives here because this is the only layer that knows it, and it reaches the
   * interface rather than staying a comment (R25).
   */
  const resolveRefusal =
    faceTrack === undefined
      ? 'Looking for a face track…'
      : faceTrack === null
        ? 'Nothing has detected faces in this recording, so there is no track to follow. ' +
          'An analysis does not schedule a face pass yet, which is why every clip is fitted.'
        : null;

  const onResolve = useCallback(async () => {
    const plan = editor.plan;
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
  }, [api, editor, faceTrack]);

  return (
    <Editor
      plan={editor.plan}
      proxyUrl={editor.proxyUrl}
      docId={editor.docId}
      loading={editor.loading}
      problem={editor.problem}
      busy={editor.busy}
      canUndo={editor.canUndo}
      canRedo={editor.canRedo}
      resolving={resolving}
      resolveRefusal={resolveRefusal}
      onOpenResults={onOpenResults}
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
