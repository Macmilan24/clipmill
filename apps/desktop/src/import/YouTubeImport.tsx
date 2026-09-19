import { useCallback, useEffect, useRef, useState } from 'react';
import { ArrowDownToLine, Check, Link, RotateCcw, Video, X } from 'lucide-react';
import { StatusBadge } from '../components/StatusBadge.js';
import { Button } from '../components/ui/button.js';
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
import type { YoutubeImport as ImportRecord } from '../daemon/client.js';
import { formatBytes } from '../deviceProfile.js';
import type { ChosenSource, ImportLoader } from './loader.js';
import {
  activeImport,
  importHeight,
  IMPORT_STATES,
  newestImport,
  recallYoutube,
  rememberYoutube,
  youtubeVideo,
} from './youtube.js';

function messageOf(error: unknown): string {
  const detail = error instanceof Error ? error.message : typeof error === 'string' ? error : '';
  return detail.trim() || 'The import could not be updated. Try again.';
}

export function YouTubeImport({
  importer,
  connected,
  disabled,
  onChosen,
}: {
  readonly importer: ImportLoader;
  readonly connected: boolean;
  readonly disabled: boolean;
  readonly onChosen: (source: ChosenSource | null) => void;
}) {
  const [saved] = useState(recallYoutube);
  const [selection, setSelection] = useState(saved);
  const [url, setUrl] = useState(saved ? `https://www.youtube.com/watch?v=${saved.videoId}` : '');
  const [maxHeight, setMaxHeight] = useState(() => importHeight(saved));
  const [rights, setRights] = useState(false);
  const [record, setRecord] = useState<ImportRecord | null>(null);
  const [recent, setRecent] = useState<readonly ImportRecord[]>([]);
  const [problem, setProblem] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [opening, setOpening] = useState(false);
  const [refresh, setRefresh] = useState(0);
  const alive = useRef(true);
  const generation = useRef(0);
  const selectedId = useRef(saved?.importId ?? null);
  const operating = useRef(false);
  const api = importer.api;
  const parsed = youtubeVideo(url);
  const active = record !== null && activeImport(record.state);
  const matchesRecord =
    record !== null && parsed?.videoId === record.videoId && maxHeight === importHeight(record);

  useEffect(() => {
    alive.current = true;
    return () => {
      alive.current = false;
      generation.current += 1;
    };
  }, []);

  const accept = useCallback((next: ImportRecord) => {
    setRecord((old) => newestImport(old, next));
    setRecent((old) => [
      newestImport(old.find((item) => item.importId === next.importId) ?? null, next),
      ...old.filter((item) => item.importId !== next.importId),
    ]);
  }, []);

  const select = useCallback(
    (next: ImportRecord) => {
      const changed = selectedId.current !== next.importId;
      if (changed) generation.current += 1;
      selectedId.current = next.importId;
      const chosen = {
        importId: next.importId,
        projectId: next.projectId,
        videoId: next.videoId,
        maxHeight: importHeight(next),
      };
      rememberYoutube(chosen);
      setSelection(chosen);
      setRecord((old) => newestImport(old, next));
      setUrl(next.canonicalUrl);
      setMaxHeight(importHeight(next));
      setProblem(null);
      if (changed) {
        setOpening(false);
        onChosen(null);
      }
    },
    [onChosen],
  );

  useEffect(() => {
    if (!connected) return;
    let live = true;
    const mine = generation.current;
    void api
      .listYoutubeImports()
      .then((items) => {
        if (!live) return;
        setRecent(items);
        if (mine !== generation.current || record !== null || !selection) return;
        const restored = items.find(
          (item) =>
            importHeight(item) === importHeight(selection) &&
            (selection.importId
              ? item.importId === selection.importId
              : item.projectId === selection.projectId && item.videoId === selection.videoId),
        );
        if (restored) select(restored);
      })
      .catch((error: unknown) => {
        if (live) setProblem(messageOf(error));
      });
    return () => {
      live = false;
    };
    // This restores an explicit saved identity; it never picks the newest import.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [api, connected, refresh]);

  useEffect(() => {
    if (!connected || !selection?.importId) return;
    let live = true;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const mine = generation.current;
    const id = selection.importId;
    const poll = async () => {
      try {
        const next = await api.getYoutubeImport(id);
        if (!live || mine !== generation.current) return;
        accept(next);
        setProblem(null);
        if (activeImport(next.state))
          timer = setTimeout(() => {
            void poll();
          }, 2_000);
      } catch (error) {
        if (!live || mine !== generation.current) return;
        setProblem(`Status could not be refreshed: ${messageOf(error)}`);
      }
    };
    void poll();
    return () => {
      live = false;
      if (timer) clearTimeout(timer);
    };
  }, [api, connected, selection?.importId, record?.attempt, refresh, accept]);

  useEffect(() => {
    if (!record || record.state !== 'completed' || !matchesRecord) return;
    let live = true;
    setOpening(true);
    void importer
      .imported(record)
      .then((source) => {
        if (live) onChosen(source);
      })
      .catch((error: unknown) => {
        if (live) setProblem(messageOf(error));
      })
      .finally(() => {
        if (live) setOpening(false);
      });
    return () => {
      live = false;
    };
  }, [
    importer,
    record?.importId,
    record?.sourceId,
    record?.state,
    matchesRecord,
    onChosen,
    refresh,
  ]);

  const begin = async () => {
    if (!parsed || !rights || operating.current || active || !connected || disabled) return;
    operating.current = true;
    setPending(true);
    setProblem(null);
    const mine = ++generation.current;
    selectedId.current = null;
    onChosen(null);
    setRecord(null);
    setOpening(false);
    try {
      const projectId =
        selection?.videoId === parsed.videoId && importHeight(selection) === maxHeight
          ? selection.projectId
          : await api.createProject(`YouTube · ${parsed.videoId}`);
      if (!alive.current || mine !== generation.current) return;
      const nextSelection = { projectId, videoId: parsed.videoId, maxHeight };
      rememberYoutube(nextSelection);
      if (alive.current && mine === generation.current) setSelection(nextSelection);
      const next = await api.startYoutubeImport(projectId, parsed.url, true, maxHeight);
      if (alive.current && mine === generation.current) {
        select(next);
        accept(next);
      }
    } catch (error) {
      if (alive.current && mine === generation.current) setProblem(messageOf(error));
    } finally {
      operating.current = false;
      if (alive.current) setPending(false);
    }
  };

  const update = async (action: 'cancel' | 'retry') => {
    if (!record || operating.current || !connected || disabled) return;
    operating.current = true;
    setPending(true);
    setProblem(null);
    const mine = ++generation.current;
    try {
      const next = await api.updateYoutubeImport(record.importId, action);
      if (alive.current && mine === generation.current) {
        accept(next);
        setRefresh((value) => value + 1);
      }
    } catch (error) {
      if (alive.current && mine === generation.current) setProblem(messageOf(error));
    } finally {
      operating.current = false;
      if (alive.current) setPending(false);
    }
  };

  const detail = record ? IMPORT_STATES[record.state] : null;
  const total = record?.totalBytes;
  const percent =
    record && total && total > 0 ? Math.min(100, (record.downloadedBytes / total) * 100) : null;
  return (
    <div className="space-y-4">
      <div className="rounded-xl border border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] p-4">
        <div className="mb-4 flex items-start gap-3">
          <div className="grid size-9 shrink-0 place-items-center rounded-lg border border-[var(--cm-glass-border)] bg-[var(--cm-surface-1)]">
            <Video className="size-4" />
          </div>
          <div>
            <h3 className="text-sm font-medium">Import a YouTube video</h3>
            <p className="mt-1 text-xs leading-relaxed text-[var(--cm-text-secondary)]">
              Download one public video, then create clips from a local copy.
            </p>
          </div>
        </div>
        <form
          onSubmit={(event) => {
            event.preventDefault();
            void begin();
          }}
          className="space-y-3"
        >
          <Label htmlFor="youtube-url">Video link</Label>
          <div className="relative">
            <Link className="pointer-events-none absolute left-3 top-3 size-4 text-[var(--cm-text-muted)]" />
            <Input
              id="youtube-url"
              type="url"
              maxLength={2048}
              placeholder="https://www.youtube.com/watch?v=…"
              className="pl-9"
              value={url}
              disabled={pending || active || disabled}
              onChange={(event) => {
                const nextUrl = event.target.value;
                setUrl(nextUrl);
                setRights(false);
                if (youtubeVideo(nextUrl)?.videoId !== (record?.videoId ?? selection?.videoId)) {
                  generation.current += 1;
                  rememberYoutube(null);
                  setOpening(false);
                  onChosen(null);
                }
              }}
              aria-describedby="youtube-url-help"
            />
          </div>
          <p id="youtube-url-help" className="text-xs leading-relaxed text-[var(--cm-text-muted)]">
            {url.trim() && !parsed
              ? 'Use one HTTPS YouTube video link.'
              : 'Video links only. Playlists, private videos and sign-in are not supported.'}
          </p>
          <div className="space-y-2">
            <Label htmlFor="youtube-quality">Import quality</Label>
            <Select
              value={String(maxHeight)}
              disabled={pending || active || disabled}
              onValueChange={(value) => {
                const height = Number(value);
                if (height === maxHeight) return;
                setMaxHeight(height);
                generation.current += 1;
                rememberYoutube(null);
                setOpening(false);
                onChosen(null);
              }}
            >
              <SelectTrigger id="youtube-quality" className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="1080">1080p · Best quality</SelectItem>
                <SelectItem value="720">720p · Smaller download</SelectItem>
                <SelectItem value="360">360p · Quick test / smallest download</SelectItem>
              </SelectContent>
            </Select>
            <p className="text-[11px] leading-relaxed text-[var(--cm-text-muted)]">
              Downloads up to this resolution when available. Lower quality saves space and is
              useful for testing.
              {record && parsed?.videoId === record.videoId && maxHeight !== importHeight(record)
                ? ' Changing quality creates a separate project and downloads another copy.'
                : ''}
            </p>
          </div>
          <Label
            htmlFor="youtube-rights"
            className="flex items-start gap-2.5 text-xs leading-relaxed"
          >
            <Checkbox
              id="youtube-rights"
              checked={rights}
              disabled={pending || active || disabled}
              className="mt-0.5"
              onCheckedChange={(value) => setRights(value === true)}
            />
            <span>
              I own this video or have the creator’s permission to download and use it in my clips.
            </span>
          </Label>
          <p className="text-[11px] leading-relaxed text-[var(--cm-text-secondary)]">
            Import contacts YouTube and downloads video and audio. It does not connect your account
            or enable cloud AI.
          </p>
          <Button
            type="submit"
            size="sm"
            disabled={!parsed || !rights || pending || active || disabled || !connected}
          >
            {pending ? <Spinner /> : <ArrowDownToLine className="size-4" />} Import video
          </Button>
        </form>
      </div>

      {problem && (
        <div
          role="alert"
          className="rounded-lg border border-[var(--cm-glass-border)] p-3 text-xs text-[var(--cm-danger-ink)]"
        >
          <p>{problem}</p>
          <Button
            size="sm"
            variant="ghost"
            className="mt-2"
            disabled={!connected || pending}
            onClick={() => setRefresh((value) => value + 1)}
          >
            Refresh import status
          </Button>
        </div>
      )}
      {!connected && (
        <p role="status" className="text-xs text-[var(--cm-warning-ink)]">
          Reconnect to the daemon to check this import. The last saved status is shown below.
        </p>
      )}
      {record && detail && matchesRecord && (
        <section
          aria-label="YouTube import status"
          className="rounded-xl border border-[var(--cm-glass-border)] p-4"
        >
          <div className="flex items-start justify-between gap-3">
            <div className="min-w-0">
              <p className="break-words text-sm font-medium">
                {record.title || `YouTube video · ${record.videoId}`}
              </p>
              <p className="mt-1 text-[11px] text-[var(--cm-text-muted)]">
                {record.channel ? `${record.channel} · ` : ''}
                {importHeight(record)}p max
                {record.attempt > 0 ? ` · Attempt ${record.attempt + 1}` : ''}
              </p>
            </div>
            <StatusBadge
              tone={
                record.state === 'completed'
                  ? 'success'
                  : active
                    ? 'progress'
                    : record.state === 'failed'
                      ? 'danger'
                      : 'warning'
              }
            >
              {detail.label}
            </StatusBadge>
          </div>
          <p role="status" className="mt-3 text-xs leading-relaxed text-[var(--cm-text-secondary)]">
            {opening ? 'Opening the imported video…' : detail.detail}
          </p>
          {(active || record.downloadedBytes > 0) && (
            <div className="mt-3">
              <p className="font-mono text-[11px] text-[var(--cm-text-muted)]">
                {formatBytes(record.downloadedBytes)} downloaded
                {total && total > 0
                  ? ` of ${formatBytes(total)}`
                  : active
                    ? ' · total size not yet known'
                    : ''}
              </p>
              {active && percent !== null && (
                <div
                  role="progressbar"
                  aria-label="Downloaded bytes"
                  aria-valuemin={0}
                  aria-valuemax={100}
                  aria-valuenow={percent}
                  className="mt-2 h-1.5 overflow-hidden rounded-full bg-[var(--cm-recessed)]"
                >
                  <div
                    className="h-full bg-[var(--color-primary)]"
                    style={{ width: `${percent}%` }}
                  />
                </div>
              )}
            </div>
          )}
          {record.error && (
            <p className="mt-3 text-xs leading-relaxed text-[var(--cm-danger-ink)]">
              {record.error}
            </p>
          )}
          <div className="mt-4 flex flex-wrap items-center gap-2">
            {active ? (
              <Button
                size="sm"
                variant="outline"
                disabled={pending || !connected || disabled}
                onClick={() => {
                  void update('cancel');
                }}
              >
                <X className="size-3.5" /> Cancel import
              </Button>
            ) : record.state !== 'completed' ? (
              <Button
                size="sm"
                variant="outline"
                disabled={pending || !connected || disabled}
                onClick={() => {
                  void update('retry');
                }}
              >
                <RotateCcw className="size-3.5" /> Retry download
              </Button>
            ) : (
              <span className="flex items-center gap-1.5 text-xs text-[var(--cm-success-ink)]">
                <Check className="size-3.5" /> Local copy saved
              </span>
            )}
          </div>
        </section>
      )}

      {recent.some((item) => !matchesRecord || item.importId !== record?.importId) && (
        <div className="border-t border-[var(--cm-glass-border)] pt-4">
          <p className="mb-2 text-[11px] font-medium uppercase tracking-wider text-[var(--cm-text-muted)]">
            Recent imports
          </p>
          <div className="space-y-1">
            {recent
              .filter((item) => !matchesRecord || item.importId !== record?.importId)
              .slice(0, 5)
              .map((item) => (
                <button
                  key={item.importId}
                  type="button"
                  disabled={pending || disabled}
                  onClick={() => select(item)}
                  className="flex w-full items-center justify-between gap-3 rounded-lg px-2 py-2 text-left hover:bg-[var(--cm-recessed)] focus-visible:outline-2 focus-visible:outline-[var(--color-primary)]"
                >
                  <span className="min-w-0 truncate text-xs">{item.title || item.videoId}</span>
                  <span className="shrink-0 text-[10px] text-[var(--cm-text-muted)]">
                    {importHeight(item)}p · {IMPORT_STATES[item.state].label}
                  </span>
                </button>
              ))}
          </div>
        </div>
      )}
    </div>
  );
}
