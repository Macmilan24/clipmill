# Phase 0 evaluation harness

The W7 harness evaluates source evidence through the same daemon IPC, durable
job engine, and CAS used by production stages. It accepts only a local corpus
whose manifest and license attestation carry valid Ed25519 signatures. Before a
request reaches the daemon, every media file is checked for a portable relative
path, regular-file type, exact byte size, and SHA-256 digest. License records
must cover exactly the signed item set.

The committed public smoke recipe generates five tiny synthetic fixtures with
the pinned FFmpeg sidecar: CFR, VFR, rotation metadata, offset multi-audio, and
one intentionally malformed input. No binary media or private signing key is
stored in Git. The gate starts a real daemon, verifies its signed device-profile
artifact, registers each source, waits for its durable probe job, verifies the
source-map CAS payload, and repeats valid sources warm. A warm run must reuse
the exact source-map artifact ID.

Run the reproducible public gate with:

```sh
./tools/fetch-ffmpeg.sh
just gate-device
just gate-eval-smoke
```

The canonical `clipmill.eval.run.v1` manifest records daemon/schema versions,
policy, hardware-profile identity and generation, source fingerprints,
artifact IDs, cold/warm timings, cache results, and expected structured
failures. It never records corpus directories or source paths. CI runs the gate
on macOS and Linux, and the Linux Local Lock job repeats it inside a namespace
where the egress canary proves networking is unavailable.

## Known gap: the reframe benchmark is not scheduled yet

Recorded here rather than discovered later. The book asks for reframe to be
measured on annotated media — `22-evaluation.tex:65` names subject coverage,
identity switches, protected-region loss, camera jerk and fallback correctness —
and `18-reframing.tex:95` is more specific still: the acceleration weight `w_a`
is to be tuned per layout state on that benchmark, "not by eye".

Two things follow from that, and neither is satisfied today.

The acceleration weight this build ships is hand-set. `gate-reframe` proves the
solver is the arithmetic it claims to be — bounded jerk, containment, a speed
never exceeded, the same evidence twice — on synthetic trajectories, which is
what W20's gate asks for and all it asks for. Synthetic trajectories cannot say
whether a real camera move feels like an operator or like a classifier, so the
weight is defensible but unmeasured.

And the harness W26 describes verifies candidates and ranking. It does not carry
the reframe row. Adding it needs annotated media, which is exactly why it
belongs in W26 and not in the unit suite: the corpus policy keeps media out of
Git, so anything measured against real faces runs behind the same private-corpus
attestation as `gate-recall`.

This is not a defect in what shipped. It is an obligation the book states, that
W20 was not scoped to meet, and that W26's current scope would not pick up on
its own.

## Private Seed-40 exit

The private Seed-40 baseline is deliberately not a repository media artifact.
The rights holder keeps the media, full signed corpus manifest, item-level
license attestation, and private evaluation signing key outside Git. A license
record must explicitly grant either redistribution or evaluation; private
evaluation permission is not mislabeled as redistribution.

The evaluation key is a raw 32-byte Ed25519 seed encoded as hexadecimal in a
regular, non-symlink file with mode `0600`. With the pinned sidecars installed,
the rights holder runs:

```sh
just gate-seed40 \
  /absolute/private/corpus \
  /absolute/private/corpus-manifest.json \
  /absolute/private/license-attestation.json \
  /absolute/private/phase0-signing.key \
  eval/seed40 \
  /absolute/private/corpus-verification-key.hex
```

The gate refuses any count other than 40, verifies every signed byte and rights
record, runs every item cold and warm through a real daemon, requires every
valid source map to verify with an identical warm artifact ID, and requires
every declared hostile item to repeat its expected structured failure. It then
writes exactly four public files under `eval/seed40/`:

- `verification-key.hex` — the dedicated run-verification public key;
- `corpus-metadata.json` — corpus ID, signed-manifest digest, signing public
  key, and valid/hostile counts;
- `license-summary.json` — signed-license-document digest and aggregate,
  publishable rights counts;
- `run-attestation.json` — the canonical, Ed25519-signed, path-free cold/warm
  run and the exact copies of both public summaries.

`just gate-phase0` verifies that exact committed file set without access to the
private corpus. It fails if a signature, count, cache identity, outcome,
license total, canonical byte, or path-leak invariant changes.

## Recorded Phase 0 baseline

The completed baseline contains 40 private technical derivatives of Blender
Foundation Open Movies: 21 derived from
[Sintel](https://durian.blender.org/sharing/) under CC BY 3.0 and 19 from
[Elephants Dream](https://orange.blender.org/blog/a-call-for-textures/) under
CC BY 2.5. It records 39 verified valid sources and one deliberately truncated
hostile source. The complete source URLs, byte hashes, attribution, license
evidence, transformations, media, and signing keys remain outside Git.

Protected `main` contains only the four safe files in `eval/seed40`. Their
signed run records all 40 cold/warm outcomes, identical warm source-map IDs,
the structured hostile failure, hardware-profile identity, contract and
sidecar versions, timings, cache results, and aggregate license counts without
private paths or media bytes. Together with the green W8 security and offline
matrix, this is the Phase 0 exit proof.

---

# Phase 1 evaluation: recall, annotation, and the SLOs

What W26 added, and the first honest number it produced.

## Ground truth is plural

An annotation is the only input to this project that nothing derives — it is a
person's opinion, and every recall figure is measured against it. So it is a
published contract (`clipmill.eval_annotation.v1`) shaped the way the book's
ch. 22 says ground truth actually is:

- **Moment sets, not a moment.** A recording has several spans worth clipping.
- **Alternative starts and ends per moment.** A cut that landed where the
  annotator's second choice was is not an error, and boundary-edge error is
  measured against every acceptable span rather than one preferred one.
- **Importance grades** — `essential`, `strong`, `acceptable`. Recall is
  reported per grade as well as pooled, because a system that finds every
  acceptable moment and misses every essential one has a good aggregate number
  and is useless.
- **Exclusions with reasons.** Not the same as a span simply not being a
  moment: not-a-moment is an absence, an exclusion is a claim, and offering one
  fails a gate whatever the bar says.
- **One document per annotator.** Disagreement is signal — a moment two of
  three editors accept is a different fact from unanimity — so agreement is
  computed across documents and never resolved inside one.

An empty `moments` list is a real answer. A recording with nothing worth
clipping is a fact a recall number has to be able to represent, which is why
the key is always present rather than omitted when unfilled.

## The metric stack

One implementation, in `clipmill_eval.recall`, called by everything that
reports a number. Four measurements, answering different questions:

| Metric               | Question                                                    | Notes                                                                |
| -------------------- | ----------------------------------------------------------- | -------------------------------------------------------------------- |
| Recall@10 at IoU 0.5 | Did the acceptable moments reach the board?                 | Scored over `selected` — what a user is shown — not the whole cohort |
| Multi-moment recall  | Did a recording with several moments give up _all_ of them? | Single-moment recordings are excluded; they cannot fail it           |
| Duplicate rate @5    | Is the _set_ useful, or one moment cut five ways?           | A clip duplicates an **earlier, better-ranked** one                  |
| Boundary-edge error  | Did the cut land where an editor would have put it?         | Median and p90, against the alternatives                             |

Two details worth knowing before reading a number:

- **IoU 0.5 is stricter than it sounds.** Two spans that overlap by half their
  length score 1/3, not 0.5. A clip may run 50% longer than the moment and
  still pass (2/3); three times as long does not (1/3).
- **Recall pools over moments, not recordings.** A recording with nine moments
  and a recording with one are different amounts of evidence, and averaging
  their rates would weight them the same.

## The planted smoke, and what it found

CI cannot run the private corpus, and a synthetic recording cannot be given
speech without a recognizer nobody has pinned for this. So the moments are
planted **at the transcript** rather than at the audio: `clipmill-discovery`
builds a recording with three well-separated moments — a question with its
answer, a claim carrying numerals, a topic that opens and closes, one for each
proposer — runs the real mesh, lattice, scorer, boundary optimizer and selector
over it, and leaves the documents on disk. The harness scores them.

The split is deliberate. The engine is Rust and the metric is Python, and
neither knows the other's answer.

The first run of this gate found a real defect, which is what it is for.

**Near-duplicate clips reached the results board.** Recall was 1.0 and boundary
error 0 ms, but the top-5 duplicate rate was **0.6**. Two candidates came out
with _identical_ chosen boundaries in _different clusters_, and three clips
covered the same moment at 78→108, 78→97 and 78→103.

The mechanism: clustering runs during `discover()`, over the candidates'
**proposed** intervals. The boundary optimizer then re-cuts each candidate in
`rank()`, and two candidates that were distinct proposals converge onto the
same cut. `select()` correctly refuses a second member of a cluster it already
took — but nothing noticed that two _different_ clusters now held the same
clip. The set simultaneously reported `all_remaining_are_duplicates` as a
shortfall reason and contained duplicates, which is self-inconsistent.

This contradicts the book's own rule (ch. 16): _"Asked for ten clips from a
recording holding four good moments, the honest answer is four and a reason,
not ten with six the system does not believe in."_

### The fix

`select()` now makes the same refusal twice, against two different statements
about what "the same moment" means. The cluster check is discovery's, made over
the proposed intervals, and it stays. The new one is the selector's own: after
the boundary optimizer has chosen, a candidate whose **chosen** interval repeats
one already selected is refused outright and counted into the shortfall.

Three things about it are load-bearing:

- **It is a refusal, not a penalty.** The MMR similarity term already saw this
  overlap, but as a preference to be traded against quality — and at the
  default diversity of 0.7 the quality term wins. Showing a user one clip twice
  is not a trade-off to be priced.
- **The threshold is the metric's**, IoU 0.5, not a number tuned until the rate
  looked good. A selector held to a looser figure than the gate would ship sets
  the gate calls duplicated, and the project would quote one number while
  believing another. It is checked from the test side against the metric's
  formula rather than the selector's, so the two cannot drift apart quietly.
- **It refuses rather than backfills.** The set gets shorter and says why; it
  does not reach further down the cohort for a replacement.

What moved: on the planted recording, 7 clips selected → **3**, one per planted
moment, with the other 7 of the 10 requested accounted for as shortfall.
Duplicate rate **0.6 → 0.0**, recall unchanged at 1.0, boundary error unchanged
at 0 ms. On the committed `interview` cohort, 7 → 4; the three dropped were an
identical cut and two repeats at IoU 0.85 and 0.63. In both cases the _cohort_
is untouched — every candidate is still ranked, scored, and carries its card and
its boundary. Only the selection narrowed, so nothing became unanswerable; a
refused clip is still there to be asked about.

`eval/recall/planted-bar.json` lowers the ratchet from 0.6 to **0.0**. That is
not an aspiration: the selector and the metric now share one threshold and one
formula, so any rate above zero means the refusal was weakened.

## The bar

`eval/recall/planted-bar.json` names only what it constrains. An absent key is
not a metric silently set to zero — it is one this bar makes no claim about,
which is the honest state before a baseline exists.

The planted bar sets recall to 1.0 because the moments were _planted_: failing
to find something put there on purpose is a failure, not a measurement. The
private corpus has no bar yet, and will not until the first annotated run
produces one.

## Fetching the corpus

`clipmill-eval fetch-corpus` is the only thing in ClipMill that reaches the
network on purpose, and it is not part of the product: it runs on a developer's
machine, outside the Local Lock, before any evaluation starts.

- **Licences are recorded per item, from the spec, before the bytes arrive.**
  An item whose licence cannot be named from a closed list is not fetched.
- **Media never enters Git.** The destination is checked against the
  repository's own ignore rules — asked of `git check-ignore` rather than
  guessed, because a rule written here would be a second implementation of
  ignore semantics. A question Git cannot answer is a refusal.
- **The output is unsigned.** Signing is a separate command with a separate
  key, so a fetch can never produce a corpus that claims to have been attested.

## Worksheets

`clipmill-eval annotate` runs the analyze DAG over each corpus item and writes
two files per recording: a Markdown transcript with timecodes and exact ticks,
and a skeleton annotation. The annotator reads one and fills the other.

Sentences come from the **evidence index**, not the raw transcript. Annotating
against a different segmentation from the one the proposers score over would
measure the segmentation as much as the ranking. Regenerating a worksheet never
overwrites an annotation somebody has filled in.

## Gates

```bash
just gate-recall-smoke
```

```bash
just gate-golden
```

```bash
just gate-render-slo
```

`gate-recall-smoke` and `gate-golden` run in CI on every push. `gate-render-slo`
does not: a shared runner's throughput says nothing about a laptop, so CI
asserts completion and determinism only, and the 1.5× ratio is measured and
attested on the machine that measured it — with the program's own duration
taken from the render manifest, so a render that produced a shorter clip than
intended cannot look fast.

`gate-recall` is the private one. It needs the corpus, the annotations, and a
running daemon, and none of the three is in this repository.
