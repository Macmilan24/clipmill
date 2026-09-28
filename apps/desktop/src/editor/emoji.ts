/**
 * Emoji over the clip: one of a pinned set, up for a span of the program like
 * a text, and drawn by the render from the same picture the editor shows.
 * Words a speaker leans on can each call for one, a few seconds apart.
 *
 * The catalogue mirrors the renderer's (`clipmill-captions`, `emoji.rs`), and
 * a test keeps the two the same: an emoji offered here that the render does
 * not know would fail the export.
 */
import type { PreviewOverlay, PreviewPlan } from '../daemon/client.js';
import { type EmojiOverlay, freshOverlayId } from './overlays.js';
import { programTicks } from './timeline.js';

export interface EmojiEntry {
  /** Its code point, as its pinned picture is named. */
  readonly code: string;
  readonly label: string;
  /** Words that call for it, plain and lower-case. */
  readonly words: readonly string[];
}

export const EMOJI: readonly EmojiEntry[] = [
  {
    code: '1f525',
    label: 'Fire',
    words: ['fire', 'hot', 'lit', 'amazing', 'insane', 'incredible'],
  },
  {
    code: '1f4a1',
    label: 'Idea',
    words: ['idea', 'ideas', 'realise', 'realize', 'realised', 'realized', 'insight'],
  },
  { code: '1f602', label: 'Laughing', words: ['funny', 'laugh', 'laughing', 'joke', 'hilarious'] },
  {
    code: '1f92f',
    label: 'Mind blown',
    words: ['crazy', 'blown', 'wild', 'unbelievable', 'shocking', 'shocked'],
  },
  { code: '1f440', label: 'Eyes', words: ['watch', 'notice', 'attention'] },
  {
    code: '1f4b0',
    label: 'Money',
    words: [
      'money',
      'cash',
      'dollars',
      'paid',
      'pay',
      'price',
      'revenue',
      'profit',
      'income',
      'salary',
      'budget',
    ],
  },
  {
    code: '1f680',
    label: 'Rocket',
    words: ['launch', 'launched', 'rocket', 'fast', 'faster', 'scale', 'boost'],
  },
  { code: '2705', label: 'Check', words: ['correct', 'done', 'works', 'worked'] },
  {
    code: '274c',
    label: 'Cross',
    words: ['wrong', 'mistake', 'mistakes', 'fail', 'failed', 'failure'],
  },
  {
    code: '26a0',
    label: 'Warning',
    words: ['warning', 'careful', 'danger', 'dangerous', 'risk', 'beware'],
  },
  { code: '2764', label: 'Heart', words: ['love', 'loved', 'heart'] },
  { code: '1f44f', label: 'Clapping', words: ['congrats', 'congratulations', 'bravo', 'applause'] },
  { code: '1f64c', label: 'Raised hands', words: ['finally', 'yay', 'hallelujah'] },
  {
    code: '1f4af',
    label: 'Hundred',
    words: ['hundred', 'absolutely', 'exactly', 'totally', 'definitely'],
  },
  { code: '1f3af', label: 'Target', words: ['goal', 'goals', 'target', 'focus', 'aim'] },
  {
    code: '1f4c8',
    label: 'Chart up',
    words: ['grow', 'growth', 'growing', 'increase', 'higher', 'rise'],
  },
  {
    code: '1f4c9',
    label: 'Chart down',
    words: ['decrease', 'lower', 'drop', 'dropped', 'fall', 'decline'],
  },
  { code: '23f0', label: 'Alarm clock', words: ['deadline', 'late', 'morning', 'hours'] },
  { code: '1f914', label: 'Thinking', words: ['question', 'wonder', 'wondering', 'hmm'] },
  { code: '1f62e', label: 'Surprised', words: ['wow', 'surprised', 'surprise', 'whoa'] },
  { code: '1f605', label: 'Sweat smile', words: ['awkward', 'oops', 'nervous'] },
  { code: '1f60d', label: 'Heart eyes', words: ['beautiful', 'gorgeous', 'adore', 'obsessed'] },
  { code: '1f64f', label: 'Folded hands', words: ['thanks', 'thank', 'grateful', 'pray'] },
  {
    code: '1f4aa',
    label: 'Strong',
    words: ['strong', 'strength', 'gym', 'discipline', 'effort'],
  },
  { code: '1f449', label: 'Pointing', words: ['tip', 'tips', 'step', 'steps'] },
  {
    code: '1f389',
    label: 'Party',
    words: ['party', 'win', 'won', 'celebrate', 'success', 'successful'],
  },
  { code: '2b50', label: 'Star', words: ['star', 'favorite', 'favourite'] },
  {
    code: '1f9e0',
    label: 'Brain',
    words: ['brain', 'smart', 'learn', 'learning', 'knowledge', 'study'],
  },
  {
    code: '1f4ac',
    label: 'Speech',
    words: ['conversation', 'podcast', 'interview', 'discuss', 'discussion'],
  },
  { code: '1f511', label: 'Key', words: ['key', 'secret', 'secrets', 'unlock', 'trick'] },
  { code: '1f6d1', label: 'Stop', words: ['stop', 'quit'] },
  {
    code: '1f91d',
    label: 'Handshake',
    words: ['deal', 'partner', 'partnership', 'team', 'agree'],
  },
  { code: '1f60e', label: 'Cool', words: ['cool', 'confident', 'chill'] },
  { code: '1f976', label: 'Cold', words: ['cold', 'freezing', 'frozen'] },
  {
    code: '1f911',
    label: 'Money face',
    words: ['rich', 'millionaire', 'million', 'millions', 'billion'],
  },
  {
    code: '1f4f1',
    label: 'Phone',
    words: ['phone', 'app', 'apps', 'social', 'instagram', 'tiktok'],
  },
  {
    code: '1f4bb',
    label: 'Laptop',
    words: ['computer', 'code', 'coding', 'software', 'laptop'],
  },
  { code: '1f4da', label: 'Books', words: ['book', 'books', 'reading', 'school'] },
  { code: '1f3c6', label: 'Trophy', words: ['champion', 'winner', 'award'] },
  { code: '1f480', label: 'Skull', words: ['dead', 'dying', 'died', 'skull'] },
];

/** An emoji's side, per mille of the frame's short side. */
export const EMOJI_SIZES = { min: 60, max: 400, default: 180 } as const;

const SECOND = 90_000;
/** How long an emoji stays up. */
const EMOJI_SPAN = (6 * SECOND) / 5;
/** The least time between two emoji put on words. */
const EMOJI_GAP = 3 * SECOND;
/** Where an emoji sits: centred, above where captions sit. */
const EMOJI_Y = 640;

/** The emoji with this code, when it is one of these. */
export function emojiOf(code: string | null | undefined): EmojiEntry | undefined {
  return EMOJI.find((entry) => entry.code === code);
}

/** The character itself, for where its picture cannot load. */
export function emojiCharacter(code: string): string {
  const points = code.split('_').map((hex) => Number.parseInt(hex, 16));
  const text = String.fromCodePoint(...points);
  // A symbol that also has a plain-text form is asked for in colour.
  return points.length === 1 && points[0]! < 0x1f000 ? `${text}️` : text;
}

/** The emoji a word calls for, whatever its case or punctuation. */
export function emojiForWord(word: string): EmojiEntry | undefined {
  const plain = word.replace(/^[^\p{L}\p{N}]+|[^\p{L}\p{N}]+$/gu, '').toLowerCase();
  if (!plain) return undefined;
  return EMOJI.find((entry) => entry.words.includes(plain));
}

/**
 * An emoji's box in output pixels, as the render lays it: an even side, and
 * its centre where the document says.
 */
export function emojiBox(
  overlay: Pick<PreviewOverlay, 'x' | 'y' | 'size'>,
  frame: { readonly width: number; readonly height: number },
): { readonly side: number; readonly cx: number; readonly cy: number } {
  const short = Math.min(frame.width, frame.height);
  return {
    side: Math.max(2, Math.floor((short * overlay.size) / 1000) & ~1),
    cx: Math.floor((frame.width * overlay.x) / 1000),
    cy: Math.floor((frame.height * overlay.y) / 1000),
  };
}

/** One emoji from a moment of the program, above the captions. */
export function emojiOverlay(
  id: string,
  code: string,
  startTicks: number,
  plan: PreviewPlan,
): EmojiOverlay {
  const end = programTicks(plan);
  const start = Math.max(0, Math.min(startTicks, end - SECOND / 10));
  return {
    overlay_id: id,
    start_ticks: start,
    end_ticks: Math.min(start + EMOJI_SPAN, end),
    content: { kind: 'emoji', emoji: code, x: 500, y: EMOJI_Y, size: EMOJI_SIZES.default },
  };
}

/** The emoji a plan shows. */
export function emojiOverlays(plan: Pick<PreviewPlan, 'overlays'>): readonly PreviewOverlay[] {
  return (plan.overlays ?? []).filter((overlay) => overlay.kind === 'emoji');
}

/**
 * An emoji for each word that calls for one, from the moment it is said,
 * never within three seconds of another emoji — one already there, or one
 * put on an earlier word.
 */
export function keywordEmoji(
  words: readonly { readonly text: string; readonly startTicks: number }[],
  plan: PreviewPlan,
): EmojiOverlay[] {
  const shown = (plan.overlays ?? []).map((overlay) => ({ overlay_id: overlay.overlayId }));
  const spans = emojiOverlays(plan).map((overlay) => [overlay.startTicks, overlay.endTicks]);
  const made: EmojiOverlay[] = [];
  for (const word of words) {
    const entry = emojiForWord(word.text);
    if (!entry) continue;
    const from = word.startTicks;
    const crowded = spans.some(
      ([start, end]) => from < end! + EMOJI_GAP && from + EMOJI_SPAN > start! - EMOJI_GAP,
    );
    if (crowded) continue;
    const overlay = emojiOverlay(freshOverlayId([...shown, ...made]), entry.code, from, plan);
    made.push(overlay);
    spans.push([overlay.start_ticks, overlay.end_ticks]);
  }
  return made;
}
