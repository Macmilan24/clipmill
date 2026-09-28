/**
 * The small controls every properties tab is built from.
 *
 * A slider or a colour well moves freely and sends one command when let go —
 * an edit per pixel would flood the log and the undo stack. While it moves it
 * may report a draft, which the caption controls use to redraw the preview
 * with the value before it is chosen.
 */
import { Minus, Plus } from 'lucide-react';
import { type ReactNode, useEffect, useRef, useState } from 'react';

import { Button } from '../components/ui/button.js';
import { Input } from '../components/ui/input.js';

const TICKS = 90_000;

export function Field({
  label,
  children,
}: {
  readonly label: string;
  readonly children: ReactNode;
}) {
  return (
    <div className="edit-field">
      <span className="edit-field-label">{label}</span>
      <div className="edit-field-control">{children}</div>
    </div>
  );
}

/** A slider that moves freely and sends its value once, when let go. */
export function CommitSlider({
  label,
  min,
  max,
  value,
  disabled,
  format,
  onCommit,
  onDraft,
}: {
  readonly label: string;
  readonly min: number;
  readonly max: number;
  readonly value: number;
  readonly disabled: boolean;
  readonly format: (value: number) => string;
  readonly onCommit: (value: number) => void;
  /** Every value the slider passes through, before it is let go. */
  readonly onDraft?: (value: number) => void;
}) {
  const [draft, setDraft] = useState(value);
  useEffect(() => setDraft(value), [value]);
  const commit = () => {
    if (draft !== value) onCommit(draft);
  };
  return (
    <div className="edit-slider">
      <input
        type="range"
        aria-label={label}
        min={min}
        max={max}
        step={1}
        value={draft}
        disabled={disabled}
        onChange={(event) => {
          const next = Number(event.target.value);
          setDraft(next);
          onDraft?.(next);
        }}
        onPointerUp={commit}
        onKeyUp={commit}
        onBlur={commit}
      />
      <span className="mono">{format(draft)}</span>
    </div>
  );
}

/** A colour well that sends its colour once the picker closes. */
export function Swatch({
  label,
  value,
  disabled,
  onCommit,
  onDraft,
}: {
  readonly label: string;
  readonly value: string;
  readonly disabled: boolean;
  readonly onCommit: (value: string) => void;
  /** Every colour the picker shows before it closes. */
  readonly onDraft?: (value: string) => void;
}) {
  const shown = value.slice(0, 7).toLowerCase();
  const [draft, setDraft] = useState(shown);
  const input = useRef<HTMLInputElement>(null);
  const latest = useRef({ shown, onCommit });
  latest.current = { shown, onCommit };
  useEffect(() => setDraft(shown), [shown]);
  // React's onChange is the input event; the picker's own change is its close.
  useEffect(() => {
    const element = input.current;
    if (!element) return;
    const closed = () => {
      if (element.value !== latest.current.shown) latest.current.onCommit(element.value);
    };
    element.addEventListener('change', closed);
    return () => element.removeEventListener('change', closed);
  }, []);
  return (
    <label className="edit-swatch" title={label}>
      <input
        ref={input}
        type="color"
        aria-label={label}
        value={draft}
        disabled={disabled}
        onChange={(event) => {
          setDraft(event.target.value);
          onDraft?.(event.target.value);
        }}
      />
      <span style={{ background: draft }} aria-hidden="true" />
      <span className="edit-swatch-label">{label}</span>
    </label>
  );
}

export function Stepper({
  label,
  value,
  min,
  max,
  onChange,
}: {
  readonly label: string;
  readonly value: number;
  readonly min: number;
  readonly max: number;
  readonly onChange: (value: number) => void;
}) {
  return (
    <div className="edit-stepper" role="group" aria-label={label}>
      <button
        type="button"
        aria-label="Fewer"
        disabled={value <= min}
        onClick={() => onChange(value - 1)}
      >
        <Minus className="size-3" aria-hidden="true" />
      </button>
      <span className="mono" aria-live="polite">
        {value}
      </span>
      <button
        type="button"
        aria-label="More"
        disabled={value >= max}
        onClick={() => onChange(value + 1)}
      >
        <Plus className="size-3" aria-hidden="true" />
      </button>
    </div>
  );
}

/** Seconds typed exactly, or taken from the playhead. */
export function SpanTiming({
  noun = 'Text',
  startTicks,
  endTicks,
  programEnd,
  playhead,
  busy,
  onCommit,
}: {
  /** What is being timed, for the fields' names. */
  readonly noun?: string;
  readonly startTicks: number;
  readonly endTicks: number;
  readonly programEnd: number;
  readonly playhead: number;
  readonly busy: boolean;
  readonly onCommit: (start: number, end: number) => void;
}) {
  const [start, setStart] = useState(String(startTicks / TICKS));
  const [end, setEnd] = useState(String(endTicks / TICKS));
  useEffect(() => {
    setStart(String(startTicks / TICKS));
    setEnd(String(endTicks / TICKS));
  }, [startTicks, endTicks]);
  const valid = (from: number, to: number) =>
    Number.isFinite(from) && Number.isFinite(to) && from >= 0 && to > from && to <= programEnd;
  const commit = (from: number, to: number) => {
    if (valid(from, to) && (from !== startTicks || to !== endTicks)) onCommit(from, to);
  };
  const typed = (value: string) => Math.round(Number(value) * TICKS);
  return (
    <div className="space-y-2">
      <div className="flex items-end gap-2">
        <label className="edit-field-stack">
          <span className="edit-field-label">From</span>
          <Input
            aria-label={`${noun} starts at`}
            className="h-8 w-24 font-mono"
            value={start}
            disabled={busy}
            inputMode="decimal"
            onChange={(event) => setStart(event.target.value)}
            onBlur={() => commit(typed(start), endTicks)}
          />
        </label>
        <label className="edit-field-stack">
          <span className="edit-field-label">To</span>
          <Input
            aria-label={`${noun} ends at`}
            className="h-8 w-24 font-mono"
            value={end}
            disabled={busy}
            inputMode="decimal"
            onChange={(event) => setEnd(event.target.value)}
            onBlur={() => commit(startTicks, typed(end))}
          />
        </label>
        <span className="pb-2 text-xs text-[var(--cm-text-muted)]">seconds</span>
      </div>
      <div className="edit-inline">
        <Button
          size="sm"
          variant="ghost"
          disabled={busy || !valid(playhead, endTicks)}
          onClick={() => commit(playhead, endTicks)}
        >
          Start at the playhead
        </Button>
        <Button
          size="sm"
          variant="ghost"
          disabled={busy || !valid(startTicks, playhead)}
          onClick={() => commit(startTicks, playhead)}
        >
          End at the playhead
        </Button>
      </div>
    </div>
  );
}
