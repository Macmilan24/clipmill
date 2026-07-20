"""ClipMill authenticated worker protocol and zero-copy shared-memory SDK."""

from .client import (
    CancellationToken,
    DeterministicTaskError,
    LeaseCancelled,
    RetryableTaskError,
    TaskContext,
    WorkerClient,
    WorkerConfiguration,
)
from .identity import SUPPORTED_PROTOCOLS, WorkerIdentity
from .shared_memory import MappedBuffer, map_shared_buffer
from .staging import StagingArea, validate_artifact_path

__version__ = "0.0.1"

__all__ = [
    "SUPPORTED_PROTOCOLS",
    "CancellationToken",
    "DeterministicTaskError",
    "LeaseCancelled",
    "MappedBuffer",
    "RetryableTaskError",
    "StagingArea",
    "TaskContext",
    "WorkerClient",
    "WorkerConfiguration",
    "WorkerIdentity",
    "map_shared_buffer",
    "validate_artifact_path",
]
