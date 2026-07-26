"""ClipMill authenticated worker protocol and zero-copy shared-memory SDK."""

from .audio import PcmAudio, read_pcm_audio
from .client import (
    CancellationToken,
    DeterministicTaskError,
    LeaseCancelled,
    RetryableTaskError,
    TaskContext,
    WorkerClient,
    WorkerConfiguration,
)
from .confidence import distribution
from .documents import canonical_bytes
from .identity import SUPPORTED_PROTOCOLS, WorkerIdentity
from .shared_memory import MappedBuffer, map_shared_buffer
from .staging import StagingArea, validate_artifact_path
from .ticks import TICKS_PER_SECOND, samples_to_ticks, samples_to_ticks_ceil, ticks_to_samples
from .weights import ModelVerificationError, VerifiedModel, require_model

__version__ = "0.0.1"

__all__ = [
    "SUPPORTED_PROTOCOLS",
    "TICKS_PER_SECOND",
    "CancellationToken",
    "DeterministicTaskError",
    "LeaseCancelled",
    "MappedBuffer",
    "ModelVerificationError",
    "PcmAudio",
    "RetryableTaskError",
    "StagingArea",
    "TaskContext",
    "VerifiedModel",
    "WorkerClient",
    "WorkerConfiguration",
    "WorkerIdentity",
    "canonical_bytes",
    "distribution",
    "map_shared_buffer",
    "read_pcm_audio",
    "require_model",
    "samples_to_ticks",
    "samples_to_ticks_ceil",
    "ticks_to_samples",
    "validate_artifact_path",
]
