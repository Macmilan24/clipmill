import { useEffect, useRef, useState } from 'react';
import { History, RefreshCw } from 'lucide-react';
import { Button } from '../components/ui/button.js';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card.js';
import { Spinner } from '../components/ui/spinner.js';
import type { ShellApi } from '../daemon/api.js';
import type { YoutubeUpload } from '../daemon/publishing.js';
import { publishingError } from './model.js';
import { UploadRecord } from './UploadPanel.js';
import { useUploads } from './useUploads.js';

/** Remote receipts outlive their source project and remain explicitly recoverable here. */
export function UploadHistory({ api }: { readonly api: ShellApi }) {
  const ledger = useUploads(api, null, null);
  const [projectNames, setProjectNames] = useState<ReadonlyMap<string, string> | null>(null);
  const [refreshId, setRefreshId] = useState(0);
  const [expanded, setExpanded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const operating = useRef(false);
  const generation = useRef(0);
  useEffect(() => {
    const mine = ++generation.current;
    void api
      .listProjects()
      .then((projects) => {
        if (mine === generation.current)
          setProjectNames(new Map(projects.map((project) => [project.projectId, project.name])));
      })
      .catch(() => {
        if (mine === generation.current) setProjectNames(null);
      });
    return () => {
      generation.current += 1;
    };
  }, [api, refreshId]);
  const refresh = () => {
    ledger.refresh();
    setRefreshId((value) => value + 1);
  };
  const run = async (action: () => Promise<YoutubeUpload | void>) => {
    if (operating.current) return;
    operating.current = true;
    setPending(true);
    setError(null);
    ledger.refresh();
    const mine = generation.current;
    try {
      const record = await action();
      if (mine === generation.current && record) ledger.accept(record);
    } catch (cause) {
      if (mine === generation.current) setError(publishingError(cause));
    } finally {
      operating.current = false;
      if (mine === generation.current) {
        setPending(false);
        ledger.refresh();
      }
    }
  };
  const rows = (ledger.uploads ?? []).toSorted(
    (left, right) => right.updatedUnixMillis - left.updatedUnixMillis,
  );
  return (
    <Card className="gap-0 overflow-hidden py-0">
      <CardHeader className="border-b border-[var(--cm-glass-border)] px-5 py-4">
        <div className="flex items-center justify-between gap-3">
          <CardTitle className="flex items-center gap-2 text-sm">
            <History className="size-4" /> Recent uploads
          </CardTitle>
          <Button size="sm" variant="ghost" disabled={pending} onClick={refresh}>
            <RefreshCw className="size-3.5" /> Refresh uploads
          </Button>
        </div>
        <p className="text-xs leading-relaxed text-[var(--cm-text-secondary)]">
          Saved YouTube receipts remain here even after a source project is removed. Opening this
          list does not contact YouTube.
        </p>
      </CardHeader>
      <CardContent className="space-y-4 px-5 py-5">
        {(error || ledger.error) && (
          <p
            role="alert"
            className="rounded-lg border border-[var(--cm-glass-border)] p-3 text-xs text-[var(--cm-danger-ink)]"
          >
            {error || ledger.error}
          </p>
        )}
        {ledger.uploads === null && !ledger.error && (
          <p role="status" className="flex items-center gap-2 text-xs text-[var(--cm-text-muted)]">
            <Spinner /> Reading saved uploads…
          </p>
        )}
        {ledger.uploads?.length === 0 && (
          <p className="text-xs text-[var(--cm-text-secondary)]">
            No uploads yet. Export a clip, review its rendered video, and choose Upload privately.
          </p>
        )}
        {(expanded ? rows : rows.slice(0, 10)).map((record) => (
          <div key={record.uploadId} className="space-y-2">
            <p className="text-[11px] text-[var(--cm-text-muted)]">
              {projectNames?.get(record.projectId) ??
                (projectNames
                  ? 'Source project removed · upload receipt retained'
                  : 'Saved upload receipt')}
            </p>
            <UploadRecord
              record={record}
              disabled={pending || Boolean(ledger.error)}
              onAction={(action) =>
                void run(() =>
                  action === 'publish'
                    ? api.publishYoutubeUpload(record.uploadId)
                    : api.updateYoutubeUpload(record.uploadId, action),
                )
              }
              onOpen={() => void run(() => api.openYoutubePage('studio', record.uploadId))}
            />
          </div>
        ))}
        {rows.length > 10 && (
          <Button size="sm" variant="ghost" onClick={() => setExpanded((value) => !value)}>
            {expanded ? 'Show recent 10' : `Show all ${rows.length} uploads`}
          </Button>
        )}
      </CardContent>
    </Card>
  );
}
