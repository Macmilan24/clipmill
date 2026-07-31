import type { JSX, ReactNode } from 'react';

import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';

/**
 * The design reserves specific meanings for colour, so state is expressed
 * through this one component rather than ad-hoc classes. `outbound` in
 * particular is reserved exclusively for anything that would send data
 * off-device, and must never be used decoratively.
 */
export type StatusTone = 'success' | 'warning' | 'danger' | 'outbound' | 'progress' | 'neutral';

const TONES: Record<StatusTone, string> = {
  success:
    'text-[var(--color-success)] border-[color-mix(in_srgb,var(--color-success)_40%,transparent)]',
  // Indigo is reserved for the primary action, selection, focus, and progress.
  // This is the progress one; nothing decorative may use it.
  progress:
    'text-[var(--color-primary)] border-[color-mix(in_srgb,var(--color-primary)_40%,transparent)]',
  warning:
    'text-[var(--color-warning)] border-[color-mix(in_srgb,var(--color-warning)_40%,transparent)]',
  danger:
    'text-[var(--color-destructive)] border-[color-mix(in_srgb,var(--color-destructive)_40%,transparent)]',
  outbound:
    'text-[var(--color-outbound)] border-[color-mix(in_srgb,var(--color-outbound)_45%,transparent)]',
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
