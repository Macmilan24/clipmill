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

## What this phase does not check

- **Nothing decodes the export.** The gate compares the plan against the render
  _plan_, not against pixels out of an encoder. Frame-accurate pixel comparison
  is the render gate's job and it already exists; what is not yet joined up is a
  single run that goes plan → render → decode → compare. That is worth building
  and is not built.
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
