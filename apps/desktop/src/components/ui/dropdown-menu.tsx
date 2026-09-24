import { DropdownMenu as MenuPrimitive } from 'radix-ui';
import type * as React from 'react';

import { cn } from '@/lib/utils';

function DropdownMenu(props: React.ComponentProps<typeof MenuPrimitive.Root>): React.JSX.Element {
  return <MenuPrimitive.Root data-slot="dropdown-menu" {...props} />;
}

function DropdownMenuTrigger(
  props: React.ComponentProps<typeof MenuPrimitive.Trigger>,
): React.JSX.Element {
  return <MenuPrimitive.Trigger data-slot="dropdown-menu-trigger" {...props} />;
}

function DropdownMenuContent({
  className,
  sideOffset = 4,
  ...props
}: React.ComponentProps<typeof MenuPrimitive.Content>): React.JSX.Element {
  return (
    <MenuPrimitive.Portal>
      <MenuPrimitive.Content
        data-slot="dropdown-menu-content"
        sideOffset={sideOffset}
        className={cn(
          'glass z-50 min-w-[11rem] overflow-hidden rounded-[var(--cm-radius-panel,8px)] p-1',
          'data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=closed]:animate-out data-[state=closed]:fade-out-0',
          className,
        )}
        {...props}
      />
    </MenuPrimitive.Portal>
  );
}

function DropdownMenuItem({
  className,
  tone,
  ...props
}: React.ComponentProps<typeof MenuPrimitive.Item> & {
  readonly tone?: 'danger';
}): React.JSX.Element {
  return (
    <MenuPrimitive.Item
      data-slot="dropdown-menu-item"
      data-tone={tone}
      className={cn(
        'flex cursor-default items-center gap-2 rounded-[var(--cm-radius-control)] px-2 py-1.5 text-label outline-none select-none',
        'data-[highlighted]:bg-[var(--cm-accent-selected)] data-[disabled]:pointer-events-none data-[disabled]:opacity-45',
        "data-[tone=danger]:text-[var(--cm-danger-ink)] [&_svg]:size-4 [&_svg]:shrink-0 [&_svg]:opacity-80",
        className,
      )}
      {...props}
    />
  );
}

function DropdownMenuSeparator({
  className,
  ...props
}: React.ComponentProps<typeof MenuPrimitive.Separator>): React.JSX.Element {
  return (
    <MenuPrimitive.Separator
      data-slot="dropdown-menu-separator"
      className={cn('my-1 h-px bg-[var(--cm-glass-border)]', className)}
      {...props}
    />
  );
}

export {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
};
