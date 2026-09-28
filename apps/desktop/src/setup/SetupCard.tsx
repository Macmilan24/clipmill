import { Check, Cog, Download, Globe, TriangleAlert, X } from 'lucide-react';
import type { JSX } from 'react';

import { StatusBadge } from '@/components/StatusBadge';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Spinner } from '@/components/ui/spinner';
import { cn } from '@/lib/utils';

import type { Component } from '../daemon/components.js';
import type { LibraryModel } from '../daemon/models.js';
import { formatBytes } from '../deviceProfile.js';
import type { SetupState } from './useSetup.js';

import './setup.css';

const MOVING = ['queued', 'downloading', 'verifying'];

/**
 * Set up ClipMill: the one-time download a packaged app needs before its
 * first analysis. Shown only while there is something to install, and only
 * in a packaged app; a development checkout sets itself up with `just setup`.
 */
export function SetupCard({ setup }: { readonly setup: SetupState }): JSX.Element | null {
  const { components, library } = setup;
  if (!setup.needed || components === null) return null;
  const updating = setup.parts.length > 0 && setup.parts.every((part) => part.state === 'outdated');
  const failed = components.parts.filter((part) => part.state === 'failed');
  const moving = setup.busy || setup.pending;
  const title = updating ? "Update ClipMill's components" : 'Set up ClipMill';

  return (
    <Card className="setup-card gap-0 py-0" data-testid="setup">
      <CardContent className="flex flex-col gap-4 px-5 py-4">
        <div className="flex items-start justify-between gap-3">
          <div className="min-w-0">
            <h3 className="flex items-center gap-2 text-sm font-semibold">
              <Cog className="size-4 text-[var(--cm-text-secondary)]" aria-hidden="true" />
              {title}
            </h3>
            <p className="mt-1 text-xs leading-relaxed text-[var(--cm-text-secondary)]">
              {updating
                ? 'This version of ClipMill brings newer components. Analysis may wait for them until they are updated.'
                : 'Everything runs on this computer. Before its first analysis ClipMill downloads its components, the programs that run the models, and the models recommended for this computer. After that, analysis works without a connection.'}
            </p>
          </div>
          <StatusBadge tone={failed.length > 0 ? 'danger' : moving ? 'progress' : 'warning'}>
            {failed.length > 0 ? 'Stopped' : moving ? 'Downloading' : 'Needed'}
          </StatusBadge>
        </div>

        {components.unavailable !== '' ? (
          <p className="text-xs text-[var(--cm-danger-ink)]">{components.unavailable}</p>
        ) : (
          <ul className="setup-rows" aria-label="What setup installs">
            {components.parts.map((part) => (
              <ComponentRow key={part.name} part={part} />
            ))}
            {library !== null && setup.models.length > 0 && <ModelsRow models={setup.models} />}
          </ul>
        )}

        {setup.error !== null && (
          <p className="flex items-start gap-2 text-xs text-[var(--cm-danger-ink)]" role="alert">
            <TriangleAlert className="mt-0.5 size-3.5 shrink-0" aria-hidden="true" />
            {setup.error}
          </p>
        )}

        <div className="flex flex-wrap items-center justify-between gap-3">
          <p className="setup-network-note">
            <Globe aria-hidden="true" />
            Downloads Python and packages from GitHub and PyPI, and models from Hugging Face.
            Nothing about your projects is sent.
          </p>
          {moving ? (
            <Button variant="outline" size="sm" onClick={setup.cancel} disabled={setup.pending}>
              <X />
              Stop
            </Button>
          ) : (
            <Button size="sm" onClick={setup.start} disabled={components.unavailable !== ''}>
              <Download />
              {failed.length > 0
                ? 'Try again'
                : setup.downloadBytes > 0
                  ? `${updating ? 'Update' : 'Set up'} · about ${formatBytes(setup.downloadBytes)}`
                  : updating
                    ? 'Update'
                    : 'Set up'}
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

function ComponentRow({ part }: { readonly part: Component }): JSX.Element {
  const status = componentStatus(part);
  return (
    <li className="setup-row" data-state={part.state}>
      <span className="setup-row-mark" aria-hidden="true">
        {part.state === 'installing' ? (
          <Spinner />
        ) : part.state === 'installed' ? (
          <Check className="size-3.5 text-[var(--cm-success-ink)]" />
        ) : part.state === 'failed' ? (
          <TriangleAlert className="size-3.5 text-[var(--cm-danger-ink)]" />
        ) : (
          <span className="setup-row-dot" />
        )}
      </span>
      <span className="setup-row-title">{part.title}</span>
      <span
        className={cn('setup-row-status', part.state === 'failed' && 'text-[var(--cm-danger-ink)]')}
      >
        {status}
      </span>
    </li>
  );
}

function ModelsRow({ models }: { readonly models: readonly LibraryModel[] }): JSX.Element {
  const total = models.reduce((sum, model) => sum + model.downloadBytes, 0);
  const received = models.reduce((sum, model) => sum + (model.download?.receivedBytes ?? 0), 0);
  const active = models.some(
    (model) => model.download !== undefined && MOVING.includes(model.download.state),
  );
  const failed = models.find((model) => model.download?.state === 'failed');
  return (
    <li className="setup-row" data-state={failed ? 'failed' : active ? 'installing' : 'missing'}>
      <span className="setup-row-mark" aria-hidden="true">
        {active ? (
          <Spinner />
        ) : failed ? (
          <TriangleAlert className="size-3.5 text-[var(--cm-danger-ink)]" />
        ) : (
          <span className="setup-row-dot" />
        )}
      </span>
      <span className="setup-row-title">
        Models: {models.map((model) => model.title).join(', ')}
      </span>
      <span className={cn('setup-row-status', failed && 'text-[var(--cm-danger-ink)]')}>
        {failed?.download?.error
          ? failed.download.error
          : active
            ? `${formatBytes(received)} of ${formatBytes(total)}`
            : formatBytes(total)}
      </span>
    </li>
  );
}

/** One component's state in a few words. */
export function componentStatus(part: Component): string {
  switch (part.state) {
    case 'installing':
      return part.detail || 'Installing';
    case 'queued':
      return 'Waiting its turn';
    case 'installed':
      return 'Installed';
    case 'outdated':
      return 'Update available';
    case 'failed':
      return part.detail || 'Could not be installed';
    default:
      return part.downloadBytes > 0 ? formatBytes(part.downloadBytes) : 'Not installed';
  }
}
