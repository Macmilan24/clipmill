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
