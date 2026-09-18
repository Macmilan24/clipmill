import { Film } from 'lucide-react';
import { useState } from 'react';

/** A missing or collected thumbnail should never leave a broken-image icon. */
export function MediaStill({ src }: { readonly src: string | null }) {
  const [failed, setFailed] = useState<string | null>(null);
  if (src && src !== failed)
    return (
      <img
        src={src}
        alt=""
        className="size-full object-cover"
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
