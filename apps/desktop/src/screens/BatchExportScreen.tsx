/** Collection delivery keeps each reviewed revision and each job independent. */
import { useCallback, useEffect, useRef, useState } from 'react';
import { ArrowLeft, Check, FolderOpen, Layers, RefreshCw, X } from 'lucide-react';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Progress } from '@/components/ui/progress';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Spinner } from '@/components/ui/spinner';
import { type ShellApi, daemonApi } from '../daemon/api.js';
import type { ExportBatch, ExportBatchItem } from '../daemon/client.js';
import { useEditDocuments } from '../editor/documents.js';
import { deliveryProgressText, deliveryWaitText, useDelivery } from '../export/delivery.js';
import {
  approvalScope,
  collectionPattern,
  confirmationGates,
  EMPTY_CHOICES,
  HOT_CAPTION_CODE,
  sourceDuration,
  type ClipChoices,
  type PreparedClip,
} from '../export/batchPreparation.js';
import { formatBytes } from '../deviceProfile.js';
import type { ClipRef } from '../shell/route.js';

export interface BatchExportScreenProps {
  readonly api?: ShellApi;
  readonly onBack?: () => void;
  readonly onEdit?: (clip: ClipRef) => void;
}

type CheckResult = { readonly prepared?: PreparedClip; readonly error?: string };
const messageOf = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));

export function BatchExportScreen({ api = daemonApi, onBack, onEdit }: BatchExportScreenProps) {
  const documents = useEditDocuments(api);
  const [selected, setSelected] = useState<readonly string[]>([]);
  const [choices, setChoices] = useState<Readonly<Record<string, ClipChoices>>>({});
  const [folder, setFolder] = useState('');
  const [pattern, setPattern] = useState('{index}-{clip}');
  const [checks, setChecks] = useState<Readonly<Record<string, CheckResult>>>({});
  const configuration = JSON.stringify([selected, choices, folder, pattern]);
  const [checkedConfiguration, setCheckedConfiguration] = useState('');
  const previousChecks = useRef(checks);
  previousChecks.current = checks;
  const [checking, setChecking] = useState(false);
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const [batches, setBatches] = useState<readonly ExportBatch[]>([]);
  const [historyProblem, setHistoryProblem] = useState<string | null>(null);
  const [savedNotice, setSavedNotice] = useState(false);
  const queueSection = useRef<HTMLElement>(null);
  const [refresh, setRefresh] = useState(0);
  const submitInFlight = useRef(false);
  const batchOperation = useRef(new Set<string>());
  const [acting, setActing] = useState<ReadonlySet<string>>(new Set());
  const planGeneration = useRef(0);

  const putBatch = useCallback((batch: ExportBatch) => {
    setBatches((current) =>
      [batch, ...current.filter((item) => item.batchId !== batch.batchId)].toSorted(
        (a, b) => b.createdUnixMillis - a.createdUnixMillis,
      ),
    );
  }, []);

  useEffect(() => {
    let live = true;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const read = async () => {
      try {
        const found = await api.listExportBatches();
        if (!live) return;
        setBatches(found);
        setHistoryProblem(null);
        // Once jobs exist, their individual delivery readers own progress.
        if (found.some((batch) => batch.items.some((item) => item.state === 'pending')))
          timer = setTimeout(() => void read(), 1500);
      } catch (cause) {
        if (!live) return;
        setHistoryProblem(messageOf(cause));
        timer = setTimeout(() => void read(), 3000);
      }
    };
    void read();
    return () => {
      live = false;
      if (timer) clearTimeout(timer);
    };
  }, [api, refresh]);

  useEffect(() => {
    const generation = ++planGeneration.current;
    if (!selected.length || !folder.trim()) {
      setChecks({});
      setChecking(false);
      return undefined;
    }
    setChecking(true);
    let live = true;
    const timer = setTimeout(() => {
      void (async () => {
        const results = await Promise.all(
          selected.map(async (docId, ordinal) => {
            const entry = documents.entries.find((item) => item.clip.docId === docId);
            if (!entry) return [docId, { error: 'This edit is no longer available.' }] as const;
            const choice = choices[docId] ?? EMPTY_CHOICES;
            try {
              const preview = await api.previewPlan(entry.clip.projectId, docId);
              const opening = preview.cues
                .flatMap((cue) => cue.lines.flat())
                .slice(0, 8)
                .map((word) => word.text)
                .join(' ');
              const request = {
                docId,
                destinationDir: folder.trim(),
                namingPattern: collectionPattern(pattern),
                sourceAttestation: choice.attestation,
                gatesPassed: confirmationGates(
                  choice,
                  previousChecks.current[docId]?.prepared?.plan.revision === preview.revision
                    ? previousChecks.current[docId]?.prepared
                    : undefined,
                ),
                aiAssistance: ['asr_captions', 'reframe'],
                index: ordinal + 1,
                date: localDate(),
                title: choice.title.trim() || opening || entry.clip.labels?.clip || 'Clip',
                expectedRevision: preview.revision,
              };
              const plan = await api.planExport(request);
              if (preview.revision !== plan.revision)
                throw new Error(
                  'The edit changed during these checks. Check it again before exporting.',
                );
              return [
                docId,
                {
                  prepared: {
                    request: { ...request, expectedRevision: plan.revision },
                    plan,
                    durationTicks: sourceDuration(preview.segments),
                    scope: approvalScope(docId, plan, choice.attestation),
                  },
                },
              ] as const;
            } catch (cause) {
              return [docId, { error: messageOf(cause) }] as const;
            }
          }),
        );
        if (live && generation === planGeneration.current) {
          setChecks(Object.fromEntries(results));
          setCheckedConfiguration(configuration);
          setChecking(false);
        }
      })();
    }, 250);
    return () => {
      live = false;
      clearTimeout(timer);
    };
  }, [api, selected, choices, folder, pattern, documents.entries, refresh, configuration]);

  const changeChoice = (docId: string, patch: Partial<ClipChoices>) => {
    setChoices((current) => ({
      ...current,
      [docId]: { ...(current[docId] ?? EMPTY_CHOICES), ...patch },
    }));
  };
  const ready =
    selected.length > 0 &&
    checkedConfiguration === configuration &&
    !checking &&
    !busy &&
    selected.every((docId) => {
      const prepared = checks[docId]?.prepared;
      if (!prepared || !choices[docId]?.attestation || !prepared.plan.passes) return false;
      const choice = choices[docId]!;
      return (
        (prepared.durationTicks <= 60 * 90000 || choice.rightsApproval === prepared.scope) &&
        (!prepared.plan.findings.some((finding) => finding.code === HOT_CAPTION_CODE) ||
          choice.captionsApproval === prepared.scope)
      );
    });
  const exportBatch = async () => {
    if (!ready || submitInFlight.current) return;
    submitInFlight.current = true;
    setBusy(true);
    setProblem(null);
    try {
      const requests = selected.map((docId) => checks[docId]!.prepared!.request);
      putBatch(await api.submitExportBatch(requests));
      setSavedNotice(true);
      setSelected([]);
      setRefresh((count) => count + 1);
    } catch (cause) {
      setProblem(messageOf(cause));
      setChoices((current) =>
        Object.fromEntries(
          Object.entries(current).map(([id, choice]) => [
            id,
            { ...choice, rightsApproval: null, captionsApproval: null },
          ]),
        ),
      );
      setRefresh((count) => count + 1);
    } finally {
      submitInFlight.current = false;
      setBusy(false);
    }
  };
  const act = async (batchId: string, index: number, action: 'retry' | 'cancel') => {
    const key = `${batchId}:${index}`;
    if (batchOperation.current.has(key)) return;
    batchOperation.current.add(key);
    setActing(new Set(batchOperation.current));
    setProblem(null);
    try {
      putBatch(await api.updateExportBatchItem(batchId, index, action));
      // A retry is first persisted as pending; restart collection polling to
      // discover its new queued job, whose delivery reader can then take over.
      setRefresh((count) => count + 1);
    } catch (cause) {
      setProblem(messageOf(cause));
    } finally {
      batchOperation.current.delete(key);
      setActing(new Set(batchOperation.current));
    }
  };
  const chooseFolder = async () => {
    try {
      const chosen = await api.chooseExportFolder();
      if (chosen) setFolder(chosen);
    } catch (cause) {
      setProblem(messageOf(cause));
    }
  };
  const estimate = selected.reduce(
    (sum, id) => sum + (checks[id]?.prepared?.plan.estimatedBytes ?? 0),
    0,
  );

  return (
    <div className="export-page">
      <header className="workspace-heading">
        <div>
          <p className="mb-2 text-xs uppercase tracking-widest text-muted-foreground">
            Delivery workspace
          </p>
          <h1 className="workspace-title">Export a collection</h1>
          <p className="workspace-subtitle mt-1">
            Review your clips together. Each export keeps its own revision, progress and recovery.
          </p>
        </div>
        {onBack && (
          <Button variant="outline" size="sm" onClick={onBack}>
            <ArrowLeft className="size-4" />
            Single clip
          </Button>
        )}
      </header>
      {problem && (
        <Alert variant="destructive">
          <AlertDescription>{problem}</AlertDescription>
        </Alert>
      )}
      {savedNotice && (
        <Alert>
          <AlertDescription className="flex flex-wrap items-center justify-between gap-2">
            <span>Your collection is saved. Each clip’s status appears below.</span>
            <Button
              size="sm"
              variant="ghost"
              onClick={() =>
                queueSection.current?.scrollIntoView({ behavior: 'smooth', block: 'start' })
              }
            >
              View collection
            </Button>
          </AlertDescription>
        </Alert>
      )}
      <div className="grid items-start gap-5 xl:grid-cols-[minmax(0,1fr)_330px]">
        <section className="min-w-0 space-y-4" aria-label="Choose collection clips">
          <div className="flex items-center justify-between gap-3">
            <h2 className="text-base font-medium">Your edits</h2>
            <span className="text-xs text-muted-foreground">{selected.length} selected</span>
          </div>
          {documents.loading && (
            <p className="flex items-center gap-2 text-sm text-muted-foreground">
              <Spinner className="size-4" />
              Loading your edits…
            </p>
          )}
          {documents.problem && (
            <Alert variant="destructive">
              <AlertDescription>{documents.problem}</AlertDescription>
            </Alert>
          )}
          {!documents.loading && !documents.entries.length && (
            <Card>
              <CardContent className="flex flex-col items-center gap-3 py-10 text-center">
                <Layers className="size-7 text-muted-foreground" />
                <p className="font-medium">Your collection starts with an edit</p>
                <p className="max-w-sm text-sm text-muted-foreground">
                  Approve a moment in Results and refine it in the editor. Your saved clips will
                  appear here.
                </p>
              </CardContent>
            </Card>
          )}
          {documents.entries.map((entry) => {
            const id = entry.clip.docId;
            const number = selected.indexOf(id) + 1;
            const choice = choices[id] ?? EMPTY_CHOICES;
            const result = checks[id];
            const prepared = result?.prepared;
            const hot =
              prepared?.plan.findings.filter((finding) => finding.code === HOT_CAPTION_CODE) ?? [];
            return (
              <Card key={id} className={number ? 'border-primary/40' : ''}>
                <CardHeader className="flex flex-row items-start gap-3 space-y-0 pb-3">
                  <Checkbox
                    className="mt-1"
                    aria-label={`Select ${entry.clip.labels?.clip ?? id}`}
                    checked={number > 0}
                    disabled={busy}
                    onCheckedChange={(checked) =>
                      setSelected((current) =>
                        checked ? [...current, id] : current.filter((item) => item !== id),
                      )
                    }
                  />
                  <div className="min-w-0 flex-1">
                    <CardTitle className="text-sm leading-5">
                      {prepared?.request.title ||
                        entry.clip.labels?.clip ||
                        entry.sourceName ||
                        'Untitled clip'}
                    </CardTitle>
                    <p className="mt-1 truncate text-xs text-muted-foreground">
                      {entry.projectName} · r{prepared?.plan.revision ?? entry.revision}
                    </p>
                  </div>
                  {number > 0 && (
                    <Badge variant="secondary">{String(number).padStart(2, '0')}</Badge>
                  )}
                  {onEdit && (
                    <Button variant="ghost" size="sm" onClick={() => onEdit(entry.clip)}>
                      Edit
                    </Button>
                  )}
                </CardHeader>
                {number > 0 && (
                  <CardContent className="space-y-4">
                    <div className="grid gap-3 md:grid-cols-2">
                      <div className="space-y-1.5">
                        <Label htmlFor={`batch-title-${id}`}>Clip title</Label>
                        <Input
                          id={`batch-title-${id}`}
                          value={choice.title}
                          disabled={busy}
                          placeholder={prepared?.request.title || 'Use opening words'}
                          onChange={(event) => changeChoice(id, { title: event.target.value })}
                        />
                      </div>
                      <div className="space-y-1.5">
                        <Label htmlFor={`batch-rights-${id}`}>Source rights</Label>
                        <Select
                          value={choice.attestation}
                          disabled={busy}
                          onValueChange={(attestation) =>
                            changeChoice(id, {
                              attestation,
                              rightsApproval: null,
                              captionsApproval: null,
                            })
                          }
                        >
                          <SelectTrigger id={`batch-rights-${id}`} className="w-full">
                            <SelectValue placeholder="Choose your permission" />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="own_content">I own this footage</SelectItem>
                            <SelectItem value="licensed_content">
                              I have permission or a license
                            </SelectItem>
                            <SelectItem value="public_domain">Public domain footage</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                    </div>
                    {prepared && (
                      <div className="space-y-2 rounded-lg bg-muted/40 p-3">
                        <div className="flex flex-wrap items-center justify-between gap-2 text-xs">
                          <span className="font-mono break-all">{prepared.plan.stem}.mp4</span>
                          <Badge variant={prepared.plan.passes ? 'secondary' : 'outline'}>
                            {prepared.plan.passes ? 'Checks passed' : 'Needs attention'}
                          </Badge>
                        </div>
                        <p className="text-xs text-muted-foreground">
                          r{prepared.plan.revision} · {(prepared.durationTicks / 90000).toFixed(1)}s
                          · about {formatBytes(prepared.plan.estimatedBytes)}
                        </p>
                        {prepared.durationTicks > 60 * 90000 && (
                          <Confirmation
                            label="I reviewed this clip’s duration and confirm permission for this use."
                            checked={choice.rightsApproval === prepared.scope}
                            disabled={busy || checking}
                            onChange={(checked) =>
                              changeChoice(id, { rightsApproval: checked ? prepared.scope : null })
                            }
                          />
                        )}
                        {hot.length > 0 && (
                          <Confirmation
                            label={`I reviewed the ${hot.length} fast caption ${hot.length === 1 ? 'passage' : 'passages'} in this revision.`}
                            checked={choice.captionsApproval === prepared.scope}
                            disabled={busy || checking}
                            onChange={(checked) =>
                              changeChoice(id, {
                                captionsApproval: checked ? prepared.scope : null,
                              })
                            }
                          />
                        )}
                        {prepared.plan.findings.length > 0 && (
                          <ul className="space-y-1 text-xs" aria-label="Export findings">
                            {prepared.plan.findings.map((finding, index) => (
                              <li
                                key={`${finding.code}:${index}`}
                                className={
                                  finding.severity === 'blocking'
                                    ? 'text-destructive'
                                    : 'text-muted-foreground'
                                }
                              >
                                {finding.detail}
                              </li>
                            ))}
                          </ul>
                        )}
                      </div>
                    )}
                    {result?.error && (
                      <p role="alert" className="text-xs text-destructive">
                        {result.error}
                      </p>
                    )}
                  </CardContent>
                )}
              </Card>
            );
          })}
        </section>
        <Card className="xl:sticky xl:top-4">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-sm">
              <FolderOpen className="size-4" />
              Collection destination
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="batch-folder">Local folder</Label>
              <Input
                id="batch-folder"
                value={folder}
                disabled={busy}
                placeholder="Choose an export folder"
                onChange={(event) => setFolder(event.target.value)}
              />
              <Button
                variant="outline"
                size="sm"
                className="w-full"
                disabled={busy}
                onClick={() => void chooseFolder()}
              >
                Browse folders
              </Button>
            </div>
            <div className="space-y-2">
              <Label htmlFor="batch-pattern">File naming</Label>
              <Input
                id="batch-pattern"
                value={pattern}
                disabled={busy}
                onChange={(event) => setPattern(event.target.value)}
              />
              <p className="text-xs leading-5 text-muted-foreground">
                Use {'{clip}'}, {'{date}'}, or {'{address}'}. The {'{index}'} number keeps every
                clip distinct.
              </p>
            </div>
            <div className="space-y-2 border-y py-4 text-xs text-muted-foreground">
              <p>1080 × 1920 · H.264 / AAC</p>
              <p>Burned captions + SRT and VTT</p>
              <p>Metadata, thumbnail and checksums</p>
            </div>
            <div className="flex items-center justify-between text-sm">
              <span>
                {selected.length} {selected.length === 1 ? 'clip' : 'clips'}
              </span>
              <span className="font-mono text-muted-foreground">
                {estimate ? `~${formatBytes(estimate)}` : '—'}
              </span>
            </div>
            <Button className="w-full" disabled={!ready} onClick={() => void exportBatch()}>
              {busy || checking ? <Spinner className="size-4" /> : <Layers className="size-4" />}
              {busy
                ? 'Saving collection…'
                : checking
                  ? 'Checking clips…'
                  : `Export ${selected.length || ''} ${selected.length === 1 ? 'clip' : 'clips'}`}
            </Button>
            <Button
              className="w-full"
              size="sm"
              variant="ghost"
              disabled={busy || checking || !selected.length || !folder.trim()}
              onClick={() => {
                setChoices((current) =>
                  Object.fromEntries(
                    Object.entries(current).map(([id, choice]) => [
                      id,
                      { ...choice, rightsApproval: null, captionsApproval: null },
                    ]),
                  ),
                );
                setRefresh((count) => count + 1);
              }}
            >
              <RefreshCw className="size-3.5" />
              Check again
            </Button>
            <p className="text-xs leading-5 text-muted-foreground">
              The queue is saved before rendering starts. You can leave this screen and return to
              the same collection.
            </p>
          </CardContent>
        </Card>
      </div>
      <section ref={queueSection} className="mt-8 space-y-4" aria-label="Saved export collections">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-lg font-medium">Saved collections</h2>
            <p className="mt-1 text-xs text-muted-foreground">
              Real job progress, delivered files and a separate retry for each clip.
            </p>
          </div>
          <Button size="sm" variant="ghost" onClick={() => setRefresh((count) => count + 1)}>
            <RefreshCw className="size-4" />
            Refresh
          </Button>
        </div>
        {historyProblem && (
          <Alert variant="destructive">
            <AlertDescription>Could not read saved collections: {historyProblem}</AlertDescription>
          </Alert>
        )}
        {!historyProblem && !batches.length && (
          <p className="rounded-xl border border-dashed px-5 py-6 text-sm text-muted-foreground">
            Your first collection will appear here when you export.
          </p>
        )}
        {batches.map((batch) => (
          <Card key={batch.batchId}>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle className="text-sm">
                {new Date(batch.createdUnixMillis).toLocaleString()} · {batch.items.length}{' '}
                {batch.items.length === 1 ? 'clip' : 'clips'}
              </CardTitle>
              <Badge variant="outline">Saved</Badge>
            </CardHeader>
            <CardContent className="divide-y">
              {batch.items.map((item) => (
                <BatchDeliveryRow
                  key={item.index}
                  item={item}
                  api={api}
                  busy={acting.has(`${batch.batchId}:${item.index}`)}
                  onAction={(action) => void act(batch.batchId, item.index, action)}
                  onProblem={setProblem}
                />
              ))}
            </CardContent>
          </Card>
        ))}
      </section>
    </div>
  );
}

function Confirmation({
  label,
  checked,
  disabled,
  onChange,
}: {
  readonly label: string;
  readonly checked: boolean;
  readonly disabled: boolean;
  readonly onChange: (checked: boolean) => void;
}) {
  return (
    <label className="flex cursor-pointer items-start gap-2 py-1 text-xs leading-5">
      <Checkbox
        className="mt-0.5"
        checked={checked}
        disabled={disabled}
        onCheckedChange={(value) => onChange(value === true)}
      />
      {label}
    </label>
  );
}

function BatchDeliveryRow({
  item,
  api,
  busy,
  onAction,
  onProblem,
}: {
  readonly item: ExportBatchItem;
  readonly api: ShellApi;
  readonly busy: boolean;
  readonly onAction: (action: 'retry' | 'cancel') => void;
  readonly onProblem: (message: string) => void;
}) {
  // A pending retry may retain the previous receipt for provenance. Its old
  // failure is not this attempt's state; wait for the new admission receipt.
  const delivery = useDelivery(
    item.projectId,
    item.state === 'queued' ? (item.queued ?? null) : null,
    api,
  );
  const failure = item.error || delivery?.failure;
  const files = delivery?.files;
  const running = delivery?.stages.find((stage) => stage.state === 'running');
  const waiting =
    !running && !delivery?.settled
      ? delivery?.stages.find((stage) => stage.state === 'waiting')
      : undefined;
  const cancelled =
    item.state === 'cancelled' || delivery?.stages.some((stage) => stage.state === 'cancelled');
  const status = files
    ? 'Delivered'
    : cancelled
      ? 'Cancelled'
      : failure || item.state === 'failed'
        ? 'Failed'
        : (running?.label ?? (item.state === 'pending' ? 'Preparing' : 'Queued'));
  const retryable = item.state === 'failed' || item.state === 'cancelled' || !!delivery?.failure;
  const cancellable = (item.state === 'pending' || item.state === 'queued') && !delivery?.settled;
  return (
    <div
      className="space-y-3 py-4 first:pt-0 last:pb-0"
      aria-label={`Export ${item.index}: ${item.request.title || 'Clip'}`}
    >
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="min-w-0">
          <p className="text-sm font-medium">
            {String(item.index).padStart(2, '0')} · {item.request.title || 'Clip'}
          </p>
          <p className="mt-1 text-xs text-muted-foreground">
            r{item.queued?.revision ?? item.request.expectedRevision} · Attempt {item.attempt}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Badge variant={files ? 'secondary' : 'outline'}>
            {files && <Check className="mr-1 size-3" />}
            {status}
          </Badge>
          {retryable && (
            <Button size="sm" variant="outline" disabled={busy} onClick={() => onAction('retry')}>
              <RefreshCw className="size-3.5" />
              Retry clip {item.index}
            </Button>
          )}
          {cancellable && (
            <Button
              size="sm"
              variant="ghost"
              disabled={busy}
              aria-label={`Cancel clip ${item.index}`}
              onClick={() => onAction('cancel')}
            >
              <X className="size-4" />
            </Button>
          )}
        </div>
      </div>
      {running?.progress && (
        <div className="space-y-1.5">
          <Progress
            aria-label={`${running.label} clip ${item.index}`}
            value={(100 * running.progress.done) / running.progress.total}
          />
          <p className="text-xs text-muted-foreground">{deliveryProgressText(running.progress)}</p>
        </div>
      )}
      {waiting && <p className="text-xs text-muted-foreground">{deliveryWaitText(waiting)}</p>}
      {failure && (
        <p role="alert" className="text-xs text-destructive">
          {failure}
        </p>
      )}
      {delivery?.interruption && (
        <p role="status" className="text-xs text-muted-foreground">
          Reconnecting: {delivery.interruption}
        </p>
      )}
      {files && (
        <details className="text-xs">
          <summary className="cursor-pointer text-muted-foreground">
            {files.length} delivered files · {delivery.destinationDir}
          </summary>
          <ul className="mt-2 space-y-1">
            {files.map((file) => (
              <li key={file.path} className="flex items-center justify-between gap-3">
                <span className="truncate font-mono">{file.name}</span>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() =>
                    void api.revealPath(file.path).catch((cause) => onProblem(messageOf(cause)))
                  }
                >
                  <FolderOpen className="size-3.5" />
                  Reveal
                </Button>
              </li>
            ))}
          </ul>
        </details>
      )}
    </div>
  );
}

function localDate(): string {
  const now = new Date();
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
}
