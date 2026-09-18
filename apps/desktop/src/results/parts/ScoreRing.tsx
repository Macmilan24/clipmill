/**
 * A score as a ring, sized for the place it is standing in.
 *
 * The ring is drawn rather than approximated with a conic gradient, because a
 * gradient cannot round its own cap and the seam at zero is visible at every
 * size. An SVG arc also gives the sweep something to animate along, so a card
 * arriving on screen draws its score rather than blinking it into place.
 *
 * The band chooses the colour and the number never does. A score is a percentile
 * within one recording's cohort, so 71 is not "amber" in any absolute sense —
 * what a reader should take from the colour is the band the ranker assigned,
 * which is the only claim the document actually makes.
 */
import { useEffect, useState } from 'react';

export type RingSize = 'sm' | 'md' | 'lg';

const GEOMETRY: Readonly<Record<RingSize, { box: number; stroke: number; type: string }>> = {
  sm: { box: 34, stroke: 3, type: 'text-[11px]' },
  md: { box: 52, stroke: 4, type: 'text-[15px]' },
  lg: { box: 104, stroke: 6, type: 'text-[28px]' },
};

export function bandInk(band: string): string {
  if (band === 'strong') {
    return 'var(--cm-success-ink)';
  }
  if (band === 'needs_review') {
    return 'var(--cm-warning-ink)';
  }
  return 'var(--cm-accent)';
}

export interface ScoreRingProps {
  readonly score: number;
  readonly band: string;
  readonly size?: RingSize;
  /** The word under the number, on the sizes with room for one. */
  readonly caption?: string;
}

export function ScoreRing({ score, band, size = 'md', caption }: ScoreRingProps) {
  const { box, stroke, type } = GEOMETRY[size];
  const radius = (box - stroke) / 2;
  const circumference = 2 * Math.PI * radius;
  const clamped = Math.max(0, Math.min(99, score));

  // Mounted at zero and then moved, so the transition has somewhere to travel
  // from. Without the second frame the dash offset is correct on first paint and
  // nothing ever animates.
  const [drawn, setDrawn] = useState(0);
  useEffect(() => {
    const frame = requestAnimationFrame(() => setDrawn(clamped));
    return () => cancelAnimationFrame(frame);
  }, [clamped]);

  const ink = bandInk(band);
  return (
    <div
      className="relative grid shrink-0 place-items-center"
      style={{ width: box, height: box }}
      role="img"
      aria-label={`Score ${score} of 99`}
    >
      <svg width={box} height={box} viewBox={`0 0 ${box} ${box}`} className="-rotate-90">
        <circle
          cx={box / 2}
          cy={box / 2}
          r={radius}
          fill="none"
          stroke="var(--cm-recessed-border)"
          strokeWidth={stroke}
        />
        <circle
          cx={box / 2}
          cy={box / 2}
          r={radius}
          fill="none"
          stroke={ink}
          strokeWidth={stroke}
          strokeLinecap="round"
          strokeDasharray={circumference}
          strokeDashoffset={circumference - (drawn / 99) * circumference}
          style={{
            transition: 'stroke-dashoffset 720ms cubic-bezier(0.22, 1, 0.36, 1)',
            filter: `drop-shadow(0 0 6px color-mix(in srgb, ${ink} 45%, transparent))`,
          }}
        />
      </svg>
      <span className="absolute flex flex-col items-center leading-none">
        <span className={`mono font-semibold ${type}`} style={{ color: ink }}>
          {score}
        </span>
        {caption && size === 'lg' && (
          <span className="mt-1 text-[10px] tracking-[0.08em] text-[var(--cm-text-muted)] uppercase">
            {caption}
          </span>
        )}
      </span>
    </div>
  );
}
