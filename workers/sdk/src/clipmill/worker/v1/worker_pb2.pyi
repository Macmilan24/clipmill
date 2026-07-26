from clipmill.shm.v1 import shm_pb2 as _shm_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class DeclineReason(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DECLINE_REASON_UNSPECIFIED: _ClassVar[DeclineReason]
    DECLINE_REASON_UNSUPPORTED_KIND: _ClassVar[DeclineReason]
    DECLINE_REASON_VERSION_SKEW: _ClassVar[DeclineReason]
    DECLINE_REASON_RESOURCE_EXHAUSTED: _ClassVar[DeclineReason]
    DECLINE_REASON_POLICY: _ClassVar[DeclineReason]

class TaskOutcome(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    TASK_OUTCOME_UNSPECIFIED: _ClassVar[TaskOutcome]
    TASK_OUTCOME_SUCCEEDED: _ClassVar[TaskOutcome]
    TASK_OUTCOME_RETRYABLE: _ClassVar[TaskOutcome]
    TASK_OUTCOME_FAILED: _ClassVar[TaskOutcome]
    TASK_OUTCOME_CANCELLED: _ClassVar[TaskOutcome]

class FailureClass(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    FAILURE_CLASS_UNSPECIFIED: _ClassVar[FailureClass]
    FAILURE_CLASS_TRANSIENT: _ClassVar[FailureClass]
    FAILURE_CLASS_DETERMINISTIC: _ClassVar[FailureClass]
    FAILURE_CLASS_CORRUPT_MODEL: _ClassVar[FailureClass]
    FAILURE_CLASS_NETWORK: _ClassVar[FailureClass]
DECLINE_REASON_UNSPECIFIED: DeclineReason
DECLINE_REASON_UNSUPPORTED_KIND: DeclineReason
DECLINE_REASON_VERSION_SKEW: DeclineReason
DECLINE_REASON_RESOURCE_EXHAUSTED: DeclineReason
DECLINE_REASON_POLICY: DeclineReason
TASK_OUTCOME_UNSPECIFIED: TaskOutcome
TASK_OUTCOME_SUCCEEDED: TaskOutcome
TASK_OUTCOME_RETRYABLE: TaskOutcome
TASK_OUTCOME_FAILED: TaskOutcome
TASK_OUTCOME_CANCELLED: TaskOutcome
FAILURE_CLASS_UNSPECIFIED: FailureClass
FAILURE_CLASS_TRANSIENT: FailureClass
FAILURE_CLASS_DETERMINISTIC: FailureClass
FAILURE_CLASS_CORRUPT_MODEL: FailureClass
FAILURE_CLASS_NETWORK: FailureClass

class CapabilityDescriptor(_message.Message):
    __slots__ = ("worker_id", "family", "capabilities", "protocol_version", "backend", "max_memory_bytes", "public_key", "signature", "cpu_threads", "vram_bytes")
    WORKER_ID_FIELD_NUMBER: _ClassVar[int]
    FAMILY_FIELD_NUMBER: _ClassVar[int]
    CAPABILITIES_FIELD_NUMBER: _ClassVar[int]
    PROTOCOL_VERSION_FIELD_NUMBER: _ClassVar[int]
    BACKEND_FIELD_NUMBER: _ClassVar[int]
    MAX_MEMORY_BYTES_FIELD_NUMBER: _ClassVar[int]
    PUBLIC_KEY_FIELD_NUMBER: _ClassVar[int]
    SIGNATURE_FIELD_NUMBER: _ClassVar[int]
    CPU_THREADS_FIELD_NUMBER: _ClassVar[int]
    VRAM_BYTES_FIELD_NUMBER: _ClassVar[int]
    worker_id: str
    family: str
    capabilities: _containers.RepeatedScalarFieldContainer[str]
    protocol_version: str
    backend: str
    max_memory_bytes: int
    public_key: bytes
    signature: bytes
    cpu_threads: int
    vram_bytes: int
    def __init__(self, worker_id: _Optional[str] = ..., family: _Optional[str] = ..., capabilities: _Optional[_Iterable[str]] = ..., protocol_version: _Optional[str] = ..., backend: _Optional[str] = ..., max_memory_bytes: _Optional[int] = ..., public_key: _Optional[bytes] = ..., signature: _Optional[bytes] = ..., cpu_threads: _Optional[int] = ..., vram_bytes: _Optional[int] = ...) -> None: ...

class TaskLease(_message.Message):
    __slots__ = ("task_id", "lease_id", "kind", "payload", "heartbeat_interval_ms", "lease_ttl_ms", "staging_id", "staging_dir", "input_artifact_ids", "shared_buffer", "output_kind", "attempt")
    TASK_ID_FIELD_NUMBER: _ClassVar[int]
    LEASE_ID_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    HEARTBEAT_INTERVAL_MS_FIELD_NUMBER: _ClassVar[int]
    LEASE_TTL_MS_FIELD_NUMBER: _ClassVar[int]
    STAGING_ID_FIELD_NUMBER: _ClassVar[int]
    STAGING_DIR_FIELD_NUMBER: _ClassVar[int]
    INPUT_ARTIFACT_IDS_FIELD_NUMBER: _ClassVar[int]
    SHARED_BUFFER_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_KIND_FIELD_NUMBER: _ClassVar[int]
    ATTEMPT_FIELD_NUMBER: _ClassVar[int]
    task_id: str
    lease_id: str
    kind: str
    payload: bytes
    heartbeat_interval_ms: int
    lease_ttl_ms: int
    staging_id: str
    staging_dir: str
    input_artifact_ids: _containers.RepeatedScalarFieldContainer[str]
    shared_buffer: _shm_pb2.BufferDescriptor
    output_kind: str
    attempt: int
    def __init__(self, task_id: _Optional[str] = ..., lease_id: _Optional[str] = ..., kind: _Optional[str] = ..., payload: _Optional[bytes] = ..., heartbeat_interval_ms: _Optional[int] = ..., lease_ttl_ms: _Optional[int] = ..., staging_id: _Optional[str] = ..., staging_dir: _Optional[str] = ..., input_artifact_ids: _Optional[_Iterable[str]] = ..., shared_buffer: _Optional[_Union[_shm_pb2.BufferDescriptor, _Mapping]] = ..., output_kind: _Optional[str] = ..., attempt: _Optional[int] = ...) -> None: ...

class ProgressUnits(_message.Message):
    __slots__ = ("unit", "done", "total")
    UNIT_FIELD_NUMBER: _ClassVar[int]
    DONE_FIELD_NUMBER: _ClassVar[int]
    TOTAL_FIELD_NUMBER: _ClassVar[int]
    unit: str
    done: int
    total: int
    def __init__(self, unit: _Optional[str] = ..., done: _Optional[int] = ..., total: _Optional[int] = ...) -> None: ...

class Heartbeat(_message.Message):
    __slots__ = ("lease_id", "progress")
    LEASE_ID_FIELD_NUMBER: _ClassVar[int]
    PROGRESS_FIELD_NUMBER: _ClassVar[int]
    lease_id: str
    progress: ProgressUnits
    def __init__(self, lease_id: _Optional[str] = ..., progress: _Optional[_Union[ProgressUnits, _Mapping]] = ...) -> None: ...

class Decline(_message.Message):
    __slots__ = ("task_id", "reason", "detail")
    TASK_ID_FIELD_NUMBER: _ClassVar[int]
    REASON_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    task_id: str
    reason: DeclineReason
    detail: str
    def __init__(self, task_id: _Optional[str] = ..., reason: _Optional[_Union[DeclineReason, str]] = ..., detail: _Optional[str] = ...) -> None: ...

class Complete(_message.Message):
    __slots__ = ("lease_id", "outcome", "artifact_ids", "failure_class", "detail", "staged_outputs")
    LEASE_ID_FIELD_NUMBER: _ClassVar[int]
    OUTCOME_FIELD_NUMBER: _ClassVar[int]
    ARTIFACT_IDS_FIELD_NUMBER: _ClassVar[int]
    FAILURE_CLASS_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    STAGED_OUTPUTS_FIELD_NUMBER: _ClassVar[int]
    lease_id: str
    outcome: TaskOutcome
    artifact_ids: _containers.RepeatedScalarFieldContainer[str]
    failure_class: FailureClass
    detail: str
    staged_outputs: _containers.RepeatedCompositeFieldContainer[StagedOutput]
    def __init__(self, lease_id: _Optional[str] = ..., outcome: _Optional[_Union[TaskOutcome, str]] = ..., artifact_ids: _Optional[_Iterable[str]] = ..., failure_class: _Optional[_Union[FailureClass, str]] = ..., detail: _Optional[str] = ..., staged_outputs: _Optional[_Iterable[_Union[StagedOutput, _Mapping]]] = ...) -> None: ...

class StagedOutput(_message.Message):
    __slots__ = ("relative_path", "byte_size", "sha256")
    RELATIVE_PATH_FIELD_NUMBER: _ClassVar[int]
    BYTE_SIZE_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    relative_path: str
    byte_size: int
    sha256: str
    def __init__(self, relative_path: _Optional[str] = ..., byte_size: _Optional[int] = ..., sha256: _Optional[str] = ...) -> None: ...

class RegistrationChallenge(_message.Message):
    __slots__ = ("nonce", "supported_protocol_versions", "issued_unix_millis")
    NONCE_FIELD_NUMBER: _ClassVar[int]
    SUPPORTED_PROTOCOL_VERSIONS_FIELD_NUMBER: _ClassVar[int]
    ISSUED_UNIX_MILLIS_FIELD_NUMBER: _ClassVar[int]
    nonce: bytes
    supported_protocol_versions: _containers.RepeatedScalarFieldContainer[str]
    issued_unix_millis: int
    def __init__(self, nonce: _Optional[bytes] = ..., supported_protocol_versions: _Optional[_Iterable[str]] = ..., issued_unix_millis: _Optional[int] = ...) -> None: ...

class RegisterWorker(_message.Message):
    __slots__ = ("descriptor",)
    DESCRIPTOR_FIELD_NUMBER: _ClassVar[int]
    descriptor: CapabilityDescriptor
    def __init__(self, descriptor: _Optional[_Union[CapabilityDescriptor, _Mapping]] = ...) -> None: ...

class RegistrationAck(_message.Message):
    __slots__ = ("accepted", "session_id", "detail")
    ACCEPTED_FIELD_NUMBER: _ClassVar[int]
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    accepted: bool
    session_id: str
    detail: str
    def __init__(self, accepted: _Optional[bool] = ..., session_id: _Optional[str] = ..., detail: _Optional[str] = ...) -> None: ...

class WorkRequest(_message.Message):
    __slots__ = ("max_wait_ms",)
    MAX_WAIT_MS_FIELD_NUMBER: _ClassVar[int]
    max_wait_ms: int
    def __init__(self, max_wait_ms: _Optional[int] = ...) -> None: ...

class NoWork(_message.Message):
    __slots__ = ("retry_after_ms",)
    RETRY_AFTER_MS_FIELD_NUMBER: _ClassVar[int]
    retry_after_ms: int
    def __init__(self, retry_after_ms: _Optional[int] = ...) -> None: ...

class LeaseAcceptance(_message.Message):
    __slots__ = ("lease_id", "accepted", "reason", "detail")
    LEASE_ID_FIELD_NUMBER: _ClassVar[int]
    ACCEPTED_FIELD_NUMBER: _ClassVar[int]
    REASON_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    lease_id: str
    accepted: bool
    reason: DeclineReason
    detail: str
    def __init__(self, lease_id: _Optional[str] = ..., accepted: _Optional[bool] = ..., reason: _Optional[_Union[DeclineReason, str]] = ..., detail: _Optional[str] = ...) -> None: ...

class HeartbeatAck(_message.Message):
    __slots__ = ("lease_id", "accepted", "cancelled", "expires_unix_millis")
    LEASE_ID_FIELD_NUMBER: _ClassVar[int]
    ACCEPTED_FIELD_NUMBER: _ClassVar[int]
    CANCELLED_FIELD_NUMBER: _ClassVar[int]
    EXPIRES_UNIX_MILLIS_FIELD_NUMBER: _ClassVar[int]
    lease_id: str
    accepted: bool
    cancelled: bool
    expires_unix_millis: int
    def __init__(self, lease_id: _Optional[str] = ..., accepted: _Optional[bool] = ..., cancelled: _Optional[bool] = ..., expires_unix_millis: _Optional[int] = ...) -> None: ...

class CancelLease(_message.Message):
    __slots__ = ("lease_id", "reason")
    LEASE_ID_FIELD_NUMBER: _ClassVar[int]
    REASON_FIELD_NUMBER: _ClassVar[int]
    lease_id: str
    reason: str
    def __init__(self, lease_id: _Optional[str] = ..., reason: _Optional[str] = ...) -> None: ...

class CompletionAck(_message.Message):
    __slots__ = ("lease_id", "accepted", "artifact_ids", "detail")
    LEASE_ID_FIELD_NUMBER: _ClassVar[int]
    ACCEPTED_FIELD_NUMBER: _ClassVar[int]
    ARTIFACT_IDS_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    lease_id: str
    accepted: bool
    artifact_ids: _containers.RepeatedScalarFieldContainer[str]
    detail: str
    def __init__(self, lease_id: _Optional[str] = ..., accepted: _Optional[bool] = ..., artifact_ids: _Optional[_Iterable[str]] = ..., detail: _Optional[str] = ...) -> None: ...

class ProtocolError(_message.Message):
    __slots__ = ("code", "detail")
    CODE_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    code: str
    detail: str
    def __init__(self, code: _Optional[str] = ..., detail: _Optional[str] = ...) -> None: ...

class WorkerRequest(_message.Message):
    __slots__ = ("register", "work_request", "lease_acceptance", "heartbeat", "complete", "decline")
    REGISTER_FIELD_NUMBER: _ClassVar[int]
    WORK_REQUEST_FIELD_NUMBER: _ClassVar[int]
    LEASE_ACCEPTANCE_FIELD_NUMBER: _ClassVar[int]
    HEARTBEAT_FIELD_NUMBER: _ClassVar[int]
    COMPLETE_FIELD_NUMBER: _ClassVar[int]
    DECLINE_FIELD_NUMBER: _ClassVar[int]
    register: RegisterWorker
    work_request: WorkRequest
    lease_acceptance: LeaseAcceptance
    heartbeat: Heartbeat
    complete: Complete
    decline: Decline
    def __init__(self, register: _Optional[_Union[RegisterWorker, _Mapping]] = ..., work_request: _Optional[_Union[WorkRequest, _Mapping]] = ..., lease_acceptance: _Optional[_Union[LeaseAcceptance, _Mapping]] = ..., heartbeat: _Optional[_Union[Heartbeat, _Mapping]] = ..., complete: _Optional[_Union[Complete, _Mapping]] = ..., decline: _Optional[_Union[Decline, _Mapping]] = ...) -> None: ...

class WorkerResponse(_message.Message):
    __slots__ = ("challenge", "registration_ack", "task_lease", "no_work", "heartbeat_ack", "cancel", "completion_ack", "error")
    CHALLENGE_FIELD_NUMBER: _ClassVar[int]
    REGISTRATION_ACK_FIELD_NUMBER: _ClassVar[int]
    TASK_LEASE_FIELD_NUMBER: _ClassVar[int]
    NO_WORK_FIELD_NUMBER: _ClassVar[int]
    HEARTBEAT_ACK_FIELD_NUMBER: _ClassVar[int]
    CANCEL_FIELD_NUMBER: _ClassVar[int]
    COMPLETION_ACK_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    challenge: RegistrationChallenge
    registration_ack: RegistrationAck
    task_lease: TaskLease
    no_work: NoWork
    heartbeat_ack: HeartbeatAck
    cancel: CancelLease
    completion_ack: CompletionAck
    error: ProtocolError
    def __init__(self, challenge: _Optional[_Union[RegistrationChallenge, _Mapping]] = ..., registration_ack: _Optional[_Union[RegistrationAck, _Mapping]] = ..., task_lease: _Optional[_Union[TaskLease, _Mapping]] = ..., no_work: _Optional[_Union[NoWork, _Mapping]] = ..., heartbeat_ack: _Optional[_Union[HeartbeatAck, _Mapping]] = ..., cancel: _Optional[_Union[CancelLease, _Mapping]] = ..., completion_ack: _Optional[_Union[CompletionAck, _Mapping]] = ..., error: _Optional[_Union[ProtocolError, _Mapping]] = ...) -> None: ...
