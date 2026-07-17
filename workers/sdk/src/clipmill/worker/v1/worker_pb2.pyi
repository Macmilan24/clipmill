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
    __slots__ = ("worker_id", "family", "capabilities", "protocol_version", "backend", "max_memory_bytes", "public_key", "signature")
    WORKER_ID_FIELD_NUMBER: _ClassVar[int]
    FAMILY_FIELD_NUMBER: _ClassVar[int]
    CAPABILITIES_FIELD_NUMBER: _ClassVar[int]
    PROTOCOL_VERSION_FIELD_NUMBER: _ClassVar[int]
    BACKEND_FIELD_NUMBER: _ClassVar[int]
    MAX_MEMORY_BYTES_FIELD_NUMBER: _ClassVar[int]
    PUBLIC_KEY_FIELD_NUMBER: _ClassVar[int]
    SIGNATURE_FIELD_NUMBER: _ClassVar[int]
    worker_id: str
    family: str
    capabilities: _containers.RepeatedScalarFieldContainer[str]
    protocol_version: str
    backend: str
    max_memory_bytes: int
    public_key: bytes
    signature: bytes
    def __init__(self, worker_id: _Optional[str] = ..., family: _Optional[str] = ..., capabilities: _Optional[_Iterable[str]] = ..., protocol_version: _Optional[str] = ..., backend: _Optional[str] = ..., max_memory_bytes: _Optional[int] = ..., public_key: _Optional[bytes] = ..., signature: _Optional[bytes] = ...) -> None: ...

class TaskLease(_message.Message):
    __slots__ = ("task_id", "lease_id", "kind", "payload", "heartbeat_interval_ms", "lease_ttl_ms")
    TASK_ID_FIELD_NUMBER: _ClassVar[int]
    LEASE_ID_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    HEARTBEAT_INTERVAL_MS_FIELD_NUMBER: _ClassVar[int]
    LEASE_TTL_MS_FIELD_NUMBER: _ClassVar[int]
    task_id: str
    lease_id: str
    kind: str
    payload: bytes
    heartbeat_interval_ms: int
    lease_ttl_ms: int
    def __init__(self, task_id: _Optional[str] = ..., lease_id: _Optional[str] = ..., kind: _Optional[str] = ..., payload: _Optional[bytes] = ..., heartbeat_interval_ms: _Optional[int] = ..., lease_ttl_ms: _Optional[int] = ...) -> None: ...

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
    __slots__ = ("lease_id", "outcome", "artifact_ids", "failure_class", "detail")
    LEASE_ID_FIELD_NUMBER: _ClassVar[int]
    OUTCOME_FIELD_NUMBER: _ClassVar[int]
    ARTIFACT_IDS_FIELD_NUMBER: _ClassVar[int]
    FAILURE_CLASS_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    lease_id: str
    outcome: TaskOutcome
    artifact_ids: _containers.RepeatedScalarFieldContainer[str]
    failure_class: FailureClass
    detail: str
    def __init__(self, lease_id: _Optional[str] = ..., outcome: _Optional[_Union[TaskOutcome, str]] = ..., artifact_ids: _Optional[_Iterable[str]] = ..., failure_class: _Optional[_Union[FailureClass, str]] = ..., detail: _Optional[str] = ...) -> None: ...
