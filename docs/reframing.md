# Reframing (W20)

Turning a wide recording into a vertical clip means deciding two things, and
they fail so differently that they are kept in separate places.

## Who to follow

`clipmill-reframe::resolve`. A judgement over evidence, and the kind of
judgement that can be wrong in a way nobody sees — a confident-looking crop of
the wrong person.

Ch. 18's focus resolver fuses active-speaker probability, diarization agreement,
track continuity, reaction salience and scene intent. Three of those five are
not measured at this phase, so what is left is continuity and detection
strength: the single-speaker case this workstream is scoped to.

That makes the **gate** the load-bearing part rather than the ranking. A track
earns the frame only by clearing all three of:

| Bar                       | Default | Why that failure                                                            |
| ------------------------- | ------- | --------------------------------------------------------------------------- |
| presence in the span      | 0.6     | below it the camera spends most of a clip pointing at an empty chair        |
| mean detector score       | 0.5     | below YuNet's own operating point its boxes are as often furniture as faces |
| margin over the runner-up | 0.15    | two people in conversation trade presence within a few points               |

Falling short produces a **fitted frame and a sentence**. The sentence is not
decoration: "why is this clip not tracking" is the first thing anybody asks, and
there are six different true answers — no faces at all, none inside this clip,
too intermittent, too weak, ambiguous, or frames nobody examined.

The ambiguous case is worth stating on its own. Two faces of equal presence are
reported as ambiguous rather than ranked, because choosing between them needs a
speaker signal this build does not measure, and a coin toss presented as a
decision is worse than a centred frame.

## How to follow

`clipmill-reframe::solve`. Arithmetic, and testable as arithmetic.

The book's objective with the terms this phase can evaluate:

```
min_x  Σ_t [ w_s‖x_t − c_t‖² + w_v‖x_t − x_{t−1}‖² + w_a‖x_t − 2x_{t−1} + x_{t−2}‖² ]
```

The protected-region term the book also lists is **absent** rather than present
and inert: protected regions are text and graphics nobody detects at this phase,
and a term recorded as never firing reads like a term that was checked.

Every one of those terms exists to prevent **chasing** — the camera lurching at
every detection flicker, which the book names as the failure users punish
hardest — and `w_a` does the heaviest lifting.

Written out the objective is a quadratic, so its stationary point is one linear
system. The acceleration term couples each sample to the two either side and
nothing further, which makes that system pentadiagonal: banded Cholesky, O(n)
rather than O(n³), microseconds for a clip. The Cholesky is hand-rolled. A
LAPACK binding would be a system dependency, a build-time toolchain requirement
and a source of platform-dependent reduction order, in exchange for forty lines
of arithmetic whose one failure mode — a matrix that is not positive definite —
the caller can be told about honestly.

**Three constraints are projections, not parts of the optimum**, and the module
says so rather than glossing it:

1. the crop stays inside the frame,
2. the camera does not exceed a speed, in frame-widths per second so the
   behaviour does not change with resolution,
3. the face occupies at least a floor of the crop, which is what fixes the
   scale.

The result is therefore a feasible near-optimal path, and a reader deserves to
know which of the two they have.

Output is **sparse keyframes**, reduced by Douglas–Peucker against the same
linear interpolation a player will do, so the tolerance means what it says. A
still camera reduces to two.

## Why the solve is an RPC and not a job

`SolveCropPath` produces a **proposal**, and nothing writes it. The answer is
wanted while somebody is looking at a clip, it is arithmetic over evidence that
already exists, and leaving the writing to the caller is what makes re-solving
after a nudge free — and what stops a re-run mutating an edit somebody accepted.

## The detector

`workers/faces/`. YuNet on onnxruntime, not through OpenCV's `FaceDetectorYN`.
The registry pins this model's runtime and `check-models.py` enforces it; the
wrapper would also have brought the desktop opencv-python wheel, which resolves
libGL at import. See R27.

The decoding is written out and unit-tested against tensors built by hand,
including that the anchor grid is read row-major — read the other way it
produces boxes mirrored about the diagonal, which is plausible, wrong, and
invisible to everything downstream.

Writing it out means owning what a wrapper would have handled, and one of those
things bit: the frames go to the model **blue first**. These weights were
trained through OpenCV, whose images are BGR, and handing them the other order
is not a crash — most faces are still found. What it costs is score on the
marginal detections, and on a test photograph of two footballers it was the
difference between two faces and three. The marginal detections are precisely
what the gate above is deciding about, so that is pinned by a test rather than
left to a comment.

Association is ByteTrack's two passes. The second one is the whole idea: a face
that turns away or gets half-lit drops below the score needed to _start_ a track
but not below what continues one, and downstream a track's length is what
decides whether the camera follows it at all.

Determinism is not incidental. The session is single-threaded and sequential,
suppression has a stable order, and frames are decoded by the FFmpeg the daemon
named on the lease — because these boxes reach a document addressed by content,
and a detector that shifted with machine load would make two runs of one
recording disagree while sharing one address.

## Gate

- `just gate-reframe` — the solver against trajectories built one behaviour
  each (still, walking, flickering, drifting to both edges): bounded jerk,
  containment, a crop that never leaves the frame, a speed never exceeded, and
  the same evidence producing the same path. Then the stage's registration, and
  the detector's decoding, graph shape and determinism against the real pinned
  weights.

It needs the pinned FFmpeg and the pinned YuNet weights, so it runs in the
`reframe` CI job on macOS and Linux rather than in the plain `rust` job.
