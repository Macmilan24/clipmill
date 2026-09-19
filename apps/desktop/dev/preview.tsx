/** Deliberately separate from main.tsx and the production build. No daemon calls. */
import { useState, type CSSProperties } from 'react';
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
import { device, readiness, storage, lock } from './settings-fixtures.js';
import { NewProject } from '../src/screens/NewProject.js';
import { LibraryLoader } from '../src/library/loader.js';
import { ImportLoader } from '../src/import/loader.js';
import { daemonApi } from '../src/daemon/api.js';
import { connection, rows as fixtures, plan as previewPlan } from './fixtures.js';
import '../src/styles.css';

const noAction = () => {};
const modelApi = { ...daemonApi, fetchReadiness: async () => readiness };
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
});
function Preview() {
  const search = new URLSearchParams(location.search);
  const [page, setPage] = useState(search.get('screen') ?? 'results');
  const [theme, setTheme] = useState<'dark' | 'light'>(
    search.get('theme') === 'light' ? 'light' : 'dark',
  );
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
      : previewPlan,
  );
  const [notice, setNotice] = useState<string | null>(null);
  const [destination, setDestination] = useState('/Users/demo/Movies/ClipMill');
  const [pattern, setPattern] = useState('{index}-{clip}');
  const media = search.get('media');
  document.documentElement.dataset['theme'] = theme;
  const labels = { project: project.name, clip: 'A better question' };
  const inspect = (id: string) => {
    setCandidate(id);
    setPage('inspector');
  };
  const decide = (decision: 'approved' | 'kept' | 'rejected') => {
    setRows((current) =>
      current.map((row) => (row.candidateId === candidate ? { ...row, decision } : row)),
    );
    if (decision === 'approved') setPage('editor');
    else setNotice(`Marked ${decision}.`);
  };
  const workspace = ['results', 'inspector', 'editor'].includes(page);
  return (
    <TooltipProvider>
      <div className="flex h-full flex-col">
        <div className="flex h-7 shrink-0 items-center justify-center border-b border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] text-[10px] text-[var(--cm-text-muted)]">
          UI development preview · Synthetic data · No files are changed
        </div>
        <SidebarProvider
          className="studio-shell min-h-0 flex-1"
          style={{ '--sidebar-width': 'var(--cm-shell-sidebar-width)' } as CSSProperties}
        >
          <AppSidebar
            activeId={page === 'inspector' ? 'results' : page}
            onSelect={setPage}
            state={connection}
          />
          <SidebarInset className="min-h-0 min-w-0 bg-transparent">
            <TopBar
              trail={[
                page === 'inspector' ? 'Results' : page.charAt(0).toUpperCase() + page.slice(1),
                ...(workspace && page !== 'results' ? [labels.clip] : []),
              ]}
              theme={theme}
              onToggleTheme={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
              state={connection}
              profile={null}
            />
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
                  tileUrl={() => null}
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
                  crop={{
                    fit: false,
                    fitReason: '',
                    containment: 1,
                    keyframes: [{ tTicks: 0, centerX: 0.5, centerY: 0.5, scale: 1 }],
                  }}
                  cues={[]}
                  peaks={null}
                  busy={false}
                  notice={notice}
                  onSelect={setCandidate}
                  onBack={() => setPage('results')}
                  onDecide={decide}
                  onUseAlternative={() => setNotice('Preview only: no alternative cut.')}
                  onTakeCut={() => setNotice('Preview only: no edit is saved.')}
                  onEdit={
                    rows.find((row) => row.candidateId === candidate)?.docId
                      ? () => setPage('editor')
                      : null
                  }
                />
              )}
              {page === 'editor' && (
                <Editor
                  plan={plan}
                  proxyUrls={new Map(media ? [['preview', media]] : [])}
                  docId="preview-edit"
                  labels={labels}
                  loading={false}
                  problem={notice}
                  busy={false}
                  canUndo={false}
                  canRedo={false}
                  resolving={false}
                  resolveRefusal="No worker connected in the UI preview."
                  picker={null}
                  onOpenResults={() => setPage('results')}
                  onExport={() => setPage('export')}
                  onApply={(command) => {
                    if (command.op === 'set_word_text')
                      setPlan({
                        ...plan,
                        revision: plan.revision + 1,
                        cues: plan.cues.map((cue) => ({
                          ...cue,
                          lines: cue.lines.map((line) =>
                            line.map((word) =>
                              word.wordId === command.word_id
                                ? { ...word, text: String(command.text) }
                                : word,
                            ),
                          ),
                        })),
                      });
                    else if (command.op === 'set_transition')
                      setPlan({
                        ...plan,
                        revision: plan.revision + 1,
                        transitionTicks: Number(command.duration_ticks),
                        // This fixture has one shot, so its saved preference
                        // produces no boundaries. It does not call the renderer.
                        transitions: [],
                      });
                    else setNotice('This development preview does not save editing commands.');
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
                  hotCaptionsConfirmed={false}
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
                  onHotCaptionsChange={noAction}
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
                  storage={storage}
                  lock={lock}
                  loading={false}
                  error={null}
                  theme={theme}
                  onThemeChange={setTheme}
                  onRefresh={noAction}
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
        sourceDurationTicks={90_000 * 720}
        proxyUrl={media}
        busy={false}
        notice={notice}
        onCreate={async () => {
          setNotice(
            'Development preview: the source selection was validated, but no edit was saved.',
          );
          return false;
        }}
      />
    </TooltipProvider>
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
