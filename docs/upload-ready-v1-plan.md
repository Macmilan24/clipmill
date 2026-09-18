**ClipMill: plan for an upload-ready first version**

Proposed 17 September 2026, following the [architecture review](/Users/sami/Personal/YT/clipmill/docs/architecture-review-2026-09-17.md). This is the working implementation proposal; its acceptance targets are goals, not measured claims about the current build.

The next product should do one job well: turn a dialogue recording into a small set of complete, well-framed, accurately captioned clips that a creator can approve and upload. Keep the existing stack and repair it incrementally. Every milestone includes the backend, its UI path, and a watchable result.

**Confirmed scope**

The user selected English podcasts and interviews, including two speakers, and optional cloud AI only when explicitly enabled. Use the current Mac as the initial reference platform. Start development with single-speaker footage; the completed version must also handle the declared two-speaker formats. Complex screen demonstrations, gameplay, sports, compilations, translation, and generative B-roll remain later work.

Use local files first, including recordings obtained from YouTube. YouTube link import is a separate acquisition adapter, added after the core loop works; ordinary clipping must not depend on it. Keep media preparation, precise timing, editing, and rendering local. Route editorial reasoning locally or to an explicitly enabled cloud provider through the same candidate/judgment interface. Do not choose an inference model solely from a leaderboard: compare a small shortlist on the actual reference machine and editorial examples before pinning one. This planning choice does not itself authorize sending footage or transcripts to a provider.

Default to one continuous source interval per clip, with a requested duration range of roughly 20–90 seconds and up to five suggestions. Those are editable product defaults. A complete moment can need a different length; never pad the result count with weak clips. Reordering speech and removing words from the audio are outside this first automatic editing mode.

**What “upload-ready” must mean**

The clip makes sense without watching the source, has a reason to keep watching, reaches its payoff, and preserves what the speaker meant. Its opening and ending sound intentional. Essential names, numbers, and negations are transcribed correctly. The subject is comfortably framed; captions are legible and do not cover important faces or content; audio is intelligible and consistent. The saved file represents the exact document revision the user approved.

The normal path is watch, approve, export. Quick correction is available when necessary, and the app explicitly distinguishes a proposed finished clip from one that needs attention. Manual repair must be measured: relying on the user to fix every result would miss the product goal. A successful encoder exit alone cannot establish editorial readiness.

Use a 1080×1920 SDR MP4 delivery preset with H.264 video, AAC-LC audio at 48 kHz, and optional SRT/VTT files. These codec choices follow [YouTube's upload guidance](https://support.google.com/youtube/answer/1722171?hl=en). Preserve supported source frame rates where practical, explicitly normalize VFR with correct time mapping, and test the supported rate matrix. Current Shorts guidance allows square or vertical uploads up to three minutes; the shorter default above is a product choice. See [YouTube Shorts uploads](https://support.google.com/youtube/answer/12779649?hl=en).

**Reuse, repair, and new work**

| Area                                                                      | Decision                                                                                                                         |
| ------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| Rust daemon, SQLite, artifact cache, scheduler, worker SDK                | Keep; extend only for the capabilities this version uses                                                                         |
| Ingest, proxy generation, ASR, alignment, shot detection                  | Reuse; verify the speech fleet actually runs through the app                                                                     |
| Tauri/React screens and generated contracts                               | Keep the screens; repair identity, loading, mutations, navigation, and progress end to end                                       |
| Edit IR, render compiler, caption engine, export package                  | Keep; repair shared caption edits, timeline mapping, layout coordinates, and revision binding                                    |
| Face detector and crop solver                                             | Connect and validate; add a two-person composition if interviews are in scope                                                    |
| Existing heuristic discovery/ranking                                      | Retain as a benchmark and optional supplemental proposals; replace its role as the primary editorial judge                       |
| Editorial inference interface                                             | New: local and explicitly enabled cloud implementations for contextual proposals, candidate verification, reasons, and rejection |
| VLM checks                                                                | Small targeted addition for visual dependencies and ambiguous layouts; it does not replace precise detection/tracking            |
| Evaluation harness                                                        | Reuse; add human usefulness/repair labels, end-to-end UI checks, and a real encoding benchmark                                   |
| Large evidence graph, many judges/proposers, new runtime/platform breadth | Defer until a measured failure justifies them; do not delete working infrastructure merely to simplify the diagram               |

**Milestone 0 — Deferred by the user on 18 September 2026**

The user explicitly chose Qwen3.5 and asked to skip M0. The following corpus proposal is retained as future quality work, not a prerequisite for M2. Functional gates still run; no measured editorial superiority is claimed.

Original proposal:

Assemble roughly 20 representative recordings, split by recording into development and untouched evaluation sets. Cover clean and difficult speech, proper names, quiet delivery, interruptions, nonzero clip starts, long answers, camera cuts, one/two-speaker layouts, and a few recordings with no useful moments. Choose acceptable moments and alternate boundaries with a human editor. Start with a smaller subset immediately rather than waiting for all annotations.

Run the current system on that subset, keep its outputs, and record failures. Capture time to first useful clip, cold/warm analysis time, peak memory, and actual export time. Repair the current throughput script so its timer surrounds a fresh render and reads that exact run's output. Report render time separately from whole-pipeline time.

**Exit:** several baseline clips can be watched next to their source and annotations; measurements identify the actual run and reference hardware. Annotation work continues during the other milestones.

**Milestone 1 — One selected clip survives the complete UI workflow**

Fix the integration defects before relying on more model output. A user should be able to open a specific clip, or select a source span, edit it, save it, and export it correctly.

- Carry the selected project, source, analysis run, candidate, and edit document through Results, Inspector, Editor, and Export. Use the analysis manifest and its referenced artifacts as one coherent snapshot. Remove “latest project/document/stage” as an implicit substitute for the user's selection.
- Make approval/document creation retry-safe. A failed creation must not leave the UI claiming that an editable clip exists; repeated approval should reopen the existing edit unless the user explicitly makes a variation.
- Extend the preview contract with source intervals, the matching proxy reference, source display dimensions, and source/program/proxy time mappings. Use them for playback, scrubbing, crop transforms, and trim commands. Bind the visible preview to the document revision and discard stale asynchronous responses.
- Make word corrections share stable word identities across burned-in captions and sidecars. Cue grouping belongs to a presentation; corrected text belongs to the words. Fix head/tail trimming and retime both presentations, crop paths, and gain automation consistently. Provide an explicit migration for saved documents affected by schema changes.
- Resolve and queue exports from the selected immutable edit snapshot. Make the approved revision explicit; revalidate if it changed between planning and submission. Show actual render/delivery progress, actionable failures, completed file paths, and Open/Reveal actions.
- Validate model availability and worker readiness before analysis. A missing model or worker must produce a useful status and recovery action rather than an indefinite spinner. Reopening the app restores the selected project/clip and the daemon's real job state.

Main locations: `apps/desktop/src/shell`, `screens`, `editor`, `results`, `daemon`; `contracts/proto/clipmill/ipc/v1/daemon.proto`; `crates/clipmilld/src/service.rs`, `inspector.rs`, and document storage; `crates/clipmill-edit-ir`; `crates/clipmill-render`. Regenerate shared types instead of independently patching three language bindings.

**Exit:** from an older project while a newer project also exists, choose a clip starting several minutes into the source; play and scrub it; correct a name; trim both ends; undo/redo; restart; export; decode the result and verify the intended footage, timing, both caption outputs, and saved revision. Exercise the real UI bridge and daemon, not just mocked component props. Old documents remain readable or migrate without losing edits.

**Milestone 2 — The system finds complete, worthwhile moments**

Add one editorial interface behind the current task/worker layer, with a local implementation and a narrow cloud adapter. One selected model per run can serve two bounded operations: proposal and review. Keep the logic modular without requiring a fleet of different judges.

The analysis setup offers Local or Cloud-assisted, with local as the default. Enabling the cloud route names the provider, data scope, and an estimated cost with a hard run budget. Begin with transcript-only cloud reasoning; sending frames or short audiovisual samples requires that scope to be explicitly enabled too. Credentials stay in the backend/OS credential store, never in renderer state, logs, or project exports. Every outbound request and retry checks the run's permission and cancellation state. Unavailable cloud service offers a clear retry or local fallback choice. Implement this small route within the existing network-policy/task mechanism; do not build a general connector platform first.

1. Build overlapping transcript windows that preserve sentence boundaries, surrounding context, and stable source word IDs. Maintain coverage across the recording. A small episode outline may help long recordings, but it must cite source ranges and cannot replace original context.
2. Ask for complete moments with a hook, necessary setup, payoff, and a concise source-grounded explanation. Return sentence/word IDs, optional title, and reasons for uncertainty; timestamps come from alignment. Permit no proposal. Long answers must be splittable into meaningful subspans rather than automatically discarded by duration constraints.
3. Validate references and duration; remove duplicated proposals; refine endpoints against word/sentence timings and nearby silence. Keep the current boundary arithmetic as a precision tool, while allowing semantic review to revise the proposed span.
4. Review candidates with adjacent context: unresolved references, missing questions, incomplete payoffs, ad reads, irrelevant introductions, misleading omissions, and visual dependencies. Run local visual checks when the interpretation depends on what is shown. A malformed model response or model failure is distinct from “no suitable moment.”
5. Rank accepted candidates for usefulness and diversity. Publish fewer when appropriate. Replace an uncalibrated “99” with a clear reason and actionable review status; any numeric quality rating must be clearly characterized and validated before it drives a readiness claim.

Cache using the source/transcript, provider/model, prompt/rubric, and decoding configuration. Record the route and model provenance. Store the actual model output locally for diagnosis, excluding credentials. Validate structure deterministically; evaluate semantic claims separately. Do not require cross-device byte-identical inference as proof of quality. Compare local and cloud-assisted results on the same examples when the relevant data use is enabled; keep their quality and runtime reports separate. A route that misses the readiness target remains labeled as needing review rather than inheriting the other route's result.

Main locations: a new editorial worker family, narrow cloud adapter, model registry entry, candidate/judgment contracts, and the existing recipe/planning/analysis pipeline; update analysis setup and Results so controls, explanations, and actions represent the new output. Preserve accepted edits when reanalysis proposes a new result set.

**Functional verification:** the selected Qwen3.5 pipeline produces reviewable, exportable clips through the existing workflow, demonstrated by a real local model/daemon/worker/export run. This is implementation evidence, not the original quality exit. Incorrect references and unsupported output are rejected, incomplete analysis is visible, and a fully assessed recording can produce zero suggestions.

**Quality exit (unverified):** human keep/reject decisions and repair effort on development footage show whether the editorial route improves over the heuristic baseline. The dedicated M0 corpus and model-comparison project are deferred at the user's request; no measured acceptance gain is currently claimed.

**Milestone 3 — Finish the clips to a consistent visual and audio standard**

Connect the existing face detector and crop solver to the normal product path. Full-source lightweight detection or candidate-scoped detection can be chosen using measured runtime; either route must preserve source timestamps and artifact reachability. Detect camera cuts and avoid interpolating a camera move across them.

For one speaker, produce a stable crop with sensible headroom and controlled movement. For two visible speakers, add a deliberate two-person composition where it works. This is new layout/IR/render work: the current `Fit` and `SpeakerFill` modes do not implement two-up. Speaker-follow switching needs a measured active-speaker signal and stable association to face tracks; face confidence alone is insufficient. A VLM can help classify a layout, but should not be the frame-by-frame camera controller. Low-confidence shots get a deliberate composition or a visible correction request, and do not count as automatically finished if the result is unattractive.

Ship one polished default caption style and one quieter alternative, shared by preview and export. Handle punctuation, emphasis, line breaks, safe placement, and per-word corrections. Avoid face overlap with simple stable placement rules; add OCR only if the supported footage needs protected on-screen text. Do not invent speech to make a caption read better.

Use the existing measured loudness pipeline, preserve sensible stereo/mono behavior, and verify gain automation and true-peak handling in the actual export. Consistent levels matter; automatic denoising or aggressive silence/filler removal should only be enabled after listening tests show an improvement. Preview must either use the same mastered audio or clearly provide a final rendered audition before approval.

**Exit:** a small set of representative clips looks intentional on a phone-sized display, sounds clean, and preserves caption content. Compare sampled browser preview frames with decoded output for subject, crop, text, line breaks, and timing. A quick rendered preview of the exact edit snapshot can be the approval authority where browser rendering cannot meet parity; label any faster draft preview accordingly.

**Milestone 4 — Make the complete product routine and prove readiness**

Finish the flow: Import → Analyze → Results → Review/Edit → Export. Show the selected clip throughout, preserve edits across navigation/reanalysis, and expose only settings that change actual behavior. Support opening any accepted clip and exporting an explicit selection as a batch, with per-clip progress and failure recovery. Missing media, full disk, worker failure, stale revisions, cancellation, and restart must leave recoverable state and a clear next action.

Run the full desktop-to-worker-to-encoder path on the held-out recordings. Test representative 25/29.97/30/60 fps sources and VFR/rotation cases claimed as supported. Decode finished files and inspect actual output; do not accept plan-to-plan agreement as the sole check. Test cold start, warm cache, and interruption without overwriting user work.

Proposed acceptance targets, to freeze before evaluating the held-out set:

| Measure                 | Initial product target                                                                                                                                               |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Automatic finishing     | At least 80% of the offered top-three clips are publishable after watching/approval, with no repair                                                                  |
| Small repairs           | At least 90% of the offered top-three clips need no more than 60 seconds of active editing, excluding watch time                                                     |
| Moment coverage         | Recall@5 of at least 70% against annotated useful moments; report per recording and speaker/layout category                                                          |
| Avoiding cherry-picking | On eligible recordings with at least three annotated strong moments, offer at least three acceptable distinct clips; count empty/failed analyses as failures         |
| No-value recordings     | The held-out negative examples are allowed to return none and must not be padded with weak suggestions                                                               |
| Editorial integrity     | No observed meaning-changing omission, invented speech, or critical name/number/negation error among clips labeled ready                                             |
| Delivery correctness    | Every exported evaluation clip uses the selected source/span/revision, decodes, and has synchronized captions/audio with no black/frozen tail introduced by the edit |
| Performance             | Measure actual cold/warm and export times on the chosen machine; choose the user-facing latency budget from the first real measurements                              |

These are ambitious initial targets, not statistical guarantees from a small sample. Report counts and failure cases with the percentages. Human review determines publishability; model self-scores cannot pass this gate. If targets fail, fix the observed failure category before adding more genres or calling the product ready.

**Implementation batches**

Work in reviewable changes, normally in this order: explicit clip/run routing and atomic/retry-safe opening; preview source mapping; canonical caption corrections and migrations; trim retiming plus end-to-end export revision checks; editorial worker and discovery integration; semantic review and selection; face integration and tested layouts; final caption/audio presentation and rendered approval preview; batch delivery and readiness evaluation. Tests that protect a feature land with that feature.

Milestone 0 is deferred at the user’s request. Start testing semantic proposals as soon as the repaired single-clip route produces a watchable file; do not wait for the entire editor to be polished. Prioritize the failure that prevents a better clip, rather than completing every infrastructure subsection in the old book. Set time estimates after the first repair batch and model benchmark reveal the real integration and performance costs.

**First implementation batch:** preserve the exact selected project/source/analysis/candidate/document from Results through Editor and Export, reopen the existing document on repeat approval, and add the older-project/multiple-clip regression scenario. Its deliverable is a working route to the intended clip; subsequent batches repair what the clip displays and renders.
