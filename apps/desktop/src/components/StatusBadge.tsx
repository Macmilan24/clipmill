import type { JSX, ReactNode } from 'react';

import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';

/**
 * The design reserves specific meanings for colour, so state is expressed
 * through this one component rather than ad-hoc classes. `outbound` in
 * particular is reserved exclusively for anything that would send data
 * off-device, and must never be used decoratively.
 *
 * The words wear the `-ink` step and the border wears the hue. The design gives
 * both themes one set of status colours, tuned against the dark surface; as text
 * on the light surface they measure 2.2–3.8:1, under the 4.5:1 a label needs. The
 * ink step holds the hue and darkens it for light mode only, so the badge still
 * reads as its own colour and the words are still legible.
 */
export type StatusTone = 'success' | 'warning' | 'danger' | 'outbound' | 'progress' | 'neutral';

const TONES: Record<StatusTone, string> = {
  success:
    'text-[var(--cm-success-ink)] border-[color-mix(in_srgb,var(--color-success)_40%,transparent)]',
  // Indigo is reserved for the primary action, selection, focus, and progress.
  // This is the progress one; nothing decorative may use it.
  progress:
    'text-[var(--color-primary)] border-[color-mix(in_srgb,var(--color-primary)_40%,transparent)]',
  warning:
    'text-[var(--cm-warning-ink)] border-[color-mix(in_srgb,var(--color-warning)_40%,transparent)]',
  danger:
    'text-[var(--cm-danger-ink)] border-[color-mix(in_srgb,var(--color-destructive)_40%,transparent)]',
  outbound:
    'text-[var(--cm-outbound-ink)] border-[color-mix(in_srgb,var(--color-outbound)_45%,transparent)]',
  neutral: 'text-[var(--cm-text-secondary)]',
};

export function StatusBadge({
  tone,
  className,
  children,
}: {
  readonly tone: StatusTone;
  readonly className?: string;
  readonly children: ReactNode;
}): JSX.Element {
  return (
    <Badge
      variant="outline"
      className={cn('gap-1.5 text-meta font-(--cm-weight-label)', TONES[tone], className)}
    >
      {children}
    </Badge>
  );
}
