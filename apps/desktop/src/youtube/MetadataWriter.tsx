import { useState } from 'react';
import { Check, Feather, RefreshCw, X } from 'lucide-react';
import { Button } from '../components/ui/button.js';
import { Spinner } from '../components/ui/spinner.js';
import type {
  PublishingApi,
  YoutubeMetadataDraft,
  YoutubeVideoMetadata,
} from '../daemon/publishing.js';
import { useMetadataWriter } from './useMetadataWriter.js';

export function MetadataWriter({
  api,
  exportJobId,
  revision,
  renderArtifactId,
  draft,
  disabled,
  onApply,
}: {
  readonly api: PublishingApi;
  readonly exportJobId: string;
  readonly revision: number;
  readonly renderArtifactId: string;
  readonly draft: YoutubeMetadataDraft;
  readonly disabled: boolean;
  readonly onApply: (metadata: YoutubeVideoMetadata) => void;
}) {
  const writer = useMetadataWriter(api, exportJobId, revision, renderArtifactId, draft);
  const [applied, setApplied] = useState(false);
  const suggestion = writer.state === 'succeeded' ? writer.draft.generatedMetadata : null;
  const failed = writer.state === 'failed' || writer.state === 'cancelled';
  const blockedSavedResult =
    writer.state === 'unavailable' && Boolean(writer.draft.generationJobId);
  const modelName =
    writer.draft.modelName?.includes('qwen3-5') || writer.draft.modelName?.includes('Qwen 3.5')
      ? 'Qwen 3.5'
      : 'Qwen';
  const title = writer.active
    ? writer.state === 'queued'
      ? 'Waiting for the local model'
      : 'Writing from this clip'
    : suggestion
      ? 'Your suggestion is ready'
      : 'Write with your local model';
  return (
    <section
      aria-label="Local metadata writer"
      className="overflow-hidden rounded-xl border border-[var(--cm-glass-border)] bg-[var(--cm-recessed)]"
    >
      <div className="flex flex-wrap items-start justify-between gap-3 p-4">
        <div className="min-w-0 flex-1">
          <p className="flex items-center gap-2 text-xs font-semibold">
            {writer.active ? (
              <Spinner />
            ) : (
              <Feather className="size-3.5 text-[var(--cm-text-secondary)]" />
            )}
            {title}
          </p>
          <p className="mt-1.5 max-w-prose text-[11px] leading-relaxed text-[var(--cm-text-secondary)]">
            {writer.active
              ? 'You can keep editing while Qwen prepares a title, description, and tags. Your text stays unchanged.'
              : 'Qwen uses the transcript of rendered r' +
                revision +
                '. Review its suggestion before applying it.'}
          </p>
          <p className="mt-1.5 text-[10px] text-[var(--cm-text-muted)]">
            {modelName} · On this device · Optional
          </p>
        </div>
        {writer.active ? (
          <Button
            size="sm"
            variant="ghost"
            disabled={disabled || writer.pending}
            onClick={() => void writer.act('cancel')}
          >
            <X className="size-3.5" /> Cancel writing
          </Button>
        ) : (
          !suggestion &&
          !blockedSavedResult && (
            <Button
              size="sm"
              variant="outline"
              disabled={disabled || writer.pending}
              onClick={() =>
                void writer.act(failed && writer.draft.generationJobId ? 'retry' : 'start')
              }
            >
              {writer.pending ? <Spinner /> : <Feather className="size-3.5" />}
              {failed ? 'Retry writing' : 'Write with Qwen'}
            </Button>
          )
        )}
      </div>
      {(writer.error ||
        writer.draft.generationMessage ||
        failed ||
        writer.state === 'unavailable') && (
        <div className="px-4 pb-4 text-xs leading-relaxed text-[var(--cm-text-secondary)]">
          <p role={writer.error || writer.state === 'failed' ? 'alert' : 'status'}>
            {writer.error ||
              writer.draft.generationMessage ||
              (writer.state === 'cancelled'
                ? 'Writing cancelled. Your details are unchanged.'
                : 'Qwen is not ready to write. You can continue editing the details yourself.')}
          </p>
          {writer.error && writer.active && (
            <Button
              size="sm"
              variant="ghost"
              className="mt-2"
              disabled={disabled || writer.pending}
              onClick={writer.refresh}
            >
              <RefreshCw className="size-3.5" /> Refresh writing status
            </Button>
          )}
        </div>
      )}
      {suggestion && (
        <div className="space-y-3 border-t border-[var(--cm-glass-border)] p-4">
          <div>
            <p className="text-[10px] font-medium uppercase tracking-wider text-[var(--cm-text-muted)]">
              Suggested title
            </p>
            <p className="mt-1 break-words text-sm font-semibold">{suggestion.title}</p>
          </div>
          <p className="whitespace-pre-wrap break-words text-xs leading-relaxed text-[var(--cm-text-secondary)]">
            {suggestion.description}
          </p>
          {suggestion.tags.length > 0 && (
            <p className="break-words text-[11px] text-[var(--cm-text-muted)]">
              Tags · {suggestion.tags.join(' · ')}
            </p>
          )}
          <div className="flex flex-wrap items-center justify-between gap-3">
            <p className="max-w-sm text-[11px] leading-relaxed text-[var(--cm-text-muted)]">
              {applied
                ? 'Suggestion applied. Review the details before uploading.'
                : 'Applying replaces the title, description, and tags below. Audience and disclosure stay as you chose them.'}
            </p>
            <Button
              size="sm"
              variant="outline"
              disabled={disabled || writer.pending}
              onClick={() => {
                onApply(suggestion);
                setApplied(true);
              }}
            >
              <Check className="size-3.5" /> Apply suggestion
            </Button>
          </div>
        </div>
      )}
    </section>
  );
}
