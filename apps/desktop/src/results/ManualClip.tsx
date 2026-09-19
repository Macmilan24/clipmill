import { Scissors } from 'lucide-react';
import { useRef, useState } from 'react';
import { Button } from '../components/ui/button.js';
import { Input } from '../components/ui/input.js';
import { Label } from '../components/ui/label.js';
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from '../components/ui/sheet.js';
import { Spinner } from '../components/ui/spinner.js';
import { manualSpanProblem, parseSourceTime, sourceTime } from './manual.js';

export interface ManualClipProps {
  readonly open: boolean;
  readonly onOpenChange: (value: boolean) => void;
  readonly sourceName: string;
  readonly sourceDurationTicks: number | null;
  readonly proxyUrl: string | null;
  readonly busy: boolean;
  readonly notice: string | null;
  readonly onCreate: (start: number, end: number) => Promise<boolean>;
}

/** A human-selected source span, never a generated recommendation. */
export function ManualClip({
  open,
  onOpenChange,
  sourceName,
  sourceDurationTicks,
  proxyUrl,
  busy,
  notice,
  onCreate,
}: ManualClipProps) {
  const [start, setStart] = useState('0:00');
  const [enteredEnd, setEnd] = useState<string | null>(null);
  const end = enteredEnd ?? sourceTime(Math.min(sourceDurationTicks ?? 60 * 90_000, 60 * 90_000));
  const [position, setPosition] = useState(0);
  const [mediaProblem, setMediaProblem] = useState<string | null>(null);
  const [requestProblem, setRequestProblem] = useState<string | null>(null);
  const creating = useRef(false);
  const video = useRef<HTMLVideoElement>(null);
  const startTicks = parseSourceTime(start);
  const endTicks = parseSourceTime(end);
  const problem = manualSpanProblem(startTicks, endTicks, sourceDurationTicks);
  const create = async () => {
    if (busy || creating.current || problem || startTicks === null || endTicks === null) return;
    creating.current = true;
    setRequestProblem(null);
    try {
      if (await onCreate(startTicks, endTicks)) onOpenChange(false);
    } catch (cause) {
      setRequestProblem(cause instanceof Error ? cause.message : String(cause));
    } finally {
      creating.current = false;
    }
  };
  return (
    <Sheet
      open={open}
      onOpenChange={(value) => {
        if (!busy) onOpenChange(value);
      }}
    >
      <SheetContent
        className="w-full gap-0 overflow-hidden bg-[var(--cm-glass)] sm:max-w-[660px]"
        showCloseButton={!busy}
      >
        <SheetHeader className="shrink-0 border-b border-[var(--cm-glass-border)] p-5 pr-10">
          <SheetTitle className="flex items-center gap-2">
            <Scissors className="size-4" />
            Make a manual clip
          </SheetTitle>
          <SheetDescription className="text-xs leading-relaxed">
            Choose a source span and open it in the editor. This is your selection; the model has
            not recommended or reviewed it.
          </SheetDescription>
        </SheetHeader>
        <div className="min-h-0 flex-1 space-y-5 overflow-y-auto p-5">
          <div>
            <p className="mb-2 truncate text-xs font-medium" title={sourceName}>
              {sourceName}
            </p>
            {proxyUrl ? (
              <video
                ref={video}
                src={proxyUrl}
                controls
                playsInline
                preload="metadata"
                className="aspect-video w-full rounded-lg bg-black"
                aria-label="Recording preview"
                onTimeUpdate={(event) =>
                  setPosition(Math.round(event.currentTarget.currentTime * 90_000))
                }
                onError={() =>
                  setMediaProblem(
                    'The source preview could not be opened. You can still enter a known source time.',
                  )
                }
              />
            ) : (
              <div className="grid h-32 place-items-center rounded-lg border border-dashed border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] p-6 text-center text-xs text-[var(--cm-text-secondary)]">
                No source proxy is available. Enter source times below.
              </div>
            )}
            <div className="mt-2 flex justify-between text-[10px] text-[var(--cm-text-muted)]">
              <span>
                {proxyUrl
                  ? 'Source preview · draft audio and picture'
                  : 'Source timing from analysis'}
              </span>
              <span className="font-mono">
                {sourceDurationTicks === null
                  ? 'Duration unavailable'
                  : sourceTime(sourceDurationTicks)}
              </span>
            </div>
            {mediaProblem && (
              <p role="status" className="mt-2 text-xs text-[var(--cm-warning-ink)]">
                {mediaProblem}
              </p>
            )}
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="manual-start">Start time</Label>
              <Input
                id="manual-start"
                value={start}
                disabled={busy}
                aria-describedby="manual-time-help"
                aria-invalid={startTicks === null}
                onChange={(event) => setStart(event.target.value)}
                placeholder="0:00"
                className="font-mono"
              />
              <Button
                variant="outline"
                size="sm"
                disabled={busy || !proxyUrl || mediaProblem !== null}
                onClick={() => setStart(sourceTime(position))}
              >
                Use playhead as start
              </Button>
            </div>
            <div className="space-y-2">
              <Label htmlFor="manual-end">End time</Label>
              <Input
                id="manual-end"
                value={end}
                disabled={busy}
                aria-describedby="manual-time-help"
                aria-invalid={
                  endTicks === null ||
                  (endTicks !== null &&
                    sourceDurationTicks !== null &&
                    endTicks > sourceDurationTicks)
                }
                onChange={(event) => setEnd(event.target.value)}
                placeholder="1:00"
                className="font-mono"
              />
              <Button
                variant="outline"
                size="sm"
                disabled={busy || !proxyUrl || mediaProblem !== null}
                onClick={() => setEnd(sourceTime(position))}
              >
                Use playhead as end
              </Button>
            </div>
          </div>
          <div className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] px-3 py-3">
            <p id="manual-time-help" className="text-xs text-[var(--cm-text-secondary)]">
              {problem ?? `${sourceTime(endTicks! - startTicks!)} selected`}
            </p>
            <Button
              variant="ghost"
              size="sm"
              disabled={
                busy || !proxyUrl || startTicks === null || startTicks >= (sourceDurationTicks ?? 0)
              }
              onClick={() => {
                if (video.current && startTicks !== null)
                  video.current.currentTime = startTicks / 90_000;
              }}
            >
              Jump to start
            </Button>
          </div>
          <p className="text-[11px] leading-relaxed text-[var(--cm-text-muted)]">
            The editor builds captions and framing from this run’s existing evidence. Check
            incomplete speech, names, timing and the final rendered audition before export.
          </p>
          {(requestProblem ?? notice) && (
            <p role="status" className="text-xs leading-relaxed text-[var(--cm-warning-ink)]">
              {requestProblem ?? notice}
            </p>
          )}
        </div>
        <div className="flex shrink-0 justify-end gap-2 border-t border-[var(--cm-glass-border)] p-4">
          <Button variant="outline" disabled={busy} onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button disabled={busy || problem !== null} onClick={() => void create()}>
            {busy ? <Spinner /> : <Scissors />}
            {busy ? 'Creating edit…' : 'Create manual edit'}
          </Button>
        </div>
      </SheetContent>
    </Sheet>
  );
}
