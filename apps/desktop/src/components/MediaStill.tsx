import { Film } from 'lucide-react';
import { useState } from 'react';

/** A missing or collected thumbnail should never leave a broken-image icon. */
export function MediaStill({
  src,
  position,
}: {
  readonly src: string | null;
  /** Where the still sits in a box of another shape, as `object-position`. */
  readonly position?: string | undefined;
}) {
  const [failed, setFailed] = useState<string | null>(null);
  if (src && src !== failed)
    return (
      <img
        src={src}
        alt=""
        className="size-full object-cover"
        style={position ? { objectPosition: position } : undefined}
        loading="lazy"
        onError={() => setFailed(src)}
      />
    );
  return (
    <div className="flex size-full flex-col items-center justify-center gap-2 px-4 text-center text-[11px] text-[var(--cm-text-muted)]">
      <Film className="size-5 opacity-60" aria-hidden />
      <span>Preview frame unavailable</span>
    </div>
  );
}
