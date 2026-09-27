/** Deliberately separate from main.tsx and the production build. No daemon calls. */
import { useEffect, useLayoutEffect, useMemo, useState, type CSSProperties } from 'react';
import {
  DEFAULT_WORKSPACE_THEME,
  ThemeController,
  isWorkspaceTheme,
  type WorkspaceTheme,
} from '@clipmill/tokens';
import { createRoot, type Root } from 'react-dom/client';
import { JobState } from '@clipmill/contracts';
import { SidebarInset, SidebarProvider } from '../src/components/ui/sidebar.js';
import { TooltipProvider } from '../src/components/ui/tooltip.js';
import { AppSidebar } from '../src/shell/Sidebar.js';
import { TopBar } from '../src/shell/TopBar.js';
import { Results } from '../src/screens/Results.js';
import { ClipInspector } from '../src/screens/ClipInspector.js';
import { ManualClip } from '../src/results/ManualClip.js';
import type { ClipRow } from '../src/results/model.js';
import { Editor } from '../src/screens/Editor.js';
import { Export } from '../src/screens/Export.js';
import { BatchExportScreen } from '../src/screens/BatchExportScreen.js';
import { batchApi } from './batch-fixtures.js';
import { publishingFixture } from './publishing-fixtures.js';
import { ConnectionCard } from '../src/youtube/ConnectionCard.js';
import { UploadPanel } from '../src/youtube/UploadPanel.js';
import { UploadHistory } from '../src/youtube/UploadHistory.js';
import { Library } from '../src/screens/Library.js';
import { Settings } from '../src/screens/Settings.js';
import { ModelsDevice } from '../src/screens/ModelsDevice.js';
import { device, readiness, lock } from './settings-fixtures.js';
import { previewClean, previewModelApi, storageWithCleanUp } from './model-fixtures.js';
import { NewProject } from '../src/screens/NewProject.js';
import { LibraryLoader } from '../src/library/loader.js';
import { ImportLoader } from '../src/import/loader.js';
import { daemonApi } from '../src/daemon/api.js';
import { connection, plan as previewPlan } from './fixtures.js';
import {
  editorDocument,
  editorFilmstrip,
  editorFilmstripUrl,
  editorLabels,
  editorPlan,
} from './editor-fixtures.js';
import { applyPreview } from './editor-preview.js';
import { ShortcutSheet, useShortcutSheet } from '../src/shell/ShortcutSheet.js';
import {
  REVIEW_DURATION_TICKS,
  reviewCrop,
  reviewPeaks,
  reviewRows as fixtures,
  reviewTranscript,
} from './review-fixtures.js';
import { reviewPlanOf } from './review-preview.js';
import { CoachEnabled } from '../src/onboarding/state.js';
import { Welcome } from '../src/onboarding/Welcome.js';
import { ClipLoop } from '../src/export/ClipLoop.js';
import { exactCaptionsOf } from '../src/editor/exactCaptions.js';
import '../src/styles.css';

const noAction = () => {};
const modelScenario =
  new URLSearchParams(location.search).get('models') === 'installed' ? 'installed' : 'fresh';
const modelApi = {
  ...daemonApi,
  ...previewModelApi(modelScenario),
  fetchReadiness: async () => readiness,
};
const project = {
  projectId: 'preview',
  name: 'The creative process · Episode 12',
  createdUnixMillis: Date.now() - 86_400_000,
};
class PreviewLibrary extends LibraryLoader {
  override async load() {
    return {
      projects: [
        project,
        { ...project, projectId: 'preview-2', name: 'Conversations about better work' },
        { ...project, projectId: 'preview-3', name: 'A different perspective' },
      ].map((entry) => ({
        project: entry,
        status: { kind: 'analyzed' as const },
        job: null,
        source: null,
        sourceMap: null,
        thumbnail: null,
      })),
      storage: null,
    };
  }
}
const libraryLoader = new PreviewLibrary();
const publishingScenario = new URLSearchParams(location.search).get('publishing');
const publishingApi = {
  ...daemonApi,
  ...publishingFixture(
    publishingScenario === 'connected' || publishingScenario === 'history',
    publishingScenario === 'history',
  ),
  listProjects: async () => [project],
};
const importLoader = new ImportLoader({
  ...daemonApi,
  fetchReadiness: async () => ({
    ready: true,
    decoderPresent: true,
    decoderPath: '/preview',
    stages: [],
    workers: [],
  }),
  chooseSourceFile: async () => null,
  listYoutubeImports: async () => [],
});
const appearanceController = new ThemeController(document.documentElement);
function Preview() {
  const search = new URLSearchParams(location.search);
  const [page, setPage] = useState(search.get('screen') ?? 'results');
  const [theme, setTheme] = useState<'dark' | 'light'>(
    search.get('theme') === 'light' ? 'light' : 'dark',
  );
  const [workspaceTheme, setWorkspaceTheme] = useState<WorkspaceTheme>(() => {
    const requested = search.get('workspaceTheme');
    return isWorkspaceTheme(requested) ? requested : DEFAULT_WORKSPACE_THEME;
  });
  useLayoutEffect(() => {
    appearanceController.apply(theme);
    appearanceController.applyWorkspace(workspaceTheme);
  }, [theme, workspaceTheme]);
  const [rows, setRows] = useState<ClipRow[]>(() => {
    const scenario = search.get('scenario');
    if (scenario === 'empty') return [];
    return fixtures.map((row, index) =>
      scenario === 'declined' || (scenario === 'mixed' && index > 2)
        ? {
            ...row,
            band: 'declined',
            bandLabel: 'Declined by editorial review',
            recommended: false,
            decision: null,
            docId: null,
            review: {
              status: 'rejected',
              route: 'local',
              reasons: [
                'The exchange introduces a question but ends before the answer. Inspect the surrounding source before making an edit.',
              ],
            },
          }
        : row,
    );
  });
  const [manualOpen, setManualOpen] = useState(false);
  const [candidate, setCandidate] = useState(fixtures[0]!.candidateId);
  const [plan, setPlan] = useState(() =>
    search.get('layout') === 'two_up'
      ? {
          ...previewPlan,
          crops: previewPlan.crops.map(() => [0, 140, 900, 800] as const),
          secondaryCrops: previewPlan.crops.map(() => [1000, 140, 900, 800] as const),
          segments: previewPlan.segments.map((segment) => ({ ...segment, hasTwoUpPaths: true })),
          cues: previewPlan.cues.map((cue) => ({ ...cue, region: 'center' })),
        }
      : editorPlan,
  );
  const [editorDoc, setEditorDoc] = useState(editorDocument);
  // With the dev server's caption route (CLIPMILL_CAPTION_ASS) and ?fonts=
  // naming the pinned font directory, the editor draws the render's own ASS.
  const fontsDir = search.get('fonts');
  // ?emoji= names the pinned emoji pictures' directory, as ?fonts= does fonts.
  const emojiDir = search.get('emoji');
  const [editorAss, setEditorAss] = useState<string | null>(null);
  const [welcoming, setWelcoming] = useState(() => search.get('welcome') === '1');
  // The Inspector's dry run: what approving the clip on screen would build.
  const reviewRow = rows.find((row) => row.candidateId === candidate);
  const reviewPlan = useMemo(
    () =>
      reviewRow
        ? reviewPlanOf(
            { startTicks: reviewRow.startTicks, endTicks: reviewRow.endTicks },
            reviewTranscript,
          )
        : null,
    [reviewRow],
  );
  const [reviewAss, setReviewAss] = useState<string | null>(null);
  useEffect(() => {
    if (!fontsDir || !reviewPlan) return;
    let live = true;
    void captionAssOf(reviewPlan.captions, 30, 1).then((ass) => {
      if (live) setReviewAss(ass);
    });
    return () => {
      live = false;
    };
  }, [fontsDir, reviewPlan]);
  useEffect(() => {
    if (!fontsDir) return;
    let live = true;
    void captionAssOf(editorDoc.captions, plan.rateNum, plan.rateDen).then((ass) => {
      if (live) setEditorAss(ass);
    });
    return () => {
      live = false;
    };
  }, [fontsDir, editorDoc, plan.rateNum, plan.rateDen]);
  const [notice, setNotice] = useState<string | null>(null);
  const [destination, setDestination] = useState('/Users/demo/Movies/ClipMill');
  const [pattern, setPattern] = useState('{index}-{clip}');
  const media = search.get('media');
  const labels = { project: project.name, clip: 'A better question' };
  const inspect = (id: string) => {
    setCandidate(id);
    setPage('inspector');
  };
  const still = search.get('still');
  const [autoAdvance, setAutoAdvance] = useState(true);
  const [history, setHistory] = useState<{ id: string; previous: ClipRow['decision'] }[]>([]);
  /** A decision on the open clip, as the daemon would record it, then on to the next. */
  const record = (decision: ClipRow['decision'], docId?: string) => {
    const before = rows.find((row) => row.candidateId === candidate);
    setHistory((stack) => [...stack, { id: candidate, previous: before?.decision ?? null }]);
    const next = rows.map((row) =>
      row.candidateId === candidate ? { ...row, decision, ...(docId ? { docId } : {}) } : row,
    );
    setRows(next);
    setNotice(
      decision === 'approved'
        ? 'Approved. The edit is ready.'
        : decision === 'kept'
          ? 'Kept for later.'
          : decision === 'rejected'
            ? 'Rejected.'
            : 'Decision cleared.',
    );
    if (decision && autoAdvance) {
      const after = next.slice(next.findIndex((row) => row.candidateId === candidate) + 1);
      const following = [...after, ...next].find(
        (row) => row.decision === null && row.candidateId !== candidate,
      );
      if (following) setCandidate(following.candidateId);
    }
  };
  const workspace = ['results', 'inspector', 'editor'].includes(page);
  const shortcuts = useShortcutSheet();
  return (
    <CoachEnabled.Provider value={search.get('coach') === '1'}>
      <TooltipProvider>
        <Welcome
          open={welcoming}
          onClose={() => setWelcoming(false)}
          onStart={() => setPage('new-project')}
        />
        <ShortcutSheet open={shortcuts.open} onOpenChange={shortcuts.setOpen} />
        <div className="flex h-full flex-col">
          <div className="flex h-7 shrink-0 items-center justify-center border-b border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] text-[10px] text-[var(--cm-text-muted)]">
            UI development preview · Synthetic data · No files are changed
          </div>
          <SidebarProvider
            className="studio-shell min-h-0 flex-1"
            style={{ '--sidebar-width': 'var(--cm-shell-sidebar-width)' } as CSSProperties}
            data-workspace={page === 'inspector' || page === 'editor' ? 'true' : undefined}
          >
            <AppSidebar
              activeId={page === 'inspector' ? 'results' : page}
              onSelect={setPage}
              state={connection}
            />
            <SidebarInset className="min-h-0 min-w-0 bg-transparent">
              {/* As the app does: the two workspaces fold the trail away. */}
              {page !== 'inspector' && page !== 'editor' && (
                <TopBar
                  trail={[page.charAt(0).toUpperCase() + page.slice(1)]}
                  theme={theme}
                  onToggleTheme={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
                  state={connection}
                  profile={null}
                />
              )}
              <main
                className={`studio-main ${workspace ? 'studio-main-workspace' : 'studio-main-page'}`}
              >
                {page === 'results' && (
                  <Results
                    loading={false}
                    rows={rows}
                    summary={{
                      selected: rows.filter((row) => row.recommended).length,
                      requested: 5,
                      cohort: rows.filter((row) => row.review?.status !== 'rejected').length,
                      declined: rows.filter((row) => row.review?.status === 'rejected').length,
                      contentProfile:
                        search.get('scenario') === 'declined' ? 'scripted' : 'interview',
                      filtered: 0,
                      shortfall: [],
                    }}
                    problem={null}
                    sourceName={project.name}
                    run={{
                      jobId: 'preview-run',
                      state: JobState.SUCCEEDED,
                      completedUnixMillis: Date.now(),
                    }}
                    tileUrl={() => still}
                    projects={[project]}
                    activeProjectId={project.projectId}
                    busy={false}
                    onChooseProject={noAction}
                    onInspect={inspect}
                    onEdit={() => setPage('editor')}
                    onApproveMany={(ids) =>
                      setRows((current) =>
                        current.map((row) =>
                          ids.includes(row.candidateId)
                            ? { ...row, decision: 'approved', docId: 'preview-edit' }
                            : row,
                        ),
                      )
                    }
                    onReload={noAction}
                    onManualClip={() => setManualOpen(true)}
                  />
                )}
                {page === 'inspector' && (
                  <ClipInspector
                    rows={rows}
                    candidateId={candidate}
                    proxyUrl={media}
                    crop={reviewCrop}
                    preview={reviewPlan?.plan ?? null}
                    previewCaptions={
                      reviewPlan && reviewAss && fontsDir
                        ? exactCaptionsOf(
                            {
                              ...reviewPlan.plan,
                              ass: reviewAss,
                              fonts: CAPTION_FONTS.map((font) => ({ ...font, installed: true })),
                            },
                            (file) => `${fontsDir}/${file}`,
                          )
                        : null
                    }
                    peaks={reviewPeaks}
                    tileUrl={() => still}
                    transcript={{ status: 'ready', transcript: reviewTranscript }}
                    sourceDurationTicks={REVIEW_DURATION_TICKS}
                    durationTarget={{ minTicks: 20 * 90_000, maxTicks: 90 * 90_000 }}
                    busy={false}
                    notice={notice}
                    autoAdvance={autoAdvance}
                    onAutoAdvance={setAutoAdvance}
                    onSelect={setCandidate}
                    onBack={() => setPage('results')}
                    onApprove={(_cut, open) => {
                      record('approved', 'preview-edit');
                      if (open) setPage('editor');
                    }}
                    onDecide={(decision) => record(decision)}
                    onUndo={
                      history.length > 0
                        ? () => {
                            const last = history.at(-1)!;
                            setHistory((stack) => stack.slice(0, -1));
                            setRows((current) =>
                              current.map((row) =>
                                row.candidateId === last.id
                                  ? { ...row, decision: last.previous }
                                  : row,
                              ),
                            );
                            setCandidate(last.id);
                            setNotice('Decision undone.');
                          }
                        : null
                    }
                    onOpenEdit={
                      rows.find((row) => row.candidateId === candidate)?.docId
                        ? () => setPage('editor')
                        : null
                    }
                  />
                )}
                {page === 'editor' && (
                  <Editor
                    plan={
                      editorAss
                        ? {
                            ...plan,
                            ass: editorAss,
                            fonts: CAPTION_FONTS.map((font) => ({ ...font, installed: true })),
                          }
                        : plan
                    }
                    fontUrl={fontsDir ? (file) => `${fontsDir}/${file}` : null}
                    emojiUrl={emojiDir ? (code) => `${emojiDir}/emoji_u${code}.png` : null}
                    previewCaptions={
                      fontsDir
                        ? (draft) =>
                            captionAssOf(
                              {
                                ...editorDoc.captions,
                                ...(draft.styleRef ? { style_ref: draft.styleRef } : {}),
                                ...(draft.options ? { options: draft.options } : {}),
                              },
                              plan.rateNum,
                              plan.rateDen,
                            )
                        : null
                    }
                    document={editorDoc}
                    transcript={reviewTranscript}
                    filmstrip={editorFilmstrip}
                    peaks={reviewPeaks}
                    filmstripUrl={editorFilmstripUrl}
                    proxyUrls={new Map(media ? [['preview', media]] : [])}
                    docId="preview-edit"
                    labels={editorLabels}
                    loading={false}
                    problem={notice}
                    busy={false}
                    canUndo={false}
                    canRedo={false}
                    resolving={false}
                    resolveRefusal="No worker is connected in the UI preview."
                    picker={null}
                    onOpenResults={() => setPage('results')}
                    onExport={() => setPage('export')}
                    onApply={(command) => {
                      const next = applyPreview({ plan, document: editorDoc }, command);
                      if (!next) {
                        setNotice('This development preview does not save that edit.');
                        return;
                      }
                      setNotice(null);
                      setPlan(next.plan);
                      setEditorDoc(next.document);
                    }}
                    onUndo={noAction}
                    onRedo={noAction}
                    onResolve={noAction}
                  />
                )}
                {page === 'batch-export' && (
                  <BatchExportScreen
                    api={batchApi}
                    onBack={() => setPage('export')}
                    onEdit={() => setPage('editor')}
                  />
                )}
                {page === 'export' && (
                  <Export
                    preview={
                      media ? (
                        <ClipLoop
                          api={{
                            ...daemonApi,
                            mediaUrl: () => media,
                            ...(fontsDir
                              ? { captionFontUrl: (file) => `${fontsDir}/${file}` }
                              : {}),
                          }}
                          projectId="preview"
                          plan={
                            editorAss
                              ? {
                                  ...plan,
                                  ass: editorAss,
                                  fonts: CAPTION_FONTS.map((font) => ({
                                    ...font,
                                    installed: true,
                                  })),
                                }
                              : plan
                          }
                        />
                      ) : null
                    }
                    publishing={
                      <UploadPanel
                        api={publishingApi}
                        projectId="preview"
                        docId="preview-edit"
                        exportJobId="preview-export"
                        revision={plan.revision}
                        renderArtifactId="preview-render"
                        currentRevision={plan.revision}
                        delivered
                        onSetup={() => setPage('settings')}
                      />
                    }
                    onEdit={() => setPage('editor')}
                    docId="preview-edit"
                    labels={labels}
                    picker={null}
                    destination={destination}
                    pattern={pattern}
                    title={labels.clip}
                    attestation="own_content"
                    rightsGateNeeded={false}
                    rightsGatePassed={true}
                    hotCaptions={[]}
                    plan={{
                      passes: true,
                      findings: [],
                      stem: '01-a-better-question',
                      fileNames: [
                        '01-a-better-question.mp4',
                        '01-a-better-question.srt',
                        '01-a-better-question.vtt',
                      ],
                      revision: plan.revision,
                      estimatedBytes: 42_000_000,
                      availableBytes: 75_000_000_000,
                    }}
                    planning={false}
                    busy={false}
                    error={notice}
                    delivery={null}
                    archive={null}
                    onDestinationChange={setDestination}
                    onPatternChange={setPattern}
                    onChooseFolder={() =>
                      setNotice('Folder selection is available in the desktop app.')
                    }
                    onRightsGateChange={noAction}
                    onExport={() => setNotice('Development preview: no video was exported.')}
                    onArchive={() => setNotice('Development preview: no archive was written.')}
                    onReveal={noAction}
                  />
                )}
                {page === 'library' && (
                  <Library
                    state={connection}
                    loader={libraryLoader}
                    onNavigate={setPage}
                    onOpenAnalysis={noAction}
                    onReconnect={noAction}
                  />
                )}
                {page === 'new-project' && (
                  <NewProject state={connection} loader={importLoader} onStarted={noAction} />
                )}
                {page === 'models' && (
                  <ModelsDevice
                    api={modelApi}
                    state={connection}
                    profile={device}
                    artifactId={`sha256:${'b'.repeat(64)}`}
                    error={null}
                    busy={false}
                    onRescan={noAction}
                    onReconnect={noAction}
                  />
                )}
                {page === 'settings' && (
                  <Settings
                    integrations={
                      <div className="space-y-5">
                        <ConnectionCard api={publishingApi} />
                        <UploadHistory api={publishingApi} />
                      </div>
                    }
                    storage={storageWithCleanUp}
                    lock={lock}
                    loading={false}
                    error={null}
                    theme={theme}
                    onThemeChange={setTheme}
                    workspaceTheme={workspaceTheme}
                    onWorkspaceThemeChange={setWorkspaceTheme}
                    onRefresh={noAction}
                    onCleanStorage={previewClean}
                    onOpenStorage={async () => undefined}
                    onOpenModels={() => setPage('models')}
                  />
                )}
              </main>
            </SidebarInset>
          </SidebarProvider>
        </div>
        <ManualClip
          open={manualOpen}
          onOpenChange={setManualOpen}
          sourceName={project.name}
          sourceDurationTicks={REVIEW_DURATION_TICKS}
          proxyUrl={media}
          busy={false}
          notice={notice}
          transcript={reviewTranscript}
          peaks={reviewPeaks}
          tileUrl={() => still}
          onCreate={async () => {
            setNotice(
              'Development preview: the source selection was validated, but no edit was saved.',
            );
            return false;
          }}
        />
      </TooltipProvider>
    </CoachEnabled.Provider>
  );
}
if (import.meta.env.DEV) {
  const root =
    (import.meta.hot?.data['root'] as Root | undefined) ??
    createRoot(document.getElementById('root')!);
  root.render(<Preview />);
  import.meta.hot?.dispose((data) => {
    data['root'] = root;
  });
}

/** The caption faces the render pins, as the daemon lists them. */
const CAPTION_FONTS = [
  { family: 'Inter', label: 'Inter', file: 'Inter-Bold.ttf' },
  { family: 'Montserrat Black', label: 'Montserrat', file: 'Montserrat-Black.ttf' },
  { family: 'Poppins ExtraBold', label: 'Poppins', file: 'Poppins-ExtraBold.ttf' },
  { family: 'Anton', label: 'Anton', file: 'Anton-Regular.ttf' },
  { family: 'Bebas Neue', label: 'Bebas Neue', file: 'BebasNeue-Regular.ttf' },
  { family: 'Luckiest Guy', label: 'Luckiest Guy', file: 'LuckiestGuy-Regular.ttf' },
  { family: 'DM Serif Display', label: 'DM Serif', file: 'DMSerifDisplay-Regular.ttf' },
] as const;

/** The render's own ASS for a caption track, from the dev server's route. */
async function captionAssOf(
  captions: unknown,
  rateNum: number,
  rateDen: number,
): Promise<string | null> {
  try {
    const response = await fetch(
      `/__clipmill/caption-ass?rate=${encodeURIComponent(`${rateNum}/${rateDen}`)}`,
      { method: 'POST', body: JSON.stringify(captions) },
    );
    return response.ok ? await response.text() : null;
  } catch {
    return null;
  }
}
