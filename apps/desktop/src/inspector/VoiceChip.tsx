/**
 * Who says a sentence, shown where the voice changes: "Speaker 2" in the
 * voice's colour until a person names it here.
 */
import { type CSSProperties, useState } from 'react';

import { type VoiceNames, type Voices, voiceColour, voiceName } from '../results/voices.js';

export function VoiceChip({
  id,
  voices,
  names,
  onRename,
}: {
  readonly id: string;
  readonly voices: Voices;
  readonly names: VoiceNames;
  readonly onRename: (id: string, name: string) => void;
}) {
  const [draft, setDraft] = useState<string | null>(null);
  const name = voiceName(id, names);
  const colour = { '--voice': voiceColour(id, voices) } as CSSProperties;
  if (draft !== null) {
    const commit = () => {
      onRename(id, draft);
      setDraft(null);
    };
    return (
      <input
        className="review-voice review-voice-input"
        style={colour}
        aria-label={`Name ${name}`}
        placeholder={`Speaker ${id.replace(/^spk_/, '')}`}
        maxLength={40}
        // eslint-disable-next-line jsx-a11y/no-autofocus -- opened by the click that asked for it
        autoFocus
        value={draft}
        onChange={(event) => setDraft(event.target.value)}
        onBlur={commit}
        onKeyDown={(event) => {
          if (event.key === 'Enter') commit();
          if (event.key === 'Escape') setDraft(null);
          event.stopPropagation();
        }}
      />
    );
  }
  return (
    <button
      type="button"
      className="review-voice"
      style={colour}
      title="Name this voice for every clip of this recording"
      onClick={(event) => {
        event.stopPropagation();
        setDraft(names[id] ?? '');
      }}
    >
      {name}
    </button>
  );
}
