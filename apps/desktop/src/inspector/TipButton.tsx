/**
 * Icon button whose label is both its accessible name and its tooltip.
 * Shortcuts work across the review station but are not printed on buttons.
 */
import type { ReactNode } from 'react';

import { Button } from '../components/ui/button.js';
import { Tooltip, TooltipContent, TooltipTrigger } from '../components/ui/tooltip.js';
import { cn } from '../lib/utils.js';

export interface TipButtonProps {
  readonly label: string;
  readonly children: ReactNode;
  readonly onClick: () => void;
  readonly disabled?: boolean;
  /** Set for a toggle, so its state is announced as well as drawn. */
  readonly pressed?: boolean;
  readonly size?: 'icon-xs' | 'icon-sm' | 'icon';
  readonly className?: string;
}

export function TipButton({
  label,
  children,
  onClick,
  disabled = false,
  pressed,
  size = 'icon-sm',
  className,
}: TipButtonProps) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          type="button"
          variant="ghost"
          size={size}
          aria-label={label}
          aria-pressed={pressed}
          data-pressed={pressed ? 'true' : undefined}
          disabled={disabled}
          onClick={onClick}
          className={cn('review-tip-button', className)}
        >
          {children}
        </Button>
      </TooltipTrigger>
      <TooltipContent>{label}</TooltipContent>
    </Tooltip>
  );
}
