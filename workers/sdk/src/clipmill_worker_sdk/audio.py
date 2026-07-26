"""Reading the ingest audio renditions a speech worker is given.

Ingest already decoded the source once and published normalized PCM (book
ch. 12); a speech worker's job is to read that, not to open the original file
again. It arrives through the verified artifact path, so the bytes below have
been hashed against the manifest before this module sees them.

No array library appears here on purpose. The SDK is shared with workers that
have no numerical dependencies at all, so this hands back raw interleaved
frames and lets each worker adopt them into whatever array type its runtime
wants — which for every current consumer is a zero-copy view.
"""

from __future__ import annotations

import json
import wave
from dataclasses import dataclass
from pathlib import Path

from .artifacts import ArtifactVerificationError, VerifiedArtifact

AUDIO_PAYLOAD = "audio.wav"
AUDIO_DESCRIPTOR = "audio.json"


@dataclass(frozen=True, slots=True)
class PcmAudio:
    """One ingest audio rendition, with the descriptor that states what it is."""

    artifact_id: str
    sample_rate: int
    channels: int
    sample_count: int
    duration_ticks: int
    source_fingerprint: str
    frames: bytes
    """Interleaved little-endian signed 16-bit samples."""

    @property
    def sample_width_bytes(self) -> int:
        return 2


def read_pcm_audio(
    artifact: VerifiedArtifact,
    payload_path: Path,
    descriptor_path: Path,
    *,
    expect_sample_rate: int | None = None,
    expect_channels: int | None = None,
) -> PcmAudio:
    """Read a verified rendition, checking the payload against its descriptor.

    Both are inside the same artifact and both were hashed, so this is not a
    tamper check — it is a wiring check. A worker handed the 48 kHz stereo
    rendition where it expected the 16 kHz mono one would otherwise produce a
    transcript whose every timestamp is off by a factor of three, and would
    produce it without complaint.
    """

    descriptor = json.loads(descriptor_path.read_text(encoding="utf-8"))
    if descriptor.get("schema_version") != "clipmill.media.audio.v1":
        raise ArtifactVerificationError("input artifact is not an audio rendition")

    with wave.open(str(payload_path), "rb") as source:
        channels = source.getnchannels()
        sample_rate = source.getframerate()
        if source.getsampwidth() != 2:
            raise ArtifactVerificationError("audio rendition is not 16-bit PCM")
        sample_count = source.getnframes()
        frames = source.readframes(sample_count)

    for label, actual, declared in (
        ("sample rate", sample_rate, descriptor.get("sample_rate")),
        ("channel count", channels, descriptor.get("channels")),
        ("sample count", sample_count, descriptor.get("sample_count")),
    ):
        if declared is not None and actual != declared:
            raise ArtifactVerificationError(
                f"audio {label} {actual} contradicts the descriptor's {declared}"
            )
    if expect_sample_rate is not None and sample_rate != expect_sample_rate:
        raise ArtifactVerificationError(
            f"this stage needs {expect_sample_rate} Hz audio, not {sample_rate} Hz"
        )
    if expect_channels is not None and channels != expect_channels:
        raise ArtifactVerificationError(
            f"this stage needs {expect_channels} channel(s), not {channels}"
        )

    return PcmAudio(
        artifact_id=artifact.artifact_id,
        sample_rate=sample_rate,
        channels=channels,
        sample_count=sample_count,
        duration_ticks=int(descriptor["duration_ticks"]),
        source_fingerprint=str(descriptor["source_fingerprint"]),
        frames=frames,
    )


__all__ = ["AUDIO_DESCRIPTOR", "AUDIO_PAYLOAD", "PcmAudio", "read_pcm_audio"]
