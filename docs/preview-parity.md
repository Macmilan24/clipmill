# Preview parity (W24)

The editor's player is a claim: _this is what the export will look like_. A
claim like that is worth nothing unless something checks it, and checking it is
this document's subject.

## One interpreter

There is exactly one implementation of the arithmetic, and it is the one that
renders. `GetPreviewPlan` runs `clipmill-render::preview_plan`, which gets the
crop at a frame from the render's own interpolation, the frame a cue begins on
from the same rounding the subtitle writer uses, and the karaoke sweep from the
function the burned-in track is written with.

The renderer applies TypeScript to nothing. It **applies** a plan: a rectangle
per frame, cue windows already in frames, lines already broken, holds already
in centiseconds. If a number can be computed, it was computed in Rust.

That is why the plan carries **an integer rectangle per frame** rather than the
keyframes it was interpolated from. Handing over keyframes would make the player
interpolate, and interpolating is precisely where two implementations would have
to agree about rounding — the agreement that cannot be assumed. A rectangle per
frame has nothing left to disagree about. For the clip lengths this product
produces it is tens of kilobytes.

## What may differ, and by how much

The renderer draws captions with libass into H.264; the player draws them with
the DOM over a proxy. Pixels will not match, and pretending otherwise would make
the parity rule unfalsifiable. These are the tolerances:

| Difference               | Tolerated                                                    | Why                                                                                                                                          |
| ------------------------ | ------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------- |
| Proxy resolution         | 720p preview vs 1080×1920 export                             | The proxy exists so an editor can scrub; scaling is linear and the crop is expressed as a share of the frame.                                |
| Text position            | ±2 px                                                        | libass hints and positions subpixel; a browser does neither identically. Same font file, same size, same anchor.                             |
| Antialiasing and outline | any                                                          | Two rasterizers.                                                                                                                             |
| Colour                   | BT.709 assumed on both sides                                 | The proxy is tagged 709 and the export is encoded 709. An untagged proxy is a bug in ingest, not a tolerance.                                |
| Audio                    | preview is WebAudio gain, export is a measured loudnorm pass | The preview shows the gain _curve_; the export normalizes to −14 LUFS. The curve is the same; the final level is not, and the meter says so. |

## What may never differ

**Semantics.** These are release-blocking:

- a different word visible at a frame,
- a different crop rectangle at a frame,
- a cue appearing or leaving on a different frame,
- a different line break,
- a different word carrying the karaoke highlight.

Every one of those is something a creator would ship without noticing, because
the preview told them it was fine.

## How it is checked

`gate-editor` does not compare the plan to a stored expectation — a golden
catches a _change_, and what matters here is a _divergence_. It renders fixture
documents through the W13 compiler and compares the plan against the renderer's
own output: the crop at every frame against `crop_rect_at`, the cue windows
against `cue_windows`, the visible text against the burned-in text, and the
karaoke holds read back out of the ASS the encoder would be handed.

If those agree, the two sides are the same interpreter. If they ever stop
agreeing, one of them started deciding.

## The program, the source, and the proxy

Three clocks meet in the player, and the plan says how. Crops, cues and gain
are indexed on the **program** — frame 0 is the clip's first frame. The
segments say which **source** ticks each run of program frames plays, so a
program frame maps to a source tick through the segment it is in. The proxies
say which artifact plays a source and which source tick its own second zero
is, so a source tick maps to a proxy second through the proxy's coverage. The
player, the scrubber, the crop transform and the trim commands all go through
those two tables rather than assuming the clip starts where the recording
does; a trim speaks source ticks because a segment's window is in source
ticks. The crop transform is measured against the source's display
dimensions, which the plan also carries, because a transform built against the
output's dimensions was right only for a source that shared its aspect.

The plan names the revision it describes, the editor sends every command
against the revision it holds, and a plan that arrives for a revision the
editor has already moved past is discarded. Two commands in flight cannot
leave the player showing the older answer.

## One word, two groupings

The reading cues and the burned-in cues are two groupings of one word list,
and every word carries an identity shared with its twin in the other
grouping — minted by the projection from the transcript's word index, or given
to an older document from timing, once. A correction is addressed to the word
(`set_word_text`) and lands in both presentations; the plan's words carry the
id so the caption panel can address it. The two groupings may not read
differently under one id, and a document that does is refused at validation
rather than found on a sidecar.

## What this phase does not check

- **Only the milestone gate decodes an export.** `gate-editor` compares the
  plan against the render _plan_, not against pixels out of an encoder.
  `gate-milestone-1` closes part of that gap for one scenario: it exports a
  trimmed, corrected clip and reads the delivered frames back to the source
  seconds they came from, on a recording whose picture names its own time.
  A general plan → render → decode → compare over arbitrary footage is still
  not built.
- **Latency is not gated.** The plan names an SLO — command under 100 ms, first
  changed frame under 500 ms — and the plan is currently fetched whole rather
  than patched per IR subtree. The revision travels with it so a caller can tell
  a stale picture from a current one, which is the cheap part of that
  optimization; incremental patches are not implemented.

## Editing, and why nothing is local

Every gesture in the editor becomes an IR command. A nudged crop, a split cue, a
gain step, a trim — each is sent, applied by the daemon, and comes back with the
command that undoes it. Nothing is held as local state and reconciled later.

That is a parity property rather than an architectural preference. The render
reads the document; a change that lived only in the renderer would look right in
the player and be absent from the file, which is the same class of failure as a
crop rectangle computed twice.

It is also why the undo stack lives in the renderer while the _log_ lives in the
daemon. Undoing is applying an inverse, so an undo is a command like any other:
logged, durable, and itself undoable.

Two consequences worth stating:

- **A live drag commits its smoothed value.** The One-Euro filter is display
  smoothing, but committing the raw pointer while showing the smoothed one would
  mean the preview and the command disagreed. So what is shown is what is sent.
- **Every apply re-fetches the plan.** Patching it incrementally is the named
  optimization and is not done; a patched plan that drifted from the document
  would be exactly the divergence this document exists to prevent.

## What a trim keeps

A trim moves a segment's window, and everything anchored to the window moves
with it. Three things are kept at the new edges rather than dropped there:

- **The crop.** A keyframe before the new in point decided where the camera
  stood at that boundary — a static crop is one keyframe at zero, and
  advancing the head past it used to leave a `speaker_fill` segment with no
  path, which the preview drew as fit and the renderer refused. The crop the
  path held at each new edge is evaluated, by the same arithmetic the
  renderer draws with (`clipmill_edit_ir::crop_along`, which `crop_rect_at`
  now calls), and written as a keyframe there. For a moving path the edge
  keyframe is an integer rectangle at a tick the renderer rounds to a frame,
  so the picture at the edge is within one frame of motion of what it was;
  for a static crop it is exact.
- **The gain.** The renderer holds the first point backwards and the last
  forwards and ramps between neighbours, so removing the points inside a cut
  is not removing the cut: a ramp that crossed the boundary started from a
  different point. The value the curve held at each edge is written as a
  point of its own before the points inside are removed, and the kept audio
  sounds as it did.
- **The caption the cut fell inside.** Its remaining words keep the window
  they had on the side that was not cut. If what remains cannot be read on
  its own — too brief, or faster than the profile allows — it is folded into
  the cue beside it and the pair is broken again by the caption engine's own
  segmenter, over exactly those two cues. The words are still said, so they
  are never dropped; every other cue, and every grouping a person chose
  elsewhere, is left as it stands. Both presentations get the same treatment
  under their own profiles.

The narrow inverse of a trim — the old window — is used only when trimming
back reproduces the arrangement byte for byte; a trim that kept a crop, a
gain value or a folded caption undoes through the whole prior arrangement
instead, and the undo is exact either way.

## Decoded-frame playback and cut continuity

The canvas uses `requestVideoFrameCallback` and the decoded frame's `mediaTime`
for both its program-frame lookup and its crop. React receives that same frame
for the playhead and captions; painting does not wait for a React commit. Webviews
without this API use a single animation-frame fallback that stops while paused.
A paused video keeps one decoded-frame callback pending so opening and seeking
can display the first available picture without an idle polling loop.

Adjacent segments play continuously only when source identity, source ticks,
program ticks and frame ranges join exactly. Real gaps, repetitions and source
changes still seek. Stale callbacks are cancelled across seeks and source/document
changes. Composition happens in a temporary canvas and replaces the visible
bitmap only after a successful draw, including its translucent blur edges.
Native end-of-file events finish or advance the edit even when the last decoded
proxy frame precedes the final program frame; Replay explicitly seeks the clip's
start instead of relying on the browser's whole-file restart.

### Native verification, 2026-09-19

The macOS desktop build was tested through its real media protocol on a saved
59-second, 1,770-frame edit with alternating single and two-portrait layouts.
Web Inspector counters recorded six unnecessary seeks over 14.3 seconds before
the fix. With the repaired frame clock, a full playback recorded 1,771 decoded
callbacks, no intermediate seeks, and only the intended seek onto the final held
frame. The largest gap between consecutive decoded timestamps was 33.367 ms.
A 16×16 canvas sample on every callback found no fully transparent or fully white
frames. The first picture appeared while paused; rapid scrubbing, stepping and
replay were also checked. No document edits were made during these checks.

An intermediate build recorded one transparent sample before adding the paused
decoded-frame subscription; the subsequent cold-open/full-playback check above
recorded none. This is a regression check for this footage and machine, not a
cross-platform frame-delivery guarantee or an export pixel-parity test. Export
still renders the saved document independently from original footage.

Regression tests cover fractional frame boundaries, delayed callbacks crossing
several shots, real source gaps and overlaps, paused edits, decode failures,
source/document replacement, native EOF and Replay. An independent review found
the detached-media, compositing-edge and EOF cases; all were fixed and retested.
EOF cases are exercised by controlled media tests rather than by the native
mid-source recording used for the compositor check.

## Soft cuts between people

New directed edits request a 120 ms blend between cropped shots. In the editor,
Reframe → Soft cuts applies to the whole clip, with Off and a 40–250 ms duration.
Older saved edits keep their original hard cuts until the setting is enabled.
The optional `video.transition_ticks` field is omitted when zero, preserving
legacy canonical document bytes; changing it is a normal undoable edit.

This is a brief fade of the outgoing shot's final composition over the live
incoming shot. It does not pan between unrelated camera views or overlap source
intervals. Audio, captions, source windows and total duration keep their timing.
Fit-to-Fit boundaries remain hard cuts. The shared renderer allocator rounds the
requested duration to frames, caps it at 250 ms and half the incoming shot, and
omits blends shorter than two frames. The editor reports when no boundary can
use the saved duration and framing.

The preview plan names each outgoing frame and the half-open incoming blend
interval. The canvas preloads that outgoing picture through a muted, paused
reference decoder; direct scrubbing therefore produces the same blend without
requiring playback history. A missing reference pauses the transport on the
requested frame until it is ready. Pause cancels automatic resume, and media
errors stop playback with a visible explanation. Stale revision references are
discarded. Captions are drawn over the blended picture.

Export uses the same frame allocation, holding the outgoing composed frame with
FFmpeg and fading its alpha over the incoming frames before captions are burned
in. The pinned-FFmpeg regression decodes changing footage at 30 and 29.97 fps,
checks the expected blended pixels and complete frame count, and compares the
decoded audio byte for byte with hard-cut output. Unit and component checks
cover short shots, legacy documents, undo, trimming, paused scrubs, reference
errors, buffering and cancelled resume.

Native verification also exposed a pre-existing save failure after shell
relaunch: counter-only request IDs collided with durable mutation receipts.
The shell now prefixes its counter with a fresh session ULID while reusing the
same complete envelope for retries. Editor failures returned as Tauri strings
are displayed instead of being lost through an `Error.message` cast. Regression
checks cover restarted counters and a dropped mutation reply.

### Native soft-cut verification, 2026-09-19

On the saved 59-second edit used above, the native Reframe control enabled
120 ms at r1, applied 180 ms as one edit at r2, and Undo restored 120 ms at r3.
A full soft-cut playback produced 1,771 decoded callbacks with no intermediate
primary-video seeks, no fully white or transparent samples, and a maximum
decoded timestamp gap of 33.367 ms. A paused seek directly to frame 40 showed
the blended picture inside the first transition; it did not depend on having
played the preceding shot. These are native proxy checks on this recording;
the separate decoded export regression above verifies render timing and weights.
Relaunch restored r3 and 120 ms. A fresh-session edit saved 150 ms at r4 and Undo
returned to 120 ms at r5, exercising the request-ID repair against the real store.
