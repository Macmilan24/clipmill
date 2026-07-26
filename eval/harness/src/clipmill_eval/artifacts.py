"""Read-only verification of daemon-published CAS artifacts.

One implementation, in the worker SDK: the harness and every worker must agree
about what "verified" means, and two copies of this logic would eventually
disagree about a corrupt file.
"""

from __future__ import annotations

from pathlib import Path

from clipmill_worker_sdk.artifacts import (
    ARTIFACT_PATTERN,
    ArtifactVerificationError,
    VerifiedArtifact,
    artifact_file,
)
from clipmill_worker_sdk.artifacts import verify_artifact as _verify_under_root

__all__ = [
    "ARTIFACT_PATTERN",
    "ArtifactVerificationError",
    "VerifiedArtifact",
    "artifact_file",
    "verify_artifact",
]


def verify_artifact(data_dir: Path, artifact_id: str) -> VerifiedArtifact:
    """Verify an artifact beneath a daemon *data directory*.

    The harness knows the data directory; a worker is handed the store root on
    its lease. Same verification either way.
    """

    return _verify_under_root(Path(data_dir) / "artifacts", artifact_id)
