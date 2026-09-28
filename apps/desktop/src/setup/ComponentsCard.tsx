import { Check, Cpu, Download, RotateCcw, TriangleAlert, X } from 'lucide-react';
import { type JSX, useCallback, useEffect, useState } from 'react';

import { StatusBadge, type StatusTone } from '@/components/StatusBadge';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Spinner } from '@/components/ui/spinner';
import { cn } from '@/lib/utils';

import {
  type Component,
  type Components,
  type ComponentsApi,
  componentDownloadBytes,
  componentsNeeded,
  installingComponents,
} from '../daemon/components.js';
import { formatBytes } from '../deviceProfile.js';
import { componentStatus } from './SetupCard.js';

import './setup.css';

const POLL_MILLIS = 1_000;

/**
 * The components of a packaged app on the Models page: each worker, whether
 * it is installed and running, and a way to update or retry one. Nothing is
 * shown in a development checkout, whose workers are started by hand.
 */
export function ComponentsCard({ api }: { readonly api: ComponentsApi }): JSX.Element | null {
  const [components, setComponents] = useState<Components | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [generation, setGeneration] = useState(0);
  const refresh = useCallback(() => setGeneration((value) => value + 1), []);

  useEffect(() => {
    let live = true;
    void api.listComponents().then(
      (next) => {
        if (live) setComponents(next);
      },
      (cause: unknown) => {
        if (live) setError(cause instanceof Error ? cause.message : String(cause));
      },
    );
    return () => {
      live = false;
    };
  }, [api, generation]);

  const installing = installingComponents(components);
  useEffect(() => {
    if (!installing) return undefined;
    const timer = setInterval(refresh, POLL_MILLIS);
    return () => clearInterval(timer);
  }, [installing, refresh]);

  const act = (action: () => Promise<Components>) => {
    setPending(true);
    setError(null);
    action().then(
      (next) => {
        setComponents(next);
        setPending(false);
      },
      (cause: unknown) => {
        setError(cause instanceof Error ? cause.message : String(cause));
        setPending(false);
      },
    );
  };

  if (components === null || !components.managed) return null;
  const needed = componentsNeeded(components);
  const bytes = componentDownloadBytes(needed);
  const logs = components.parts[0]?.logPath.replace(/[/\\][^/\\]+$/, '');

  return (
    <Card className="gap-0 py-0" data-testid="components">
      <CardContent className="flex flex-col gap-3 px-5 py-4">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="min-w-0">
            <h3 className="flex items-center gap-2 text-sm font-semibold">
              <Cpu className="size-4 text-[var(--cm-text-secondary)]" aria-hidden="true" />
              Components
            </h3>
            <p className="mt-1 text-xs leading-relaxed text-[var(--cm-text-secondary)]">
              The programs that run the models, each in its own process that ClipMill starts and
              restarts.
              {components.pythonVersion !== '' && ` Python ${components.pythonVersion}.`}
            </p>
          </div>
          {installing ? (
            <Button
              size="sm"
              variant="outline"
              disabled={pending}
              onClick={() => act(() => api.cancelComponentInstall())}
            >
              <X />
              Stop
            </Button>
          ) : (
            needed.length > 0 && (
              <Button
                size="sm"
                disabled={pending || components.unavailable !== ''}
                onClick={() => act(() => api.installComponents([]))}
              >
                <Download />
                {needed.every((part) => part.state === 'outdated') ? 'Update' : 'Install'}
                {bytes > 0 && ` · about ${formatBytes(bytes)}`}
              </Button>
            )
          )}
        </div>
        {components.unavailable !== '' && (
          <p className="text-xs text-[var(--cm-danger-ink)]">{components.unavailable}</p>
        )}
        {error !== null && (
          <p className="flex items-start gap-2 text-xs text-[var(--cm-danger-ink)]" role="alert">
            <TriangleAlert className="mt-0.5 size-3.5 shrink-0" aria-hidden="true" />
            {error}
          </p>
        )}
        <ul className="setup-rows" aria-label="Components">
          {components.parts.map((part) => (
            <PartRow
              key={part.name}
              part={part}
              disabled={pending || installing}
              onRetry={() => act(() => api.installComponents([part.name]))}
            />
          ))}
        </ul>
        {logs !== undefined && logs !== '' && (
          <p className="text-[11px] text-[var(--cm-text-muted)]">
            What each one says is kept in <span className="mono break-all">{logs}</span>.
          </p>
        )}
      </CardContent>
    </Card>
  );
}

function PartRow({
  part,
  disabled,
  onRetry,
}: {
  readonly part: Component;
  readonly disabled: boolean;
  readonly onRetry: () => void;
}): JSX.Element {
  const installed = part.state === 'installed' || part.state === 'outdated';
  return (
    <li className="setup-row components-row" data-state={part.state}>
      <span className="setup-row-mark" aria-hidden="true">
        {part.state === 'installing' ? (
          <Spinner />
        ) : part.state === 'failed' ? (
          <TriangleAlert className="size-3.5 text-[var(--cm-danger-ink)]" />
        ) : installed ? (
          <Check className="size-3.5 text-[var(--cm-success-ink)]" />
        ) : (
          <span className="setup-row-dot" />
        )}
      </span>
      <span className="setup-row-title">{part.title}</span>
      <span
        className={cn('setup-row-status', part.state === 'failed' && 'text-[var(--cm-danger-ink)]')}
        title={part.detail || undefined}
      >
        {installed
          ? [
              part.state === 'outdated' ? 'Update available' : null,
              part.installedBytes > 0 ? formatBytes(part.installedBytes) : null,
            ]
              .filter(Boolean)
              .join(' · ') || 'Installed'
          : componentStatus(part)}
      </span>
      <span className="components-row-process">
        {installed && !part.tool && <ProcessBadge part={part} />}
        {part.state === 'failed' && (
          <Button size="xs" variant="outline" disabled={disabled} onClick={onRetry}>
            <RotateCcw />
            Try again
          </Button>
        )}
      </span>
    </li>
  );
}

function ProcessBadge({ part }: { readonly part: Component }): JSX.Element {
  const [tone, label]: [StatusTone, string] =
    part.process === 'running'
      ? ['success', 'Running']
      : part.process === 'starting'
        ? ['progress', 'Starting']
        : part.process === 'waiting'
          ? ['warning', part.restarts > 1 ? `Restarting (${part.restarts})` : 'Restarting']
          : ['neutral', 'Stopped'];
  return (
    <span title={part.process === 'waiting' ? part.detail : undefined}>
      <StatusBadge tone={tone}>{label}</StatusBadge>
    </span>
  );
}
