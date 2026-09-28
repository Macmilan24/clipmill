/**
 * An emoji as the export draws it: its pinned picture, or the character
 * itself where the picture cannot load.
 */
import { useState } from 'react';

import { emojiCharacter, emojiOf } from './emoji.js';

export function EmojiPicture({
  code,
  url,
  className,
}: {
  readonly code: string;
  readonly url?: ((code: string) => string) | null | undefined;
  readonly className?: string;
}) {
  const [failed, setFailed] = useState(false);
  const label = emojiOf(code)?.label ?? code;
  if (url && !failed) {
    return (
      <img
        className={className}
        src={url(code)}
        alt={label}
        draggable={false}
        onError={() => setFailed(true)}
      />
    );
  }
  return (
    <span className={className} role="img" aria-label={label}>
      {emojiCharacter(code)}
    </span>
  );
}
