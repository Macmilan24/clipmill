import {
  CircleAlert,
  Download,
  RotateCw,
  ShieldCheck,
  Trash2,
  TriangleAlert,
  X,
} from 'lucide-react';
import { type JSX, useState } from 'react';

import { StatusBadge } from '@/components/StatusBadge';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { Spinner } from '@/components/ui/spinner';
import { cn } from '@/lib/utils';

import { formatBytes } from '../deviceProfile.js';
import type { LibraryModel, ModelJob, ModelLibrary } from '../daemon/models.js';
import {
  fitWarning,
  isDownloading,
  modelFacts,
  percentOf,
  progressText,
  workerWarning,
} from './describe.js';

export interface ModelRowActions {
  readonly onDownload: () => void;
  readonly onCancel: () => void;
  readonly onUse: () => void;
  readonly onVerify: () => void;
  readonly onRemove: () => void;
}

/**
 * One model in a job: what it is, whether it is here, and what can be done.
 *
 * Destructive actions confirm in place, naming what will be lost, rather than
 * behind a modal: the model the sentence is about is right above it.
 */
export function ModelRow({
  model,
  job,
  library,
  pending,
  actions,
}: {
  readonly model: LibraryModel;
  readonly job: ModelJob;
  readonly library: ModelLibrary;
  /** The action in flight anywhere in the library, by key. */
  readonly pending: string | null;
  readonly actions: ModelRowActions;
}): JSX.Element {
  const [confirming, setConfirming] = useState(false);
  const installed = model.installState === 'installed';
  const downloading = isDownloading(model);
  const download = model.download;
  const planned = job.model === model.name;
  const warning = fitWarning(model, library);
  const missingWorker =
    installed && model.supported
      ? workerWarning(
          model,
          job,
          library.models.find((entry) => entry.name === job.model),
        )
      : null;
  const busy = pending !== null && pending.endsWith(`:${model.name}`);
  const alternatives = job.models.length > 1;
  const headingId = `model-${model.name}`;

  return (
    <li
      data-tour={
        installed || model.installState === 'partial' || model.custom
          ? 'models-removable-row'
          : undefined
      }
      className="model-row px-5 py-4"
      aria-labelledby={headingId}
    >
      <div className="flex flex-wrap items-start justify-between gap-x-4 gap-y-3">
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <h4 id={headingId} className="model-row-title">
              {model.title}
            </h4>
            {planned && installed && <StatusBadge tone="success">In use</StatusBadge>}
            {planned && !installed && <StatusBadge tone="warning">Needed</StatusBadge>}
            {model.recommended && !model.custom && (
              <StatusBadge tone="neutral">Recommended</StatusBadge>
            )}
            {model.custom && <StatusBadge tone="neutral">Added by you</StatusBadge>}
          </div>
          {model.summary !== '' && <p className="model-row-summary">{model.summary}</p>}
          <p className="model-row-facts">{modelFacts(model)}</p>
          {!model.supported && (
            <p className="model-row-note text-[var(--cm-danger-ink)]">
              <CircleAlert aria-hidden="true" />
              {model.unsupportedReason}
            </p>
          )}
          {model.supported && warning !== null && (
            <p
              className={cn(
                'model-row-note',
                warning.tone === 'danger'
                  ? 'text-[var(--cm-danger-ink)]'
                  : 'text-[var(--cm-warning-ink)]',
              )}
            >
              <TriangleAlert aria-hidden="true" />
              {warning.text}
            </p>
          )}
          {missingWorker !== null && (
            <p className="model-row-note text-[var(--cm-warning-ink)]" role="status">
              <TriangleAlert aria-hidden="true" />
              {missingWorker}
            </p>
          )}
          {download !== undefined && downloading && (
            <div className="model-row-progress" role="status">
              {download.state !== 'queued' && (
                <Progress
                  value={percentOf(download) ?? 0}
                  aria-label={`${model.title} download progress`}
                  className="h-1.5"
                />
              )}
              <p>
                <span className="min-w-0 truncate">{progressText(download)}</span>
                {download.state !== 'queued' && percentOf(download) !== undefined && (
                  <span className="mono shrink-0">{percentOf(download)}%</span>
                )}
              </p>
            </div>
          )}
          {download !== undefined && download.state === 'failed' && (
            <p className="model-row-note text-[var(--cm-danger-ink)]" role="alert">
              <CircleAlert aria-hidden="true" />
              {progressText(download)}
            </p>
          )}
          {download !== undefined && download.state === 'cancelled' && (
            <p className="model-row-note text-[var(--cm-text-secondary)]">
              {progressText(download)}. Downloading again keeps what already arrived.
            </p>
          )}
          {download === undefined && model.installState === 'partial' && (
            <p className="model-row-note text-[var(--cm-text-secondary)]">
              Partly downloaded: {formatBytes(model.installedBytes)} of{' '}
              {formatBytes(model.downloadBytes)} is here.
            </p>
          )}
          {confirming && (
            <div className="model-row-confirm" role="group" aria-label={`Remove ${model.title}`}>
              <p>
                {model.custom
                  ? `Remove ${model.title}? Its ${formatBytes(Math.max(model.installedBytes, 0))} of files are deleted and it leaves the library; you can add it again later.`
                  : `Remove ${model.title}? Its ${formatBytes(Math.max(model.installedBytes, 0))} of files are deleted; you can download it again later.`}
                {planned &&
                  ` It is in use for ${job.title.toLowerCase()}; ClipMill will use another installed model if there is one.`}
              </p>
              <div className="flex flex-wrap gap-2">
                <Button
                  size="sm"
                  variant="destructive"
                  disabled={busy}
                  onClick={() => {
                    setConfirming(false);
                    actions.onRemove();
                  }}
                >
                  <Trash2 />
                  Remove
                </Button>
                <Button size="sm" variant="ghost" onClick={() => setConfirming(false)}>
                  Keep it
                </Button>
              </div>
            </div>
          )}
        </div>
        <div className="model-row-actions">
          {downloading ? (
            <Button
              size="sm"
              variant="outline"
              disabled={busy}
              onClick={actions.onCancel}
              aria-label={`Cancel ${model.title} download`}
            >
              {busy ? <Spinner /> : <X />}
              Cancel
            </Button>
          ) : installed ? (
            <>
              {alternatives && !planned && model.supported && (
                <Button
                  size="sm"
                  variant="outline"
                  disabled={busy}
                  onClick={actions.onUse}
                  aria-label={`Use ${model.title} for ${job.title.toLowerCase()}`}
                >
                  Use this one
                </Button>
              )}
              <Button
                size="sm"
                variant="ghost"
                disabled={busy}
                onClick={actions.onVerify}
                aria-label={`Check ${model.title} files`}
              >
                <ShieldCheck />
                Check files
              </Button>
              <Button
                size="sm"
                variant="ghost"
                disabled={busy || confirming}
                onClick={() => setConfirming(true)}
                aria-label={`Remove ${model.title}`}
              >
                <Trash2 />
                Remove
              </Button>
            </>
          ) : (
            <>
              <Button
                size="sm"
                variant={model.recommended || planned ? 'default' : 'outline'}
                disabled={busy || !model.supported}
                onClick={actions.onDownload}
                aria-label={`${download?.state === 'failed' ? 'Retry' : model.installState === 'partial' || download?.state === 'cancelled' ? 'Resume' : 'Download'} ${model.title}`}
              >
                {busy ? <Spinner /> : download?.state === 'failed' ? <RotateCw /> : <Download />}
                {download?.state === 'failed'
                  ? 'Try again'
                  : model.installState === 'partial' || download?.state === 'cancelled'
                    ? 'Resume'
                    : `Download ${formatBytes(model.downloadBytes)}`}
              </Button>
              {(model.installState === 'partial' || model.custom) && (
                <Button
                  size="sm"
                  variant="ghost"
                  disabled={busy || confirming}
                  onClick={() => setConfirming(true)}
                  aria-label={`Remove ${model.title}`}
                >
                  <Trash2 />
                  Remove
                </Button>
              )}
            </>
          )}
        </div>
      </div>
    </li>
  );
}
