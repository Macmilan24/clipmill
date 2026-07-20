# Phase 0 public smoke corpus

The repository stores the corpus recipe, not binary media. `clipmill-eval
smoke` uses the pinned FFmpeg build to generate five tiny local fixtures:
CFR, VFR, rotation metadata, offset multi-audio, and intentionally malformed
media. It creates ephemeral Ed25519-signed corpus and license attestations,
verifies every byte, then runs cold and warm source-map evaluation through the
real daemon.

All generated audiovisual content comes only from FFmpeg `lavfi` synthetic
sources. No restricted or third-party media is included.
