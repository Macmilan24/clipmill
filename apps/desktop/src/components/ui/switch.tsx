import { Switch as SwitchPrimitive } from 'radix-ui';
import type * as React from 'react';

import { cn } from '@/lib/utils';

function Switch({
  className,
  ...props
}: React.ComponentProps<typeof SwitchPrimitive.Root>): React.JSX.Element {
  return (
    <SwitchPrimitive.Root
      data-slot="switch"
      className={cn(
        'peer inline-flex h-[18px] w-8 shrink-0 items-center rounded-full border border-transparent outline-none transition-all',
        'bg-[var(--cm-recessed)] data-[state=checked]:bg-primary',
        'focus-visible:ring-[3px] focus-visible:ring-ring/50',
        // A disabled switch is still readable: this shell uses one to state a
        // fact the user cannot change, not merely to grey out a choice.
        'disabled:cursor-not-allowed disabled:opacity-60',
        className,
      )}
      {...props}
    >
      <SwitchPrimitive.Thumb
        data-slot="switch-thumb"
        className={cn(
          'pointer-events-none block size-[14px] rounded-full bg-[var(--cm-text-primary)] ring-0 transition-transform',
          'translate-x-[2px] data-[state=checked]:translate-x-[16px] data-[state=checked]:bg-primary-foreground',
        )}
      />
    </SwitchPrimitive.Root>
  );
}

export { Switch };
