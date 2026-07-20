"""Signed corpus verification, daemon evaluation, and run attestations."""

from .artifacts import ArtifactVerificationError, verify_artifact
from .client import DaemonClient, DaemonClientError
from .corpus import CorpusError, VerifiedCorpus, verify_corpus
from .profiles import DeviceProfileVerificationError, verify_device_profile
from .runner import EvaluationError, run_corpus, write_run_manifest

__version__ = "0.0.1"

__all__ = [
    "ArtifactVerificationError",
    "CorpusError",
    "DaemonClient",
    "DaemonClientError",
    "DeviceProfileVerificationError",
    "EvaluationError",
    "VerifiedCorpus",
    "run_corpus",
    "verify_artifact",
    "verify_corpus",
    "verify_device_profile",
    "write_run_manifest",
]
