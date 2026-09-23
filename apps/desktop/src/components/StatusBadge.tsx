import type { JSX, ReactNode } from 'react';

import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';

/**
 * Shared status colors. Reserve `outbound` exclusively for off-device data transfer.
 * Use the `-ink` step for text: base hues provide only 2.2–3.8:1 contrast on light
 * surfaces, below the 4.5:1 label target. Borders retain the base hue.
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
