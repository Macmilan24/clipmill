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
import { AlertTriangle, FolderOpen, Info, PackageCheck, Upload } from 'lucide-react';
import type { JSX } from 'react';

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

import type { ExportPlan } from '../daemon/client.js';
import { formatBytes } from '../deviceProfile.js';

/**
 * What the render actually does, stated rather than offered.
 *
 * Every one of these is fixed in Phase 1, and a control that pretended
 * otherwise would be a control that does nothing. They are shown because an
 * editor about to upload needs to know them, not because they are adjustable.
 */
const DELIVERY: readonly (readonly [string, string])[] = [
  ['Picture', '1080 × 1920, H.264, CRF 18'],
  ['Sound', 'AAC, −14 LUFS integrated, −1.0 dBTP ceiling'],
  ['Captions', 'burned in, plus SRT and WebVTT sidecars'],
  ['Also written', 'thumbnail, metadata JSON, render manifest, sha256 sums'],
];

export interface ExportProps {
  readonly docId: string | null;
  readonly destination: string;
  readonly pattern: string;
  readonly title: string;
  readonly attestation: string;
  readonly rightsGateNeeded: boolean;
  readonly rightsGatePassed: boolean;
  readonly plan: ExportPlan | null;
  readonly planning: boolean;
  readonly busy: boolean;
  readonly error: string | null;
  readonly queued: string | null;
  readonly archive: { readonly path: string; readonly entryCount: number } | null;
  readonly onDestinationChange: (value: string) => void;
  readonly onPatternChange: (value: string) => void;
  readonly onChooseFolder: () => void;
  readonly onRightsGateChange: (passed: boolean) => void;
  readonly onExport: () => void;
  readonly onArchive: () => void;
}

export function Export(props: ExportProps): JSX.Element {
  if (props.docId === null) {
    return (
      <Empty className="h-full">
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <Upload />
          </EmptyMedia>
          <EmptyTitle>Nothing approved yet</EmptyTitle>
          <EmptyDescription>
            An export delivers an edit document. Approve a clip on the Results board and it will
            appear here.
          </EmptyDescription>
        </EmptyHeader>
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

  return (
    <div className="flex h-full flex-col gap-4 overflow-y-auto p-4">
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
              placeholder="{index}-{clip}"
              onChange={(event) => props.onPatternChange(event.target.value)}
            />
            <p className="mt-1 text-xs text-[var(--cm-ink-3)]">
              {'{project} {clip} {index} {duration} {date} {address}'}
            </p>
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

      {props.error !== null && (
        <Alert variant="destructive">
          <AlertTriangle />
          <AlertDescription>{props.error}</AlertDescription>
        </Alert>
      )}

      {props.queued !== null && (
        <Alert>
          <PackageCheck />
          <AlertDescription>
            Queued as <span className="font-mono text-xs">{props.queued}</span>. It renders, then it
            delivers; watch it on the run this project is on.
          </AlertDescription>
        </Alert>
      )}

      <div className="flex flex-wrap items-center gap-2">
        <Button onClick={props.onExport} disabled={!ready}>
          {props.busy ? 'Working…' : 'Export'}
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
