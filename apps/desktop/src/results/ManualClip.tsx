/**
 * A clip nobody proposed, chosen on the same player the Inspector reviews
 * with: the recording plays in the Result or Source view, the cut is two
 * handles on the waveform strip, and In and Out mark it at the playhead,
 * landing on the word edges. The Result view shows the clip the edit will
 * start as — framing and captions — once the cut stops moving.
 */
import { ArrowLeftToLine, ArrowRightToLine, Scissors } from 'lucide-react';
import { useEffect, useMemo, useRef, useState, type KeyboardEvent } from 'react';

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
import type { PreviewPlan } from '../daemon/client.js';
import type { ExactCaptions } from '../editor/exactCaptions.js';
import { Monitor, type MonitorView } from '../inspector/Monitor.js';
import { PlaybackController } from '../inspector/playback.js';
import { reviewSpeed } from '../inspector/preferences.js';
import type { Cut } from '../inspector/review.js';
import { BoundaryStrip, initialView } from '../inspector/Timeline.js';
import type { Peaks } from './loader.js';
import { manualSpanProblem, parseSourceTime, sourceTime } from './manual.js';
import { type Transcript, snapEnd, snapStart } from './transcript.js';

const SECOND = 90_000;

export interface ManualClipProps {
  readonly open: boolean;
  readonly onOpenChange: (value: boolean) => void;
  readonly sourceName: string;
  readonly sourceDurationTicks: number | null;
  readonly proxyUrl: string | null;
  readonly busy: boolean;
  readonly notice: string | null;
  readonly onCreate: (start: number, end: number) => Promise<boolean>;
  /** The recording's words, for word-edge marks and the strip's snap points. */
  readonly transcript?: Transcript | null;
  readonly peaks?: Peaks | null;
  readonly tileUrl?: (atTicks: number) => string | null;
  /** The clip this span would make, and its captions, once asked for. */
  readonly preview?: PreviewPlan | null;
  readonly previewCaptions?: ExactCaptions | null;
  /** Ask for that clip for a span. */
  readonly onPreview?: ((cut: Cut) => void) | null;
}

/** A human-selected source span, never a generated recommendation. */
export function ManualClip(props: ManualClipProps) {
  const { open, onOpenChange, busy } = props;
  return (
    <Sheet
      open={open}
      onOpenChange={(value) => {
        if (!busy) onOpenChange(value);
      }}
    >
      <SheetContent
        className="w-full gap-0 overflow-hidden bg-[var(--cm-glass)] sm:max-w-[760px]"
        showCloseButton={!busy}
      >
        <SheetHeader className="shrink-0 border-b border-[var(--cm-glass-border)] p-5 pr-10">
          <SheetTitle className="flex items-center gap-2">
            <Scissors className="size-4" />
            Make a manual clip
          </SheetTitle>
          <SheetDescription className="text-xs leading-relaxed">
            Choose a span of the recording and open it in the editor. This is your selection; the
            model has not recommended or reviewed it.
          </SheetDescription>
        </SheetHeader>
        {open && <Chooser {...props} />}
      </SheetContent>
    </Sheet>
  );
}

function Chooser({
  onOpenChange,
  sourceName,
  sourceDurationTicks,
  proxyUrl,
  busy,
  notice,
  onCreate,
  transcript = null,
  peaks = null,
  tileUrl = () => null,
  preview = null,
  previewCaptions = null,
  onPreview = null,
}: ManualClipProps) {
  const duration = sourceDurationTicks ?? 0;
  // The first minute, ending where a word does, until a person moves it —
  // and taken again when the recording's length or words arrive before then.
  const opening = (): Cut => {
    const end = Math.min(duration || 60 * SECOND, 60 * SECOND);
    return { startTicks: 0, endTicks: transcript ? snapEnd(transcript, end) : end };
  };
  const [cut, setChosen] = useState<Cut>(opening);
  const touched = useRef(false);
  const setCut = (next: Cut | ((current: Cut) => Cut)) => {
    touched.current = true;
    setChosen(next);
  };
  useEffect(() => {
    if (!touched.current) setChosen(opening());
    // eslint-disable-next-line react-hooks/exhaustive-deps -- the evidence is the signal
  }, [duration, transcript]);
  // The strip and the player need a span that runs forwards; the fields may
  // briefly hold one that does not while a time is being typed.
  const shown: Cut =
    cut.endTicks > cut.startTicks
      ? cut
      : { startTicks: cut.startTicks, endTicks: cut.startTicks + SECOND };
  const [view, setView] = useState<Cut>(() => initialView(cut, Math.max(duration, cut.endTicks)));
  const [monitorView, setMonitorView] = useState<MonitorView>('result');
  const [safeArea, setSafeArea] = useState(false);
  const [requestProblem, setRequestProblem] = useState<string | null>(null);
  const creating = useRef(false);
  // eslint-disable-next-line react-hooks/exhaustive-deps -- one clock per opening
  const controller = useMemo(() => new PlaybackController(cut.startTicks, reviewSpeed()), []);
  useEffect(() => () => controller.dispose(), [controller]);
  useEffect(() => {
    controller.setCut(shown);
  }, [controller, shown.startTicks, shown.endTicks]);

  // The clip this span would make, a moment after the cut stops moving.
  const ask = useRef(onPreview);
  ask.current = onPreview;
  useEffect(() => {
    if (cut.endTicks <= cut.startTicks) return undefined;
    const timer = setTimeout(() => ask.current?.(cut), 300);
    return () => clearTimeout(timer);
  }, [cut]);

  const problem = manualSpanProblem(cut.startTicks, cut.endTicks, sourceDurationTicks);
  const mark = (edge: 'in' | 'out') => {
    const at = controller.getState().ticks;
    const landed = transcript
      ? edge === 'in'
        ? snapStart(transcript, at)
        : snapEnd(transcript, at)
      : at;
    setCut((current) =>
      edge === 'in'
        ? { startTicks: Math.min(landed, current.endTicks - SECOND), endTicks: current.endTicks }
        : {
            startTicks: current.startTicks,
            endTicks: Math.max(landed, current.startTicks + SECOND),
          },
    );
  };
  const create = async () => {
    if (busy || creating.current || problem) return;
    creating.current = true;
    setRequestProblem(null);
    try {
      if (await onCreate(cut.startTicks, cut.endTicks)) onOpenChange(false);
    } catch (cause) {
      setRequestProblem(cause instanceof Error ? cause.message : String(cause));
    } finally {
      creating.current = false;
    }
  };
  // Space plays, I and O mark — the Inspector's keys, while the sheet has focus.
  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.target instanceof HTMLInputElement || event.metaKey || event.ctrlKey) return;
    if (event.key === ' ') {
      event.preventDefault();
      controller.toggle();
    } else if (event.key === 'i' || event.key === 'I') {
      mark('in');
    } else if (event.key === 'o' || event.key === 'O') {
      mark('out');
    }
  };
  const suggested = useMemo(
    () => ({
      starts: transcript?.sentences.map((sentence) => sentence.startTicks) ?? [],
      ends: transcript?.sentences.map((sentence) => sentence.endTicks) ?? [],
    }),
    [transcript],
  );

  return (
    <>
      <div
        className="manual-clip min-h-0 flex-1 space-y-4 overflow-y-auto p-5"
        onKeyDown={onKeyDown}
        role="group"
        aria-label="Choose the span"
      >
        <p className="truncate text-xs font-medium" title={sourceName}>
          {sourceName}
        </p>
        <div className="manual-clip-monitor">
          <Monitor
            src={proxyUrl}
            crop={null}
            plan={preview}
            captions={previewCaptions}
            controller={controller}
            cut={shown}
            view={monitorView}
            onView={setMonitorView}
            safeArea={safeArea}
            onSafeArea={setSafeArea}
            alternative={null}
          />
        </div>
        {duration > 0 && (
          <BoundaryStrip
            cut={shown}
            chosen={shown}
            alternative={null}
            suggestedStarts={suggested.starts}
            suggestedEnds={suggested.ends}
            transcript={transcript}
            peaks={peaks}
            tileUrl={tileUrl}
            durationTicks={duration}
            view={view}
            onView={setView}
            controller={controller}
            onCut={busy ? null : setCut}
          />
        )}
        <div className="flex flex-wrap items-end justify-between gap-3 rounded-lg border border-[var(--cm-glass-border)] bg-[var(--cm-recessed)] px-3 py-2.5">
          <div className="flex items-end gap-3">
            <TimeField
              id="manual-start"
              label="Start time"
              ticks={cut.startTicks}
              disabled={busy}
              onTicks={(ticks) => setCut((current) => ({ ...current, startTicks: ticks }))}
            />
            <TimeField
              id="manual-end"
              label="End time"
              ticks={cut.endTicks}
              disabled={busy}
              onTicks={(ticks) => setCut((current) => ({ ...current, endTicks: ticks }))}
            />
            <p id="manual-time-help" className="pb-2 text-xs text-[var(--cm-text-secondary)]">
              {problem ?? `${sourceTime(cut.endTicks - cut.startTicks)} selected`}
            </p>
          </div>
          <div className="flex gap-2">
            <Button variant="outline" size="sm" disabled={busy} onClick={() => mark('in')}>
              <ArrowLeftToLine className="size-4" aria-hidden="true" />
              Start here
            </Button>
            <Button variant="outline" size="sm" disabled={busy} onClick={() => mark('out')}>
              <ArrowRightToLine className="size-4" aria-hidden="true" />
              End here
            </Button>
          </div>
        </div>
        <p className="text-[11px] leading-relaxed text-[var(--cm-text-muted)]">
          Drag the handles on the strip, or mark at the playhead: the start and end land on the
          nearest word edge. The editor builds captions and framing from this run’s evidence; check
          names and timing in the rendered audition before export.
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
    </>
  );
}

/**
 * A time typed exactly, for a known timecode or a keyboard. It moves the cut
 * as soon as it reads as one, and shows the cut's own time when left.
 */
function TimeField({
  id,
  label,
  ticks,
  disabled,
  onTicks,
}: {
  readonly id: string;
  readonly label: string;
  readonly ticks: number;
  readonly disabled: boolean;
  readonly onTicks: (ticks: number) => void;
}) {
  const [text, setText] = useState(() => sourceTime(ticks));
  const [editing, setEditing] = useState(false);
  useEffect(() => {
    if (!editing) setText(sourceTime(ticks));
  }, [ticks, editing]);
  return (
    <div className="space-y-1">
      <Label htmlFor={id} className="text-[11px]">
        {label}
      </Label>
      <Input
        id={id}
        value={text}
        disabled={disabled}
        aria-describedby="manual-time-help"
        aria-invalid={parseSourceTime(text) === null}
        className="h-8 w-28 font-mono"
        onFocus={() => setEditing(true)}
        onBlur={() => setEditing(false)}
        onChange={(event) => {
          setText(event.target.value);
          const parsed = parseSourceTime(event.target.value);
          if (parsed !== null) onTicks(parsed);
        }}
      />
    </div>
  );
}
