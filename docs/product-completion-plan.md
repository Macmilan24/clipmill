# Upload-ready product completion

This extends `upload-ready-v1-plan.md` using the user's confirmed scope: English interviews/podcasts **and scripted TV/movie scenes**, Qwen3.5 as the local editorial model, and optional cloud reasoning only when explicitly enabled. The user deferred M0 and asked us to prioritize implementation; human acceptance targets remain unmeasured.

Each verified milestone receives a commit and push and an opened or updated PR. This is a development checkpoint cadence, not a recurring automation. Required CI and independent criticism are release gates.

## Editorial completion

- Persist the chosen content profile in analysis requests and provenance, and include it in prompt/cache identity. Evaluate a self-contained dramatic beat for scripted scenes rather than requiring the wider plot to resolve.
- Make every duration-valid sentence span eligible across window seams. Preserve bounded model input and deduplicate repeated nominations.
- Keep declined nominations inspectable separately from recommended clips, with reasons and explicit recovery. Never inflate the recommended list by relabeling rejection.
- Exercise a real scripted recording and distinguish empty editorial judgment from inference/coverage failure.

### Implemented behavior and evidence

The working tree now carries `interview` or `scripted` from the import controls through analysis requests, editorial prompts, artifact provenance and result presentation. Older documents and tasks default to `interview`. The scripted rubric asks for a complete local dramatic beat, such as a reveal and reaction, confrontation or joke; it does not demand resolution of the wider plot. Evidenced missing openings, endings and misleading omissions remain rejection reasons. Transcript or visual uncertainty alone is presented as needing review, and visual-dependent declines can still receive a visual check without erasing their original semantic review.

Editorial windows retain their nominal word budget and extend through the possible endpoints of a maximum-duration moment starting in that core. The word target is not a hard maximum after this extension. Exhaustive tests cover dense and sparse speech and a 70-second seam case. A read-only arithmetic check over the existing Reacher transcript index found all **13,350** duration-valid 20–90 second sentence spans eligible, compared with **1,762** previously excluded spans. The model-window count stayed at **10**, with **679** words in the largest window. The reproducible check is `cargo run -p clipmill-editorial --example window_coverage -- <index.json>`; it performs no model inference and stores no source text in this repository.

Actual semantic declines retain their title, span and reasons in a separate `declined` collection. They never enter the recommended list or bulk approval. Results and the Inspector expose an explicit individual “Edit despite review” choice while retaining the model's original assessment. If the model nominates nothing, Results offers manual source-interval recovery instead of empty filters and a dead detail panel. The manual drawer accepts source timecodes, validates them against a matching source map, previews the existing proxy when available, and opens the document identity returned by the daemon for the named run. It does not label a human selection as a recommendation or model-approved clip.

Verification completed for this implementation: **44 Rust editorial tests**, **44 editorial worker tests**, desktop type checking, and focused frontend regressions for profile submission, declined-only/mixed results, explicit override, manual time bounds, named-run identity and stale asynchronous responses. Browser interaction with synthetic fixtures checked dark and light presentation, declined inspection, empty-result recovery and manual interval validation at ordinary and compact desktop sizes, including 960×720. Browser checks do not exercise the native media protocol or the real daemon.

A real local Reacher run completed on 19 September: **10/10 windows answered**, **five nominations**, **two selected moments needing review**, **three retained semantic declines**, and no failed review or visual-check calls. This was the original 15–60 second, six-requested-clips configuration with the explicit scripted profile. All 16,078 face-analysis frames completed after the live decoder issue was repaired. The independent critic found the visual prompt still named interviews; that prompt was corrected and its digest invalidates the visual-check recipe. The warm-cache rerun (`job_01M2V71PZN11G9VPFDSM4X2689`) then completed with the same two needs-review moments and three declines, no failed calls, and no interview-only requirement in its visual answers. Earlier speech, proposals and reviews were reused; the corrected visual stage reran.

This is functional evidence, not a human acceptance measurement. Both selected moments retained transcription and visual uncertainty; the moment at the recording head may be a recap or montage. Neither is claimed ready to publish. Coverage proves that spans can be proposed; it does not prove coherent editorial quality or the plan's acceptance target. Human positive/negative judgments remain unmeasured.

## Milestone 3: finished picture, captions and sound

- Connect existing face detection to normal analysis and preserve source coordinates.
- Use actual display geometry, stable crop scale and composition boundaries at shot cuts. Add a deliberate two-person layout; fit uncertain shots with an explanation. Do not claim active-speaker detection.
- Carry both viewport crops through edits, preview and rendering.
- Apply gain automation during draft playback and expose an exact-revision rendered audition for final audio/caption approval. Keep caption text and presets consistent.
- Verify decoded outputs and representative frame-rate/rotation cases, not only render plans.

### Implemented behavior and evidence

The director uses display dimensions, resets framing at camera cuts, and keeps one crop size within a shot. One reliably visible face can receive a portrait crop. Exactly two sustained, separated faces can receive two equal portraits from the same decoded frame. Groups, weak evidence and shots shorter than one second stay fitted and carry a review explanation; this is not active-speaker detection. Both portrait paths survive trim, ripple edits and undo. The caption lane is selected from the available face geometry, including the seam between two portraits; text/OCR avoidance is not implemented.

All segments now share one quantized program frame clock. Video and audio concatenate independently, so rounding a camera cut cannot insert audio padding or discard the final footage. Contiguous same-source camera changes are not treated as audible cuts through words. Re-solving targets the shot under the playhead, replaces its complete path as one undoable command, discards stale replies and surfaces errors. Begin/end controls trim the whole program. Fit preserves the stored two-person paths and offers an explicit restore action. Legacy documents missing a required crop path receive an export preflight refusal and a visible draft warning with a Fit repair action.

Draft playback draws both portraits from one video element, applies the saved gain curve through Web Audio, and uses the resolved caption preset and bundled pinned Inter font. It is labelled a draft: the rendered revision remains the authority for libass typography and mastered audio. Native draft playback subsequently advanced in the actual WKWebView without a Web Audio error. This verifies the media path, not a human listening comparison of gain curves.

Mastering keeps the existing −14 LUFS / −1 dBTP targets and measures the decoded AAC result. The −1 dBTP value is a filter target, not a guaranteed post-encode ceiling: the pre-existing daemon conformance test allows up to −0.5 dBTP, and the focused finishing fixture allows up to −0.8 dBTP. Native QA measured −0.86 dBTP, within both gates. No automatic corrective re-encode is implemented; the manifest records the actual result.

The explicit decoder gate is `cargo test --offline -p clipmill-render --test finishing_decode -- --ignored --nocapture` with the repository's pinned FFmpeg and font. Five scenarios passed against actual encoded files:

- A four-second 1080×1920 two-person render decoded the intended upper/lower source colors. The existing audio pass measured **−14.02 LUFS** and **−11.15 dBTP** on the synthetic tone.
- Twenty contiguous 0.52-second shots from a 25 fps source produced **312** frames at 30000/1001 fps, retained source frame 259 at the tail, kept an audio pulse at **9.9000 seconds**, and showed the final caption only at its intended time.
- 25, 30000/1001, 30 and 60 fps sources, plus a verified variable-frame-rate source, produced the expected **31** frames for a 1.03-second edit across a non-frame-aligned cut. Decoded time-encoded luminance checked the source position and final partial frame.
- Files carrying 90°, 180° and 270° display matrices used display-space crop coordinates correctly. Encoded output had the intended dimensions and no residual rotation for a player to apply twice.
- A source with no audio encoded and decoded digital silence. This exposed and fixed an existing failure where non-finite loudness measurements were passed back to FFmpeg; measured silence is now preserved without an infinite normalization gain. A sixth, focused 200 ms audible-edit regression also passed: when integrated loudness is unavailable, the output remains finite and audible rather than failing or turning silent.

Focused Rust, frontend and clippy checks cover these changes. The final full Rust run passed 787 tests and the full desktop run passed 400; subsequent fixes received focused regressions, including 38 export/collection tests and full desktop typechecking. Browser interaction checked the synthetic two-person preview and framing controls without console errors. The small rate/rotation fixtures test the compiler/encoder boundary, not the entire source-ingestion path. Human face-retention scores, caption-quality percentages and acceptance rates remain separate evidence requirements. The complete bridge/worker workflow is also rerun after integration repairs; its final result is recorded separately. The shipping output preset still normalizes to 30000/1001 fps; source-rate preservation is not claimed.

## Milestone 4: routine delivery

- Bind caption/rights approvals to the actual document revision and findings, and record the user's real rights assertion.
- Export an explicit selection with durable per-clip progress, independent retry and restart recovery.
- Refine Models and Settings around actual operational controls, readable evidence, independent loading errors and useful recovery.
- Complete the normal import-to-delivery flow, including declined results and unfinished jobs.

### Implemented behavior and evidence

The collection export screen selects named edit documents across projects, previews the daemon-resolved filenames, and sends an explicit reviewed revision for each clip. Each selected clip requires its own source-rights choice. Fast-caption and long-duration confirmations are bound to that document's revision and findings, and are invalidated when a newer revision is read. Common naming includes a one-based collection index. The saved collection list follows each real export job separately, shows delivered paths, and offers item-specific retry and cancellation.

Nine frontend regressions passed for cross-project selection, explicit rights, stale-revision confirmations, independent preflight failure, restored delivery outcomes, per-item retry/cancel collision-resistant index naming, retry recovery through pending to a new queued job, and readable media-time progress without irrelevant downstream waiting text. Synthetic browser checks exercised permission selection, resolved names, queuing, saved progress and failure recovery in dark and light themes. These UI tests use typed daemon fakes; durable persistence, interruption and frozen-revision retry are covered separately by the real-socket `export_recovery` integration target. Native desktop interaction submitted a real collection for a separate synthetic edit (`edt_01M2V5BCYBGNW6CEN4T41SF211`, revision 0), observed rendering progress and seven delivered files, and recovered the same completed collection after app and daemon restart. This did not assert rights to third-party TV footage or alter existing edits. The delivered MP4 fully decoded: H.264/AAC, 1080×1920, 477 frames, 15.9 seconds; every checksum entry matched, and SRT/VTT carried the intended words. The exact rendered revision played from 0 to 14 seconds in the native app and sought back to eight seconds with the burned caption visible. This exposed and repaired a render inventory bug: internal ASS output must not prevent the supported MP4 from reaching the existing media allowlist. The real `just gate-milestone-1` rerun passed in **145.68 seconds** after correcting two integration defects: approval now reports the complete multi-shot source interval, and whole-program trims reflow shortened caption fragments. The gate exported revision 13 across two shots, retained source 599.43–613.79 seconds, preserved the corrected name in both presentations, exercised undo/redo and daemon restart, and decoded source-time samples at the beginning, middle and end. Its output is `target/milestone-1/delivered/01-older-clip.mp4`. New edge-reflow behavior is explicitly opted into by new trim commands; old serialized ripple commands retain their prior replay semantics. Release CI remains the final gate.

Results now reads the project-bound filmstrip timing descriptor and verifies its source fingerprint before selecting a thumbnail; native interaction confirmed the actual source frame appears. Media authorization parses its validated manifest once per request instead of once per image. The same 2,680-file inventory took **10.502 seconds** on a warm call before the fix and **0.0535/0.0534 seconds** on two warm calls afterward. The first request immediately following daemon restart took **36.4533 seconds**; that startup sample is not evidence of a cold-start speedup, and its cause has not been isolated. Invalid paths, missing advertised files and cross-project requests remain refused. Two artifact and three real-socket tests cover the final inventory change.

## YouTube delivery — after the core milestones

The user reconfirmed this order: finish and verify selection, finishing, routine export and UI refinements before implementing YouTube publishing. Early research and an isolated transport sketch do not change that order.

Connect a channel through system-browser desktop OAuth with PKCE, state validation and a loopback callback. Keep credentials in the OS credential store. Show the actual connected channel and let the user review clip-grounded title, description and tags.

Upload the approved immutable export **privately first** using a persisted resumable session. Recover interruptions without blindly creating another video. Offer a separate explicit **Publish** action on the completed private upload, preserving other status fields.

Google Cloud desktop OAuth configuration and the user's browser consent are external prerequisites. YouTube API projects that have not passed Google's upload compliance audit may be restricted to private uploads. Protocol tests can verify our implementation without asserting a live account connection or public publishing capability.

## Independent completion review

The critic remains separate from implementation. Findings must be fixed or explicitly recorded with a concrete limitation. Evidence must distinguish automated tests, actual decoded media, actual UI interaction and human editorial judgment. No milestone is complete because a document says so.
