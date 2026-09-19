import { useEffect, useRef, useState } from 'react';
import {
  Check,
  ExternalLink,
  LockKeyhole,
  Pause,
  Play,
  RefreshCw,
  Send,
  ShieldCheck,
  Upload,
  Video,
} from 'lucide-react';
import { Button } from '../components/ui/button.js';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card.js';
import { Checkbox } from '../components/ui/checkbox.js';
import { Input } from '../components/ui/input.js';
import { Label } from '../components/ui/label.js';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../components/ui/select.js';
import { Spinner } from '../components/ui/spinner.js';
import { StatusBadge } from '../components/StatusBadge.js';
import type {
  PublishingApi,
  StartYoutubeUpload,
  YoutubeConnection,
  YoutubeMetadataDraft,
  YoutubeUpload,
} from '../daemon/publishing.js';
import { formatBytes } from '../deviceProfile.js';
import {
  metadataLimits,
  parseTags,
  publishingError,
  UPLOAD_STATES,
  uploadActive,
} from './model.js';
import { usePublishingStatus } from './usePublishingStatus.js';
import { useUploads } from './useUploads.js';

interface UploadPanelProps {
  readonly api: PublishingApi;
  readonly projectId: string;
  readonly docId: string;
  readonly exportJobId: string | null;
  readonly revision: number | null;
  readonly renderArtifactId: string | null;
  readonly irArtifactId?: string | null;
  readonly currentRevision: number | null;
  readonly delivered: boolean;
  readonly onSetup?: () => void;
}

export function UploadPanel(props: UploadPanelProps) {
  const {
    api,
    projectId,
    docId,
    exportJobId,
    revision,
    renderArtifactId,
    irArtifactId,
    currentRevision,
    delivered,
    onSetup,
  } = props;
  const connection = usePublishingStatus(api);
  const ledger = useUploads(api, projectId, docId);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [selectedConnectionId, setSelectedConnectionId] = useState('');
  const operating = useRef(false);
  const generation = useRef(0);
  useEffect(() => {
    generation.current += 1;
    setError(null);
    setPending(false);
    return () => {
      generation.current += 1;
    };
  }, [projectId, docId]);
  const run = async (action: () => Promise<YoutubeUpload>) => {
    if (operating.current) return;
    operating.current = true;
    setPending(true);
    setError(null);
    ledger.refresh();
    const mine = generation.current;
    try {
      const record = await action();
      if (mine === generation.current) ledger.accept(record);
    } catch (cause) {
      if (mine === generation.current) setError(publishingError(cause));
    } finally {
      operating.current = false;
      if (mine === generation.current) {
        setPending(false);
        ledger.refresh();
        connection.refresh();
      }
    }
  };
  const open = async (uploadId: string) => {
    try {
      await api.openYoutubePage('studio', uploadId);
    } catch (cause) {
      setError(publishingError(cause));
    }
  };
  const channels =
    connection.status?.connections.filter((item) => item.state === 'connected') ?? [];
  const channel =
    channels.find((item) => item.connectionId === selectedConnectionId) ??
    (!selectedConnectionId && channels.length === 1 ? channels[0] : undefined);
  const currentUploads =
    ledger.uploads?.filter(
      (item) =>
        item.channelId === channel?.channelId &&
        (item.exportJobId === exportJobId ||
          (item.revision === revision &&
            (irArtifactId
              ? item.irArtifactId === irArtifactId
              : item.renderArtifactId === renderArtifactId))),
    ) ?? [];
  const ready = delivered && exportJobId !== null && revision !== null && renderArtifactId !== null;
  return (
    <Card className="gap-0 overflow-hidden py-0" data-testid="youtube-publishing">
      <CardHeader className="border-b border-[var(--cm-glass-border)] px-5 py-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <CardTitle className="flex items-center gap-2 text-sm">
            <Video className="size-4" /> Deliver to YouTube
          </CardTitle>
          <StatusBadge tone="neutral">
            <LockKeyhole className="size-3" /> Private first
          </StatusBadge>
        </div>
        <p className="text-xs leading-relaxed text-[var(--cm-text-secondary)]">
          Upload the exact rendered clip, review it on your channel, then choose whether to make it
          public.
        </p>
      </CardHeader>
      <CardContent className="space-y-5 px-5 py-5">
        {(error || ledger.error || connection.error) && (
          <div
            role="alert"
            className="rounded-lg border border-[var(--cm-glass-border)] p-3 text-xs text-[var(--cm-danger-ink)]"
          >
            <p>{error || ledger.error || connection.error}</p>
            <Button
              className="mt-2"
              size="sm"
              variant="ghost"
              disabled={pending}
              onClick={() => {
                ledger.refresh();
                connection.refresh();
              }}
            >
              <RefreshCw className="size-3.5" /> Refresh saved status
            </Button>
          </div>
        )}
        {!connection.status && !connection.error && (
          <p role="status" className="flex items-center gap-2 text-xs text-[var(--cm-text-muted)]">
            <Spinner /> Checking channel connection…
          </p>
        )}
        {connection.status && !connection.status.available && (
          <p className="text-xs text-[var(--cm-warning-ink)]">
            Secure channel credentials are unavailable on this platform. You can still export and
            upload the file manually.
          </p>
        )}
        {connection.status?.available && channels.length === 0 && (
          <div className="flex flex-wrap items-center justify-between gap-4 rounded-xl border border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] p-4">
            <div>
              <p className="text-sm font-medium">Connect your channel</p>
              <p className="mt-1 text-xs text-[var(--cm-text-secondary)]">
                Set up your desktop client and sign in from Settings.
              </p>
            </div>
            {onSetup && (
              <Button size="sm" variant="outline" onClick={onSetup}>
                Channel settings
              </Button>
            )}
          </div>
        )}
        {!ready && (
          <p className="text-xs text-[var(--cm-text-secondary)]">
            Export this clip first. Its completed, immutable render will appear here for review and
            upload.
          </p>
        )}
        {ready && channels.length > 0 && connection.status?.available && (
          <div className="space-y-2">
            <Label htmlFor="youtube-upload-channel">Destination channel</Label>
            <Select
              value={channel?.connectionId ?? ''}
              disabled={pending}
              onValueChange={setSelectedConnectionId}
            >
              <SelectTrigger id="youtube-upload-channel" className="w-full">
                <SelectValue placeholder="Choose a connected channel" />
              </SelectTrigger>
              <SelectContent>
                {channels.map((item) => (
                  <SelectItem key={item.connectionId} value={item.connectionId}>
                    {item.title || item.channelId}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {channel && (
              <p className="break-all font-mono text-[10px] text-[var(--cm-text-muted)]">
                {channel.channelId}
              </p>
            )}
            {currentUploads.length > 0 && (
              <p className="text-xs text-[var(--cm-text-secondary)]">
                This rendered revision already has an upload on this channel. Continue with its
                saved status below.
              </p>
            )}
          </div>
        )}
        {ready &&
          ledger.uploads !== null &&
          channels.length > 0 &&
          connection.status?.available &&
          currentUploads.length === 0 && (
            <MetadataForm
              key={exportJobId}
              api={api}
              exportJobId={exportJobId}
              revision={revision}
              renderArtifactId={renderArtifactId}
              currentRevision={currentRevision}
              channel={channel}
              disabled={pending || Boolean(ledger.error || connection.error)}
              onStart={(request) => void run(() => api.startYoutubeUpload(request))}
            />
          )}
        {ledger.uploads?.map((record) => (
          <UploadRecord
            key={record.uploadId}
            record={record}
            disabled={pending || Boolean(ledger.error)}
            onAction={(action) =>
              void run(() =>
                action === 'publish'
                  ? api.publishYoutubeUpload(record.uploadId)
                  : api.updateYoutubeUpload(record.uploadId, action),
              )
            }
            onOpen={() => void open(record.uploadId)}
            onSetup={onSetup}
          />
        ))}
        <p className="flex items-start gap-2 text-[11px] leading-relaxed text-[var(--cm-text-muted)]">
          <ShieldCheck className="mt-0.5 size-3.5 shrink-0" />
          Uploads use your rendered video and approved metadata. They do not enable cloud analysis
          or publish automatically.
        </p>
      </CardContent>
    </Card>
  );
}

function MetadataForm({
  api,
  exportJobId,
  revision,
  renderArtifactId,
  currentRevision,
  channel,
  disabled,
  onStart,
}: {
  readonly api: PublishingApi;
  readonly exportJobId: string;
  readonly revision: number;
  readonly renderArtifactId: string;
  readonly currentRevision: number | null;
  readonly channel: YoutubeConnection | undefined;
  readonly disabled: boolean;
  readonly onStart: (request: StartYoutubeUpload) => void;
}) {
  const [draft, setDraft] = useState<YoutubeMetadataDraft | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [refresh, setRefresh] = useState(0);
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [tags, setTags] = useState('');
  const [audience, setAudience] = useState('');
  const [synthetic, setSynthetic] = useState('');
  const [confirmed, setConfirmed] = useState(false);
  const dirty = useRef(false);
  useEffect(() => {
    let live = true;
    setError(null);
    void api
      .draftYoutubeMetadata(exportJobId, revision)
      .then((value) => {
        if (!live) return;
        if (value.revision !== revision || value.renderArtifactId !== renderArtifactId)
          throw new Error(
            'The metadata draft belongs to a different rendered revision. Export again before uploading.',
          );
        setDraft(value);
        if (!dirty.current) {
          setTitle(value.metadata.title);
          setDescription(value.metadata.description);
          setTags(value.metadata.tags.join(', '));
        }
      })
      .catch((cause: unknown) => {
        if (live) setError(publishingError(cause));
      });
    return () => {
      live = false;
    };
  }, [api, exportJobId, revision, renderArtifactId, refresh]);
  useEffect(() => {
    setConfirmed(false);
  }, [channel?.channelId, channel?.title]);
  const tagList = parseTags(tags);
  const limits = metadataLimits(title, description, tagList);
  const metadataValid = limits.problem === null;
  const editable = (setter: (value: string) => void, value: string) => {
    dirty.current = true;
    setConfirmed(false);
    setter(value);
  };
  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <p className="text-sm font-semibold">Prepare rendered revision r{revision}</p>
        <span className="text-[11px] text-[var(--cm-text-muted)]">
          Editable metadata · grounded in this clip
        </span>
      </div>
      {currentRevision !== null && currentRevision !== revision && (
        <p className="rounded-lg border border-[var(--cm-glass-border)] p-3 text-xs text-[var(--cm-warning-ink)]">
          Your current edit is r{currentRevision}. This upload uses the rendered r{revision} shown
          above. Export again to include newer edits.
        </p>
      )}
      {error && (
        <div role="alert" className="text-xs text-[var(--cm-danger-ink)]">
          {error}
          <Button
            className="ml-2"
            size="sm"
            variant="ghost"
            onClick={() => setRefresh((value) => value + 1)}
          >
            Retry draft
          </Button>
        </div>
      )}
      {!draft && !error && (
        <p role="status" className="flex items-center gap-2 text-xs text-[var(--cm-text-muted)]">
          <Spinner /> Reading the rendered clip’s saved metadata…
        </p>
      )}
      <div className="space-y-2">
        <div className="flex justify-between gap-2">
          <Label htmlFor="youtube-title">Video title</Label>
          <span className="font-mono text-[10px] text-[var(--cm-text-muted)]">
            {limits.titleCharacters}/100
          </span>
        </div>
        <Input
          id="youtube-title"
          value={title}
          disabled={disabled}
          onChange={(event) => editable(setTitle, event.target.value)}
        />
      </div>
      <div className="space-y-2">
        <Label htmlFor="youtube-description">Description &amp; hashtags</Label>
        <textarea
          id="youtube-description"
          rows={5}
          value={description}
          disabled={disabled}
          onChange={(event) => editable(setDescription, event.target.value)}
          className="w-full resize-y rounded-md border border-input bg-transparent px-3 py-2 text-sm leading-relaxed shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:opacity-50"
        />
        <p className="text-[11px] text-[var(--cm-text-muted)]">
          {limits.descriptionBytes.toLocaleString()}/5,000 UTF-8 bytes. Keep claims accurate to this
          clip. Add relevant hashtags here if you want them in the description.
        </p>
      </div>
      <div className="space-y-2">
        <Label htmlFor="youtube-tags">Tags</Label>
        <Input
          id="youtube-tags"
          value={tags}
          disabled={disabled}
          onChange={(event) => editable(setTags, event.target.value)}
          placeholder="Separate tags with commas"
        />
        <p className="text-[11px] text-[var(--cm-text-muted)]">
          {limits.tagBytes}/500 UTF-8 bytes including separators. Optional. Tags are metadata;
          hashtags belong in the description.
        </p>
      </div>
      {draft?.transcriptExcerpt && (
        <details className="rounded-lg border border-[var(--cm-glass-border)] p-3">
          <summary className="cursor-pointer text-xs font-medium">
            Check the clip’s transcript
          </summary>
          <p className="mt-3 whitespace-pre-wrap text-xs leading-relaxed text-[var(--cm-text-secondary)]">
            {draft.transcriptExcerpt}
          </p>
        </details>
      )}
      <div className="grid gap-4 md:grid-cols-2">
        <div className="space-y-2">
          <Label htmlFor="youtube-audience">Audience</Label>
          <Select
            value={audience}
            disabled={disabled}
            onValueChange={(value) => {
              setAudience(value);
              setConfirmed(false);
            }}
          >
            <SelectTrigger id="youtube-audience" className="w-full">
              <SelectValue placeholder="Choose audience" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="not_kids">Not made for kids</SelectItem>
              <SelectItem value="kids">Made for kids</SelectItem>
            </SelectContent>
          </Select>
        </div>
        <div className="space-y-2">
          <Label htmlFor="youtube-synthetic">Realistic altered or synthetic content</Label>
          <Select
            value={synthetic}
            disabled={disabled}
            onValueChange={(value) => {
              setSynthetic(value);
              setConfirmed(false);
            }}
          >
            <SelectTrigger id="youtube-synthetic" className="w-full">
              <SelectValue placeholder="Choose disclosure" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="no">No</SelectItem>
              <SelectItem value="yes">Yes</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>
      <Label
        htmlFor="youtube-upload-approved"
        className="flex items-start gap-2.5 text-xs leading-relaxed"
      >
        <Checkbox
          id="youtube-upload-approved"
          checked={confirmed}
          disabled={disabled || !draft || !channel}
          onCheckedChange={(value) => setConfirmed(value === true)}
          className="mt-0.5"
        />
        <span>
          I reviewed rendered r{revision} and these details, and have permission to upload this clip
          to {channel?.title || 'the selected channel'} privately.
        </span>
      </Label>
      {!metadataValid && (title || description || tags) && (
        <p role="status" className="text-xs text-[var(--cm-warning-ink)]">
          {limits.problem}
        </p>
      )}
      <Button
        disabled={
          disabled ||
          !draft ||
          Boolean(error) ||
          !metadataValid ||
          !channel ||
          !confirmed ||
          !audience ||
          !synthetic
        }
        onClick={() => {
          if (channel)
            onStart({
              exportJobId,
              connectionId: channel.connectionId,
              expectedRevision: revision,
              rightsConfirmed: confirmed,
              metadata: {
                title: title.trim(),
                description,
                tags: tagList,
                madeForKids: audience === 'kids',
                containsSyntheticMedia: synthetic === 'yes',
              },
            });
        }}
      >
        {disabled ? <Spinner /> : <Upload className="size-4" />} Upload r{revision} privately
      </Button>
    </div>
  );
}

export function UploadRecord({
  record,
  disabled,
  onAction,
  onOpen,
  onSetup,
}: {
  readonly record: YoutubeUpload;
  readonly disabled: boolean;
  readonly onAction: (action: 'pause' | 'resume' | 'reconcile' | 'publish') => void;
  readonly onOpen: () => void;
  readonly onSetup?: (() => void) | undefined;
}) {
  const [confirming, setConfirming] = useState(false);
  const unlisted = record.visibility === 'unlisted' && ['private', 'public'].includes(record.state);
  const expiredIncomplete = record.state === 'failed' && record.errorCode === 'session_expired';
  const detail = unlisted
    ? {
        label: 'Unlisted on YouTube',
        detail:
          'Anyone with the video link can watch it. Choose Publish to make the existing video public.',
      }
    : expiredIncomplete
      ? {
          label: 'Incomplete upload expired',
          detail:
            'The upload session expired before the complete video was sent. Restart sends the same saved revision through a new session.',
        }
      : UPLOAD_STATES[record.state];
  const active = uploadActive(record.state);
  const percent =
    record.totalBytes > 0
      ? Math.min(100, (record.acknowledgedBytes / record.totalBytes) * 100)
      : null;
  const pause = ['queued', 'verifying', 'starting', 'uploading', 'reconciling'].includes(
    record.state,
  );
  return (
    <section
      aria-label={`YouTube upload r${record.revision}`}
      className="space-y-3 rounded-xl border border-[var(--cm-glass-border)] p-4"
    >
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0">
          <p className="break-words text-sm font-medium">{record.metadata.title}</p>
          <p className="mt-1 text-[11px] text-[var(--cm-text-muted)]">
            {record.channelTitle || record.channelId} · Rendered r{record.revision}
          </p>
        </div>
        <StatusBadge
          tone={
            record.state === 'public' || record.state === 'private'
              ? 'success'
              : active
                ? 'progress'
                : 'warning'
          }
        >
          {detail.label}
        </StatusBadge>
      </div>
      <p role="status" className="text-xs leading-relaxed text-[var(--cm-text-secondary)]">
        {detail.detail}
      </p>
      {record.totalBytes > 0 && (
        <div>
          <p className="font-mono text-[10px] text-[var(--cm-text-muted)]">
            {formatBytes(record.acknowledgedBytes)} of {formatBytes(record.totalBytes)} received by
            YouTube
          </p>
          {active && percent !== null && (
            <div
              role="progressbar"
              aria-label="Bytes received by YouTube"
              aria-valuemin={0}
              aria-valuemax={100}
              aria-valuenow={percent}
              className="mt-2 h-1.5 overflow-hidden rounded-full bg-[var(--cm-recessed)]"
            >
              <div className="h-full bg-[var(--color-primary)]" style={{ width: `${percent}%` }} />
            </div>
          )}
        </div>
      )}
      {record.error && (
        <p className="text-xs leading-relaxed text-[var(--cm-danger-ink)]">{record.error}</p>
      )}
      <div className="flex flex-wrap gap-2">
        {pause && (
          <Button size="sm" variant="outline" disabled={disabled} onClick={() => onAction('pause')}>
            <Pause className="size-3.5" /> Pause upload
          </Button>
        )}
        {['paused', 'failed', 'auth_required'].includes(record.state) && (
          <Button
            size="sm"
            variant="outline"
            disabled={disabled}
            onClick={() => onAction('resume')}
          >
            <Play className="size-3.5" />{' '}
            {expiredIncomplete ? 'Restart incomplete upload' : 'Resume existing upload'}
          </Button>
        )}
        {record.state === 'completion_uncertain' && (
          <Button
            size="sm"
            variant="outline"
            disabled={disabled}
            onClick={() => onAction('reconcile')}
          >
            <RefreshCw className="size-3.5" /> Check existing upload
          </Button>
        )}
        {record.state === 'auth_required' && onSetup && (
          <Button size="sm" variant="ghost" onClick={onSetup}>
            Reconnect channel
          </Button>
        )}
        {record.videoId && (
          <Button size="sm" variant="outline" disabled={disabled} onClick={onOpen}>
            <ExternalLink className="size-3.5" /> Review in YouTube Studio
          </Button>
        )}
        {record.state === 'private' && record.videoId && !confirming && (
          <Button size="sm" disabled={disabled} onClick={() => setConfirming(true)}>
            <Send className="size-3.5" /> Publish…
          </Button>
        )}
      </div>
      {record.state === 'private' && confirming && (
        <div className="space-y-3 rounded-lg border border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] p-3">
          <p className="text-xs leading-relaxed">
            Make “{record.metadata.title}” public on {record.channelTitle || record.channelId}? This
            publishes the existing uploaded revision r{record.revision}.
          </p>
          <p className="text-[11px] text-[var(--cm-text-muted)]">
            Google may restrict your API project to private uploads until its upload audit is
            complete. The confirmed remote status is shown here.
          </p>
          <div className="flex gap-2">
            <Button size="sm" disabled={disabled} onClick={() => onAction('publish')}>
              <Check className="size-3.5" /> Publish publicly
            </Button>
            <Button
              size="sm"
              variant="ghost"
              disabled={disabled}
              onClick={() => setConfirming(false)}
            >
              {unlisted ? 'Keep unlisted' : 'Keep private'}
            </Button>
          </div>
        </div>
      )}
    </section>
  );
}
