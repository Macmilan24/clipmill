/**
 * The studio tour's steps: where everything is, from a long recording to a
 * posted clip. Each points at a real control where one is on screen for a
 * new installation; for the screens that fill up only once there is a
 * project (Results, review, the Editor, Export), it points at the rail and
 * shows a picture of what will be there. It navigates, and starts nothing.
 */
import type { ReactNode } from 'react';

export type TourPlacement = 'right' | 'left' | 'bottom' | 'top' | 'center';

/** A drawing of a screen a new installation has nothing on yet. */
export type TourPicture = 'welcome' | 'results' | 'review' | 'editor' | 'export';

export interface TourStep {
  readonly id: string;
  /** Where on the way it is, as number and name: "02 · Prepare". */
  readonly chapter: string;
  /** The place it is about: "Models". */
  readonly place: string;
  readonly title: string;
  readonly body: ReactNode;
  /** The section to show first, when the step is about one. */
  readonly section?: string;
  /** `data-tour` values to point at; the first on screen wins. None: the card sits in the middle. */
  readonly targets?: readonly string[];
  readonly placement?: TourPlacement;
  readonly picture?: TourPicture;
}

export const TOUR_STEPS: readonly TourStep[] = [
  {
    id: 'welcome',
    chapter: '01 · Start',
    place: 'Welcome',
    title: 'A quick tour of the studio',
    body: (
      <p>
        Two minutes through where everything is, from a long recording to a posted clip. The tour
        only looks: it downloads, analyses and uploads nothing.
      </p>
    ),
    section: 'library',
    placement: 'center',
    picture: 'welcome',
  },
  {
    id: 'rail',
    chapter: '01 · Start',
    place: 'The rail',
    title: 'Everything is one step away',
    body: (
      <p>
        The rail follows the work. <strong>Library</strong> keeps your projects and{' '}
        <strong>New Project</strong> starts one; <strong>Results</strong>, <strong>Editor</strong>{' '}
        and <strong>Export</strong> take a clip from found to finished. <strong>Models</strong> and{' '}
        <strong>Settings</strong> hold the setup.
      </p>
    ),
    section: 'library',
    targets: ['rail'],
    placement: 'right',
  },
  {
    id: 'status',
    chapter: '01 · Start',
    place: 'Top bar',
    title: 'What is running, and whether the engine is ready',
    body: (
      <p>
        <strong>Engine ready</strong> means the local engine that does the work is up.{' '}
        <strong>Activity</strong> lists analyses, exports and downloads while they run, from any
        screen.
      </p>
    ),
    targets: ['topbar-status'],
    placement: 'bottom',
  },
  {
    id: 'models',
    chapter: '02 · Prepare',
    place: 'Models',
    title: 'Download what analysis needs',
    body: (
      <p>
        The first analysis needs a few models, and one download gets the set recommended for this
        computer. <strong>Editorial AI</strong>, the model that reads like an editor, is checked
        here before its first run, and Models says how fast it is on this computer.
      </p>
    ),
    section: 'models',
    targets: ['models-setup', 'models', 'models-title'],
    placement: 'bottom',
  },
  {
    id: 'source',
    chapter: '03 · Bring',
    place: 'New Project',
    title: 'Start from a recording',
    body: (
      <p>
        Drop a video file here, or paste a YouTube link. The original recording is never changed.
      </p>
    ),
    section: 'new-project',
    targets: ['new-project-source'],
    placement: 'right',
  },
  {
    id: 'preferences',
    chapter: '03 · Bring',
    place: 'New Project',
    title: 'Say what you are looking for',
    body: (
      <p>
        The kind of footage, the caption look, how long the clips should be, how many to find and
        the language. Any clip can still change later, in the Editor.
      </p>
    ),
    section: 'new-project',
    targets: ['new-project-preferences'],
    placement: 'left',
  },
  {
    id: 'start',
    chapter: '03 · Bring',
    place: 'New Project',
    title: 'Then analyse',
    body: (
      <p>
        Confirm you may use the footage, and start. ClipMill says how long the run should take on
        this computer, and it carries on while you do other things.
      </p>
    ),
    section: 'new-project',
    targets: ['new-project-start'],
    placement: 'left',
  },
  {
    id: 'results',
    chapter: '04 · Find',
    place: 'Results',
    title: 'The moments worth a look',
    body: (
      <p>
        Each clip comes with why it was chosen, framed the way it will be posted. Move over a card
        to scrub through it, and filter to the ones still to decide.
      </p>
    ),
    targets: ['nav-results'],
    placement: 'right',
    picture: 'results',
  },
  {
    id: 'review',
    chapter: '04 · Find',
    place: 'Clip review',
    title: 'Judge each clip as it will be built',
    body: (
      <p>
        Play it vertical, read its words, hear past its edges and move its start and end. Then
        approve it, keep it for later or reject it, and the next one opens.
      </p>
    ),
    targets: ['nav-results'],
    placement: 'right',
    picture: 'review',
  },
  {
    id: 'editor',
    chapter: '05 · Shape',
    place: 'Editor',
    title: 'Edit by the words',
    body: (
      <p>
        Cut or correct words in the transcript, restyle the captions and reframe the picture. Every
        change can be undone, even after a restart.
      </p>
    ),
    targets: ['nav-editor'],
    placement: 'right',
    picture: 'editor',
  },
  {
    id: 'export',
    chapter: '06 · Deliver',
    place: 'Export',
    title: 'Export, or upload privately',
    body: (
      <p>
        A 9:16 video and its subtitles go to the folder you choose. With your channel connected in
        Settings, a clip can go to YouTube as a private upload first.
      </p>
    ),
    targets: ['nav-export'],
    placement: 'right',
    picture: 'export',
  },
  {
    id: 'local-lock',
    chapter: '07 · Control',
    place: 'Local Lock',
    title: 'See when the network is used',
    body: (
      <p>
        Local Lock stays on while nothing in this session has used the network. Model downloads,
        YouTube and the update check each count, and Settings shows the detail.
      </p>
    ),
    targets: ['local-lock'],
    placement: 'right',
  },
  {
    id: 'finish',
    chapter: '08 · Begin',
    place: 'Your turn',
    title: 'Your first clip starts here',
    body: (
      <p>
        Pick a recording and let ClipMill find the moments. The tour is in Settings, under Getting
        started, whenever you want it again.
      </p>
    ),
    placement: 'center',
  },
];
