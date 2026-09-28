/**
 * Synthetic review data for the development preview: an eight-minute talk with
 * eight candidates, a word-timed transcript, a loudness contour that pauses
 * where the speaker does, and a crop path that drifts. Nothing here is read
 * from a real recording.
 */
import type { CropPath } from '../src/daemon/client.js';
import type { Peaks } from '../src/results/loader.js';
import type { ClipRow } from '../src/results/model.js';
import type { Transcript, TranscriptSentence, TranscriptWord } from '../src/results/transcript.js';

const SECOND = 90_000;
export const REVIEW_DURATION_TICKS = 480 * SECOND;

const SENTENCES = [
  'Most people wait until they feel ready before they start.',
  'The thing we get wrong about confidence is that we treat it like a feeling you need before you act.',
  'It works the other way round.',
  'You act, it goes badly, you survive it, and that is the evidence your nervous system was waiting for.',
  'Confidence is a receipt, not a ticket.',
  'So the practical version of this is really simple.',
  'Pick the smallest version of the thing that scares you.',
  'Do it once this week, on purpose, and write down what actually happened.',
  'Not what you were afraid would happen, what actually happened.',
  'The gap between those two lists is where your confidence comes from.',
  'I used to think the best ideas arrived fully formed.',
  'They almost never do.',
  'The best ideas start with a better question, and a better question usually sounds a little naive.',
  'Why do we do it this way?',
  'What would this look like if it were easy?',
  'Those questions feel too simple to ask in a meeting, which is exactly why nobody asks them.',
  'Doing less is not the same as caring less.',
  'When I cut my project list in half, the work that was left got better, not worse.',
  'Attention is the real budget, and most of us are overspent.',
  'Listening is a habit you can practise like any other.',
  'Try this in your next conversation: wait two seconds before you answer.',
  'It feels long, but the other person almost always says the most important thing in that pause.',
  'Leave room for the unexpected, because the unexpected is where the story is.',
  'The advice I wish I had heard sooner is that nobody is thinking about you as much as you are.',
  'That sounds harsh, but it is the most freeing thing I know.',
  'You can try things, look foolish, and nobody will remember it next week.',
  'Start with one small thing you can try tomorrow.',
  'Then do it again the day after that.',
];

/**
 * Words timed like a steady speaker: three words a second, a breath per
 * sentence. A host asks the questions and cuts in now and then; the guest
 * says the rest, so the transcript has two voices to tell apart.
 */
function speak(): Transcript {
  const words: TranscriptWord[] = [];
  const sentences: TranscriptSentence[] = [];
  const turns: { startTicks: number; endTicks: number; speakerId: string }[] = [];
  let at = 4 * SECOND;
  let sentence = 0;
  while (at < REVIEW_DURATION_TICKS - 6 * SECOND) {
    const text = SENTENCES[sentence % SENTENCES.length]!;
    const firstWord = words.length;
    const speakerId = text.endsWith('?') || sentence % 7 === 3 ? 'spk_2' : 'spk_1';
    for (const word of text.split(' ')) {
      const length = Math.round((0.18 + word.length * 0.035) * SECOND);
      words.push({ text: word, startTicks: at, endTicks: at + length });
      at += length + Math.round(0.07 * SECOND);
    }
    sentences.push({
      startTicks: words[firstWord]!.startTicks,
      endTicks: words.at(-1)!.endTicks,
      firstWord,
      wordCount: words.length - firstWord,
    });
    const last = turns.at(-1);
    if (last?.speakerId === speakerId) last.endTicks = words.at(-1)!.endTicks;
    else
      turns.push({
        startTicks: words[firstWord]!.startTicks,
        endTicks: words.at(-1)!.endTicks,
        speakerId,
      });
    at += Math.round((sentence % 3 === 0 ? 0.9 : 0.55) * SECOND);
    sentence += 1;
  }
  return {
    words,
    sentences,
    voices: { sourceFingerprint: 'preview', ids: ['spk_1', 'spk_2'], turns },
  };
}

export const reviewTranscript: Transcript = speak();

/** A loudness contour that follows the words: loud while speaking, quiet between. */
function contour(transcript: Transcript): Peaks {
  const bucketTicks = SECOND / 10;
  const buckets = Math.ceil(REVIEW_DURATION_TICKS / bucketTicks);
  const values: (readonly [number, number])[] = [];
  let word = 0;
  let seed = 7;
  const random = () => {
    seed = (seed * 16807) % 2147483647;
    return seed / 2147483647;
  };
  for (let bucket = 0; bucket < buckets; bucket += 1) {
    const at = bucket * bucketTicks;
    while (word < transcript.words.length && transcript.words[word]!.endTicks < at) word += 1;
    const speaking =
      word < transcript.words.length &&
      transcript.words[word]!.startTicks <= at + bucketTicks &&
      transcript.words[word]!.endTicks >= at;
    const level = speaking ? 9_000 + random() * 17_000 : 300 + random() * 900;
    values.push([-Math.round(level * (0.8 + random() * 0.2)), Math.round(level)]);
  }
  return { bucketTicks, values };
}

export const reviewPeaks: Peaks = contour(reviewTranscript);

/** A crop that follows a speaker who drifts a little and leans in once. */
export const reviewCrop: CropPath = {
  fit: false,
  fitReason: '',
  containment: 0.97,
  keyframes: Array.from({ length: 97 }, (_, index) => ({
    tTicks: index * 5 * SECOND,
    centerX: 0.5 + 0.035 * Math.sin(index / 3),
    centerY: 0.44,
    scale: index % 9 === 4 ? 0.82 : 0.9,
  })),
};

/** The start of the sentence nearest a moment, and the end of the one it closes. */
function sentenceSpan(from: number, count: number): { start: number; end: number } {
  const sentences = reviewTranscript.sentences;
  const first = Math.max(
    0,
    sentences.findIndex((sentence) => sentence.startTicks >= from),
  );
  const last = sentences[Math.min(sentences.length - 1, first + count - 1)]!;
  return {
    start: sentences[first]!.startTicks - Math.round(0.1 * SECOND),
    end: last.endTicks + Math.round(0.1 * SECOND),
  };
}

const PLAN = [
  { at: 18, count: 5, headline: 'Confidence is a receipt, not a ticket', band: 'strong' },
  {
    at: 92,
    count: 4,
    headline: 'Pick the smallest version of the thing that scares you',
    band: 'strong',
  },
  {
    at: 150,
    count: 6,
    headline: 'The best ideas start with a better question',
    band: 'needs_review',
  },
  { at: 170, count: 5, headline: 'Questions that feel too simple to ask', band: 'strong' },
  { at: 228, count: 4, headline: 'Doing less is not the same as caring less', band: 'strong' },
  { at: 286, count: 4, headline: 'Wait two seconds before you answer', band: 'strong' },
  {
    at: 350,
    count: 5,
    headline: 'Nobody is thinking about you as much as you are',
    band: 'declined',
  },
  { at: 420, count: 3, headline: 'Start with one small thing tomorrow', band: 'strong' },
] as const;

export const reviewRows: ClipRow[] = PLAN.map((item, index) => {
  const { start, end } = sentenceSpan(item.at * SECOND, item.count);
  const earlier = sentenceSpan(item.at * SECOND - 9 * SECOND, item.count + 1);
  const declined = item.band === 'declined';
  return {
    candidateId: `review-${index}`,
    rank: index + 1,
    displayScore: 0,
    band: item.band,
    bandLabel: declined
      ? 'Declined by editorial review'
      : item.band === 'needs_review'
        ? 'Needs review'
        : 'Ready to review',
    review: {
      status: declined ? 'rejected' : item.band === 'needs_review' ? 'uncertain' : 'accepted',
      route: 'local',
      summary: declined
        ? 'A strong line, but it lands as a conclusion without the story that earns it.'
        : 'A complete thought with a clear opening and a takeaway a viewer can use today.',
      reasons: declined
        ? ['The payoff depends on an anecdote told two minutes earlier.']
        : [
            'The opening makes sense without the preceding conversation.',
            'It ends on the takeaway rather than trailing into the next topic.',
          ],
    },
    warnings: item.band === 'needs_review' ? ['One word near the start has uncertain timing.'] : [],
    startTicks: start,
    endTicks: end,
    durationSeconds: (end - start) / SECOND,
    headline: item.headline,
    axes: [],
    penalties: [],
    boundary: {
      startTicks: start,
      endTicks: end,
      score: 0.82,
      terms: [],
      alternative: index % 2 === 0 ? { startTicks: earlier.start, endTicks: end } : null,
    },
    decision: index === 1 ? 'approved' : index === 4 ? 'kept' : null,
    docId: index === 1 ? 'preview-edit' : null,
    docJobId: null,
    latticeStarts: [earlier.start, start],
    latticeEnds: [end],
    recommended: !declined,
    proposer: 'editorial',
    clusterId: null,
    hook: { text: `${item.headline}.`, atTicks: start },
    payoff: null,
    flagged: item.band === 'needs_review',
  } satisfies ClipRow;
});
