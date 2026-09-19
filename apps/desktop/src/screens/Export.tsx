/**
 * 12 Export: the strip, the names, and the queue.
 *
 * Three things a person needs before they let a file leave: what is wrong with
 * it, what it will be called, and what the settings actually are. All three are
 * read rather than composed here — the findings come from the daemon's
 * validation strip, the names come from the daemon resolving the pattern, and
 * the delivery settings are a read-only statement of what the renderer does.
 *
 * The naming preview is the part worth being careful about. It would be easy to
 * resolve the pattern in this file and show the result, and it would be wrong
 * for the same reason the editor's player does not compute its own crops: there
 * would be two implementations of the naming rules, and the preview a user
 * approved would eventually not be the name they got. So every keystroke asks
 * the daemon, and what is drawn is the daemon's answer.
 */
import {
  AlertTriangle,
  ArrowLeft,
  Eye,
  FolderOpen,
  Info,
  PackageCheck,
  Upload,
} from 'lucide-react';
import type { JSX, ReactNode } from 'react';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from '@/components/ui/empty';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Separator } from '@/components/ui/separator';
import { Spinner } from '@/components/ui/spinner';

import type { ExportFinding, ExportPlan } from '../daemon/client.js';
import { formatBytes } from '../deviceProfile.js';
import {
  type Delivery,
  type DeliveryStage,
  deliveryProgressText,
  deliveryWaitText,
} from '../export/delivery.js';

/**
 * What the render actually does, stated rather than offered.
 *
 * Every one of these is fixed in Phase 1, and a control that pretended
 * otherwise would be a control that does nothing. They are shown because an
 * editor about to upload needs to know them, not because they are adjustable.
 */
/** What a user gets before they have an opinion; the daemon's default too. */
const DEFAULT_PATTERN = '{index}-{clip}';

/** The fastest of the hot captions, as the daemon put it. */
function hottestRate(findings: readonly ExportFinding[]): string {
  const rates = findings
    .map((finding) => /([\d.]+) characters a second/.exec(finding.detail)?.[1])
    .filter((rate): rate is string => rate !== undefined)
    .map(Number);
  const top = Math.max(...rates);
  return Number.isFinite(top) ? `up to ${top.toFixed(1)} characters a second` : 'too fast';
}

/** Stable machine codes stay in the export record, not in the reading flow. */
function findingTitle(code: string): string {
  if (code.startsWith('framing.')) return 'Framing needs attention';
  if (code === 'boundary.inside_word') return 'A cut interrupts a word';
  if (code === 'rights.gate_not_passed') return 'Confirm this clip’s permission';
  if (code.startsWith('rights.')) return 'Choose your source permission';
  if (code === 'disk.insufficient') return 'More storage is needed';
  if (code === 'disk.unknown') return 'Storage could not be checked';
  if (code.startsWith('disk.')) return 'Storage is running low';
  if (code.startsWith('captions.burn_in.')) return 'On-screen caption notice';
  if (code.startsWith('captions.')) return 'Subtitle check';
  return 'Export check';
}

/**
 * The daemon's refusal of the name pattern, when that is what the error is.
 *
 * The pattern is the one field a person can type something reasonable into
 * and be refused for it — a plain name, with no `{index}` or `{clip}`, would
 * give every clip in an export the same file. The refusal belongs under the
 * field it is about, with the way back beside it, not in a red bar at the
 * bottom that names nothing on screen.
 */
function patternProblemOf(error: string | null): string | null {
  return error !== null && error.includes('pattern') ? error : null;
}

const DELIVERY: readonly (readonly [string, string])[] = [
  ['Video', '1080 × 1920, H.264, CRF 18'],
  ['Audio', 'AAC, −14 LUFS integrated, −1.0 dBTP target'],
  ['Captions', 'Burned in, with SRT and WebVTT files'],
  ['Additional files', 'Thumbnail, metadata, render manifest and checksums'],
];

export interface ExportProps {
  readonly onEdit?: (() => void) | undefined;
  readonly onBatch?: () => void;
  readonly docId: string | null;
  /** What the clip is called — the project and the clip — when the route knew. */
  readonly labels: { readonly project?: string; readonly clip?: string } | null;
  /** The list of edits to choose from, shown only when no clip is named. */
  readonly picker: ReactNode;
  readonly destination: string;
  readonly pattern: string;
  readonly title: string;
  readonly attestation: string;
  readonly onAttestationChange?: (value: string) => void;
  readonly audition?: string | null;
  readonly auditionProblem?: string | null;
  readonly onAuditionError?: () => void;
  readonly rightsGateNeeded: boolean;
  readonly rightsGatePassed: boolean;
  /**
   * Whether the sidecar captions run faster than the reading profile allows,
   * and whether the person exporting has said they know.
   */
  readonly hotCaptions: readonly ExportFinding[];
  readonly hotCaptionsConfirmed: boolean;
  readonly plan: ExportPlan | null;
  readonly planning: boolean;
  readonly busy: boolean;
  readonly error: string | null;
  /** The export that was queued, followed to its files. Null before one is. */
  readonly delivery: Delivery | null;
  readonly archive: { readonly path: string; readonly entryCount: number } | null;
  readonly onDestinationChange: (value: string) => void;
  readonly onPatternChange: (value: string) => void;
  readonly onChooseFolder: () => void;
  readonly onRightsGateChange: (passed: boolean) => void;
  readonly onHotCaptionsChange: (confirmed: boolean) => void;
  readonly onExport: () => void;
  readonly onArchive: () => void;
  /** Show a delivered file in the file manager. */
  readonly onReveal: (path: string) => void;
}

export function Export(props: ExportProps): JSX.Element {
  if (props.docId === null) {
    return (
      <Empty className="h-full">
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <Upload />
          </EmptyMedia>
          <EmptyTitle>No clip is chosen for export</EmptyTitle>
          <EmptyDescription>
            An export delivers one edit document. Open a clip from the editor, or choose one of the
            edits below.
          </EmptyDescription>
        </EmptyHeader>
        {props.onBatch && (
          <Button variant="outline" onClick={props.onBatch}>
            Export a collection
          </Button>
        )}
        {props.picker}
      </Empty>
    );
  }

  const blocking = (props.plan?.findings ?? []).filter(
    (finding) =>
      finding.severity === 'blocking' &&
      !(finding.code === 'captions.reading_rate' && props.hotCaptions.length > 0),
  );
  const advisory = (props.plan?.findings ?? []).filter(
    (finding) =>
      finding.severity === 'advisory' &&
      !(finding.code === 'captions.reading_rate' && props.hotCaptions.length > 0),
  );
  const ready =
    props.plan?.passes === true && props.attestation !== '' && !props.busy && !props.planning;
  const delivering = props.delivery !== null && !props.delivery.settled;
  const patternProblem = patternProblemOf(props.error);

  return (
    <div className="export-page">
      <header className="workspace-heading" data-testid="export-clip">
        <div>
          <h1 className="workspace-title">Export clip</h1>
          <p className="workspace-subtitle mt-1">
            {props.labels
              ? [props.labels.project, props.labels.clip].filter(Boolean).join(' · ')
              : 'Your edited clip'}
          </p>
        </div>
        <div className="flex items-center gap-2">
          {props.onBatch && (
            <Button variant="outline" size="sm" onClick={props.onBatch}>
              Export a collection
            </Button>
          )}
          {props.onEdit && (
            <Button variant="outline" size="sm" onClick={props.onEdit}>
              <ArrowLeft className="size-4" />
              Back to editor
            </Button>
          )}
        </div>
      </header>
      <div className="export-grid">
        <div className="export-column">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-sm">
                <FolderOpen className="size-4" /> Export location
              </CardTitle>
            </CardHeader>
            <CardContent className="flex flex-col gap-3">
              <div className="flex items-end gap-2">
                <div className="flex-1">
                  <Label htmlFor="export-destination">Folder</Label>
                  <Input
                    id="export-destination"
                    value={props.destination}
                    placeholder="Choose a local folder"
                    onChange={(event) => props.onDestinationChange(event.target.value)}
                  />
                </div>
                <Button variant="outline" onClick={props.onChooseFolder} disabled={props.busy}>
                  Browse
                </Button>
              </div>
              <p className="text-xs text-[var(--cm-ink-3)]">
                Choose a folder on this device for the video and its accompanying files.
              </p>

              <div>
                <Label htmlFor="export-pattern">Name pattern</Label>
                <Input
                  id="export-pattern"
                  value={props.pattern}
                  placeholder={DEFAULT_PATTERN}
                  aria-invalid={patternProblem !== null}
                  onChange={(event) => props.onPatternChange(event.target.value)}
                />
                {patternProblem === null ? (
                  <p className="mt-1 text-xs text-[var(--cm-ink-3)]">
                    Enter a name or use {'{index}'}, {'{clip}'}, {'{project}'}, {'{duration}'},{' '}
                    {'{date}'} or {'{address}'}. Plain names get a clip number automatically.
                  </p>
                ) : (
                  <p
                    className="mt-1 flex flex-wrap items-center gap-2 text-xs text-[var(--cm-danger-ink)]"
                    data-testid="pattern-problem"
                  >
                    <span>{patternProblem}</span>
                    <Button
                      variant="outline"
                      size="xs"
                      onClick={() => props.onPatternChange(DEFAULT_PATTERN)}
                    >
                      Use {DEFAULT_PATTERN}
                    </Button>
                  </p>
                )}
              </div>

              <NamePreview plan={props.plan} planning={props.planning} />
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-sm">Export checks</CardTitle>
            </CardHeader>
            <CardContent className="flex flex-col gap-3">
              <div className="space-y-2">
                <Label htmlFor="source-rights">Source rights</Label>
                <Select
                  value={props.attestation}
                  onValueChange={(value) => props.onAttestationChange?.(value)}
                >
                  <SelectTrigger id="source-rights" className="w-full">
                    <SelectValue placeholder="Choose the permission you hold" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="own_content">I own this footage</SelectItem>
                    <SelectItem value="licensed_content">
                      I have permission or a license for this use
                    </SelectItem>
                    <SelectItem value="public_domain">
                      This footage is in the public domain
                    </SelectItem>
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  Your choice is saved with this export.
                </p>
              </div>
              {props.rightsGateNeeded && (
                <label className="flex items-start gap-2 rounded-lg border border-[var(--cm-line-1)] bg-[var(--cm-surface-1)] p-3 text-xs">
                  <input
                    type="checkbox"
                    className="mt-0.5"
                    checked={props.rightsGatePassed}
                    disabled={props.busy || props.planning}
                    onChange={(event) => props.onRightsGateChange(event.target.checked)}
                  />
                  <span>
                    This clip runs past a minute. I confirm that my selected source permission
                    covers this use.
                  </span>
                </label>
              )}

              {props.hotCaptions.length > 0 && (
                <div className="space-y-2">
                  <label
                    className="flex items-start gap-2 rounded-lg border border-[var(--cm-line-1)] bg-[var(--cm-surface-1)] p-3 text-xs"
                    data-testid="hot-captions-gate"
                  >
                    <input
                      type="checkbox"
                      className="mt-0.5"
                      checked={props.hotCaptionsConfirmed}
                      disabled={props.busy || props.planning}
                      onChange={(event) => props.onHotCaptionsChange(event.target.checked)}
                    />
                    <span>
                      {props.hotCaptions.length === 1
                        ? 'One caption'
                        : `${props.hotCaptions.length} captions`}{' '}
                      in the subtitle file exceed the reading-speed target (
                      {hottestRate(props.hotCaptions)}). I reviewed them and want to export them as
                      they are.
                    </span>
                  </label>
                  <p className="text-xs text-muted-foreground">
                    {props.hotCaptionsConfirmed
                      ? `Confirmed for revision r${props.plan?.revision ?? '—'}.`
                      : 'Review required before export. You can also adjust these captions in the editor.'}
                  </p>
                  <details className="rounded-lg border px-3 py-2 text-xs">
                    <summary className="cursor-pointer text-muted-foreground">
                      Review caption details ({props.hotCaptions.length})
                    </summary>
                    <ul className="mt-3 space-y-2 leading-5">
                      {props.hotCaptions.map((finding, index) => (
                        <li key={`${finding.detail}:${index}`}>
                          <span className="mr-1.5 text-muted-foreground">{index + 1}.</span>
                          {finding.detail.replace(/ — confirmed as read\.$/, '')}
                        </li>
                      ))}
                    </ul>
                  </details>
                </div>
              )}

              {props.planning && (
                <p className="flex items-center gap-2 text-xs text-[var(--cm-ink-2)]">
                  <Spinner className="size-3" /> Checking…
                </p>
              )}

              {blocking.map((finding) => (
                <Alert key={`${finding.code}:${finding.detail}`} variant="destructive">
                  <AlertTriangle />
                  <AlertDescription>
                    <span className="font-medium">{findingTitle(finding.code)}</span>
                    <span className="mt-1 block">{finding.detail}</span>
                  </AlertDescription>
                </Alert>
              ))}
              {advisory.map((finding) => (
                <Alert key={`${finding.code}:${finding.detail}`}>
                  <Info />
                  <AlertDescription>
                    <span className="font-medium">{findingTitle(finding.code)}</span>
                    <span className="mt-1 block">{finding.detail}</span>
                  </AlertDescription>
                </Alert>
              ))}
              {props.plan !== null && !props.planning && props.plan.findings.length === 0 && (
                <p className="flex items-center gap-2 text-xs text-[var(--cm-success-ink)]">
                  <PackageCheck className="size-4 shrink-0" /> All checks passed. Your clip is ready
                  to export.
                </p>
              )}
            </CardContent>
          </Card>
        </div>
        <div className="export-column">
          <Card>
            <CardHeader>
              <CardTitle className="text-sm">Delivery format</CardTitle>
            </CardHeader>
            <CardContent>
              <dl className="space-y-1 text-xs">
                {DELIVERY.map(([label, value]) => (
                  <div key={label} className="flex justify-between gap-4">
                    <dt className="text-[var(--cm-ink-2)]">{label}</dt>
                    <dd className="text-right text-[var(--cm-ink-1)]">{value}</dd>
                  </div>
                ))}
                <Separator className="my-2" />
                <div className="flex justify-between gap-4">
                  <dt className="text-[var(--cm-ink-2)]">Estimated size</dt>
                  <dd className="font-mono text-[var(--cm-ink-1)]">
                    {props.plan === null ? '—' : formatBytes(props.plan.estimatedBytes)}
                  </dd>
                </div>
                <div className="flex justify-between gap-4">
                  <dt className="text-[var(--cm-ink-2)]">Free disk space</dt>
                  <dd className="font-mono text-[var(--cm-ink-1)]">
                    {!props.destination.trim()
                      ? 'Choose a folder'
                      : props.planning
                        ? 'Checking…'
                        : props.plan?.availableBytes === undefined
                          ? 'Unavailable'
                          : formatBytes(props.plan.availableBytes)}
                  </dd>
                </div>
              </dl>
              <p className="mt-2 text-xs text-[var(--cm-ink-3)]">
                Vertical video with captions, ready for your final review before uploading.
              </p>
            </CardContent>
          </Card>
        </div>
      </div>

      {props.error !== null && patternProblem === null && (
        <Alert variant="destructive">
          <AlertTriangle />
          <AlertDescription>{props.error}</AlertDescription>
        </Alert>
      )}

      {props.delivery !== null && (
        <DeliveryCard delivery={props.delivery} onReveal={props.onReveal} />
      )}

      {props.audition && props.delivery && (
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">
              Final rendered preview · r{props.delivery.revision}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <video
              aria-label={`Final rendered revision r${props.delivery.revision}`}
              src={props.audition}
              onError={props.onAuditionError}
              controls
              playsInline
              preload="metadata"
              className="mx-auto max-h-[560px] max-w-full rounded-lg bg-black"
            />
            <p className="mt-3 text-xs text-muted-foreground">
              This is the encoded file with its final captions, framing and mastered audio. Review
              this version before uploading.
              {props.plan && props.plan.revision !== props.delivery.revision
                ? ` Your current edit is r${props.plan.revision}; export it again to review those changes.`
                : ''}
            </p>
          </CardContent>
        </Card>
      )}
      {props.auditionProblem && (
        <p role="status" className="text-sm text-muted-foreground">
          {props.auditionProblem}
        </p>
      )}
      <div className="export-actions">
        <Button onClick={props.onExport} disabled={!ready || delivering}>
          {props.busy
            ? 'Working…'
            : props.plan
              ? `Export revision r${props.plan.revision}`
              : 'Export'}
        </Button>
        <Button variant="outline" onClick={props.onArchive} disabled={props.busy}>
          Save project archive
        </Button>
        {props.archive !== null && (
          <span className="text-xs text-[var(--cm-ink-2)]">
            {props.archive.entryCount} documents written to{' '}
            <span className="font-mono">{props.archive.path}</span>
          </span>
        )}
      </div>
      <p className="text-xs text-[var(--cm-ink-3)]">
        A project archive saves your edits and export records. Source recordings stay in their
        original location.
      </p>
    </div>
  );
}

/** What a stage is doing, in a word a person reads. */
function stageWord(stage: DeliveryStage): string {
  switch (stage.state) {
    case 'running':
      return stage.progress ? deliveryProgressText(stage.progress) : 'running';
    case 'done':
      return 'done';
    case 'failed':
      return 'failed';
    case 'cancelled':
      return 'cancelled';
    default:
      return deliveryWaitText(stage);
  }
}

/**
 * The export as it happens, and what it left behind.
 *
 * The files listed are the ones the delivery's own package names — the
 * daemon's receipt for what it wrote — at the folder the export was resolved
 * to, so every path here is one that exists.
 */
function DeliveryCard({
  delivery,
  onReveal,
}: {
  readonly delivery: Delivery;
  readonly onReveal: (path: string) => void;
}): JSX.Element {
  return (
    <Card data-testid="delivery">
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-sm">
          <PackageCheck className="size-4" />
          {delivery.files
            ? `Delivered revision r${delivery.revision}`
            : delivery.failure
              ? `Revision r${delivery.revision} was not delivered`
              : `Delivering revision r${delivery.revision}`}
        </CardTitle>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        <p className="font-mono text-[11px] text-[var(--cm-ink-3)]">{delivery.destinationDir}</p>
        <ul className="space-y-1 text-xs" aria-label="Delivery stages">
          {delivery.stages.map((stage) => (
            <li key={stage.kind} className="flex justify-between gap-4">
              <span className="text-[var(--cm-ink-2)]">{stage.label}</span>
              <span
                className={
                  stage.state === 'failed'
                    ? 'text-[var(--cm-danger-ink)]'
                    : stage.state === 'done'
                      ? 'text-[var(--cm-success-ink)]'
                      : 'text-[var(--cm-ink-1)]'
                }
                data-testid={`stage-${stage.kind}`}
              >
                {stageWord(stage)}
              </span>
            </li>
          ))}
        </ul>
        {delivery.interruption !== null && (
          <p className="text-xs text-[var(--cm-ink-3)]" data-testid="delivery-interruption">
            Export is still running in the background. Reconnecting to its progress… (
            {delivery.interruption})
          </p>
        )}
        {delivery.failure !== null && (
          <Alert variant="destructive">
            <AlertTriangle />
            <AlertDescription>
              {delivery.failure} The folder holds nothing from this export; fix the cause and export
              again.
            </AlertDescription>
          </Alert>
        )}
        {delivery.files !== null && (
          <ul className="space-y-1" aria-label="Delivered files">
            {delivery.files.map((file) => (
              <li key={file.name} className="flex items-center justify-between gap-3 text-xs">
                <span className="min-w-0 truncate font-mono text-[11px] text-[var(--cm-ink-1)]">
                  {file.path}
                </span>
                <span className="flex shrink-0 items-center gap-2">
                  <span className="font-mono text-[10px] text-[var(--cm-ink-3)]">
                    {formatBytes(file.bytes)}
                  </span>
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => onReveal(file.path)}
                    aria-label={`Reveal ${file.name}`}
                  >
                    <Eye className="size-3" /> Reveal
                  </Button>
                </span>
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  );
}

/**
 * The names, as the daemon resolved them.
 *
 * Deliberately not computed here. See the note at the top of the file.
 */
function NamePreview({
  plan,
  planning,
}: {
  readonly plan: ExportPlan | null;
  readonly planning: boolean;
}): JSX.Element {
  if (plan === null) {
    return (
      <p className="text-xs text-[var(--cm-ink-3)]">
        {planning ? 'Resolving…' : 'Choose a folder to see what the files will be called.'}
      </p>
    );
  }
  return (
    <div className="rounded-lg border border-[var(--cm-line-1)] bg-[var(--cm-surface-1)] p-2">
      <p className="mb-1 flex items-center gap-2 text-xs text-[var(--cm-ink-2)]">
        Files <Badge variant="outline">{plan.fileNames.length}</Badge>
      </p>
      <ul className="space-y-0.5 font-mono text-[11px] text-[var(--cm-ink-1)]">
        {plan.fileNames.map((name) => (
          <li key={name}>{name}</li>
        ))}
      </ul>
    </div>
  );
}
