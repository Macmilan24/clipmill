/**
 * Who speaks when, read onto a transcript.
 *
 * The analysis tells voices apart and numbers them in the order they are
 * first heard (`speech.speakers.v1`). A voice is how someone sounds, not who
 * they are, so it is shown as "Speaker 1" until a person names it. Names are
 * kept per recording on this computer: every clip of an episode shares them,
 * and naming two voices the same is how two voices that are one person — a
 * guest heard through two microphones — become one.
 */
import type { SpeechSpeakers } from '@clipmill/contracts';
import { useCallback, useEffect, useState } from 'react';

export interface VoiceTurn {
  readonly startTicks: number;
  readonly endTicks: number;
  readonly speakerId: string;
}

export interface Voices {
  /** The recording they were told apart in: names are kept by it. */
  readonly sourceFingerprint: string;
  /** In the order first heard. */
  readonly ids: readonly string[];
  /** Ordered and non-overlapping. */
  readonly turns: readonly VoiceTurn[];
}

/** The voices a published document states, or null for one of another recording. */
export function readVoices(document: SpeechSpeakers, sourceFingerprint: string): Voices | null {
  if (
    document.schema_version !== 'clipmill.speech.speakers.v1' ||
    document.source_fingerprint !== sourceFingerprint
  ) {
    return null;
  }
  return {
    sourceFingerprint,
    ids: document.speakers.map((speaker) => speaker.speaker_id),
    turns: document.turns.map((turn) => ({
      startTicks: turn.start_ticks,
      endTicks: turn.end_ticks,
      speakerId: turn.speaker_id,
    })),
  };
}

/** The voice speaking at a moment, or null in a pause between voices. */
export function voiceAt(voices: Voices, ticks: number): string | null {
  let low = 0;
  let high = voices.turns.length - 1;
  while (low <= high) {
    const middle = (low + high) >> 1;
    const turn = voices.turns[middle]!;
    if (ticks < turn.startTicks) high = middle - 1;
    else if (ticks >= turn.endTicks) low = middle + 1;
    else return turn.speakerId;
  }
  return null;
}

/**
 * The voice a stretch of words is said in: the one heard for most of it,
 * word by word. Null when no voice covers any of it.
 */
export function voiceOfWords(
  voices: Voices,
  words: readonly { readonly startTicks: number; readonly endTicks: number }[],
): string | null {
  const heard = new Map<string, number>();
  for (const word of words) {
    const id = voiceAt(voices, (word.startTicks + word.endTicks) / 2);
    if (id) heard.set(id, (heard.get(id) ?? 0) + Math.max(1, word.endTicks - word.startTicks));
  }
  let best: string | null = null;
  for (const [id, length] of heard) {
    if (best === null || length > heard.get(best)!) best = id;
  }
  return best;
}

export type VoiceNames = Readonly<Record<string, string>>;

const NAMES_KEY = (fingerprint: string) => `clipmill.voices.${fingerprint}`;

export function recallNames(fingerprint: string): VoiceNames {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(NAMES_KEY(fingerprint)) ?? 'null');
    return parsed && typeof parsed === 'object' ? (parsed as VoiceNames) : {};
  } catch {
    return {};
  }
}

export function rememberNames(fingerprint: string, names: VoiceNames): void {
  try {
    if (Object.keys(names).length === 0) localStorage.removeItem(NAMES_KEY(fingerprint));
    else localStorage.setItem(NAMES_KEY(fingerprint), JSON.stringify(names));
  } catch {
    // This session keeps them.
  }
}

/** What a voice is called: its name, or "Speaker 2". */
export function voiceName(id: string, names: VoiceNames): string {
  return names[id]?.trim() || `Speaker ${id.replace(/^spk_/, '')}`;
}

/** A colour per voice, so turns can be told apart at a glance. */
const VOICE_COLOURS = ['#5fb3a1', '#e0a13a', '#8b8ff0', '#e0617a', '#6fb0e8', '#c7a86b'];

export function voiceColour(id: string, voices: Voices): string {
  const at = voices.ids.indexOf(id);
  return VOICE_COLOURS[(at < 0 ? 0 : at) % VOICE_COLOURS.length]!;
}

/** The names of a recording's voices, and a way to change one. */
export function useVoiceNames(
  fingerprint: string | null,
): readonly [VoiceNames, (id: string, name: string) => void] {
  const [names, setNames] = useState<VoiceNames>(() =>
    fingerprint ? recallNames(fingerprint) : {},
  );
  useEffect(() => {
    setNames(fingerprint ? recallNames(fingerprint) : {});
    if (!fingerprint) return undefined;
    // Named in another window of the app: follow it.
    const follow = (event: StorageEvent) => {
      if (event.key === NAMES_KEY(fingerprint)) setNames(recallNames(fingerprint));
    };
    window.addEventListener('storage', follow);
    return () => window.removeEventListener('storage', follow);
  }, [fingerprint]);
  const rename = useCallback(
    (id: string, name: string) => {
      if (!fingerprint) return;
      setNames((current) => {
        const next: Record<string, string> = { ...current };
        if (name.trim()) next[id] = name.trim().slice(0, 40);
        else delete next[id];
        rememberNames(fingerprint, next);
        return next;
      });
    },
    [fingerprint],
  );
  return [names, rename] as const;
}
