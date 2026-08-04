"""Authenticated pull client, lease heartbeat loop, and durable completion retries."""

from __future__ import annotations

import logging
import socket
import threading
from collections.abc import Callable, Sequence
from concurrent.futures import ThreadPoolExecutor
from concurrent.futures import TimeoutError as FutureTimeout
from dataclasses import dataclass, field
from pathlib import Path

from clipmill.worker.v1 import worker_pb2

from .artifacts import (
    ArtifactVerificationError,
    VerifiedArtifact,
    artifact_file,
    open_artifact,
)
from .framing import recv_frame, send_frame
from .identity import WorkerIdentity
from .shared_memory import MappedBuffer, map_shared_buffer
from .staging import StagingArea

LOGGER = logging.getLogger(__name__)
TaskHandler = Callable[["TaskContext"], Sequence[str]]


class RetryableTaskError(RuntimeError):
    """The lease may safely be retried under the daemon retry policy."""


class DeterministicTaskError(RuntimeError):
    """The same input and implementation will fail again."""


class LeaseCancelled(RuntimeError):
    """The daemon cancelled or revoked a running lease."""


class _SessionReset(ConnectionError):
    """Close a lease-owning connection before requesting more work."""


@dataclass(slots=True)
class CancellationToken:
    _event: threading.Event = field(default_factory=threading.Event)
    _parent: threading.Event | None = None

    def cancel(self) -> None:
        self._event.set()

    @property
    def cancelled(self) -> bool:
        return self._event.is_set() or (self._parent is not None and self._parent.is_set())

    def raise_if_cancelled(self) -> None:
        if self.cancelled:
            raise LeaseCancelled("lease cancelled")


@dataclass(slots=True)
class TaskContext:
    lease: worker_pb2.TaskLease
    staging: StagingArea
    shared: MappedBuffer
    cancellation: CancellationToken
    _progress: worker_pb2.ProgressUnits | None = None
    _progress_lock: threading.Lock = field(default_factory=threading.Lock)

    def report_progress(self, unit: str, done: int, total: int = 0) -> None:
        if (
            not unit
            or len(unit) > 64
            or any(ord(character) < 32 or 0x7F <= ord(character) <= 0x9F for character in unit)
        ):
            raise ValueError("invalid progress unit")
        if done < 0 or total < 0 or (total > 0 and done > total):
            raise ValueError("invalid progress values")
        with self._progress_lock:
            self._progress = worker_pb2.ProgressUnits(unit=unit, done=done, total=total)

    def progress(self) -> worker_pb2.ProgressUnits | None:
        with self._progress_lock:
            if self._progress is None:
                return None
            value = worker_pb2.ProgressUnits()
            value.CopyFrom(self._progress)
            return value

    def open_artifact(self, artifact_id: str) -> VerifiedArtifact:
        """Open one of this task's inputs, verified against its manifest.

        The store belongs to the daemon; a worker that read it blindly would
        turn a corrupt object into a corrupt result and publish it under a
        content address claiming it is fine. Every payload is hashed before it
        is used, and only artifacts this lease named may be opened.
        """

        if artifact_id not in self.lease.input_artifact_ids:
            raise ArtifactVerificationError("this lease does not name that artifact as an input")
        if not self.lease.artifact_root:
            raise ArtifactVerificationError("the lease carried no artifact store root")
        return open_artifact(Path(self.lease.artifact_root), artifact_id)

    def artifact_file(self, artifact: VerifiedArtifact, relative: str) -> Path:
        """Resolve one verified payload file inside an opened artifact."""

        return artifact_file(artifact, relative)


@dataclass(frozen=True, slots=True)
class WorkerConfiguration:
    socket_path: Path
    shm_socket_path: Path
    identity: WorkerIdentity
    family: str
    capabilities: tuple[str, ...]
    protocol_version: str = "1.2"
    backend: str = "cpu"
    max_memory_bytes: int = 256 * 1024 * 1024
    # Declared honestly: under-declaring starves the worker, over-declaring
    # gets it capacity-rejected rather than admitted into a machine that
    # cannot hold it.
    cpu_threads: int = 1
    vram_bytes: int = 0


class WorkerClient:
    """Runs a disposable worker until the caller's stop event is set."""

    def __init__(self, configuration: WorkerConfiguration) -> None:
        self.configuration = configuration

    def run(self, handler: TaskHandler, stop: threading.Event | None = None) -> None:
        stopped = stop or threading.Event()
        backoff = 0.1
        while not stopped.is_set():
            try:
                with self._connect_registered() as stream:
                    backoff = 0.1
                    self._run_session(stream, handler, stopped)
            except _SessionReset:
                pass
            except (ConnectionError, OSError, TimeoutError, ValueError) as error:
                LOGGER.warning("worker connection reset: %s", type(error).__name__)
            stopped.wait(backoff)
            backoff = min(backoff * 2, 5.0)

    def run_one(self, handler: TaskHandler, deadline_seconds: float = 30) -> bool:
        """Run at most one lease; useful for deterministic harnesses and tests."""

        with self._connect_registered() as stream:
            stream.settimeout(deadline_seconds)
            send_frame(
                stream,
                worker_pb2.WorkerRequest(work_request=worker_pb2.WorkRequest(max_wait_ms=0)),
            )
            response = recv_frame(stream, worker_pb2.WorkerResponse)
            if response.WhichOneof("body") == "no_work":
                return False
            if response.WhichOneof("body") != "task_lease":
                raise ValueError("daemon returned an unexpected work response")
            self._execute_lease(stream, response.task_lease, handler, threading.Event())
            return True

    def _run_session(
        self,
        stream: socket.socket,
        handler: TaskHandler,
        stopped: threading.Event,
    ) -> None:
        while not stopped.is_set():
            send_frame(
                stream,
                worker_pb2.WorkerRequest(work_request=worker_pb2.WorkRequest(max_wait_ms=250)),
            )
            response = recv_frame(stream, worker_pb2.WorkerResponse)
            kind = response.WhichOneof("body")
            if kind == "no_work":
                stopped.wait(min(response.no_work.retry_after_ms / 1000, 1.0))
                continue
            if kind == "error":
                raise ValueError(f"daemon rejected work request: {response.error.code}")
            if kind != "task_lease":
                raise ValueError("daemon returned an unexpected work response")
            self._execute_lease(stream, response.task_lease, handler, stopped)

    def _execute_lease(
        self,
        stream: socket.socket,
        lease: worker_pb2.TaskLease,
        handler: TaskHandler,
        stopped: threading.Event,
    ) -> None:
        send_frame(
            stream,
            worker_pb2.WorkerRequest(
                lease_acceptance=worker_pb2.LeaseAcceptance(
                    lease_id=lease.lease_id,
                    accepted=True,
                )
            ),
        )
        accepted = recv_frame(stream, worker_pb2.WorkerResponse)
        if accepted.WhichOneof("body") != "heartbeat_ack" or not accepted.heartbeat_ack.accepted:
            raise LeaseCancelled("daemon rejected lease acceptance")
        try:
            staging = StagingArea(lease.staging_id, lease.staging_dir)
        except (OSError, ValueError):
            LOGGER.warning("worker lease setup failed: staging unavailable")
            raise
        cancellation = CancellationToken(_parent=stopped)
        try:
            shared = map_shared_buffer(self.configuration.shm_socket_path, lease.shared_buffer)
        except (OSError, ValueError):
            LOGGER.warning("worker lease setup failed: shared memory unavailable")
            staging.abandon()
            raise
        context = TaskContext(lease, staging, shared, cancellation)
        try:
            completion = self._run_handler_with_heartbeats(stream, context, handler)
            self._send_completion(stream, completion)
        except (ConnectionError, OSError, TimeoutError):
            cancellation.cancel()
            staging.abandon()
            raise
        except LeaseCancelled:
            cancellation.cancel()
            staging.abandon()
            raise _SessionReset("worker lease was cancelled") from None
        finally:
            shared.close()

    def _run_handler_with_heartbeats(
        self,
        stream: socket.socket,
        context: TaskContext,
        handler: TaskHandler,
    ) -> worker_pb2.Complete:
        interval = max(context.lease.heartbeat_interval_ms / 1000, 0.05)
        with ThreadPoolExecutor(max_workers=1, thread_name_prefix="clipmill-task") as executor:
            future = executor.submit(handler, context)
            while True:
                try:
                    paths = tuple(future.result(timeout=interval))
                    context.cancellation.raise_if_cancelled()
                    staged = [context.staging.declare(path) for path in paths]
                    return worker_pb2.Complete(
                        lease_id=context.lease.lease_id,
                        outcome=worker_pb2.TASK_OUTCOME_SUCCEEDED,
                        staged_outputs=staged,
                    )
                except FutureTimeout:
                    heartbeat = worker_pb2.Heartbeat(lease_id=context.lease.lease_id)
                    progress = context.progress()
                    if progress is not None:
                        heartbeat.progress.CopyFrom(progress)
                    try:
                        send_frame(stream, worker_pb2.WorkerRequest(heartbeat=heartbeat))
                        response = recv_frame(stream, worker_pb2.WorkerResponse)
                    except (ConnectionError, OSError, TimeoutError):
                        context.cancellation.cancel()
                        raise
                    kind = response.WhichOneof("body")
                    if kind == "cancel" or (
                        kind == "heartbeat_ack" and response.heartbeat_ack.cancelled
                    ):
                        context.cancellation.cancel()
                    elif kind != "heartbeat_ack" or not response.heartbeat_ack.accepted:
                        context.cancellation.cancel()
                        raise LeaseCancelled("heartbeat rejected") from None
                except RetryableTaskError as error:
                    return worker_pb2.Complete(
                        lease_id=context.lease.lease_id,
                        outcome=worker_pb2.TASK_OUTCOME_RETRYABLE,
                        failure_class=worker_pb2.FAILURE_CLASS_TRANSIENT,
                        detail=type(error).__name__,
                    )
                except DeterministicTaskError as error:
                    return worker_pb2.Complete(
                        lease_id=context.lease.lease_id,
                        outcome=worker_pb2.TASK_OUTCOME_FAILED,
                        failure_class=worker_pb2.FAILURE_CLASS_DETERMINISTIC,
                        detail=type(error).__name__,
                    )
                except LeaseCancelled:
                    raise
                except Exception as error:
                    # The daemon is told the class only — a detail string
                    # crossing the boundary could carry a path or a fragment of
                    # somebody's recording. But it has to be written down
                    # somewhere, or an unexpected failure is a word with no
                    # cause attached and the only way to find it is to guess.
                    LOGGER.exception("task %s failed unexpectedly", context.lease.task_id)
                    return worker_pb2.Complete(
                        lease_id=context.lease.lease_id,
                        outcome=worker_pb2.TASK_OUTCOME_RETRYABLE,
                        failure_class=worker_pb2.FAILURE_CLASS_TRANSIENT,
                        detail=type(error).__name__,
                    )

    def _send_completion(
        self,
        stream: socket.socket,
        completion: worker_pb2.Complete,
    ) -> worker_pb2.CompletionAck:
        request = worker_pb2.WorkerRequest(complete=completion)
        try:
            send_frame(stream, request)
            response = recv_frame(stream, worker_pb2.WorkerResponse)
        except (ConnectionError, OSError, TimeoutError):
            return self._retry_completion(request)
        return _completion_ack(response)

    def _retry_completion(
        self,
        request: worker_pb2.WorkerRequest,
    ) -> worker_pb2.CompletionAck:
        backoff = 0.1
        last_error: Exception | None = None
        for _attempt in range(5):
            try:
                with self._connect_registered() as stream:
                    send_frame(stream, request)
                    return _completion_ack(recv_frame(stream, worker_pb2.WorkerResponse))
            except (ConnectionError, OSError, TimeoutError, ValueError) as error:
                last_error = error
                threading.Event().wait(backoff)
                backoff = min(backoff * 2, 2.0)
        raise ConnectionError("completion acknowledgement was not recovered") from last_error

    def _connect_registered(self) -> socket.socket:
        stream = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        stream.settimeout(30)
        try:
            stream.connect(str(self.configuration.socket_path))
            challenge_response = recv_frame(stream, worker_pb2.WorkerResponse)
            if challenge_response.WhichOneof("body") != "challenge":
                raise ValueError("daemon omitted registration challenge")
            descriptor = self.configuration.identity.signed_descriptor(
                challenge_response.challenge,
                family=self.configuration.family,
                capabilities=self.configuration.capabilities,
                protocol_version=self.configuration.protocol_version,
                backend=self.configuration.backend,
                max_memory_bytes=self.configuration.max_memory_bytes,
                cpu_threads=self.configuration.cpu_threads,
                vram_bytes=self.configuration.vram_bytes,
            )
            send_frame(
                stream,
                worker_pb2.WorkerRequest(register=worker_pb2.RegisterWorker(descriptor=descriptor)),
            )
            response = recv_frame(stream, worker_pb2.WorkerResponse)
            if response.WhichOneof("body") != "registration_ack":
                raise ValueError("daemon rejected worker registration")
            if not response.registration_ack.accepted:
                raise ValueError("daemon rejected worker identity")
            return stream
        except Exception:
            stream.close()
            raise


def _completion_ack(response: worker_pb2.WorkerResponse) -> worker_pb2.CompletionAck:
    kind = response.WhichOneof("body")
    if kind == "error":
        raise ValueError(f"daemon rejected completion: {response.error.code}")
    if kind != "completion_ack" or not response.completion_ack.accepted:
        raise ValueError("daemon did not acknowledge completion")
    return response.completion_ack
