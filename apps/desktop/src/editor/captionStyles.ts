/**
 * Caption styles a creator keeps, and the key words a clip's captions stress.
 *
 * A saved style is a look and its adjustments under a name, applied to any
 * clip as one edit. It lives on this machine: styles are a person's taste,
 * not a project's state, and a new project can start from one.
 *
 * Key words are suggested, never imposed: the suggestion is a batch of
 * ordinary emphasis edits a person can undo or change word by word.
 */
import type { EditIr } from '@clipmill/contracts';

import type { EditCommandJson, PreviewCue } from '../daemon/client.js';
import { batch, setCaptionOptions, setCaptionStyle, setWordEmphasis } from './commands.js';

export type CaptionOptions = NonNullable<EditIr['captions']['options']>;

export interface SavedStyle {
  readonly name: string;
  readonly styleRef: string;
  readonly options: CaptionOptions;
}

const KEY = 'clipmill.captionStyles';

/**
 * The parts of the options that are a look rather than this clip's layout.
 * Where captions sit and how many words they show belong to the clip.
 */
export function lookOptions(options: CaptionOptions): CaptionOptions {
  const { position: _position, words_on_screen: _words, ...look } = options;
  return look;
}

function isSaved(value: unknown): value is SavedStyle {
  if (typeof value !== 'object' || value === null) return false;
  const style = value as SavedStyle;
  return (
    typeof style.name === 'string' &&
    style.name.trim() !== '' &&
    typeof style.styleRef === 'string' &&
    typeof style.options === 'object' &&
    style.options !== null
  );
}

export function savedStyles(): readonly SavedStyle[] {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(KEY) ?? '[]');
    return Array.isArray(parsed) ? parsed.filter(isSaved) : [];
  } catch {
    return [];
  }
}

function store(styles: readonly SavedStyle[]): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(styles));
  } catch {
    // Saved for this session's list only; nothing else depends on it.
  }
}

/** Keep a style under a name, replacing one of the same name. */
export function saveStyle(style: SavedStyle): readonly SavedStyle[] {
  const name = style.name.trim();
  const next = [
    ...savedStyles().filter((saved) => saved.name !== name),
    { name, styleRef: style.styleRef, options: lookOptions(style.options) },
  ];
  store(next);
  return next;
}

export function forgetStyle(name: string): readonly SavedStyle[] {
  const next = savedStyles().filter((saved) => saved.name !== name);
  store(next);
  return next;
}

/** One edit that gives a clip a saved style, keeping where its captions sit. */
export function applyStyle(style: SavedStyle, current: CaptionOptions): EditCommandJson {
  const kept: CaptionOptions = {
    ...(current.position ? { position: current.position } : {}),
    ...(current.words_on_screen ? { words_on_screen: current.words_on_screen } : {}),
  };
  return batch([
    setCaptionStyle(style.styleRef),
    setCaptionOptions({ ...lookOptions(style.options), ...kept }),
  ]);
}

/**
 * Words that say nothing on their own. A key word is never one of these: a
 * caption that stresses "the" has stressed nothing.
 */
const PLAIN = new Set(
  (
    'a about after again all also am an and any are as at be because been before being both ' +
    'but by can could did do does doing down for from get got had has have having he her here ' +
    'hers him his how i if in into is it its just like me more most my no not now of off on ' +
    'once only or other our out over really she should so some such than that the their them ' +
    'then there these they this those through to too under until up us very was we were what ' +
    'when where which while who whom why will with would yeah yes you your gonna wanna kind ' +
    'thing things know think mean right okay oh um uh'
  ).split(' '),
);

function scoreOf(text: string, first: boolean): number {
  const clean = text.replace(/[^\p{L}\p{N}%$€£]/gu, '');
  if (clean.length < 2 || PLAIN.has(clean.toLowerCase())) return 0;
  let score = 0;
  if (/\d/.test(clean)) score += 3;
  if (/[%$€£]/.test(clean)) score += 1;
  if (clean.length >= 7) score += 2;
  else if (clean.length >= 5) score += 1;
  if (!first && /^\p{Lu}/u.test(clean)) score += 2;
  if (clean.length >= 2 && clean === clean.toUpperCase() && /\p{L}/u.test(clean)) score += 1;
  return score;
}

/**
 * At most one word per on-screen caption worth stressing: numbers, names and
 * long words, never a function word. Captions of one or two words are left
 * alone; stressing a word that is the whole caption stresses nothing.
 */
export function suggestKeyWords(cues: readonly PreviewCue[]): readonly string[] {
  const chosen: string[] = [];
  for (const cue of cues) {
    const words = cue.lines.flat();
    if (words.length < 3) continue;
    let best: { id: string; score: number } | null = null;
    words.forEach((word, index) => {
      if (!word.wordId) return;
      const score = scoreOf(word.text, index === 0);
      if (score >= 3 && (!best || score > best.score)) best = { id: word.wordId, score };
    });
    if (best) chosen.push((best as { id: string }).id);
  }
  return chosen;
}

/** The ids of the words a document marks as key words. */
export function keyWords(document: EditIr | null): readonly string[] {
  if (!document) return [];
  const ids = new Set<string>();
  for (const cue of [...(document.captions.cues ?? []), ...(document.captions.burn_in ?? [])]) {
    for (const line of cue.lines) {
      for (const word of line.words) {
        if (word.emphasis && word.word_id) ids.add(word.word_id);
      }
    }
  }
  return [...ids];
}

/** Mark the suggested words, as one edit. Null when there is nothing to mark. */
export function markKeyWords(
  cues: readonly PreviewCue[],
  document: EditIr | null,
): EditCommandJson | null {
  const marked = new Set(keyWords(document));
  const fresh = suggestKeyWords(cues).filter((id) => !marked.has(id));
  return fresh.length === 0 ? null : batch(fresh.map((id) => setWordEmphasis(id, true)));
}

/** Unmark every key word, as one edit. Null when none is marked. */
export function clearKeyWords(document: EditIr | null): EditCommandJson | null {
  const marked = keyWords(document);
  return marked.length === 0 ? null : batch(marked.map((id) => setWordEmphasis(id, false)));
}

/** Words the recognizer wrote that are not speech: dashes, markers, annotations. */
export function nonSpeechCount(document: EditIr | null): number {
  if (!document) return 0;
  let count = 0;
  for (const cue of [...(document.captions.cues ?? []), ...(document.captions.burn_in ?? [])]) {
    for (const line of cue.lines) {
      for (const word of line.words) {
        const text = word.text.trim();
        const annotation =
          (text.startsWith('[') && text.endsWith(']')) ||
          (text.startsWith('(') && text.endsWith(')'));
        if (annotation || !/[\p{L}\p{N}]/u.test(text)) count += 1;
      }
    }
  }
  return count;
}
