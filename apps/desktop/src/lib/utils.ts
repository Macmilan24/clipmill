import { type ClassValue, clsx } from 'clsx';
import { extendTailwindMerge } from 'tailwind-merge';

/**
 * tailwind-merge only knows Tailwind's stock class groups, so our design type
 * scale (text-body, text-meta, …) is invisible to it: a `text-body` passed to a
 * shadcn component would sit alongside that component's built-in `text-sm`
 * rather than replacing it, and the stylesheet order would silently decide the
 * winner. Teaching it the custom scale makes the override deterministic.
 */
const twMerge = extendTailwindMerge({
  extend: {
    classGroups: {
      'font-size': [
        {
          text: ['page-title', 'card-title', 'section-title', 'body', 'label', 'meta', 'technical'],
        },
      ],
    },
  },
});

/** Merge conditional class names, letting later Tailwind utilities win. */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
