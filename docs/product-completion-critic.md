# Independent product completion critique

Initial audit: 18 September 2026, against the implementation merged in `55ea0de`.
The user subsequently confirmed that scripted TV/movie scenes belong in the
first version, alongside English podcasts/interviews. YouTube uploads should
default to private, with a separate explicit Publish action.

This is an independent, initially read-only assessment. Findings below describe
the audited revision, not the status of later repairs. Passing implementation
tests does not establish the deferred human acceptance targets in the
[upload-ready plan](upload-ready-v1-plan.md).

The historical tables below record discoveries, not the final open-issue list.
The latest real-gate result and repair dispositions appear under
[latest verified closures](#latest-verified-closures).

## Release blockers and falsification checks

| Priority | Finding and evidence                                                                                                                                                                                                                                                                                                                                                                                                       | Minimum repair and check                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| P1       | **Some complete moments are impossible to propose.** `crates/clipmill-editorial/src/windows.rs:50` overlaps cores by 80 words; `workers/editorial/src/clipmill_worker_editorial/inference.py:52` restricts endpoints to one core. The real Reacher window artifact contains 13,350 sentence spans lasting 20–90 seconds; 1,762 fit no core. Those spans are duration-valid possibilities, not proof of worthwhile content. | Make eligibility/overlap cover the requested maximum duration, retaining bounded input and duplicate removal. A 70-second complete beat centered on a core seam must be nominatable. Test sparse dialogue as well as dense speech.                                                                                                                                                                                                                                                    |
| P1       | **The editorial rubric excludes the newly supported genre.** Both `prompts/propose.v1.txt:1` and `prompts/review.v1.txt:1` explicitly describe podcast/interview editing. The actual episode generated one nomination, rejected for incomplete payoff, confused transcript text and visual context. This does not by itself prove general model pessimism.                                                                 | Persist an editorial content profile through request, cache identity, prompts and result provenance. Scripted review should assess a complete local dramatic beat, without demanding resolution of the entire plot. Preserve rejection for evidenced missing setup/end or misleading omission. Distinguish recognition uncertainty from proven incomplete speech. Check a real understandable scripted exchange and a deliberately incomplete negative example; do not force a quota. |
| P1       | **A rejected nomination cannot be inspected or repaired.** `crates/clipmill-editorial/src/review.rs:106` removes it from the cohort; `apps/desktop/src/results/model.ts:222` renders only the cohort; `crates/clipmill-director/src/lib.rs:167` requires a cohort row to create an edit. `inference.py:368` also skips visual checks for rejected candidates.                                                              | Preserve a clearly separate inspectable declined collection, with concrete reasons and an explicit human choice to edit. Provide source-span recovery when no nomination exists. Test rejected-only and empty-nomination results. Neither should falsely imply recommended clips exist. Visual-only uncertainty must reach visual review or remain explicitly uncertain.                                                                                                              |
| P1       | **Saved gain changes are not heard in draft playback.** `Editor.tsx:556` plays the proxy without a gain node. `editor/player.ts:217` holds gain between points, whereas `clipmill-render/src/graph.rs:374` ramps it. The existing preview document incorrectly claims WebAudio parity.                                                                                                                                     | Apply the actual gain interpolation and provide a rendered audition of the exact revision for mastered audio. A varying positive/negative gain curve must be audible, seek correctly and agree with the delivered automation. Label the faster proxy playback as a draft.                                                                                                                                                                                                             |
| P1       | **Caption appearance in the editor is not the export appearance.** `Editor.tsx:585` hardcodes size, position, shadow and UI accent, independently of the resolved export style in `clipmill-render/src/profile.rs:57`. Shared word timing does not prove visual parity.                                                                                                                                                    | Carry resolved style into the preview and inspect decoded output for both supported styles, at phone scale and different viewport sizes. Check text, wrapping, placement and timing. Where browser parity is insufficient, the exact rendered audition is the approval authority.                                                                                                                                                                                                     |
| P1       | **Automatic finishing is absent from the normal path.** The analysis DAG in `clipmilld/src/jobs.rs:1469` does not include face detection. The director defaults to fitted landscape without faces (`lib.rs:278`). Whole-span focus resolution rejects two equally present faces (`clipmill-reframe/src/tracks.rs:211`).                                                                                                    | Connect face evidence to the normal analysis and immutable manifest. Compose per shot, with stable one-person framing and a deliberate two-person layout. Decode a two-person interview and a shot/reverse-shot scene; no artificial pan may cross a cut. Do not claim active-speaker following from face confidence.                                                                                                                                                                 |
| P2       | **Export acknowledgments outlive what was acknowledged.** `ExportScreen.tsx:109` changes documents without clearing caption/rights acknowledgment, and stale-revision recovery at line 247 retains them. Line 68 always declares `own_content`, which cannot represent licensed third-party footage.                                                                                                                       | Bind acknowledgment to document, revision and finding identity; collect and persist the user's actual rights assertion. A clip switch or a new revision with different hot-caption findings must require a fresh relevant acknowledgment.                                                                                                                                                                                                                                             |
| P1       | **Routine delivery is still incomplete.** Export currently addresses one document; durable explicit batch selection, per-item recovery and channel publishing are not implemented. Models and Settings mostly report information rather than resolve common problems.                                                                                                                                                      | Bind each batch item to an approved immutable revision and track independent outcomes. Test cancellation, restart, stale revision, missing media and disk failure without losing edits or duplicating completed work. Models/settings actions must perform real bounded operations and show current readiness; avoid decorative toggles.                                                                                                                                              |

## Smallest acceptable finishing scope

1. Repair selection coverage and scripted-scene semantics; preserve inspectable
   declines and a manual recovery path. Later visual/audio work cannot repair
   an empty nomination set.
2. Connect shot-aware framing, two-person composition, shared caption styles
   and real audio behavior. Provide an exact-revision rendered audition.
3. Deliver an explicit selection as a recoverable batch, preserving source,
   document and revision identity through completion.
4. Add channel connection, editable source-grounded metadata, private upload
   and separate publishing. Treat interrupted or ambiguous remote completion
   as a reconciliation problem, not permission to upload a second video.
5. Refine Models and Settings around actual readiness, recoveries and persisted
   preferences. A polished status dashboard alone does not complete the flow.

These are functional checks tied to observed failure modes, not a new corpus
milestone or an invented requirement to complete the deferred benchmark. The
original human publishability and repair-effort targets remain unverified until
measured. Report that limitation rather than calling successful encoding proof
of editorial quality.

## Evidence boundaries

- The episode artifacts were inspected locally without copying private footage
  or transcript text into this repository.
- The real rerun answered all ten windows, nominated one moment and rejected it
  during semantic review. Finishing stages did not cause that empty result.
- Initial code inspection did not exercise the native window or a YouTube
  account. Provider protocol tests and a genuine authorized account run are
  different evidence and must be labeled separately.
- `apps/desktop/src/pipeline/stages.ts:78` says shots prevent clips straddling
  cuts. Editorial validation does not implement that rule, and multi-shot
  dialogue is legitimate. Explain shot-aware composition instead.

## YouTube integration: independent architecture recommendation

Keep durable publishing coordination in the daemon, using its existing SQLite
actor. A narrow transport implementation may use a new Rust HTTPS client or a
separate Python helper; neither choice requires a general connector platform.
Do not give a Python helper write access to the application's SQLite store, or
let the renderer send arbitrary URLs or filesystem paths to it.

The current worker path is designed for reproducible artifact production:
`clipmilld/src/worker.rs:344` can complete a task directly from a cache hit.
`checkpoint_support` is a capability flag, not an implemented durable upload
checkpoint protocol. Remote publication must therefore have an explicit
operation ledger, not merely another cached artifact recipe. If implemented as
a registered network worker, it needs daemon-acknowledged checkpoint transitions
and must never reuse another operation's remote receipt as a cache hit.

Suggested bounded state:

- **Connection:** opaque account ID, actual YouTube channel ID/name, granted
  scopes and credential reference. Tokens remain in Keychain; access tokens,
  authorization codes and refresh tokens never enter views, task payloads,
  logs or project archives.
- **Upload:** stable operation ID, project/document/revision, immutable rendered
  artifact and verified file digest/length, channel ID, approved metadata,
  private destination, state, acknowledged byte offset, protected session
  reference, remote video ID and safe failure/recovery details.
- **Publish:** separate explicit intent bound to the existing remote video ID,
  channel and approved visibility. Never invoke a second insert to publish an
  already uploaded video.

`clipmilld/src/db.rs` already owns the serial database actor, schema migrations
and request deduplication. Add the publishing ledger there. A stable upload
operation identity must survive new frontend request IDs, window closure and
daemon restart. Prevent concurrent claims on the same operation. Bind the
upload to the immutable snapshot/render already produced by
`service.rs::export_clip`, not a mutable export-folder filename. Keep its
artifact rooted while pending, paused or recoverable.

For authentication, use a system-browser authorization flow, random state,
PKCE and a bounded loopback callback. Bind the callback listener before opening
the browser; accept only its expected path/state and expire it on cancellation.
The existing Tauri bridge can expose a narrow Connect command without adding a
general renderer HTTP or shell permission. Google documents the desktop flow
and PKCE in its [installed-app OAuth guide](https://developers.google.com/identity/protocols/oauth2/native-app).

Treat every upload session URL as sensitive untrusted input. The existing cloud
adapter disables redirects, but there is no shared host-enforcement layer in
the scheduler: `NetworkAllowed` is task admission policy, not a hostname
allowlist. Restrict HTTPS scheme, exact approved hosts, port and paths in the
transport; validate the returned session URL before sending credentials or
video bytes. Do not accept userinfo, a lookalike suffix, or arbitrary redirects.
Keep session URLs out of project exports and human-facing errors.

Google's resumable protocol lets the client query the same session after a
lost response; completed sessions return their final response, while incomplete
sessions report the acknowledged byte range. Persist the session before sending
video and query server state after interruption. Do not trust only the local
byte counter. See the [resumable-upload protocol](https://developers.google.com/youtube/v3/guides/using_resumable_upload_protocol).

An expired session after an ambiguous final request is the dangerous case:
local request deduplication cannot prove that no remote video was created.
Reconcile against the known remote result if possible; otherwise show an
explicit uncertain-completion state and require recovery. Automatically
starting another insert would turn retry into duplicate publication. This is
an architectural conclusion, not a claim that the provider supports a remote
idempotency key.

Falsification tests should simulate: a wrong OAuth state; a hostile session
Location/redirect; a replaced rendered file; changed channel after reconnect;
crash after session creation; crash after an acknowledged chunk; lost final
success; expired session after ambiguous completion; duplicate frontend submit;
and an interrupted Publish update. Resume must use server acknowledgment,
successful operations must return the existing video, and ambiguous completion
must never silently initiate a replacement upload.

## Independent M3 implementation review

Second inspection: the in-progress M3 working tree, before the following
findings were repaired. YouTube work is now deferred until the core milestones
are finished, per the user's subsequent direction. The recommendations above
remain future architecture notes, not current completion requirements.

The new two-path IR uses default-empty fields, preserving readability of old
documents. Source-aware crop geometry, per-shot director composition, explicit
draft labeling, shared caption styling and one decoded frame for both portraits
are substantial improvements. The single-segment encoder drill demonstrates
two-up pixel routing and measured loudness; it does not exercise the new
multi-shot editing/timing seams.

| Priority | Concrete finding                                                                                                                                                                                                                                                                                                                                                                                                                           | Falsification check                                                                                                                                                                                                                                                                                             |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| P1       | **Per-shot rounding accumulates against the program clock.** `clipmill-render/src/preview.rs::segments` and `crops` sum `ceil(segment duration × fps)`, but `frame_count` and caption timing use the whole program. The encoder also independently normalizes each segment before audio/video concatenation.                                                                                                                               | Twenty 0.52-second shots from a 25 fps source produce 320 summed shot frames at 29.97 fps, versus 312 frames for the 10.4-second program. Use global quantization and verify the final shot, decoded source time, captions and audio across many cuts. A single aligned four-second segment cannot detect this. |
| P1       | **Re-solve targets the first shot regardless of playhead.** `screens/EditorScreen.tsx::onResolve` reads `plan.segments[0]`.                                                                                                                                                                                                                                                                                                                | Move to shot two and re-solve; only shot two may change. Hold the selected segment/document/revision across the asynchronous request. A switched document must not receive an old result.                                                                                                                       |
| P1       | **Re-solve can save an unrenderable crop.** `editor/commands.ts::solvedKeyframe` still floors width and then forces even dimensions, while the director now rounds to the nearest even width. A full-height 1080px 9:16 crop becomes 606×1080; its aspect error is 2880, above the renderer's 1920 tolerance. Re-solve also adds points without replacing the previous path, potentially retaining older points with different dimensions. | Re-solve the full-height crop, export successfully, and compare its geometry with the director. Re-solve after a manual nudge; no stale dimension-changing point may survive. Undo must restore the exact previous path.                                                                                        |
| P1       | **Clip trims now leave unwanted shots in place.** `editor/commands.ts::trimStartAt` and `trimEndAt` trim only the current segment. Once the director creates one segment per shot, trimming the start in shot two retains shot one, and trimming the end leaves later shots.                                                                                                                                                               | On a three-shot clip, trim into shot two from both ends. The resulting program must contain exactly the intended retained span, with both caption presentations, gain and both portrait paths correctly retimed; undo restores all three shots.                                                                 |
| P2       | **The preview names a font it does not load.** The app has no pinned Inter font asset or `@font-face`; `fontFamily: 'Inter'` can use host fallback while export uses its pinned font.                                                                                                                                                                                                                                                      | Exercise a machine/browser without system Inter. Verify the actual preview font is loaded from the same controlled asset, or retain explicit draft limitations and use the exact rendered audition for approval.                                                                                                |

Bounded arithmetic checks independently reproduced the 320/312 frame mismatch
and the 606×1080 aspect refusal. No heavy build or process restart was performed
by this review. Implementation ownership was left with the finishing agent and
root coordinator.

## Core repair review, 19 September 2026

This section supersedes the open/closed status implied by the historical
findings above. It reviews the current working tree, not a merged release.
YouTube remains explicitly deferred until core completion; it is not a blocker
for this pass. The critic edited only this document and did not restart the
application or daemon.

### Repairs inspected

| Area                              | Current disposition and evidence                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| --------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Editorial coverage and genre      | The duration-aware window overlap, persisted scripted/interview profile and profile-bound cache/review identity address the specific seam and rubric defects. A duration-valid span is still not evidence of a good clip.                                                                                                                                                                                                                                                                                                                                                                                       |
| Declines and manual recovery      | Declines now retain their span and review separately from recommendations. Explicit human editing does not relabel them as model recommendations. `clipmill-director/src/lib.rs::direct_span` refuses a boundary inside a word and supplies an outward, millisecond-safe boundary suggestion. Uncertainty alone does not prove an incomplete story; missing setup/payoff still can. The review tests include `declines_preserve_the_exact_span_and_review_for_human_inspection`, `uncertainty_alone_is_not_a_semantic_rejection` and `a_visual_answer_attaches_to_a_decline_without_overriding_missing_payoff`. |
| M3 shot timing                    | Global program-frame allocation replaces the per-shot rounding error. Audio and video concatenate separately. The finishing agent's decoded twenty-shot fixture now reports 312 frames, preserved source tail frame 259 and the audio pulse at 9.9000 seconds; the critic inspected the implementation and regression, but did not independently rerun that encoder gate. Continuous-source camera cuts no longer count as clipped-word boundaries.                                                                                                                                                             |
| M3 editing and composition        | Re-solve targets the active segment, uses renderable nearest-even geometry and replaces the whole crop path atomically. Whole-program head/tail trims preserve the intended retained interval. Undo restores both portrait paths. One-face and two-person framing remain shot-local; no active-speaker claim is inferred from face confidence.                                                                                                                                                                                                                                                                  |
| Draft versus final playback       | Shared caption style, a bundled Inter font and linear-dB WebAudio automation replace the earlier false-parity claims. Draft playback remains labeled as such. The delivered encoded revision is the final audition authority. Native playback and audible behavior are a separate check below.                                                                                                                                                                                                                                                                                                                  |
| Legacy framing and silence        | `Layout::needs_crop_repair`, preview warnings and export `framing.missing_crop` findings stop a missing saved crop from looking export-ready. Reframe's Fit action writes the repair. `RenderPlan::encode_graph` preserves measured digital silence and avoids non-finite loudnorm hints for very short audible material. Decoded silent and 200 ms audible regressions exist; their successful runs were reported by the finishing agent.                                                                                                                                                                      |
| Export approval                   | Rights are an explicit own/licensed/public-domain choice. Caption and duration acknowledgments bind to document, reviewed revision and relevant findings. Stale submit completions are guarded by selection generation. `useDelivery` binds its job and delivered files to the queued job ID, preventing an old job's files being attributed to a replacement export.                                                                                                                                                                                                                                           |
| Durable batch admission           | All selected requests and expected revisions are persisted atomically. Stable item/attempt request IDs close the job-accepted-before-item-linked crash gap. Pending intents resume after daemon restart. First admission revalidates the expected revision; retry of an already admitted export uses its existing immutable snapshot, not the subsequently edited document. An independently failed item does not cancel its peers.                                                                                                                                                                             |
| Batch RPC replay and cancellation | Outer retry/cancel request hashes now enter the same transactional deduplication store as the state transition. Replaying a lost response returns that receipt without incrementing the attempt again. Hash mismatch is a conflict. Cancellation checks the job state returned by the cancellation operation, refuses a completed export and reports errors under the caller's outer request ID. Late item links are conditional on attempt and pending state.                                                                                                                                                  |
| Batch UI recovery                 | The final review found that retry returned a pending item but did not restart collection polling. This was repaired. The regression retains the previous failed receipt, verifies Preparing without fetching that old job, then observes the new queued job and real frame progress. Pending items no longer display the previous attempt's failure as their current state.                                                                                                                                                                                                                                     |

The batch architecture is sufficient for the requested durable selected
collection. It reuses existing render/delivery jobs rather than creating a
second render scheduler. The important constraints are atomic intent storage,
strict first-admission revision checks, immutable retry snapshots, durable
request replay and independent item outcomes; these are present in the code
reviewed here.

The final review also caught a brief stale-audition label race: a previously
resolved video URL could survive one render after the queued revision changed.
`ExportScreen` now stores project/artifact identity beside that URL and exposes
it only when both match the current rendered artifact. This repair was inspected
after the frontend suite run reported below; it is not included in that test
result.

The new `clipmilld/tests/export_recovery.rs` exercises export replay after a
revision change/restart, batch retry retaining the approved snapshot, replayed
old cancellation not cancelling a new attempt, and recovery before admission
and after job acceptance before item linking. Its crash boundaries are explicit
stopped-store injections, not a claim to have randomly killed a live encoder.
The database regression
`batch_intents_are_atomic_and_replayed_updates_cannot_create_another_attempt`
also checks rollback, request-hash conflict and a late link after cancellation.
This critic inspected those tests; their Rust execution is the coordinator's
verification responsibility.

### Real face-stage failures uncovered by the live run

The newly connected face stage initially failed before receiving a usable
lease: its planned implementation identifier did not resolve in the registry.
Existing capability-presence tests could not catch a wrong concrete planner
identifier. The exact-planned-implementation regression now walks modeled
analysis tasks through `recipes::model_for`.

The next run reached inference and then failed decoding a later JPEG batch.
The last progress heartbeat was 120 frames; it did not identify the failing
batch. Independent local reproduction found the failure in frames 360–479:
FFmpeg image2pipe autodetection exited 234 with no stream, despite valid JPEGs.
The first two batches decoded successfully. A private-footage-free 120-frame
solid-color JPEG fixture reproduced the same failure. Explicit input
`-c:v mjpeg` fixed both cases; changing output frame pacing alone did not.

The critic independently decoded all 16,078 real sampled frames across 134
batches one-for-one in 18.49 seconds with the explicit JPEG codec and output
passthrough pacing. This was a decoder check, not face inference or an
editorial-quality check. No images or transcript text were copied into the
repository. The worker now declares the input codec and is version 0.1.1 so
the changed implementation cannot reuse a prior failed identity. Its owner
reports 17 frame tests, including solid-color batches and a 243-frame mixed
fixture, passing. SDK failures now use a fixed allowlisted diagnostic catalog;
unknown codes preserve class-only reporting rather than exposing exception
text, paths or decoder stderr.

### Independent checks and remaining evidence

- Earlier in this review, the editorial inference unit file passed 32 tests
  using the existing worker virtual environment. That checks prompt/reply
  behavior, not whether Qwen finds publishable moments in the episode.
- The final frontend invocation ran the complete desktop suite: 37 files and
  395 tests passed in 9.10 seconds. It included batch preparation, retry
  polling, export selection/revision handling and legacy framing recovery.
  React fakes do not establish native media playback.
- **Open native check:** exercise actual Tauri/WKWebView proxy playback,
  seeking, gain changes and exact encoded audition through the app's media
  route. Confirm that the rendered file shown and its revision label refer to
  the same artifact, including immediately after queuing a replacement export.
- **Live editorial check:** the subsequent scripted-profile run completed;
  its result and remaining caveats are recorded below. Inspecting and editing
  those actual moments is still distinct from successful pipeline completion.
- **Not established:** higher human acceptance, broad scripted-scene quality
  or upload-readiness of every automatic recommendation. The user deferred
  the benchmark milestone; neither synthetic media nor successful encoding
  should be substituted for that evidence.

No additional unresolved durable-batch blocker was found in this bounded final
pass after the retry-polling repair. Core completion still requires the live
and native checks above plus the coordinator's normal repository checks.

### Continued gate verification: new findings

The critic subsequently ran `just gate-milestone-1` with
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_INCREMENTAL=0`, `CARGO_NET_OFFLINE=true`,
`UV_OFFLINE=true` and `CARGO_BUILD_JOBS=2`. The initial sandboxed attempt could
not create its private Unix socket (`EPERM`), before any analysis. The approved
rerun used its own temporary daemon, database, socket and worker fleet, without
restarting the live application.

**P1, unresolved at this inspection:** the real gate failed after 165.60 seconds
at `apps/desktop/src-tauri/tests/milestone_1.rs:658`. Approval reported end tick
54,002,949 instead of the selected end tick 55,580,940.
`clipmilld/src/db/edit_store.rs::direct_response` takes both bounds from the
first segment. The saved document correctly contains two contiguous shots,
53,662,140–54,002,949 and 54,002,949–55,580,940. This is a real response-contract
regression exposed by multi-shot direction, not permission to remove the gate's
assertion. Approval and reopening must report the whole clip's source envelope.
The gate also contains subsequent single-segment crop/trim/manifest assumptions;
adapt those to the actual shot program while retaining its correction, undo,
restart, immutable revision, captions and decoded source-timing checks. Do not
disable shot detection or collapse the new program merely to make the old test
pass.

After the audition identity repair, the critic reran the export, ExportScreen
and batch frontend files: 36 tests in three files passed in 9.53 seconds. This
is additional unit evidence, not native playback proof. The coordinator reports
that native batch delivery wrote seven files and draft playback works, but the
exact rendered audition currently fails while fetching its media manifest;
that native failure remains a release check to resolve.

The live scripted-profile job `job_01M2V6088F8JXSCTZRQH6VXCJ5` completed with
all ten windows answered, no failed reviews or visual checks, two selected
moments marked **needs_review**, and three retained declines. Both selected
moments carry transcript/timing and visual uncertainty. One starts at the
recording's beginning and the visual answer describes multiple settings;
whether it is a coherent local scene or introductory montage needs human
inspection. Counts alone do not establish publishability or improvement over
the prior route.

**P2, unresolved at this inspection:** `prompts/look.v1.txt` still instructs
the VLM to assess a podcast/interview clip despite receiving
`content_profile=scripted`. One actual answer repeats that irrelevant genre
requirement. Make the visual prompt profile-aware or neutral, invalidate its
prompt/cache identity and keep the question limited to the stated visible
referent. Four static samples cannot establish an unseen dynamic reaction or
prove that particular dialogue was spoken; uncertainty in those cases is
appropriate. This finding does not justify automatically accepting the two
selected moments or removing the three semantic declines.

### Multi-shot repair rerun and caption regression

The first/last segment response repair and strengthened database approval/reopen
test were inspected. The revised native gate preserves every detected shot,
checks source/frame contiguity and every rendered segment, and uses the same
whole-program ripple head/tail operations as the editor. Its decoded source-time
tolerance was not relaxed. The coordinator reports the database regression
passing.

The critic's next real gate run progressed past multi-shot approval, correction,
both trims, undo/redo and restart, then failed export preflight after **152.51
seconds**. Store and daemon/worker logs: `/private/tmp/cm-m1-11165`. Saved revision
13 contains a first reading cue with one retained word, three characters, and a
0–8,700 tick window (0.0967 seconds). Export correctly rejects both its 31.03
characters/second reading rate and its insufficient duration.

**P1, open:** `clipmill-edit-ir/src/command.rs::apply_ripple_delete` calls
`splice_program_content` without the edge-fragment reflow already present in
the trim command. Moving the editor's head/tail controls to whole-program
ripple deletion therefore regressed readable caption repair. Reflow an actually
shortened edge fragment in both presentations using the existing caption logic;
preserve untouched user grouping and all retained speech. Keep the gate's word,
caption, undo and export assertions rather than granting a rate override or
dropping the fragment.

The render inventory repair was also inspected: `shell.rs` now filters render
outputs through the existing MIME allowlist while preserving project ownership,
descriptor integrity, safe artifact paths and published-manifest membership.
The new real-socket test includes internal ASS output, denies another project's
request and refuses an advertised but unpublished MP4. It uses fixture bytes
and does not claim to test an encoder or native player. The visual prompt now
honors interview/scripted context and the limits of static evidence; its
embedded prompt digest changes the look task's cache identity when the daemon
is rebuilt. These two code repairs still require their applicable runtime
checks; the earlier live ranking was produced before this prompt repair.

At the earlier inspection, the native CORS/audio boundary was a verification gap, not an observed
failure: the new origin allowlist is appropriately narrow, but the current
audio test mocks `AudioContext`, and the media test checks the origin predicate.
A real macOS WebView must play the custom-scheme proxy with
`crossOrigin="anonymous"`, show both portraits, audibly apply gain and seek
without losing audio. Playing a public MP4 in the development browser does not
exercise that path.

## Latest verified closures

The critic inspected the edge-ripple repair: it shares the existing trim
reflow, applies to an actually shortened surviving cue at a program edge, and
handles both presentations. Its regression preserves retained word text and
identities, unrelated user grouping, and exact undo/redo.

The critic independently reran **`just gate-milestone-1`: passed, one real
scenario, 145.68 seconds**. It used the same offline/debug/incremental settings
recorded above and an isolated daemon/fleet. The two-shot source selection at
596–617 seconds was corrected in both caption presentations, trimmed to
599.43–613.79 seconds, undone/redone and restored after restart. Revision 13
exported successfully with the selected immutable IR and caption outputs.
Decoded samples mapped clip times 0.50, 7.18 and 13.86 seconds to source times
599.91, 606.61 and 613.30 seconds within the unchanged tolerance. The saved
program and rendered manifest retain every kept shot.

Gate evidence is in `/private/tmp/cm-m1-13783/daemon.log`,
`/private/tmp/cm-m1-13783/workers.log` and its isolated `state/clipmill.db`.
The synthetic delivered video is
`target/milestone-1/delivered/01-older-clip.mp4`. This closes the multi-shot
response and edge-caption blockers found by the earlier real runs. The gate
uses the heuristic analysis route to exercise the edit/export workflow; it
does not replace the separate live Qwen check or native playback check.

A subsequent compatibility review made reflow explicitly opt-in:
`RippleDelete.reflow_edges` defaults to false and serializes only when true.
Old commands retain both their original JSON shape and old grouping behavior;
new editor head/tail commands and both M1 gate operations explicitly set true.
The critic inspected that unchanged opt-in runtime path and the missing/false
legacy head/tail regressions. The owner reports 36 Edit IR tests, clippy, 22
editor command/screen tests and full UI typecheck passing. The full gate was
not redundantly rerun after this compatibility-only guard.

The coordinator separately reports actual rebuilt-app verification: a completed
seven-file collection restored after daemon/application restart; the exact
rendered revision loaded, played from zero through 14 seconds and sought back
to eight seconds with burned captions visible. Its synthetic QA export decoded
477 frames at 1080×1920, H.264/AAC, 15.9 seconds; all six checksum entries matched,
and SRT/VTT files were present. This was synthetic QA material, not a rights
assertion or editorial approval for the user's television footage. The critic
inspected the media-allowlist repair and its socket test but did not operate
that native window or independently measure audible WebAudio gain.

The updated-look live job `job_01M2V71PZN11G9VPFDSM4X2689` completed. The critic
independently reread its ranking: scripted profile, ten of ten windows answered,
no failed reviews/visual checks, two selected **needs_review** moments and three
preserved declines. Neither selected moment's visual reason retains the
erroneous interview-format requirement. This closes that prompt mismatch; it
does not upgrade either moment to human-approved or upload-ready.

A further native Results defect was fixed: the filmstrip timing descriptor was
not on the document allowlist, so thumbnail selection could not read it. The
repair adds only the project-owned `media.filmstrip.v1` `index.json`; image bytes
remain on the media route. The loader checks its source fingerprint and schema.
The critic inspected the bounded change. The owner reports two real-socket
tests passing in 0.33 seconds, including cross-project refusal, plus seven shell
policy tests, 39 Results tests, typecheck and formatting.

**Performance repair inspected:** the 2,680-tile real filmstrip previously took
10.502 seconds to resolve even warm because each tile's `declared_bytes` lookup
reparsed the entire manifest. The implementation now builds one validated size
map per request. The critic checked that ownership, type, safe-path, membership
and verified-read behavior remain in place; manifest validation already refuses
duplicate paths, so collecting the map does not introduce ambiguous membership.
No global cache was added. The owner reports two declared-inventory tests and
three real-socket media tests passing, including every one of 2,680 tile
names/sizes/types and unsafe-path, missing-file and wrong-project refusals;
clippy, formatting and diff checks pass. The coordinator measured the same
2,680-tile inventory on the rebuilt daemon at **0.0535 and 0.0534 seconds warm**,
versus 10.502 seconds previously (about 196 times faster). Native Results now
visibly renders the selected moment's thumbnail. This closes the repeated-parse
performance finding.

The first request immediately after daemon restart took **36.4533 seconds**.
That is a separate observed startup sample, not a demonstrated cold-start
improvement. This review has not isolated its cause and does not attribute it
to hashing, model startup, storage or the inventory loop without evidence.
Warm performance must not be presented as the latency of every first load.

The critic also inspected the coordinator's final broad test logs, recorded
before this last inventory-only optimization: 72 Rust test-result groups,
787 passed, zero failed, 35 ignored; 37 desktop files, 400 tests passed in
14.53 seconds. Those logs include the explicit reflow opt-in and filmstrip
allowlist changes. The ignored real M1 scenario was separately executed and
passed as documented above; ignored tests were not silently counted as passes.

Human acceptance and automatic editorial quality remain unmeasured. YouTube
integration remains explicitly deferred; neither is established by synthetic
encoding success or the two review-required recommendations.

Final bounded-review disposition: the identified multi-shot response, caption
reflow, legacy replay, native render inventory, visual-profile prompt,
thumbnail authorization and repeated-parse defects have been repaired and
received the checks documented above. No additional concrete release blocker
was found in this core pass. The first-load timing sample and unmeasured human
editorial quality remain explicit limitations. Commit, PR, CI and merge are
the coordinator's remaining delivery steps.

## Follow-up release checks

The first cross-platform CI pass found a stale evidence-index harness: it did
not start the face worker now required by analysis. The repair runs both real
visual workers with verified pinned YuNet weights and adds frame-count and
model-provenance assertions. Existing digest, warm-cache and recovery checks
remain. All three real analysis tests and the full ranking drill passed locally.
The finishing agent independently reviewed this repair and found no blocker.

Further tracing established that the previously recorded startup delay was a
release blocker: cleanup occupied the artifact actor for 55.097 seconds, exceeding
the native bridge's ten-second timeout. Cleanup now yields during verification
when foreground work arrives and retries with fresh roots and pins. The CI-repair
agent independently reviewed the change and found no blocker. Full digest checks,
no sweep after incomplete verification, retention, pins and safe filesystem
deletion remain intact; individual filesystem operations remain uninterruptible.
The live first request improved from 55.1798 to 0.1747 seconds, and idle cleanup
subsequently completed. Artifact and maintenance regressions and strict clippy
passed. These follow-up fixes still require the final CI run before merge.
