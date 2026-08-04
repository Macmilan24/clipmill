"""Small synchronous Protobuf client for the local daemon control socket."""

from __future__ import annotations

import socket
import time
import uuid
from collections.abc import Iterator
from pathlib import Path

from clipmill.ipc.v1 import daemon_pb2

MAX_FRAME_BYTES = 4 * 1024 * 1024


class DaemonClientError(RuntimeError):
    def __init__(self, code: int, message: str) -> None:
        self.code = code
        self.message = message
        super().__init__(f"daemon error {code}: {message}")


class FramingError(RuntimeError):
    """The daemon returned malformed length-delimited Protobuf framing."""


class DaemonClient:
    def __init__(self, socket_path: Path, timeout_seconds: float = 30.0) -> None:
        self.socket_path = socket_path
        self.timeout_seconds = timeout_seconds

    def health(self) -> daemon_pb2.HealthResponse:
        response = self._call(daemon_pb2.Request(health=daemon_pb2.HealthRequest()))
        return _require_body(response, "health")

    def create_project(self, name: str) -> daemon_pb2.Project:
        response = self._call(
            daemon_pb2.Request(
                create_project=daemon_pb2.CreateProjectRequest(name=name),
            )
        )
        return _require_body(response, "create_project").project

    def register_source(
        self, project_id: str, absolute_path: Path
    ) -> daemon_pb2.RegisterSourceResponse:
        response = self._call(
            daemon_pb2.Request(
                register_source=daemon_pb2.RegisterSourceRequest(
                    project_id=project_id,
                    absolute_path=str(absolute_path),
                )
            )
        )
        return _require_body(response, "register_source")

    def submit_probe(self, project_id: str, source_id: str) -> daemon_pb2.Job:
        payload = daemon_pb2.ProbeSourcePayloadV1(
            key_version="clipmill.probe-source.v1",
            source_id=source_id,
        ).SerializeToString(deterministic=True)
        response = self._call(
            daemon_pb2.Request(
                submit_job=daemon_pb2.SubmitJobRequest(
                    project_id=project_id,
                    kind="probe-source",
                    payload=payload,
                )
            )
        )
        return _require_body(response, "submit_job").job

    def submit_analyze(
        self,
        project_id: str,
        source_id: str,
        *,
        language: str = "",
        count: int = 0,
        diversity_milli: int = 0,
    ) -> daemon_pb2.Job:
        """Submit the whole analyze DAG: probe through rank.

        Every tunable is left at zero, which the payload defines as "no
        opinion". An evaluation that passed its own duration range or set size
        would be measuring a configuration nobody ships rather than the one a
        user gets.
        """

        payload = daemon_pb2.AnalyzeSourcePayloadV1(
            key_version="clipmill.analyze-source.v1",
            source_id=source_id,
            language=language,
            count=count,
            diversity_milli=diversity_milli,
        ).SerializeToString(deterministic=True)
        response = self._call(
            daemon_pb2.Request(
                submit_job=daemon_pb2.SubmitJobRequest(
                    project_id=project_id,
                    kind="analyze-source",
                    payload=payload,
                )
            )
        )
        return _require_body(response, "submit_job").job

    def read_artifact(self, project_id: str, artifact_id: str) -> bytes:
        """A published document, whole.

        Read in chunks and reassembled here rather than asking for everything
        at once: the protocol caps a frame, and a caller that assumed one call
        would suffice would silently truncate the first document large enough
        to matter — which is exactly the ranked set of a long recording.
        """

        collected = bytearray()
        while True:
            response = self._call(
                daemon_pb2.Request(
                    read_artifact=daemon_pb2.ReadArtifactRequest(
                        project_id=project_id,
                        artifact_id=artifact_id,
                        offset=len(collected),
                    )
                )
            )
            body = _require_body(response, "read_artifact")
            collected.extend(body.chunk)
            if not body.chunk or len(collected) >= body.total_bytes:
                break
        return bytes(collected)

    def get_job(self, job_id: str) -> daemon_pb2.Job:
        response = self._call(daemon_pb2.Request(get_job=daemon_pb2.GetJobRequest(job_id=job_id)))
        return _require_body(response, "get_job").job

    def list_jobs(self, project_id: str) -> tuple[daemon_pb2.Job, ...]:
        response = self._call(
            daemon_pb2.Request(
                list_jobs=daemon_pb2.ListJobsRequest(project_id=project_id),
            )
        )
        return tuple(_require_body(response, "list_jobs").jobs)

    def cancel_job(self, job_id: str) -> daemon_pb2.Job:
        response = self._call(
            daemon_pb2.Request(
                cancel_job=daemon_pb2.CancelJobRequest(job_id=job_id),
            )
        )
        return _require_body(response, "cancel_job").job

    def get_source(self, source_id: str) -> daemon_pb2.Source:
        response = self._call(
            daemon_pb2.Request(get_source=daemon_pb2.GetSourceRequest(source_id=source_id))
        )
        return _require_body(response, "get_source").source

    def list_sources(self, project_id: str) -> tuple[daemon_pb2.Source, ...]:
        response = self._call(
            daemon_pb2.Request(
                list_sources=daemon_pb2.ListSourcesRequest(project_id=project_id),
            )
        )
        return tuple(_require_body(response, "list_sources").sources)

    def direct_clip(
        self,
        project_id: str,
        source_id: str,
        candidate_id: str,
        *,
        cut: int = daemon_pb2.CLIP_CUT_V1_CHOSEN,
    ) -> daemon_pb2.DirectClipResponse:
        """Assemble a candidate into an edit document.

        The chosen cut by default, because that is the one a ranking argues
        for; the alternative exists for a person who disagrees, and asking for
        it from a harness would measure a path no default run takes.
        """

        response = self._call(
            daemon_pb2.Request(
                direct_clip=daemon_pb2.DirectClipRequest(
                    project_id=project_id,
                    source_id=source_id,
                    candidate_id=candidate_id,
                    cut=cut,
                )
            )
        )
        return _require_body(response, "direct_clip")

    def export_clip(self, request: daemon_pb2.ExportRequestV1) -> str:
        """Render and deliver a document, returning the job to wait on.

        The whole request travels as the contract type rather than as loose
        arguments: the naming pattern, the attestation and the assistance
        disclosure are one document the daemon validates together, and
        spreading them across keywords here would let a caller build half of
        one.
        """

        response = self._call(
            daemon_pb2.Request(export_clip=daemon_pb2.ExportClipRequest(request=request))
        )
        return _require_body(response, "export_clip").job_id

    def get_device_profile(
        self, *, remeasure: bool = False, request_id: str | None = None
    ) -> daemon_pb2.GetDeviceProfileResponse:
        response = self._call(
            daemon_pb2.Request(
                request_id=request_id or self._request_id("device"),
                get_device_profile=daemon_pb2.GetDeviceProfileRequest(remeasure=remeasure),
            ),
            assign_request_id=False,
        )
        return _require_body(response, "get_device_profile")

    def wait_for_job(self, job_id: str, timeout_seconds: float = 30.0) -> daemon_pb2.Job:
        deadline = time.monotonic() + timeout_seconds
        while time.monotonic() < deadline:
            job = self.get_job(job_id)
            if job.state in {
                daemon_pb2.JOB_STATE_SUCCEEDED,
                daemon_pb2.JOB_STATE_FAILED,
                daemon_pb2.JOB_STATE_CANCELLED,
            }:
                return job
            time.sleep(0.05)
        raise TimeoutError(f"job {job_id} did not become terminal")

    def subscribe_events(
        self,
        *,
        project_id: str = "",
        job_id: str = "",
        after_event_id: int = 0,
        maximum_events: int | None = None,
    ) -> Iterator[daemon_pb2.TaskEvent]:
        request = daemon_pb2.Request(
            request_id=self._request_id("events"),
            subscribe_task_events=daemon_pb2.SubscribeTaskEventsRequest(
                project_id=project_id,
                job_id=job_id,
                after_event_id=after_event_id,
            ),
        )
        with self._connect() as connection:
            _write_frame(connection, request.SerializeToString(deterministic=True))
            ready = _read_response(connection)
            if ready.request_id != request.request_id:
                raise FramingError("subscription response request_id does not match the request")
            self._raise_error(ready)
            if ready.WhichOneof("body") != "subscribe_task_events":
                raise FramingError("subscription omitted its ready response")
            last_cursor = after_event_id
            received = 0
            while maximum_events is None or received < maximum_events:
                response = _read_response(connection)
                if response.request_id != request.request_id:
                    raise FramingError("event response request_id does not match the subscription")
                self._raise_error(response)
                if response.WhichOneof("body") != "task_event":
                    raise FramingError("subscription returned a non-event response")
                event = response.task_event
                if event.event_id <= last_cursor:
                    raise FramingError("event cursor replay contained a gap or duplicate")
                last_cursor = event.event_id
                received += 1
                yield event

    def _call(
        self, request: daemon_pb2.Request, *, assign_request_id: bool = True
    ) -> daemon_pb2.Response:
        if assign_request_id:
            request.request_id = self._request_id("eval")
        if not request.request_id:
            raise ValueError("request_id is required")
        with self._connect() as connection:
            _write_frame(connection, request.SerializeToString(deterministic=True))
            response = _read_response(connection)
        if response.request_id != request.request_id:
            raise FramingError("response request_id does not match the request")
        self._raise_error(response)
        return response

    def _connect(self) -> socket.socket:
        connection = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        connection.settimeout(self.timeout_seconds)
        connection.connect(str(self.socket_path))
        return connection

    @staticmethod
    def _raise_error(response: daemon_pb2.Response) -> None:
        if response.WhichOneof("body") == "error":
            raise DaemonClientError(response.error.code, response.error.message)

    @staticmethod
    def _request_id(prefix: str) -> str:
        return f"{prefix}-{uuid.uuid4().hex}"


def _write_frame(connection: socket.socket, payload: bytes) -> None:
    if len(payload) > MAX_FRAME_BYTES:
        raise FramingError("request exceeds the daemon frame limit")
    connection.sendall(_encode_varint(len(payload)) + payload)


def _read_response(connection: socket.socket) -> daemon_pb2.Response:
    length = _read_varint(connection)
    if length > MAX_FRAME_BYTES:
        raise FramingError("response exceeds the daemon frame limit")
    payload = _read_exact(connection, length)
    response = daemon_pb2.Response()
    try:
        response.ParseFromString(payload)
    except Exception as error:  # protobuf raises implementation-specific decode errors
        raise FramingError(f"response is not valid Protobuf: {error}") from error
    return response


def _read_varint(connection: socket.socket) -> int:
    value = 0
    for index in range(10):
        byte = _read_exact(connection, 1)[0]
        if index == 9 and byte > 1:
            raise FramingError("malformed frame length")
        value |= (byte & 0x7F) << (index * 7)
        if byte & 0x80 == 0:
            return value
    raise FramingError("malformed frame length")


def _read_exact(connection: socket.socket, length: int) -> bytes:
    chunks: list[bytes] = []
    remaining = length
    while remaining:
        chunk = connection.recv(remaining)
        if not chunk:
            raise FramingError("truncated daemon response")
        chunks.append(chunk)
        remaining -= len(chunk)
    return b"".join(chunks)


def _encode_varint(value: int) -> bytes:
    encoded = bytearray()
    while value >= 0x80:
        encoded.append((value & 0x7F) | 0x80)
        value >>= 7
    encoded.append(value)
    return bytes(encoded)


def _require_body(response: daemon_pb2.Response, expected: str):
    actual = response.WhichOneof("body")
    if actual != expected:
        raise FramingError(f"daemon returned {actual or 'no body'}; expected {expected}")
    return getattr(response, expected)
