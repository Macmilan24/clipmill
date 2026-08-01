# Captions (W21)

Captions are the most-read typography this product ships, and they are read
over a moving image by people who often cannot hear it. That is the whole
reason there is a crate here and not a filter argument.

## One IR, two intents

`captions.cues.v1` holds **one token array and two groupings of it**. The
accessibility grouping is the conservative one every sidecar is written from.
The burn-in grouping is kinetic — a few words at a time, deliberately faster —
because that is the register a muted feed is read in.

They are two rhythms and never two vocabularies. The book's design rule is that
divergence between what a viewer reads and what a deaf viewer reads is not a bug
class this product will host, and the shape is what makes it unreachable: both
groupings index the same tokens, so a correction applied once is read by both,
and no code path can give one of them a word the other does not have.

**Line breaks are decided here and stored here.** A break re-decided at render
time is a break that can differ between the preview and the file; the renderer
is already configured never to re-wrap, and now there is something upstream that
means it.

## The segmentation is exact

`clipmill-captions::segment`. Where to break a line is the craft of captioning
and it is also arithmetic. Every consideration ch. 19 lists — reading speed,
break quality, not stranding an article, line balance, cue duration, cut
avoidance — is a cost on **one candidate cue**. Costs that depend only on which
tokens a cue holds make the best partition a shortest path over token
boundaries, and a shortest path is exact.

"Exact" is load-bearing rather than decorative, so it is tested rather than
claimed: for runs small enough to enumerate, the dynamic program is compared
against **every possible segmentation**. A greedy segmenter agrees with the
optimum on most inputs, which is exactly why the disagreement has to be measured
instead of reviewed by eye — every individual break a greedy segmenter makes is
defensible, and the result still reads badly.

### What is absolute and what is merely expensive

Two constraints are hard:

1. a line may not exceed the profile's character ceiling — **that ceiling is how
   the safe area reaches the segmenter**, since it is the width that fits inside
   it at the preset's type size;
2. a cue may not span a shot cut **that falls in a silence**, because a caption
   that survives a change of picture reads as a glitch.

A cut that falls inside a spoken word is deliberately allowed. Nothing can be
done about it without dropping the word, and a missing word is worse than a
caption that outlives a cut nobody could have avoided.

Everything else is a cost, **reading speed included**. The alternative to a
slightly fast cue is no cue, and a viewer would rather read quickly than read
nothing. What makes the ceiling reachable at all is that a cue may be held past
its speech — bounded by the next cue, the profile's maximum, the next cut, and
the span. Every one of those bounds is a function of the cue's own tokens, which
is precisely what keeps the cost local and the program exact.

## The numbers are published, not invented

The accessibility profile is Netflix's English timed-text guidance: 42
characters a line, at most two lines, twenty characters a second, five sixths of
a second on screen at the shortest and seven at the longest, two frames of blank
between cues. A viewer who relies on captions has already learned to read at
those numbers.

They are **values rather than constants** because the reason they are right is
English. A ceiling counted in Latin characters says nothing useful about a CJK
cue, and reading-rate norms move with the script. A language with no entry is
reported as unknown rather than quietly given English ceilings.

Three-line profiles are **refused rather than approximated**. The balance term
compares a cue's lines against each other, so a third line is a different
optimization and not one more loop.

## Emphasis, fillers, corrections

Emphasis comes from evidence or it does not happen: a term the topic index found
salient, appearing more than once. Not length, not position, not a model asked
to be interesting.

Fillers are **tagged and never removed**. A caption is a record of what was said;
a viewer reading a tidied sentence is being told what a program thought they
should have heard. What the tag buys is that a filler may never carry emphasis.

Corrections are an **overlay keyed to the token they replace**, never a rewrite
of it. That is what lets a better model re-transcribe a recording and propose
updates without erasing a word somebody already fixed, and a user's correction
outranks a re-transcription's for the same token.

## Presets

Clean, Minimal, Boxed — and a reduced-motion twin for each, which is not an
afterthought toggle. Motion sensitivity is real and a caption style with no
still version is a style some viewers cannot use. The twin is the same
typography with the sweep removed, so choosing it changes how the words arrive
and never which words they are.

The style placeholder W13 left behind is now one of these. It named a look
nothing defined, which was harmless while nothing resolved the name and wrong
the moment something did — the renderer refuses an unknown style rather than
defaulting to one.

## What this phase does not do

Stated rather than implied.

- **Placement is one stable anchor.** Per-scene avoidance of faces, OCR regions
  and high-motion zones is ch. 19's placement section and is not attempted here.
  A caption that hops lanes every cue reads as broken even when every hop was
  locally right, and there is nothing measured yet to hop away from.
- **Only English has its own numbers.** The profile is parameterized and the
  direction is carried per document, so a translated track is a sibling rather
  than a schema change — but no other language has an entry yet, and an unknown
  one is reported as such.
- **Nothing writes corrections yet.** The overlay exists and is applied; the
  surface that creates entries in it is the Clip Inspector's job.
- **Word-level ripple editing is not here.** Cues reference source-timed tokens,
  which is what will make it compose, but the editing surface is W23's.

## Gate

- `just gate-captions` — the optimality property test against exhaustive
  enumeration; the goldens, so a change to the weights has to be argued for
  rather than merged; **zero reading-speed violations in the accessibility
  intent**, which is not "few" and not "acceptable"; no cue over a cut in a
  silence; and the round trip out through the render's own writers, parsed back
  as a player would rather than compared to a stored string.

It needs no pinned media and no weights — cue segmentation is arithmetic over
documents — so it runs in the plain `captions` CI job on macOS and Linux.
