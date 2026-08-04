# Phase 1 exit

What Phase 1 built, what holds it to that, and what is still owed.

A phase ends on evidence, not on a feeling that the work looks finished. This
document names the gate behind every claim and the run that produced it, so
that a reader can disagree with any line by running the thing it points at.
Where a claim cannot be made yet it is written down as unmet rather than
softened — a phase declared over an absent measurement is the failure this
register exists to prevent.

## What runs

A recording goes in, and a vertical clip with burned-in captions comes out,
with nothing leaving the machine.

That sentence was not demonstrable until this workstream. Every Local Lock
proof before it stopped short of the stages that would actually phone home: the
analyze gate detects shots on a silent video and says so, the shell gate stops
at ingest, and the speech conformance harness imports the model classes
in-process — which proves the models work, not that a lease delivers them. The
three stages that load half a gigabyte of weights, and the one that writes the
file a user keeps, had never run inside a denied namespace.

`gate-lock-phase1` runs them: the egress canary, then analyze, direct and render
against a live daemon and five worker processes, then the canary again, because
a stage that opened a socket and gave up is still a stage that tried.

## Exit conditions

| Book condition                                        | Gate                                                        | State                                                       |
| ----------------------------------------------------- | ----------------------------------------------------------- | ----------------------------------------------------------- |
| End-to-end offline on reference machines              | `gate-lock-phase1` (full pipeline under `--network=none`)   | **Met** on Linux CI and the dev Mac; Windows deferred (R47) |
| Seed-40 top-10 recall met                             | `gate-recall` on the annotated corpus                       | **Not met** — the corpus is not annotated yet               |
| Render SLO ≥1.5× real time at 1080×1920               | `gate-render-slo`, attested on the machine that measured it | **Met** — 1.587× on Darwin arm64                            |
| Recovery <30 s at any boundary                        | `gate-kill` plus the per-stage kill suites                  | **Met**                                                     |
| Zero hidden cloud dependencies                        | `network-denial` + `lock-phase1` + model licence checks     | **Met**                                                     |
| Spine reality: exact preview, word-snapped, determin. | `gate-render`, `gate-editor`, `gate-golden`                 | **Met**                                                     |

`just gate-phase1` runs every reproducible gate and then verifies the evidence
for the three that no hosted runner can produce. It fails today, correctly, on
the recall report.

## What is owed

**The recall attestation.** `gate-recall` needs an editor's annotations over the
private corpus — acceptable-moment sets, importance, alternative boundaries —
and those are a person's judgement, not something a run can synthesize. Until
they exist, the only recall number that has been measured is the synthetic
floor: `gate-recall-smoke` recovers 3 of 3 planted moments at a duplicate rate
of 0.000, which says the engine finds a moment placed in front of it on purpose
and says nothing about a real recording. `verify-phase1-attestation.sh` refuses
until the real report is committed, and names the command that produces it.

**Reframing never runs.** Nothing plans a `detect-faces` stage, so every
directed clip is fitted rather than following a speaker, and the inspector says
so honestly in its rationale. The detector, its recipe and its worker all exist
and are exercised by `gate-reframe`; what is missing is the job that would
produce a face track during an analysis.

**Frame-rate coverage.** The Lock gate renders from a 30 fps source, which is
the case that found R57. It does not sweep the rate matrix; a source at 25 or
60 is untested end to end.

## What the exit run found

Five defects, none of which any existing gate could see, all of them in the
stages nothing had run through the daemon:

- A gated loudness window serialized as `null` into a field the schema requires
  to be a number, failing the analysis of any recording with a quiet passage.
- `direct_clip` decoded a response envelope as its payload, so it returned a
  document id that was really a request id and an empty document — which is why
  the caption overlay after approving a clip had always been blank.
- The render refused its own output on any source not already at 30000/1001,
  because `-frames:v` caps rather than pads (R57).
- Task capacity assumed a flat half-gigabyte of free disk regardless of the
  machine, so `deliver-export` — which declares a gigabyte — was unschedulable
  everywhere and export through the daemon had never run.
- The delivered thumbnail named no format, and its partial suffix left FFmpeg
  nothing to infer a muxer from.

The pattern is worth stating, because it is the argument for this gate
existing: each stage had a passing gate that tested it a level below where it
runs. The export gate runs `cargo test` against the export crates and never
submits a job. The speech harness calls the recognizer directly and never takes
a lease. A test one level below the seam cannot see the seam.

## Definition of done, per stage

The book requires thirteen things of a vertical slice (ch. 3). Seven of them
are not per-stage properties here — they are structural, and a stage that
lacked them could not be a stage at all:

- **Capability contract, artifact schema with versioning, and cache key.** A
  stage exists as a `Recipe` naming its output kind and semantic version, and
  publishes to a content-addressed store whose key is derived from the inputs,
  the payload and the implementation identity. There is no way to add a stage
  that skips this.
- **Cancellation, progress, declared resources, explicit failure states.** Every
  stage runs as a task row carrying progress, a wait reason, a resource
  declaration and a failure class split into deterministic and transient.
- **Local-only telemetry.** Artifact operations log to the daemon's own log; the
  network-denial and Lock gates are what say nothing else leaves.
- **Upgrade/migration.** Schema versions are in every artifact's
  `schema_version`; the store migrates with a backup gate.

The four that genuinely vary are worth a table. A blank is a claim nobody
should make.

| Stage     | UI correction path             | Exact render support | Byte-stable golden      | Licence / model card        |
| --------- | ------------------------------ | -------------------- | ----------------------- | --------------------------- |
| ingest    | re-analyze                     | n/a — derives media  | — conformance drill     | FFmpeg pinned in `bom.toml` |
| speech    | caption text editing           | ✓ cue windows        | — conformance drill     | ✓ three model cards         |
| shots     | —                              | n/a — evidence only  | — conformance drill     | none — no model (R52)       |
| evidence  | —                              | n/a — evidence only  | ✓ `golden.rs`           | none — no model             |
| discovery | boundary alternative           | ✓ snapped boundaries | ✓ `golden.rs`           | none — no model             |
| ranking   | approve / keep / reject        | n/a — selects only   | ✓ `ranking_golden.rs`   | none — no model             |
| reframe   | editor nudge, refusal sentence | ✓ crop path          | — properties, not bytes | ✓ YuNet card (R27)          |
| captions  | text and grouping edits        | ✓ burned-in cues     | ✓ `goldens.rs`          | ✓ pinned font (R53)         |
| edit IR   | undo / redo                    | ✓ the document is it | ✓ `invertibility.rs`    | none                        |
| render    | re-render after an edit        | ✓ it is the render   | ✓ `compilation.rs`      | FFmpeg + font digests       |
| export    | rename, re-export              | ✓ validation strip   | — round-trip, not bytes | ✓ rights attestation        |

Five stages carry something other than a byte-stable golden, and the
distinction is not cosmetic. A golden says the bytes are the same everywhere; a
conformance drill says the output is well-formed and within tolerance on this
machine; a property test says an invariant holds over cases built to stress it;
a round-trip says two descriptions of one format have not drifted apart.

Ingest and speech cannot have a golden — a decoder and a platform voice differ
across machines by design (W13, W15) — so their drills are the strongest claim
available. Reframe's `crop_path.rs` pins bounded jerk and containment over
synthetic trajectories, deliberately, because a test over real footage would
measure the detector as much as the camera; a golden over a fixed face track
would still be worth having and does not exist. Export's `contract.rs` pins its
structs against the generated types, which catches drift but not a changed
byte. Shot detection could have a golden and has none.

Those four are gaps, not design decisions, and none of them is load-bearing for
an exit claim made above.

Shot detection also has no UI correction path. An editor who disagrees with a
cut cannot say so; they can only move the clip boundary, which is a different
statement. Recorded here rather than in the register because it is unfinished
work rather than a decision.

## Reproducing this

```
just gate-phase1
```

Every reproducible gate, then the evidence for the three that are not. The
private ones, on hardware that can make their claims:

```
just gate-render-slo                 # the reference Mac
just gate-recall <corpus> <manifest> <licences> <annotations> <socket> <output>
just gate-asr-mlx <signing-key>      # an Apple GPU
```
