"""Read the ingest proxy's declared geometry and constant frame rate.

All analysis uses the derived proxy rather than the source file. Frame rate
comes from its artifact descriptor, which defines the resolution of published
shot positions."""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

from clipmill_worker_sdk.artifacts import ArtifactVerificationError, VerifiedArtifact

PROXY_DESCRIPTOR = "proxy.json"


@dataclass(frozen=True, slots=True)
class Proxy:
    artifact_id: str
    file: str
    width: int
    height: int
    rate_num: int
    rate_den: int
    duration_ticks: int
    source_fingerprint: str


def read_proxy(artifact: VerifiedArtifact, descriptor_path: Path) -> Proxy:
    """Read a verified proxy descriptor, refusing anything that is not one."""

    descriptor = json.loads(descriptor_path.read_text(encoding="utf-8"))
    if descriptor.get("schema_version") != "clipmill.media.proxy.v1":
        raise ArtifactVerificationError("input artifact is not a mezzanine proxy")
    video = descriptor.get("video")
    if not isinstance(video, dict):
        raise ArtifactVerificationError("proxy descriptor states no video stream")
    rate = video.get("frame_rate")
    if not isinstance(rate, dict):
        raise ArtifactVerificationError("proxy descriptor states no frame rate")
    rate_num = int(rate.get("num", 0))
    rate_den = int(rate.get("den", 0))
    if rate_num <= 0 or rate_den <= 0:
        raise ArtifactVerificationError("proxy declares a frame rate of zero")
    return Proxy(
        artifact_id=artifact.artifact_id,
        file=str(descriptor["file"]),
        width=int(video["width"]),
        height=int(video["height"]),
        rate_num=rate_num,
        rate_den=rate_den,
        duration_ticks=int(descriptor["duration_ticks"]),
        source_fingerprint=str(descriptor["source_fingerprint"]),
    )


__all__ = ["PROXY_DESCRIPTOR", "Proxy", "read_proxy"]
