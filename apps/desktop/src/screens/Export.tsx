/**
 * One clip, exported.
 *
 * The clip itself stays in view on one side. The other side is the one path to
 * its files — where they go, what they are, the permission they rest on and
 * what the checks found — ending in the button that makes them. What the
 * export becomes follows beneath: its progress and files, then publishing.
 *
 * The daemon resolves naming patterns so the preview uses the same rules as delivery.
 */
import {
  ArrowLeft,
  Captions,
  CircleAlert,
  CircleCheck,
  Eye,
  FolderOpen,
  Layers,
  Link2,
  TriangleAlert,
  Upload,
} from 'lucide-react';
import { type JSX, type ReactNode, type Ref, useEffect, useRef, useState } from 'react';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
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
import { Spinner } from '@/components/ui/spinner';
import { TooltipProvider } from '@/components/ui/tooltip';

import type { ExportFinding, ExportPlan } from '../daemon/client.js';
import { formatBytes } from '../deviceProfile.js';
import { FRAME_SHAPES, type FrameShape, frameOfShape } from '../editor/layouts.js';
import { TipButton } from '../inspector/TipButton.js';
import { clock } from '../results/model.js';
import type { EditorFocus } from '../shell/route.js';
import {
  type Delivery,
  type DeliveryStage,
  deliveryProgressText,
  deliveryWaitText,
  estimatedExportSeconds,
} from '../export/delivery.js';
import {
  DEFAULT_FORMAT,
  type FormatChoice,
  HEIGHT_CHOICES,
  RATE_CHOICES,
  type RateChoice,
  formatSummary,
  fpsText,
  frameAt,
  heightLabel,
  rateLabel,
  sizeName,
} from '../export/format.js';
import '../export/export.css';

/** What a user gets before they have an opinion; the daemon's default too. */
const DEFAULT_PATTERN = '{index}-{clip}';

/** The fastest of the hot captions, as the daemon put it. */
function hottestRate(findings: readonly ExportFinding[]): number | null {
  const rates = findings
    .map((finding) => /([\d.]+) characters a second/.exec(finding.detail)?.[1])
    .filter((rate): rate is string => rate !== undefined)
    .map(Number);
  const top = Math.max(...rates);
  return Number.isFinite(top) ? top : null;
}

/**
 * Where "Fix captions" should land: the first caption the strip refused, or
 * failing that the first it merely pointed at.
 *
 * The finding names the cue and which grouping it is in — `captions.burn_in.*`
 * is the on-screen track, everything else under `captions.` is the subtitle
 * file. A finding with no cue (there are none today, but the field is
 * optional) still opens the right track; the editor then lands on the track's
 * first problem itself.
 */
function captionFocusOf(findings: readonly ExportFinding[]): {
  readonly finding: ExportFinding;
  readonly focus: EditorFocus;
  readonly blocking: boolean;
} | null {
  const captions = findings.filter((finding) => finding.code.startsWith('captions.'));
  const first = captions.find((finding) => finding.severity === 'blocking') ?? captions[0];
  if (!first) return null;
  return {
    finding: first,
    blocking: first.severity === 'blocking',
    focus: {
      panel: 'captions',
      track: first.code.startsWith('captions.burn_in.') ? 'on-screen' : 'reading',
      ...(first.cueId ? { cueId: first.cueId } : {}),
    },
  };
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
  if (code === 'source.missing') return 'The recording has moved';
  if (code.startsWith('destination.')) return 'This folder cannot be used';
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

function deliverySpecs(
  format: FormatChoice,
  shape: FrameShape,
): readonly (readonly [string, string])[] {
  const frame = frameAt(format.height, shape);
  return [
    ['Video', `${frame.width} × ${frame.height}, H.264, CRF 18`],
    ['Audio', 'AAC, −14 LUFS integrated, −1.0 dBTP target'],
    ['Captions', 'Burned in, with SRT and WebVTT files'],
    ['Additional files', 'Thumbnail, metadata, render manifest and checksums'],
  ];
}

/** What changing the frame rate does to this recording, when it does anything. */
function rateNote(rate: RateChoice, sourceFps: number | null): string | null {
  if (rate === 'source' || sourceFps === null) return null;
  const chosen = Number(rate);
  if (Math.abs(chosen - sourceFps) < 0.01) return null;
  return chosen > sourceFps
    ? `Your recording runs at ${fpsText(sourceFps)} fps, so some frames will repeat and pans may judder.`
    : `Your recording runs at ${fpsText(sourceFps)} fps, so some frames will be left out.`;
}

/** Whether a person has asked for less motion. */
function calm(): boolean {
  return (
    typeof window !== 'undefined' &&
    typeof window.matchMedia === 'function' &&
    window.matchMedia('(prefers-reduced-motion: reduce)').matches
  );
}

export interface ExportProps {
  /** Open the editor — on a particular caption, when handed the focus for one. */
  readonly onEdit?: ((focus?: EditorFocus) => void) | undefined;
  readonly onBatch?: () => void;
  readonly publishing?: ReactNode;
  /** The clip itself, playing, above what it will be delivered as. */
  readonly preview?: ReactNode;
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
  readonly plan: ExportPlan | null;
  readonly planning: boolean;
  readonly busy: boolean;
  readonly error: string | null;
  /** The export that was queued, followed to its files. Null before one is. */
  readonly delivery: Delivery | null;
  readonly mediaSeconds?: number;
  readonly archive: { readonly path: string; readonly entryCount: number } | null;
  readonly onDestinationChange: (value: string) => void;
  readonly onPatternChange: (value: string) => void;
  readonly onChooseFolder: () => void;
  readonly onRightsGateChange: (passed: boolean) => void;
  readonly onExport: () => void;
  readonly onCancel?: (() => void) | undefined;
  readonly onRetry?: (() => void) | undefined;
  readonly onRelink?: (() => void) | undefined;
  /** Frame rate and size of the delivered picture. */
  readonly format?: FormatChoice;
  /** The clip's shape, which the sizes are given in. */
  readonly shape?: FrameShape;
  /** The recording's own frame rate, for "Match recording". Null when unknown. */
  readonly sourceFps?: number | null;
  readonly onFormatChange?: (format: FormatChoice) => void;
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
  return <ExportClip {...props} />;
}

function ExportClip(props: ExportProps): JSX.Element {
  const format = props.format ?? DEFAULT_FORMAT;
  const shape = props.shape ?? 'vertical';
  const findings = props.plan?.findings ?? [];
  // Fast captions, once there are any, are counted together rather than listed.
  const grouped = (finding: ExportFinding) =>
    finding.code === 'captions.reading_rate' && props.hotCaptions.length > 0;
  const blocking = findings.filter(
    (finding) => finding.severity === 'blocking' && !grouped(finding),
  );
  const advisory = findings.filter(
    (finding) => finding.severity === 'advisory' && !grouped(finding),
  );
  const captionFocus = captionFocusOf(findings);
  const ready =
    props.plan?.passes === true && props.attestation !== '' && !props.busy && !props.planning;
  const delivering = props.delivery !== null && !props.delivery.settled;
  const patternProblem = patternProblemOf(props.error);
  const status = readiness(props, blocking.length, delivering);

  // The export someone just asked for is brought into view as it starts, so
  // its progress is not left below the fold.
  const deliveryPanel = useRef<HTMLElement>(null);
  const following = useRef(false);
  const { delivery } = props;
  useEffect(() => {
    if (!following.current || delivery === null) return;
    following.current = false;
    deliveryPanel.current?.scrollIntoView?.({
      block: 'nearest',
      behavior: calm() ? 'auto' : 'smooth',
    });
  }, [delivery]);
  // An export that was refused never arrives, so there is nothing to follow.
  useEffect(() => {
    if (props.error !== null) following.current = false;
  }, [props.error]);

  const hottest = hottestRate(props.hotCaptions);
  const fix = (finding: ExportFinding | null) =>
    captionFocus !== null && (finding === null || captionFocus.finding === finding) ? (
      <CaptionFix
        focus={captionFocus}
        grouped={finding === null}
        busy={props.busy}
        onEdit={props.onEdit}
      />
    ) : null;

  return (
    <div className="export-page export-single">
      <header className="export-header" data-testid="export-clip">
        {props.onEdit && (
          <TooltipProvider delayDuration={300}>
            <TipButton label="Back to editor" onClick={() => props.onEdit?.()}>
              <ArrowLeft />
            </TipButton>
          </TooltipProvider>
        )}
        <div className="export-heading">
          <h1 className="workspace-title">Export clip</h1>
          <p className="workspace-subtitle">
            {props.labels
              ? [props.labels.project, props.labels.clip].filter(Boolean).join(' · ')
              : 'Your edited clip'}
          </p>
        </div>
        {props.onBatch && (
          <Button variant="outline" size="sm" onClick={props.onBatch}>
            <Layers />
            Export a collection
          </Button>
        )}
      </header>

      <div className="export-layout" data-shape={shape}>
        <Stage
          preview={props.preview}
          exported={props.delivery !== null ? (props.audition ?? null) : null}
          problem={props.auditionProblem ?? null}
          onError={props.onAuditionError}
          staleRevision={
            props.plan && props.delivery && props.plan.revision !== props.delivery.revision
              ? props.plan.revision
              : null
          }
          summary={formatSummary(format, props.sourceFps ?? null, shape)}
          shape={shape}
          seconds={props.mediaSeconds ?? 0}
        />

        <div className="export-flow">
          <section className="export-panel" aria-label="Export settings">
            <Section title="Save to">
              <div className="export-field">
                <Label htmlFor="export-destination" className="export-field-label">
                  Folder
                </Label>
                <div className="export-folder">
                  <Input
                    id="export-destination"
                    value={props.destination}
                    placeholder="Choose a local folder"
                    onChange={(event) => props.onDestinationChange(event.target.value)}
                  />
                  <Button variant="outline" onClick={props.onChooseFolder} disabled={props.busy}>
                    <FolderOpen />
                    Browse
                  </Button>
                </div>
              </div>
              <div className="export-field">
                <Label htmlFor="export-pattern" className="export-field-label">
                  Name pattern
                </Label>
                <Input
                  id="export-pattern"
                  value={props.pattern}
                  placeholder={DEFAULT_PATTERN}
                  aria-invalid={patternProblem !== null}
                  onChange={(event) => props.onPatternChange(event.target.value)}
                />
                {patternProblem === null ? (
                  <details className="export-disclosure">
                    <summary>Naming options</summary>
                    <p>
                      Enter a name or use {'{index}'}, {'{clip}'}, {'{project}'}, {'{duration}'},{' '}
                      {'{date}'} or {'{address}'}. Plain names get a clip number automatically.
                    </p>
                  </details>
                ) : (
                  <p className="export-field-problem" data-testid="pattern-problem">
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
              <FileNames plan={props.plan} planning={props.planning} />
            </Section>

            {props.onFormatChange && (
              <Section title="Format">
                <div className="export-field-row">
                  <Choice
                    id="export-size"
                    label="Resolution"
                    value={String(format.height)}
                    options={HEIGHT_CHOICES.map((height) => ({
                      value: String(height),
                      text: sizeName(height),
                      hint: heightLabel(height, shape),
                    }))}
                    onChange={(height) =>
                      props.onFormatChange?.({
                        ...format,
                        height: Number(height) as FormatChoice['height'],
                      })
                    }
                  />
                  <Choice
                    id="export-rate"
                    label="Frame rate"
                    value={format.rate}
                    options={RATE_CHOICES.map((rate) => ({
                      value: rate,
                      text: rate === 'source' ? 'Match recording' : `${rate} fps`,
                      hint: rateLabel(rate, props.sourceFps ?? null),
                    }))}
                    onChange={(rate) => props.onFormatChange?.({ ...format, rate })}
                  />
                </div>
                {rateNote(format.rate, props.sourceFps ?? null) && (
                  <p className="export-note">{rateNote(format.rate, props.sourceFps ?? null)}</p>
                )}
                <details className="export-disclosure">
                  <summary>Format specifications</summary>
                  <dl className="export-specs">
                    {deliverySpecs(format, shape).map(([label, value]) => (
                      <div key={label}>
                        <dt>{label}</dt>
                        <dd>{value}</dd>
                      </div>
                    ))}
                  </dl>
                </details>
              </Section>
            )}

            <Section title="Permission">
              <div className="export-field">
                <Label htmlFor="source-rights" className="export-field-label">
                  Source rights
                </Label>
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
                <p className="export-note">Your choice is saved with this export.</p>
              </div>
              {props.rightsGateNeeded && (
                <label className="export-consent">
                  <input
                    type="checkbox"
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
            </Section>

            <Section title="Checks">
              <ul className="export-checks" aria-label="Export checks">
                {props.planning && (
                  <li className="export-check" data-tone="quiet">
                    <Spinner />
                    <p className="export-check-title">Checking…</p>
                  </li>
                )}
                {props.plan === null && !props.planning && (
                  <li className="export-check" data-tone="quiet">
                    <CircleAlert />
                    <p className="export-check-detail">The checks run once a folder is chosen.</p>
                  </li>
                )}
                {blocking.map((finding) => (
                  <Check
                    key={`${finding.code}:${finding.detail}`}
                    tone="danger"
                    title={findingTitle(finding.code)}
                    detail={finding.detail}
                  >
                    {finding.code === 'source.missing' && props.onRelink && (
                      <Button
                        variant="outline"
                        size="sm"
                        disabled={props.busy}
                        onClick={props.onRelink}
                      >
                        <Link2 /> Locate recording…
                      </Button>
                    )}
                    {fix(finding)}
                  </Check>
                ))}
                {props.hotCaptions.length > 0 && (
                  <Check
                    tone="warning"
                    title={`${props.hotCaptions.length} fast subtitle ${
                      props.hotCaptions.length === 1 ? 'passage' : 'passages'
                    }`}
                    detail={`${
                      hottest === null ? '' : `Up to ${hottest.toFixed(1)} characters a second. `
                    }They do not hold up export; you can review them in Captions.`}
                  >
                    <details className="export-disclosure">
                      <summary>Review caption details ({props.hotCaptions.length})</summary>
                      <ol className="export-caption-list">
                        {props.hotCaptions.map((finding, index) => (
                          <li key={`${finding.detail}:${index}`}>
                            {finding.detail.replace(/ — confirmed as read\.$/, '')}
                          </li>
                        ))}
                      </ol>
                    </details>
                    {captionFocus !== null && grouped(captionFocus.finding) && fix(null)}
                  </Check>
                )}
                {advisory.map((finding) => (
                  <Check
                    key={`${finding.code}:${finding.detail}`}
                    tone="warning"
                    title={findingTitle(finding.code)}
                    detail={finding.detail}
                  >
                    {fix(finding)}
                  </Check>
                ))}
                {props.plan !== null && !props.planning && props.plan.findings.length === 0 && (
                  <li className="export-check" data-tone="success">
                    <CircleCheck />
                    <p className="export-check-title">All checks passed</p>
                  </li>
                )}
              </ul>
            </Section>

            <div className="export-footer">
              {props.error !== null && patternProblem === null && (
                <Alert variant="destructive">
                  <TriangleAlert />
                  <AlertDescription>{props.error}</AlertDescription>
                </Alert>
              )}
              <div className="export-go">
                <div className="export-go-text">
                  <p className="export-status" data-tone={statusTone(status)} role="status">
                    {status}
                  </p>
                  <dl className="export-figures">
                    <div>
                      <dt>Estimated size</dt>
                      <dd>{props.plan === null ? '—' : formatBytes(props.plan.estimatedBytes)}</dd>
                    </div>
                    <div>
                      <dt>Free disk space</dt>
                      <dd>
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
                </div>
                <Button
                  size="lg"
                  className="export-go-button"
                  onClick={() => {
                    following.current = true;
                    props.onExport();
                  }}
                  disabled={!ready || delivering}
                >
                  {props.busy ? 'Working…' : 'Export clip'}
                </Button>
              </div>
            </div>
          </section>

          {props.delivery !== null && (
            <DeliveryPanel
              ref={deliveryPanel}
              delivery={props.delivery}
              mediaSeconds={props.mediaSeconds ?? 0}
              onReveal={props.onReveal}
              onCancel={props.onCancel}
              onRetry={props.onRetry}
            />
          )}

          {props.publishing}

          <section className="export-archive" aria-labelledby="export-archive-title">
            <div className="export-archive-text">
              <h2 id="export-archive-title">Project archive</h2>
              <p>
                Saves this project’s edits and export records in the export folder. Source
                recordings stay in their original location.
              </p>
              {props.archive !== null && (
                <p className="export-archive-result" role="status">
                  {props.archive.entryCount} documents written to <span>{props.archive.path}</span>
                </p>
              )}
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={props.onArchive}
              disabled={props.busy || !props.destination.trim()}
            >
              Save project archive
            </Button>
          </section>
        </div>
      </div>
    </div>
  );
}

/**
 * The clip, as it will be delivered and, once it has been, as it was.
 *
 * The loop is the edit drawn the way the Editor draws it; the exported file is
 * the encoder's own output with its sound. A file that has just been made is
 * the next thing to look at, so it is shown as it arrives.
 */
function Stage({
  preview,
  exported,
  problem,
  onError,
  staleRevision,
  summary,
  shape,
  seconds,
}: {
  readonly preview: ReactNode;
  readonly exported: string | null;
  readonly problem: string | null;
  readonly onError: (() => void) | undefined;
  readonly staleRevision: number | null;
  readonly summary: string;
  readonly shape: FrameShape;
  readonly seconds: number;
}): JSX.Element {
  const [view, setView] = useState<'preview' | 'exported'>('preview');
  // The file is shown when it is the edit as it stands; after further edits
  // the loop is the truer picture, and the file is one press away.
  useEffect(() => {
    if (exported) setView(staleRevision === null ? 'exported' : 'preview');
  }, [exported, staleRevision]);
  const playable = exported !== null && problem === null;
  const showing = playable && view === 'exported' ? 'exported' : 'preview';
  const frame = frameOfShape(shape);
  const kind = FRAME_SHAPES.find((entry) => entry.shape === shape);
  return (
    <aside className="export-stage" aria-label="The clip">
      {playable && (
        <div className="export-choice export-stage-switch" role="group" aria-label="Show">
          <button
            type="button"
            aria-pressed={showing === 'preview'}
            onClick={() => setView('preview')}
          >
            Preview
          </button>
          <button
            type="button"
            aria-pressed={showing === 'exported'}
            onClick={() => setView('exported')}
          >
            Exported file
          </button>
        </div>
      )}
      <div className="export-picture">
        {showing === 'exported' && exported ? (
          <video
            aria-label="The exported clip"
            src={exported}
            onError={onError}
            controls
            playsInline
            preload="metadata"
            className="export-exported"
            style={{ aspectRatio: `${frame.width} / ${frame.height}` }}
          />
        ) : (
          preview
        )}
      </div>
      <div className="export-stage-caption">
        <p className="export-stage-format">{summary}</p>
        <p className="export-stage-meta">
          {[
            kind ? `${kind.label} ${kind.ratio}` : null,
            seconds > 0 ? clock(seconds * 90_000) : null,
          ]
            .filter(Boolean)
            .join(' · ')}
        </p>
        <p className="export-stage-meta">Captions and mastered audio included.</p>
        {showing === 'exported' && (
          <p className="export-stage-note">
            The encoded file, with its final captions, framing and mastered audio. Review it before
            uploading.
            {staleRevision !== null
              ? ` Your edit has changed since (now r${staleRevision}); export again to review it.`
              : ''}
          </p>
        )}
        {problem && (
          <p className="export-stage-note" role="status">
            {problem}
          </p>
        )}
      </div>
    </aside>
  );
}

function Section({
  title,
  children,
}: {
  readonly title: string;
  readonly children: ReactNode;
}): JSX.Element {
  return (
    <section className="export-section">
      <h2 className="export-section-title">{title}</h2>
      <div className="export-section-body">{children}</div>
    </section>
  );
}

/** A handful of choices, all in view: one press, and the summary says what it made. */
function Choice<T extends string>({
  id,
  label,
  value,
  options,
  onChange,
}: {
  readonly id: string;
  readonly label: string;
  readonly value: T;
  readonly options: readonly { readonly value: T; readonly text: string; readonly hint: string }[];
  readonly onChange: (value: T) => void;
}): JSX.Element {
  return (
    <div className="export-field">
      <span className="export-field-label" id={id}>
        {label}
      </span>
      <div className="export-choice" role="group" aria-labelledby={id}>
        {options.map((option) => (
          <button
            key={option.value}
            type="button"
            title={option.hint}
            aria-pressed={option.value === value}
            onClick={() => onChange(option.value)}
          >
            {option.text}
          </button>
        ))}
      </div>
    </div>
  );
}

function Check({
  tone,
  title,
  detail,
  children,
}: {
  readonly tone: 'danger' | 'warning';
  readonly title: string;
  readonly detail: string;
  readonly children?: ReactNode;
}): JSX.Element {
  return (
    <li className="export-check" data-tone={tone}>
      {tone === 'danger' ? <TriangleAlert /> : <CircleAlert />}
      <div className="export-check-body">
        <p className="export-check-title">{title}</p>
        <p className="export-check-detail">{detail}</p>
        {children}
      </div>
    </li>
  );
}

/** The way from a caption finding to the caption, with the fix waiting there. */
function CaptionFix({
  focus,
  grouped,
  busy,
  onEdit,
}: {
  readonly focus: NonNullable<ReturnType<typeof captionFocusOf>>;
  /** Whether it stands for a group of fast passages rather than one finding. */
  readonly grouped: boolean;
  readonly busy: boolean;
  readonly onEdit: ExportProps['onEdit'];
}): JSX.Element {
  return (
    <div className="export-check-fix">
      <p>
        {grouped
          ? 'The editor opens on the first, with a fix ready to apply.'
          : `${
              focus.blocking
                ? 'This caption cannot be exported as it is.'
                : 'This caption is worth a look before exporting.'
            } The editor opens on it, with a fix ready to apply.`}
      </p>
      {onEdit && (
        <Button
          size="sm"
          variant={focus.blocking ? 'default' : 'outline'}
          onClick={() => onEdit(focus.focus)}
          disabled={busy}
        >
          <Captions /> Fix captions
        </Button>
      )}
    </div>
  );
}

/** What still stands between this screen and an export, in one line. */
function readiness(props: ExportProps, blocking: number, delivering: boolean): string {
  if (delivering) return 'Exporting…';
  if (!props.destination.trim()) return 'Choose a folder to export into.';
  if (props.planning) return 'Checking…';
  if (!props.attestation) return 'Choose the permission you hold for this footage.';
  if (props.rightsGateNeeded && !props.rightsGatePassed)
    return 'Confirm your permission covers a clip longer than a minute.';
  if (blocking > 0)
    return blocking === 1 ? 'One check to fix first.' : `${blocking} checks to fix first.`;
  if (props.plan?.passes === true) return 'Ready to export.';
  return '';
}

function statusTone(status: string): 'ready' | 'blocked' | 'quiet' {
  if (status === 'Ready to export.') return 'ready';
  if (status.endsWith('to fix first.')) return 'blocked';
  return 'quiet';
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
function DeliveryPanel({
  ref,
  delivery,
  mediaSeconds,
  onReveal,
  onCancel,
  onRetry,
}: {
  readonly ref: Ref<HTMLElement>;
  readonly delivery: Delivery;
  readonly mediaSeconds: number;
  readonly onReveal: (path: string) => void;
  readonly onCancel?: (() => void) | undefined;
  readonly onRetry?: (() => void) | undefined;
}): JSX.Element {
  const render = delivery.stages.find((stage) => stage.kind === 'render');
  const progress = render?.progress;
  const percent = progress?.unit.startsWith('export.')
    ? Math.min(100, Math.round((100 * progress.done) / Math.max(1, progress.total)))
    : 0;
  const remaining = estimatedExportSeconds(mediaSeconds, percent / 100);
  const plainFailure = delivery.failure?.includes('stopped making progress')
    ? 'Rendering stopped advancing. Try the export again.'
    : delivery.failure?.includes('overall safety limit')
      ? 'Rendering took too long. Try the export again.'
      : delivery.failure?.includes('invalid local source') ||
          delivery.failure?.includes('path does not exist')
        ? 'The source recording could not be found. Locate it, then retry.'
        : delivery.failure
          ? 'The export could not finish. You can try again.'
          : null;
  const state = delivery.files ? 'done' : delivery.failure ? 'failed' : 'running';
  return (
    <section
      ref={ref}
      className="export-panel export-delivery"
      data-state={state}
      data-testid="delivery"
      aria-labelledby="export-delivery-title"
    >
      <header className="export-delivery-head">
        {state === 'done' ? <CircleCheck /> : state === 'failed' ? <TriangleAlert /> : <Spinner />}
        <div className="export-delivery-title">
          <h2 id="export-delivery-title">
            {state === 'done'
              ? 'Exported'
              : state === 'failed'
                ? 'This export did not finish'
                : 'Exporting…'}
          </h2>
          {delivery.files === null && <p>{delivery.destinationDir}</p>}
        </div>
        {!delivery.settled && onCancel && (
          <Button size="sm" variant="outline" onClick={onCancel}>
            Cancel export
          </Button>
        )}
      </header>
      <div className="export-delivery-body">
        {progress?.unit.startsWith('export.') && !delivery.settled && (
          <div
            role="progressbar"
            aria-label="Export progress"
            aria-valuenow={percent}
            aria-valuemin={0}
            aria-valuemax={100}
            className="export-progress"
          >
            <div className="export-progress-track">
              <div className="export-progress-fill" style={{ width: `${percent}%` }} />
            </div>
            {remaining !== null && (
              <p>
                About {Math.max(1, Math.ceil(remaining / 60))} min remaining, from recent exports on
                this machine
              </p>
            )}
          </div>
        )}
        <ul className="export-stages" aria-label="Delivery stages">
          {delivery.stages.map((stage) => (
            <li key={stage.kind}>
              <span>{stage.label}</span>
              <span data-state={stage.state} data-testid={`stage-${stage.kind}`}>
                {stageWord(stage)}
              </span>
            </li>
          ))}
        </ul>
        {delivery.interruption !== null && (
          <p className="export-note" data-testid="delivery-interruption">
            Export is still running in the background. Reconnecting to its progress… (
            {delivery.interruption})
          </p>
        )}
        {delivery.failure !== null && (
          <>
            <Alert variant="destructive">
              <TriangleAlert />
              <AlertDescription>{plainFailure}</AlertDescription>
            </Alert>
            <div className="export-delivery-actions">
              {onRetry && (
                <Button size="sm" onClick={onRetry}>
                  Retry export
                </Button>
              )}
              <Button
                size="sm"
                variant="outline"
                onClick={() => void navigator.clipboard?.writeText(delivery.failure ?? '')}
              >
                Copy details
              </Button>
            </div>
            <details className="export-disclosure">
              <summary>Technical details</summary>
              <p className="export-technical">{delivery.failure}</p>
            </details>
          </>
        )}
        {delivery.files !== null && (
          <ul className="export-delivered" aria-label="Delivered files">
            {delivery.files.map((file) => {
              const folder = file.path.endsWith(file.name)
                ? file.path.slice(0, file.path.length - file.name.length)
                : '';
              return (
                <li key={file.name}>
                  <span className="export-delivered-path" title={file.path}>
                    <span className="export-delivered-folder">{folder}</span>
                    <span className="export-delivered-name">{folder ? file.name : file.path}</span>
                  </span>
                  <span className="export-delivered-size">{formatBytes(file.bytes)}</span>
                  <Button
                    size="xs"
                    variant="ghost"
                    onClick={() => onReveal(file.path)}
                    aria-label={`Reveal ${file.name}`}
                  >
                    <Eye /> Reveal
                  </Button>
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </section>
  );
}

/**
 * The names, as the daemon resolved them.
 *
 * Deliberately not computed here. See the note at the top of the file.
 */
function FileNames({
  plan,
  planning,
}: {
  readonly plan: ExportPlan | null;
  readonly planning: boolean;
}): JSX.Element {
  if (plan === null) {
    return (
      <p className="export-note">
        {planning ? 'Resolving…' : 'Choose a folder to see what the files will be called.'}
      </p>
    );
  }
  const main = plan.fileNames.find((name) => name.endsWith('.mp4')) ?? plan.fileNames[0];
  const rest = plan.fileNames.filter((name) => name !== main);
  return (
    <div className="export-files">
      <span className="export-field-label">Files</span>
      {rest.length === 0 ? (
        <p className="export-files-main">{main}</p>
      ) : (
        <details className="export-disclosure export-files-more">
          <summary>
            <span className="export-files-main">{main}</span>
            <span className="export-files-count">and {rest.length} more</span>
          </summary>
          <ul>
            {rest.map((name) => (
              <li key={name}>{name}</li>
            ))}
          </ul>
        </details>
      )}
    </div>
  );
}
