import {
  Boxes,
  ChevronRight,
  Download,
  Eye,
  Globe,
  MessageSquareText,
  Plus,
  RefreshCw,
  ScanFace,
  Sparkles,
  TriangleAlert,
  Users,
  Waves,
  Workflow,
  X,
} from 'lucide-react';
import { type JSX, type ReactNode, useState } from 'react';

import { StatusBadge } from '@/components/StatusBadge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Spinner } from '@/components/ui/spinner';
import { cn } from '@/lib/utils';

import { useReadiness } from '../analysis/readiness.js';
import type { ShellApi } from '../daemon/api.js';
import type { LibraryModel, ModelJob, ModelLibrary } from '../daemon/models.js';
import { formatBytes } from '../deviceProfile.js';
import { AddModelSheet } from './AddModelSheet.js';
import {
  isDownloading,
  jobReason,
  jobSentence,
  recommendedTitles,
  upgradeNote,
} from './describe.js';
import { ModelRow } from './ModelRow.js';
import { ComponentsCard } from '../setup/ComponentsCard.js';
import { type ModelLibraryState, useModelLibrary } from './useModelLibrary.js';

import './models.css';

const JOB_ICONS: Readonly<Record<string, ReactNode>> = {
  editorial: <MessageSquareText />,
  asr: <Workflow />,
  'forced-align': <Waves />,
  vad: <Eye />,
  'speaker-embed': <Users />,
  'detect-faces': <ScanFace />,
};

/**
 * The model library: every model ClipMill can run, grouped by the job it does.
 *
 * Installation, choice and readiness stay separate facts: a model can be here
 * and not chosen, chosen and still downloading, installed with no worker
 * connected to run it. Each is shown as what it is.
 */
export function ModelLibraryPanel({ api }: { readonly api: ShellApi }): JSX.Element {
  const state = useModelLibrary(api);
  const { readiness } = useReadiness(false, api);
  const [adding, setAdding] = useState(false);
  const { library, problem, error } = state;
  const installed = library?.models.filter((model) => model.installState === 'installed') ?? [];
  const installedBytes = installed.reduce((total, model) => total + model.installedBytes, 0);

  return (
    <section aria-labelledby="models-heading" className="space-y-4">
      <div data-tour="models" className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h2 id="models-heading" className="flex items-center gap-2 text-sm font-semibold">
            <Boxes className="size-4 text-[var(--cm-text-secondary)]" />
            Models
          </h2>
          <p className="mt-1 text-xs text-[var(--cm-text-secondary)]">
            {library === null
              ? problem
                ? 'The library could not be read.'
                : 'Reading the library…'
              : `${installed.length} of ${library.models.length} installed · ${formatBytes(installedBytes)} on this computer`}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={state.refresh}
            aria-label="Refresh the model library"
          >
            <RefreshCw />
            Refresh
          </Button>
          <Button size="sm" variant="outline" onClick={() => setAdding(true)} disabled={!library}>
            <Plus />
            Add from Hugging Face
          </Button>
        </div>
      </div>

      {problem !== null && (
        <Alert>
          <TriangleAlert />
          <AlertDescription>
            The model library is unavailable. {problem}
            {library !== null && ' Showing the last successful reading.'}
          </AlertDescription>
        </Alert>
      )}
      {error !== null && (
        <Alert variant="destructive">
          <TriangleAlert />
          <AlertDescription className="flex flex-wrap items-start justify-between gap-2">
            <span className="min-w-0 flex-1">{error}</span>
            <Button
              size="icon-xs"
              variant="ghost"
              aria-label="Dismiss"
              onClick={state.dismissError}
            >
              <X />
            </Button>
          </AlertDescription>
        </Alert>
      )}

      {library === null && problem === null && (
        <div className="flex items-center gap-3 px-1 py-6 text-xs text-[var(--cm-text-secondary)]">
          <Spinner />
          Reading the model library…
        </div>
      )}

      <ComponentsCard api={api} />

      {library !== null && library.recommendedMissing.length > 0 && (
        <SetupCard library={library} state={state} api={api} />
      )}

      {library?.jobs.map((job) => (
        <JobCard
          key={job.capability}
          job={job}
          library={library}
          state={state}
          api={api}
          workerIssue={
            readiness?.stages.find(
              (stage) =>
                stage.capability === job.capability &&
                stage.modelPresent &&
                !stage.ready &&
                !stage.stage.endsWith('-cloud'),
            )?.remedy ?? null
          }
        />
      ))}

      {library !== null && (
        <p className="model-library-footnote">
          Models are kept in <span className="mono break-all">{library.installPath}</span>
          {library.availableBytes !== undefined && ` · ${formatBytes(library.availableBytes)} free`}
          . Downloads come from Hugging Face at a pinned commit, and every file is checked against
          its pinned SHA-256 before it is used. Each download counts as a network operation in Local
          Lock.
        </p>
      )}

      <AddModelSheet
        open={adding}
        onOpenChange={setAdding}
        api={api}
        onAdded={(next) => {
          void state.run('add', () => Promise.resolve(next));
        }}
      />
    </section>
  );
}

/** Shown until everything a fresh install needs is here. */
function SetupCard({
  library,
  state,
  api,
}: {
  readonly library: ModelLibrary;
  readonly state: ModelLibraryState;
  readonly api: ShellApi;
}): JSX.Element {
  const busy = state.pending === 'download:recommended';
  const downloading = library.recommendedMissing.every((name) => {
    const model = library.models.find((candidate) => candidate.name === name);
    return model !== undefined && isDownloading(model);
  });
  return (
    <Card className="model-setup gap-0 py-0" data-testid="model-setup" data-tour="models-setup">
      <CardContent className="flex flex-wrap items-center justify-between gap-4 px-5 py-4">
        <div className="min-w-0 flex-1">
          <h3 className="text-sm font-semibold">Download what analysis needs</h3>
          <p className="mt-1 text-xs leading-relaxed text-[var(--cm-text-secondary)]">
            Recommended for this computer: {recommendedTitles(library)}. You can change or remove
            any of them later.
          </p>
          <p className="model-network-note">
            <Globe aria-hidden="true" />
            Downloads from huggingface.co. Nothing about your projects is sent.
          </p>
        </div>
        <Button
          disabled={busy || downloading}
          onClick={() => {
            void state.run('download:recommended', () =>
              api.downloadModels(library.recommendedMissing),
            );
          }}
        >
          {busy || downloading ? <Spinner /> : <Download />}
          {downloading
            ? 'Downloading…'
            : `Download all · ${formatBytes(library.recommendedMissingBytes)}`}
        </Button>
      </CardContent>
    </Card>
  );
}

function JobCard({
  job,
  library,
  state,
  api,
  workerIssue,
}: {
  readonly job: ModelJob;
  readonly library: ModelLibrary;
  readonly state: ModelLibraryState;
  readonly api: ShellApi;
  /** Why the stage would not run although its model is here, when it would not. */
  readonly workerIssue: string | null;
}): JSX.Element {
  const byName = new Map(library.models.map((model) => [model.name, model] as const));
  const models = job.models
    .map((name) => byName.get(name))
    .filter((model): model is LibraryModel => model !== undefined);
  const planned = byName.get(job.model);
  const plannedReady = planned?.installState === 'installed';
  const upgrade = upgradeNote(job, library);
  // In use: the model the job runs, and any other one with a download under
  // way, paused or failed, or only half here. The rest fold away until asked for.
  const current = models.filter(
    (model) =>
      model.name === job.model || model.download !== undefined || model.installState === 'partial',
  );
  const inUse = current.length > 0 ? current : models;
  const available = models.filter((model) => !inUse.includes(model));
  const availableHere = available.filter((model) => model.installState === 'installed').length;
  const [showAvailable, setShowAvailable] = useState(false);
  const row = (model: LibraryModel) => (
    <ModelRow
      key={model.name}
      model={model}
      job={job}
      library={library}
      pending={state.pending}
      actions={{
        onDownload: () => {
          void state.run(`download:${model.name}`, () => api.downloadModels([model.name]));
        },
        onCancel: () => {
          void state.run(`cancel:${model.name}`, () => api.cancelModelDownload(model.name));
        },
        onUse: () => {
          void state.run(`choose:${model.name}`, () =>
            api.setModelChoice(job.capability, model.name),
          );
        },
        onVerify: () => {
          void state.run(`verify:${model.name}`, () => api.verifyModel(model.name));
        },
        onRemove: () => {
          void state.run(`remove:${model.name}`, () =>
            model.custom ? api.forgetModel(model.name) : api.removeModel(model.name),
          );
        },
      }}
    />
  );
  return (
    <section aria-labelledby={`job-${job.capability}`}>
      <Card className="preference-section model-job gap-0 overflow-hidden py-0">
        <CardHeader className="flex flex-row flex-wrap items-start justify-between gap-3 border-b border-[var(--cm-glass-border)] px-5 py-4">
          <div className="flex min-w-0 flex-1 items-start gap-3">
            <span className="model-role-icon" aria-hidden="true">
              {JOB_ICONS[job.capability] ?? <Boxes />}
            </span>
            <div className="min-w-0">
              <CardTitle id={`job-${job.capability}`} className="text-sm">
                {job.title}
              </CardTitle>
              <p className="mt-1 text-xs leading-relaxed text-[var(--cm-text-secondary)]">
                {job.summary} {jobSentence(job, planned)}
              </p>
            </div>
          </div>
          <div className="flex shrink-0 flex-wrap items-center gap-2">
            <StatusBadge
              tone={
                job.selectedBy === 'unavailable' || !plannedReady
                  ? 'warning'
                  : job.selectedBy === 'installed_fallback'
                    ? 'warning'
                    : 'neutral'
              }
            >
              {jobReason(job)}
            </StatusBadge>
            {job.choice !== '' && (
              <Button
                size="sm"
                variant="ghost"
                disabled={state.pending !== null}
                onClick={() => {
                  void state.run(`choose:${job.capability}`, () =>
                    api.setModelChoice(job.capability, ''),
                  );
                }}
              >
                Use automatic
              </Button>
            )}
          </div>
        </CardHeader>
        {workerIssue !== null && plannedReady && (
          <p className="model-job-issue" role="status">
            <TriangleAlert aria-hidden="true" />
            {workerIssue}
          </p>
        )}
        {upgrade !== null && (
          <p className="model-job-issue model-job-upgrade">
            <Sparkles aria-hidden="true" />
            {upgrade}
          </p>
        )}
        <CardContent className="p-0">
          <ul className="divide-y divide-[var(--cm-glass-border)]">{inUse.map(row)}</ul>
          {available.length > 0 && (
            <button
              type="button"
              className="model-job-more"
              aria-expanded={showAvailable}
              onClick={() => setShowAvailable((shown) => !shown)}
            >
              <ChevronRight className={cn(showAvailable && 'rotate-90')} aria-hidden="true" />
              Other models ({available.length})
              {availableHere > 0 && <span>· {availableHere} on this computer</span>}
            </button>
          )}
          {showAvailable && (
            <ul className="divide-y divide-[var(--cm-glass-border)] border-t border-[var(--cm-glass-border)]">
              {available.map(row)}
            </ul>
          )}
          {models.length === 0 && (
            <p className="px-5 py-5 text-xs text-[var(--cm-text-secondary)]">
              No model is registered for this job.
            </p>
          )}
        </CardContent>
      </Card>
    </section>
  );
}
