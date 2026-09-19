import { useEffect, useRef, useState } from 'react';
import {
  Check,
  ExternalLink,
  FileKey,
  Link,
  RefreshCw,
  ShieldCheck,
  Unplug,
  Video,
  X,
} from 'lucide-react';
import { Button } from '../components/ui/button.js';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card.js';
import { Spinner } from '../components/ui/spinner.js';
import { StatusBadge } from '../components/StatusBadge.js';
import type { PublishingApi, YoutubeConnection } from '../daemon/publishing.js';
import { publishingError } from './model.js';
import { usePublishingStatus } from './usePublishingStatus.js';

export function ConnectionCard({ api }: { readonly api: PublishingApi }) {
  const { status, error: statusError, refresh } = usePublishingStatus(api);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [disconnecting, setDisconnecting] = useState<string | null>(null);
  const operating = useRef(false);
  const live = useRef(true);
  useEffect(() => {
    live.current = true;
    return () => {
      live.current = false;
    };
  }, []);
  const run = async (action: () => Promise<unknown>) => {
    if (operating.current) return;
    operating.current = true;
    setPending(true);
    setError(null);
    try {
      await action();
    } catch (cause) {
      if (live.current) setError(publishingError(cause));
    } finally {
      operating.current = false;
      if (live.current) {
        setPending(false);
        refresh();
      }
    }
  };
  const connecting =
    status?.connections.some((connection) => connection.state === 'connecting') ?? false;
  const channels =
    status?.connections.filter((connection) => connection.state !== 'disconnected') ?? [];
  return (
    <Card className="gap-0 overflow-hidden py-0">
      <CardHeader className="border-b border-[var(--cm-glass-border)] px-5 py-4">
        <div className="flex items-center justify-between gap-3">
          <CardTitle className="flex items-center gap-2 text-sm">
            <Video className="size-4" /> YouTube channel
          </CardTitle>
          <Button size="sm" variant="ghost" disabled={pending} onClick={refresh}>
            <RefreshCw className="size-3.5" /> Refresh
          </Button>
        </div>
        <p className="text-xs leading-relaxed text-[var(--cm-text-secondary)]">
          Send an approved export to your channel privately, review it, then choose when to publish.
        </p>
      </CardHeader>
      <CardContent className="space-y-5 px-5 py-5">
        {(error || statusError) && (
          <p
            role="alert"
            className="rounded-lg border border-[var(--cm-glass-border)] p-3 text-xs text-[var(--cm-danger-ink)]"
          >
            {error || statusError}
          </p>
        )}
        {!status && !statusError && (
          <p role="status" className="flex items-center gap-2 text-xs text-[var(--cm-text-muted)]">
            <Spinner /> Checking saved connection…
          </p>
        )}
        {status && !status.available && (
          <p className="text-xs leading-relaxed text-[var(--cm-warning-ink)]">
            Secure channel credentials are unavailable on this platform. Export your files locally
            to upload them manually.
          </p>
        )}
        {status?.available && (
          <>
            <div className="flex items-start gap-3 rounded-xl border border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] p-4">
              <div className="grid size-8 shrink-0 place-items-center rounded-lg bg-[var(--cm-surface-1)]">
                {status.configured ? (
                  <Check className="size-4 text-[var(--cm-success-ink)]" />
                ) : (
                  <FileKey className="size-4" />
                )}
              </div>
              <div className="min-w-0 flex-1">
                <h3 className="text-xs font-semibold">
                  {status.configured
                    ? 'Desktop client configured'
                    : 'Set up your Google desktop client'}
                </h3>
                <p className="mt-1 text-xs leading-relaxed text-[var(--cm-text-secondary)]">
                  {status.configured
                    ? 'Your client configuration is stored securely by the local engine.'
                    : 'Use your own Google Cloud project with YouTube Data API v3 enabled and an OAuth client of type Desktop app.'}
                </p>
                <div className="mt-3 flex flex-wrap gap-2">
                  <Button
                    size="sm"
                    variant={status.configured ? 'outline' : 'default'}
                    disabled={pending || connecting}
                    onClick={() => void run(() => api.chooseYoutubeClientConfig())}
                  >
                    <FileKey className="size-3.5" />{' '}
                    {status.configured ? 'Replace client JSON' : 'Choose client JSON'}
                  </Button>
                  {!status.configured && (
                    <Button
                      size="sm"
                      variant="ghost"
                      disabled={pending}
                      onClick={() => void run(() => api.openYoutubePage('console'))}
                    >
                      <ExternalLink className="size-3.5" /> Google Cloud Console
                    </Button>
                  )}
                </div>
              </div>
            </div>
            <details className="rounded-lg border border-[var(--cm-glass-border)] px-4 py-3">
              <summary className="cursor-pointer text-xs font-medium">
                First-time setup guide
              </summary>
              <ol className="mt-3 list-decimal space-y-2 pl-4 text-xs leading-relaxed text-[var(--cm-text-secondary)]">
                <li>Create a Google Cloud project and enable YouTube Data API v3.</li>
                <li>
                  Configure the consent screen. While the project is in testing, add the Google
                  account that owns your channel as a test user.
                </li>
                <li>
                  Create an OAuth client with application type <strong>Desktop app</strong>,
                  download its JSON, and choose it above.
                </li>
                <li>
                  Connect below, complete sign-in in your browser, and check the channel name shown
                  here.
                </li>
              </ol>
              <p className="mt-3 text-[11px] text-[var(--cm-text-muted)]">
                An API key or a Web application client cannot be used for this connection. Passwords
                and tokens are never entered into this screen.
              </p>
              <Button
                size="sm"
                variant="link"
                className="mt-2 h-auto px-0"
                disabled={pending}
                onClick={() => void run(() => api.openYoutubePage('setup'))}
              >
                Google desktop sign-in documentation <ExternalLink className="size-3" />
              </Button>
            </details>
            <div className="space-y-3">
              {channels.map((connection) => (
                <ConnectionRow
                  key={connection.connectionId}
                  connection={connection}
                  disabled={pending}
                  confirming={disconnecting === connection.connectionId}
                  onConfirm={(value) => setDisconnecting(value ? connection.connectionId : null)}
                  onAction={(action) =>
                    void run(async () => {
                      await api.updateYoutubeConnection(connection.connectionId, action);
                      if (live.current) setDisconnecting(null);
                    })
                  }
                />
              ))}
              {channels.length === 0 && (
                <p className="text-xs text-[var(--cm-text-secondary)]">
                  No channel connected. Source-video imports work independently of this connection.
                </p>
              )}
              <Button
                disabled={!status.configured || pending || connecting}
                onClick={() => void run(() => api.connectYoutubeChannel())}
              >
                {pending ? <Spinner /> : <Link className="size-4" />}{' '}
                {connecting
                  ? 'Waiting for browser sign-in…'
                  : channels.some((connection) => connection.state === 'connected')
                    ? 'Connect another channel'
                    : 'Connect channel'}
              </Button>
            </div>
            <div className="flex items-start gap-2.5 text-[11px] leading-relaxed text-[var(--cm-text-muted)]">
              <ShieldCheck className="mt-0.5 size-4 shrink-0" />
              <p>
                Sign-in opens your system browser. Credentials stay in the operating system
                credential store. Connecting never uploads a video or enables cloud AI.
              </p>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}

function ConnectionRow({
  connection,
  disabled,
  confirming,
  onConfirm,
  onAction,
}: {
  readonly connection: YoutubeConnection;
  readonly disabled: boolean;
  readonly confirming: boolean;
  readonly onConfirm: (value: boolean) => void;
  readonly onAction: (action: 'cancel' | 'disconnect') => void;
}) {
  const ready = connection.state === 'connected';
  const waiting = connection.state === 'connecting';
  return (
    <div className="rounded-xl border border-[var(--cm-glass-border)] p-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0">
          <p className="break-words text-sm font-medium">
            {connection.title ||
              (waiting ? 'Complete sign-in in your browser' : 'Channel connection')}
          </p>
          {connection.channelId && (
            <p className="mt-1 break-all font-mono text-[10px] text-[var(--cm-text-muted)]">
              {connection.channelId}
            </p>
          )}
        </div>
        <StatusBadge tone={ready ? 'success' : waiting ? 'progress' : 'warning'}>
          {ready
            ? 'Connected'
            : waiting
              ? 'Waiting for sign-in'
              : connection.state === 'interrupted'
                ? 'Sign-in interrupted'
                : 'Needs attention'}
        </StatusBadge>
      </div>
      {connection.error && (
        <p className="mt-3 text-xs leading-relaxed text-[var(--cm-danger-ink)]">
          {connection.error}
        </p>
      )}
      {confirming ? (
        <div className="mt-3 space-y-2">
          <p className="text-xs text-[var(--cm-text-secondary)]">
            Disconnect this channel from ClipMill? Its upload history remains saved.
          </p>
          <div className="flex gap-2">
            <Button
              size="sm"
              variant="outline"
              disabled={disabled}
              onClick={() => onAction('disconnect')}
            >
              Disconnect channel
            </Button>
            <Button size="sm" variant="ghost" disabled={disabled} onClick={() => onConfirm(false)}>
              Keep connected
            </Button>
          </div>
        </div>
      ) : (
        <Button
          className="mt-3"
          size="sm"
          variant="ghost"
          disabled={disabled}
          onClick={() => (waiting ? onAction('cancel') : onConfirm(true))}
        >
          {waiting ? <X className="size-3.5" /> : <Unplug className="size-3.5" />}
          {waiting ? 'Cancel sign-in' : 'Disconnect'}
        </Button>
      )}
    </div>
  );
}
