# Milestone 1 recheck and Milestone 2 implementation

The audit began at `86988a7` with a clean working tree. The earlier Milestone 1
findings referred to `57d6dd4`, not this revision. Its crop/gain trim fixes,
caption-fragment recovery and durable export tracking were already present.
No PR was merged during that initial audit and implementation pass.

## Milestone 1

The real daemon/worker/shell-bridge acceptance scenario passed before and after
the editorial integration. The final strengthened run analyzes the same
recording in two projects concurrently. The final rerun passed in 123.11 seconds:

- Select the older project's candidate at 596–617 seconds.
- Play/scrub through the shell media route; correct Yuki to Okonkwo in both
  caption presentations; trim both ends to 599.43–613.79 seconds.
- Verify static crop and boundary gain preservation, undo/redo, daemon restart,
  retry-safe document approval, stale revision refusal and export revision 10.
- Decode the delivered clip: clip seconds 0.50, 7.18 and 13.86 map back to
  source seconds 599.91, 606.61 and 613.30. Check both caption sidecars and the
  ASS input to burn-in.

The previously disclosed same-recording concurrency failure had a concrete
cause: competing tasks could repeatedly fail against an in-flight artifact.
Admission now serializes the same stage over the same recording fingerprint,
including across projects, without consuming attempts. The second run reuses
published work. The real scenario above now exercises this race.

A further selected-source bug was found in Analysis Progress: it read the
first source in a project. It now uses the failed job's source identity, and
a renderer test proves that local reanalysis does not use another recording.

Native-window automation was attempted with an isolated development build.
The computer-use tool timed out opening/attaching to it; no hand-clicked UI
pass is claimed. The temporary app/daemon process trees were stopped. The
bridge acceptance scenario and renderer tests are separate evidence.

## Milestone 2

Implemented the local Qwen3.5 proposal → deterministic validation → semantic
review → targeted visual-check → ranking route, reusing the existing selected
run editor/export. Added opt-in transcript-only cloud proposal/review with
Keychain credentials, durable per-run budget reservations and explicit local
reanalysis after failure. No paid/cloud inference request was made.

The old scheduler admitted only `local-lock` tasks; the new cloud variants
needed explicit admission support as well as recipe registration. Admission
now derives permitted network task kinds from the implementation registry and
rejects recipe/task policy mismatches at planning and leasing. Resource accounting includes these tasks. Consent/model/cap validation
remains at submission and inside the worker.

The user chose Qwen3.5, skipped the dedicated M0 work, and asked not to focus
this pass on benchmarking. Functional delivery proceeded on that basis. This
does not waive the original quality goal: measured acceptance gain and repair
burden remain unverified. Platform scope is the current Apple-silicon build. No comparative
quality, speed, Linux/Windows, or live cloud compatibility claim is made.

The real-runtime checks also required concrete fixes beyond the mocked tests:

- Preserve verified GPU capacity after the scheduler consumes its update.
  Refresh free-memory admission from new measurements, and show insufficient
  memory as not-ready with a recovery action.
- Run readiness inference in a subprocess and measure free memory after it
  exits; otherwise the check counts its own model against future tasks.
- Pin MLX-VLM 0.7.1 with image preprocessing dependencies and use its native
  constrained decoder. Runtime proof is tied to this runtime version as well
  as the machine and weights. The earlier image implementation failed before
  seeing any frame.
- Constrain proposal choices to duration-valid sentence spans. Real unconstrained
  references produced 6–13 second quotes despite a 20-second minimum. The fixed
  decoder produced an 81.268-second complete moment; Rust still validates it.
- Keep uncertain interior word timing visible as Needs review. Missing speech
  or interpolated timing at a cut boundary still prevents that cut. A few
  uncertain interior caption words no longer discard a whole complete moment.
- Read the frame artifact's actual `index.json` payload in the visual stage.
- Bound long-recording outlines, match the shared title limit, and test span constraints.

## Initial verification evidence

These counts describe the earlier implementation pass. The recording-failure
recheck and updated verification follow below. The daemon count was `--lib`,
not a claim that the integration suite ran.

- Rust daemon: 215 passed, 1 intentionally ignored.
- Editorial Rust: 28 passed, including empty results, bad references,
  duplicate spans, rejected/visual-dependent candidates and failed reviews.
- Python editorial worker: 17 passed, including cancellation, failures not
  entering the success cache, in-span frame selection, consent, durable
  shared budgets, uncertain-call reservations and secret redaction.
- Renderer: 311 passed across 29 files; TypeScript type checking passed.
- Shared TypeScript contracts: 98 passed. Rust/Python editorial contract
  fixtures, schema lint and registry policy passed through the unit gate.
- All Rust workspace test executables compile. Targeted Rust Clippy passes
  with warnings denied. New Python code passes Ruff checks.
- Real pinned Qwen/model-to-export gate: **passed**. The synthetic talk
  produced one selected moment, Qwen's semantic review accepted it, and the
  timing validator retained a visible Needs review warning for interior
  interpolated captions. It was not padded to five suggestions.
- Export produced an **81.5815-second MP4**, decoded without errors, with
  nonempty SRT and VTT sidecars. The MP4 is 1080×1920 H.264 with 48 kHz AAC;
  a decoded frame was visually inspected and shows burned-in captions.
  The exact selected run, candidate, document
  revision 0 and immutable snapshot were verified through the daemon API.
- Real negative/visual checks passed: setup chatter returned **none**; the
  model answered **no** when a diagram was not visible in the sampled frame.
  A separate four-image inference check also completed on the same model.

The final rerun reused successful real proposal/review artifacts after the
validation and frame-descriptor fixes, exercising normal cache recovery.
No fake editorial response or cloud call was used in the functional gate.

Evidence:

- [M2 machine-readable result](/private/tmp/cm-editorial-delivery/result.json)
- [M2 rendered clip](/private/tmp/cm-editorial-delivery/delivered/01-editorial-smoke.mp4)
- [M2 ranking and visible review warning](/private/tmp/cm-editorial-delivery/ranking.json)
- [M1 rendered clip](/private/tmp/clipmill-m2-readiness-work/delivered/01-older-clip.mp4)

Logs are local under `/private/tmp/clipmill-m2-*.log`. The implementation and
functional evidence demonstrate the implemented M2 path. Human editorial
quality remains unmeasured; the user deferred benchmark-focused work, not the
product goal of better accepted clips. Native-window interaction and a live cloud-provider
call remain unverified; neither is claimed by these tests. Milestone 3's
framing, caption polish and audio work remains separate.

At the end of that initial pass, changes were uncommitted on
`feature/editorial-windows` and no merge had been performed. The completed work
and subsequent recording-failure corrections are collected in
[PR #71](https://github.com/Macmilan24/clipmill/pull/71).
The temporary Rust build cache and unused isolated test app/data were removed
after verification. The pinned Qwen weights, worker environment, delivered
videos and final evidence remain available.

## Real-recording failure correction

The reported Reacher run (`job_01M2TTP7X0SFVPJQGT6KKRCZJW`) answered all ten
editorial windows: one proposal and nine explicit “none” answers. Validation
refused its single 0.142–42.594-second spoken span because an interior
12.77–12.99-second utterance had unavailable word alignment. Its endpoint
words were aligned, and there was no missing audio/ASR region in the proposed
span. This failure was caused by the distinction between unavailable timing
and unavailable speech, not a malformed model window.

The validator now permits interior alignment uncertainty with an explicit
caption-timing Needs review warning. Missing audio or recognition remains a
hard refusal, as do unaligned endpoint words or an invalid region at a cut.
Padding cannot enter an unavailable region. The retained span in the saved
artifact replay is 0.071–42.744 seconds after bounded gap padding.

The replay reads the original transcript, windows, proposals and index without
calling a model or modifying the store. It produces one candidate and one
pre-review ranking row, with no validation rejection. This does not establish
semantic acceptance. Its helper is `crates/clipmill-editorial/examples/replay.rs`;
private recording text is not checked into the repository. Local evidence is
`/private/tmp/clipmill-reacher-replay`.

The same fix pass also implements:

- Independent proposal/review failure rows; later windows and other candidates
  continue. No answered windows or no usable semantic reviews still fail the
  stage. Optional visual failures preserve assessed nonvisual candidates.
- Partial counts in Results, including when the requested count is met or the
  result is empty. Failed semantic reviews are excluded. Visual uncertainty
  and contradictory accepted/problem verdicts stay Needs review.
- Typed candidate/review arithmetic, retained titles and proposal rationale,
  merged duplicate nominations, and guarded padding.
- Durable partial artifacts that trigger recovery recipes rather than complete
  cache hits for explicit failed/malformed replies. Earlier immutable evidence
  is preserved; later runs may incur additional inference.
- Separate explicitly started cloud worker, default local-only capabilities,
  registry-derived network admission, CPU-only cloud backend registration,
  specific redacted provider errors, and updated Local Lock/threat documents.
- Editorial worker inclusion in CI/setup/Dependabot, reviewed Python license
  expressions with fail-closed parsing, and corrected readiness expectations.

Updated local verification:

- Daemon unit tests: **220 passed, 1 intentionally ignored**; focused readiness
  integration passes. No claim that the full daemon integration suite was rerun.
- Editorial Rust tests: **37 passed**, including the real-failure boundary
  regression, partial windows/reviews, contradictory judgments, and missing
  optional visual checks. Discovery and Rust contract suites also pass.
- Editorial Python: **35 passed**; worker SDK: **163 passed**.
- Renderer: **332 passed**, with TypeScript checking; TypeScript contracts:
  **98 passed**.
- Full license gate across **12 locked environments**, **11 license-policy
  regressions**, all **32 JSON schemas**, changed-file Prettier, and relevant
  Ruff checks pass.
- Workspace/all-target Clippy passed before the final contradiction guard;
  editorial/daemon/discovery all-target Clippy passed again after it. Warnings
  are denied in both runs.
- Offline regeneration of the changed candidate/ranking contracts produces
  six byte-identical Rust, Python and TypeScript outputs.

The existing database was backed up to
`/private/tmp/clipmill-before-editorial-repair-98ux_5p5/clipmill.db`. The corrected
backend, six local workers, frontend server and desktop shell were started.
A fresh local analysis, `job_01M2TZDWS7HC5W7FC9YQTKECDC`, uses the user's original
15–60-second duration range and requested count of six. Its original failed
run remains unchanged. Fresh run evidence is saved under
`/private/tmp/clipmill-reacher-recheck`. The fresh run **succeeded through every
stage**, reused ingestion/speech artifacts and made new local model calls. All
ten windows were assessed with zero failed windows, reviews or visual checks.
Qwen again proposed one moment, which passed deterministic validation. Its
semantic review **rejected** that moment for incomplete payoff, missing context
and dependence on the picture, so the final selected set is **empty**. Those
are the model’s judgments, not independently established facts about the scene.

This confirms the pipeline repair. It does not establish useful recommendations
for this episode. Scripted TV is outside the current English podcast/interview
target. No candidate was approved or exported merely to satisfy a count, and
no cloud call was made. The app and local services remain running for testing.
