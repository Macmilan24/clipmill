import type { Step } from 'react-joyride';

/** The chapter also drives the tooltip's visual progress and accessible label. */
export type TourChapter =
  | 'welcome'
  | 'workspace'
  | 'models'
  | 'local-import'
  | 'youtube-import'
  | 'preferences'
  | 'readiness'
  | 'analysis'
  | 'results'
  | 'review'
  | 'editor'
  | 'export'
  | 'publishing'
  | 'privacy'
  | 'finish';

export type TourStep = Step & {
  readonly before: NonNullable<Step['before']>;
  readonly data: {
    readonly phase: string;
    readonly chapter: TourChapter;
    readonly visual?: 'model-actions';
  };
};

/**
 * An orientation, never a simulated project. The later stages use a permanent
 * shell anchor because a fresh install has no analysis, clip, or export to show.
 * `before` lets the runner select the real screen before resolving its target.
 */
export function createTourSteps(navigate: (sectionId: string) => void): TourStep[] {
  const compact = window.innerWidth < 1050;
  const removableModel = () =>
    document.querySelector<HTMLElement>('[data-tour="models-removable-row"]') ??
    document.querySelector<HTMLElement>('[data-tour="models-heading"]');
  const visit =
    (
      sectionId: string,
      target?: string | (() => HTMLElement | null),
      block: ScrollLogicalPosition = 'center',
    ) =>
    async () => {
      navigate(sectionId);
      const main = document.querySelector<HTMLElement>('.studio-main');
      if (main) main.scrollTop = 0;
      if (!target) return;

      // React may need a frame to mount the next screen, and Models is lazy.
      // Position the real target before Joyride measures its tooltip.
      const deadline = performance.now() + 3500;
      await new Promise<void>((resolve) => {
        const seek = () => {
          const element =
            typeof target === 'string' ? document.querySelector<HTMLElement>(target) : target();
          if (element && element.getClientRects().length > 0) {
            element.scrollIntoView({ block, behavior: 'auto' });
            resolve();
          } else if (performance.now() >= deadline) {
            resolve();
          } else {
            requestAnimationFrame(seek);
          }
        };
        requestAnimationFrame(seek);
      });
    };

  return [
    {
      target: '[data-tour="tour-stage"]',
      placement: 'center',
      skipBeacon: true,
      title: 'Make a clip worth sharing',
      content: (
        <div className="tour-copy">
          <p>
            ClipMill takes a long recording from source to finished short video. You choose the
            footage, review the suggested moments, shape the edit, and approve the final file.
          </p>
          <div className="tour-mini-flow" aria-label="ClipMill workflow">
            <span>Import</span>
            <span>Analyze</span>
            <span>Review</span>
            <span>Edit</span>
            <span>Deliver</span>
          </div>
          <p className="tour-mini-note">
            This walkthrough only explains the studio. It will not start a download, analysis,
            upload, or channel connection.
          </p>
        </div>
      ),
      data: { phase: '01 · Start', chapter: 'welcome' },
      before: visit('library'),
    },
    {
      target: '[data-tour="app-sidebar"]',
      placement: 'right',
      skipBeacon: true,
      title: 'Your work has a home',
      content: (
        <div className="tour-copy">
          <p>
            Library keeps your projects and analysis runs. The rail follows the workflow through New
            Project, Results, Editor, and Export. Models and Settings hold setup and controls.
          </p>
          <p className="tour-mini-note">
            On a new install, the later pages are empty until you make a project. Use the Library’s
            <strong> Import video</strong> action to begin.
          </p>
        </div>
      ),
      data: { phase: '01 · Start', chapter: 'workspace' },
      before: visit('library'),
    },
    {
      target: '[data-tour="models-heading"]',
      placement: 'bottom',
      skipBeacon: true,
      title: 'Get the models ready',
      content: (
        <div className="tour-copy">
          <p>
            Models &amp; Device shows your computer’s memory and the models for each analysis job.
            If models are missing, <strong>Download all</strong> gets the recommended set; each
            model row also has its own download action.
          </p>
          <p className="tour-mini-note">
            You can cancel a download, then resume what arrived or try again after a failure.
            Downloads contact Hugging Face and verify pinned files; your project footage is not
            sent. An installed model still needs a connected worker to run.
          </p>
        </div>
      ),
      data: { phase: '02 · Prepare', chapter: 'models' },
      before: visit('models', '[data-tour="models-heading"]'),
    },
    {
      target: '[data-tour="models-heading"]',
      placement: 'bottom',
      skipBeacon: true,
      title: 'Choose what each job uses',
      content: (
        <div className="tour-copy">
          <p>
            The <strong>In use</strong> badge marks the model selected for that job’s next analysis.
            For an installed alternative, choose <strong>Use this one</strong>;
            <strong> Use automatic</strong> returns selection to ClipMill.{' '}
            <strong>Check files</strong>
            verifies an installed model against its pinned checksums.
          </p>
          <p className="tour-mini-note">
            <strong>Add from Hugging Face</strong> lets you inspect a compatible job, repository,
            and revision before pinning a custom model. Check memory and worker warnings: an
            installed alternative may still be unable to run here.
          </p>
        </div>
      ),
      data: { phase: '02 · Prepare', chapter: 'models' },
      before: visit('models', '[data-tour="models-heading"]'),
    },
    {
      target: removableModel,
      placement: 'center',
      skipBeacon: true,
      title: 'Remove a model when you need the space',
      content: (
        <div className="tour-copy">
          <p>
            Find the model in its job row and choose <strong>Remove</strong>. A confirmation opens
            beside it; <strong>Keep it</strong> cancels. Confirming deletes the downloaded files. A
            model added by you also leaves the library; a built-in model stays listed for another
            download.
          </p>
          <ul className="tour-points">
            <li>
              If it was selected, ClipMill uses another installed model when one is available.
            </li>
            <li>
              Cancel its download, wait for a file check to finish, and finish or stop any analysis
              using it before removal.
            </li>
          </ul>
        </div>
      ),
      data: { phase: '02 · Prepare', chapter: 'models', visual: 'model-actions' },
      before: visit('models', removableModel),
    },
    {
      target: '[data-tour="new-project-source"]',
      placement: compact ? 'center' : 'right',
      skipBeacon: true,
      title: 'Start with a recording',
      content: (
        <div className="tour-copy">
          <p>
            On <strong>New Project</strong>, drop a recording or choose{' '}
            <strong>Browse files</strong>. ClipMill accepts common video formats and leaves your
            original local file untouched.
          </p>
          <p className="tour-mini-note">
            The source comes first. Analysis starts only when you confirm the settings on this page.
          </p>
        </div>
      ),
      data: { phase: '03 · Import', chapter: 'local-import' },
      before: visit('new-project', '[data-tour="new-project-source"]'),
    },
    {
      target: '[data-tour="new-project-source-tabs"]',
      placement: 'bottom',
      skipBeacon: true,
      title: 'Or bring in one YouTube video',
      content: (
        <div className="tour-copy">
          <p>
            Choose <strong>YouTube</strong> to paste one public video link, select a maximum
            quality, and confirm permission to download and use it. ClipMill saves a local copy; a
            failed download has <strong>Retry download</strong>.
          </p>
          <p className="tour-mini-note">
            Playlists, private videos, and sign-in are unsupported. Importing is separate from
            channel connection, cloud AI, and analysis.
          </p>
        </div>
      ),
      data: { phase: '03 · Import', chapter: 'youtube-import' },
      before: visit('new-project', '[data-tour="new-project-source-tabs"]'),
    },
    {
      target: '[data-tour="new-project-preferences"]',
      placement: compact ? 'center' : 'left',
      skipBeacon: true,
      title: 'Tell ClipMill what to look for',
      content: (
        <div className="tour-copy">
          <p>
            Choose <strong>Podcast / interview</strong> or <strong>TV / movie scene</strong>, then
            set clip length, how many clips to find, language, and the first caption look. Each clip
            can be changed later in the Editor.
          </p>
          <p className="tour-mini-note">
            Local Qwen is the default editorial route. Cloud-assisted reasoning needs explicit
            consent and a per-run budget; it shares transcript context, not video or audio. The
            heuristic baseline uses no editorial model.
          </p>
        </div>
      ),
      data: { phase: '03 · Import', chapter: 'preferences' },
      before: visit('new-project', '[data-tour="new-project-preferences"]'),
    },
    {
      target: '[data-tour="new-project-start"]',
      placement: compact ? 'center' : 'left',
      skipBeacon: true,
      title: 'Check readiness, then start',
      content: (
        <div className="tour-copy">
          <p>
            <strong>Engine status</strong> explains missing models, workers, or the decoder and
            gives a recovery path. Confirm that you have the rights to clip and publish the source,
            then choose <strong>Analyze video</strong>.
          </p>
          <p className="tour-mini-note">
            Missing models block the run. A missing worker can leave a stage waiting until it
            connects.
          </p>
        </div>
      ),
      data: { phase: '03 · Import', chapter: 'readiness' },
      before: visit('new-project', '[data-tour="new-project-start"]'),
    },
    {
      target: '[data-tour="tour-stage"]',
      placement: 'center',
      skipBeacon: true,
      title: 'Follow the analysis',
      content: (
        <div className="tour-copy">
          <p>
            When a run starts, its progress page shows every pipeline stage and what it is waiting
            for. You can return to Library and reopen that run. A finished run unlocks
            <strong> View results</strong>.
          </p>
          <p className="tour-mini-note">
            If a run fails or is stopped, its progress page explains the state and can offer
            <strong> Analyze again locally</strong>. Your saved edits remain separate.
          </p>
        </div>
      ),
      data: { phase: '04 · Discover', chapter: 'analysis' },
      before: visit('new-project'),
    },
    {
      target: '[data-tour="tour-stage"]',
      placement: 'center',
      skipBeacon: true,
      title: 'Results are suggestions',
      content: (
        <div className="tour-copy">
          <p>
            When you have results, filter <strong>Recommended</strong>, <strong>Flagged</strong>, or{' '}
            <strong>Needs review</strong>. Open a candidate to inspect its context, reasons, and
            warnings before approving it.
          </p>
          <p className="tour-mini-note">
            No candidate is a promise that it works as a clip. Declined moments stay inspectable; if
            no clip is ready, you can choose a source interval and make a manual edit.
          </p>
        </div>
      ),
      data: { phase: '04 · Discover', chapter: 'results' },
      before: visit('results'),
    },
    {
      target: '[data-tour="tour-stage"]',
      placement: 'center',
      skipBeacon: true,
      title: 'Make the editorial decision',
      content: (
        <div className="tour-copy">
          <p>
            In the Inspector, play the suggested cut and surrounding source. Use
            <strong> Transcript</strong>, <strong>Why</strong>, and <strong>Details</strong> to
            verify the claim and context. Adjust the cut edges before choosing
            <strong> Reject</strong>, <strong>Keep for later</strong>, or
            <strong> Approve &amp; edit</strong>.
          </p>
          <p className="tour-mini-note">
            You may edit a model-declined moment anyway; it remains marked as declined.
          </p>
        </div>
      ),
      data: { phase: '04 · Discover', chapter: 'review' },
      before: visit('results'),
    },
    {
      target: '[data-tour="tour-stage"]',
      placement: 'center',
      skipBeacon: true,
      title: 'Shape the clip in the Editor',
      content: (
        <div className="tour-copy">
          <p>
            When a clip is open, correct words and line breaks in the transcript, watch the vertical
            preview, and use <strong>Captions</strong>, <strong>Framing</strong>, and
            <strong> Audio</strong> properties. The timeline controls trims, splits, caption timing,
            framing keyframes, and volume points.
          </p>
          <p className="tour-mini-note">
            Edits save as revisions. Undo and Redo are available while you edit; check the saved
            state before leaving.
          </p>
        </div>
      ),
      data: { phase: '05 · Create', chapter: 'editor' },
      before: visit('editor'),
    },
    {
      target: '[data-tour="tour-stage"]',
      placement: 'center',
      skipBeacon: true,
      title: 'Export the version you reviewed',
      content: (
        <div className="tour-copy">
          <p>
            When an edit is ready, choose a local folder and name, review source rights and any
            caption or duration findings, then select <strong>Export clip</strong>. ClipMill renders
            a vertical MP4 with burned captions plus SRT and VTT sidecars.
          </p>
          <p className="tour-mini-note">
            Watch <strong>The exported clip</strong>: it is the encoded file with final framing,
            captions, and mastered audio. You can also <strong>Export a collection</strong> of saved
            edits.
          </p>
        </div>
      ),
      data: { phase: '06 · Deliver', chapter: 'export' },
      before: visit('export'),
    },
    {
      target: '[data-tour="tour-stage"]',
      placement: 'center',
      skipBeacon: true,
      title: 'YouTube delivery stays in your hands',
      content: (
        <div className="tour-copy">
          <p>
            Where channel delivery is available, connect your own channel in Settings using a Google
            Desktop OAuth client. After export, review the rendered video, title, description,
            audience, and synthetic-media disclosure before <strong>Upload privately</strong>.
          </p>
          <p className="tour-mini-note">
            Review the private upload in YouTube Studio. <strong>Publish publicly</strong> is a
            second explicit action; Google may restrict public publishing for your API project.
            Local file export remains available without a connection.
          </p>
        </div>
      ),
      data: { phase: '06 · Deliver', chapter: 'publishing' },
      before: visit('export'),
    },
    {
      target: '[data-tour="settings-privacy"]',
      placement: 'top',
      skipBeacon: true,
      title: 'Know when the network is used',
      content: (
        <div className="tour-copy">
          <p>
            Settings contains appearance, storage, privacy, and available channel connections. Local
            analysis is the default. YouTube import, model downloads, optional cloud reasoning, and
            channel delivery each have their own explicit action.
          </p>
          <p className="tour-mini-note">
            Local Lock reports network operations started in this engine session. It is an activity
            indicator, not an operating-system firewall.
          </p>
        </div>
      ),
      data: { phase: '07 · Control', chapter: 'privacy' },
      before: visit('settings', '[data-tour="settings-privacy"]', 'end'),
    },
    {
      target: '[data-tour="tour-stage"]',
      placement: 'center',
      skipBeacon: true,
      title: 'Your first clip starts here',
      content: (
        <div className="tour-copy">
          <p>
            You have the map. Pick a recording from Library’s <strong>Import video</strong>, make
            your choices, and let ClipMill find candidates. Your judgment carries the clip through
            review, editing, and delivery.
          </p>
          <div className="tour-mini-flow" aria-label="Your next steps">
            <span>Choose footage</span>
            <span>Review the moment</span>
            <span>Export the result</span>
          </div>
        </div>
      ),
      data: { phase: '08 · Begin', chapter: 'finish' },
      before: visit('library'),
    },
  ];
}
