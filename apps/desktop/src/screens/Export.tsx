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
import { AlertTriangle, Eye, FolderOpen, Info, PackageCheck, Upload } from 'lucide-react';
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
import { Separator } from '@/components/ui/separator';
import { Spinner } from '@/components/ui/spinner';

import type { ExportFinding, ExportPlan } from '../daemon/client.js';
import { formatBytes } from '../deviceProfile.js';
import type { Delivery, DeliveryStage } from '../export/delivery.js';

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
  ['Picture', '1080 × 1920, H.264, CRF 18'],
  ['Sound', 'AAC, −14 LUFS integrated, −1.0 dBTP ceiling'],
  ['Captions', 'burned in, plus SRT and WebVTT sidecars'],
  ['Also written', 'thumbnail, metadata JSON, render manifest, sha256 sums'],
];

export interface ExportProps {
  readonly docId: string | null;
  /** What the clip is called — the project and the clip — when the route knew. */
  readonly labels: { readonly project?: string; readonly clip?: string } | null;
  /** The list of edits to choose from, shown only when no clip is named. */
  readonly picker: ReactNode;
  readonly destination: string;
  readonly pattern: string;
  readonly title: string;
  readonly attestation: string;
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
        {props.picker}
      </Empty>
    );
  }

  const blocking = (props.plan?.findings ?? []).filter(
    (finding) => finding.severity === 'blocking',
  );
  const advisory = (props.plan?.findings ?? []).filter(
    (finding) => finding.severity === 'advisory',
  );
  const ready = props.plan?.passes === true && !props.busy;
  const delivering = props.delivery !== null && !props.delivery.settled;
  const patternProblem = patternProblemOf(props.error);

  return (
    <div className="flex h-full flex-col gap-4 overflow-y-auto p-4">
      <header className="flex flex-wrap items-baseline gap-x-3 gap-y-1" data-testid="export-clip">
        <h1 className="text-sm font-semibold text-[var(--cm-ink-1)]">
          {props.labels
            ? [props.labels.project, props.labels.clip].filter(Boolean).join(' · ')
            : 'This clip'}
        </h1>
        <span className="font-mono text-[10px] text-[var(--cm-ink-3)]">{props.docId}</span>
      </header>
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-sm">
            <FolderOpen className="size-4" /> Where it goes
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
            Local disks only. A transfer over a network share that drops leaves a file that looks
            finished, and nothing here could tell you afterwards which one you had.
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
                A plain name works; {'{index}'} is added to it so each clip gets its own. Fills:{' '}
                {'{index} {clip} {project} {duration} {date} {address}'}.
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
          <CardTitle className="text-sm">Before it leaves</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-3">
          {props.rightsGateNeeded && (
            <label className="flex items-start gap-2 rounded-lg border border-[var(--cm-line-1)] bg-[var(--cm-surface-1)] p-3 text-xs">
              <input
                type="checkbox"
                className="mt-0.5"
                checked={props.rightsGatePassed}
                onChange={(event) => props.onRightsGateChange(event.target.checked)}
              />
              <span>
                This clip runs past a minute. I hold the rights to this footage, or it is licensed
                for this use.{' '}
                <span className="text-[var(--cm-ink-3)]">
                  Recorded verbatim in the delivered metadata as “{props.attestation}”.
                </span>
              </span>
            </label>
          )}

          {props.hotCaptions.length > 0 && (
            <label
              className="flex items-start gap-2 rounded-lg border border-[var(--cm-line-1)] bg-[var(--cm-surface-1)] p-3 text-xs"
              data-testid="hot-captions-gate"
            >
              <input
                type="checkbox"
                className="mt-0.5"
                checked={props.hotCaptionsConfirmed}
                onChange={(event) => props.onHotCaptionsChange(event.target.checked)}
              />
              <span>
                {props.hotCaptions.length === 1
                  ? 'One caption'
                  : `${props.hotCaptions.length} captions`}{' '}
                in the subtitle file run faster than a reader can follow (
                {hottestRate(props.hotCaptions)}; the profile allows 20 a second). The speech is
                that fast, and slowing the captions would mean hiding words that were said. Export
                them as they are.{' '}
                <span className="text-[var(--cm-ink-3)]">
                  Recorded in the delivered metadata as “captions_reading_rate”.
                </span>
              </span>
            </label>
          )}

          {props.planning && (
            <p className="flex items-center gap-2 text-xs text-[var(--cm-ink-2)]">
              <Spinner className="size-3" /> Checking…
            </p>
          )}

          {blocking.map((finding) => (
            <Alert key={finding.code} variant="destructive">
              <AlertTriangle />
              <AlertDescription>
                <span className="font-mono text-[10px] opacity-70">{finding.code}</span>{' '}
                {finding.detail}
              </AlertDescription>
            </Alert>
          ))}
          {advisory.map((finding) => (
            <Alert key={finding.code}>
              <Info />
              <AlertDescription>
                <span className="font-mono text-[10px] opacity-70">{finding.code}</span>{' '}
                {finding.detail}
              </AlertDescription>
            </Alert>
          ))}
          {props.plan !== null && !props.planning && props.plan.findings.length === 0 && (
            <p className="flex items-center gap-2 text-xs text-[var(--cm-success-ink)]">
              <PackageCheck className="size-4" /> Rights recorded, no cut inside a word, sidecars
              readable at speed, room on the disk.
            </p>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-sm">What gets written</CardTitle>
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
              <dt className="text-[var(--cm-ink-2)]">Free where it lands</dt>
              <dd className="font-mono text-[var(--cm-ink-1)]">
                {props.plan?.availableBytes === undefined
                  ? 'not readable'
                  : formatBytes(props.plan.availableBytes)}
              </dd>
            </div>
          </dl>
          <p className="mt-2 text-xs text-[var(--cm-ink-3)]">
            These are not settings. Phase 1 delivers one profile, and a control that let you change
            it would be a control that changed nothing.
          </p>
        </CardContent>
      </Card>

      {props.error !== null && patternProblem === null && (
        <Alert variant="destructive">
          <AlertTriangle />
          <AlertDescription>{props.error}</AlertDescription>
        </Alert>
      )}

      {props.delivery !== null && (
        <DeliveryCard delivery={props.delivery} onReveal={props.onReveal} />
      )}

      <div className="flex flex-wrap items-center gap-2">
        <Button onClick={props.onExport} disabled={!ready || delivering}>
          {props.busy
            ? 'Working…'
            : props.plan
              ? `Export revision r${props.plan.revision}`
              : 'Export'}
        </Button>
        <Button variant="outline" onClick={props.onArchive} disabled={props.busy}>
          Archive this project
        </Button>
        {props.archive !== null && (
          <span className="text-xs text-[var(--cm-ink-2)]">
            {props.archive.entryCount} documents written to{' '}
            <span className="font-mono">{props.archive.path}</span>
          </span>
        )}
      </div>
      <p className="text-xs text-[var(--cm-ink-3)]">
        An archive carries the project&rsquo;s state, its edit documents, their command logs, and
        the render manifests, under a published schema. Your recordings are named in it rather than
        copied — they are already on your disk, and an archive that duplicated them is one nobody
        makes twice.
      </p>
    </div>
  );
}

/** What a stage is doing, in a word a person reads. */
function stageWord(stage: DeliveryStage): string {
  switch (stage.state) {
    case 'running':
      return stage.progress
        ? `${stage.progress.done} of ${stage.progress.total} ${stage.progress.unit}`
        : 'running';
    case 'done':
      return 'done';
    case 'failed':
      return 'failed';
    case 'cancelled':
      return 'cancelled';
    default:
      return stage.waitReason === '' ? 'waiting' : stage.waitReason;
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
            The export goes on in the daemon; the last look at it failed ({delivery.interruption}).
            Asking again.
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
