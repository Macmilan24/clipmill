import type { ExportPlan, ExportRequest } from '../daemon/client.js';

export const HOT_CAPTION_CODE = 'captions.reading_rate';

/** A confirmation belongs to exactly the document, revision, and findings read. */
export function approvalScope(docId: string, plan: ExportPlan, attestation: string): string {
  return JSON.stringify([
    docId,
    plan.revision,
    attestation,
    plan.findings
      .filter((finding) => finding.code === HOT_CAPTION_CODE)
      .map((finding) => finding.detail.replace(/ — confirmed as read\.$/, ''))
      .toSorted(),
  ]);
}

/** Every collection member has an explicit, stable one-based ordinal. */
export function collectionPattern(pattern: string): string {
  const value = pattern.trim() || '{index}-{clip}';
  return value.includes('{index}') ? value : `${value}-{index}`;
}

export interface PreparedClip {
  readonly request: ExportRequest;
  readonly plan: ExportPlan;
  readonly durationTicks: number;
  readonly scope: string;
}

export interface ClipChoices {
  readonly title: string;
  readonly attestation: string;
  readonly rightsApproval: string | null;
  readonly captionsApproval: string | null;
}

export const EMPTY_CHOICES: ClipChoices = {
  title: '',
  attestation: '',
  rightsApproval: null,
  captionsApproval: null,
};

export function confirmationGates(
  choices: ClipChoices,
  prepared: PreparedClip | undefined,
): string[] {
  if (!prepared) return [];
  const scope = approvalScope(prepared.request.docId, prepared.plan, choices.attestation);
  return [
    ...(choices.rightsApproval === scope ? ['duration_60s'] : []),
    ...(choices.captionsApproval === scope ? ['captions_reading_rate'] : []),
  ];
}

/** Use all program segments; frame rounding is not a source-duration measurement. */
export function sourceDuration(segments: readonly { inTicks: number; outTicks: number }[]): number {
  return segments.reduce((sum, segment) => sum + segment.outTicks - segment.inTicks, 0);
}
