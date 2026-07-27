from clipmill.ipc.v1 import ping_pb2 as _ping_pb2
from clipmill.worker.v1 import worker_pb2 as _worker_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ErrorCode(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    ERROR_CODE_UNSPECIFIED: _ClassVar[ErrorCode]
    ERROR_CODE_INVALID_ARGUMENT: _ClassVar[ErrorCode]
    ERROR_CODE_NOT_FOUND: _ClassVar[ErrorCode]
    ERROR_CODE_CONFLICT: _ClassVar[ErrorCode]
    ERROR_CODE_UNAVAILABLE: _ClassVar[ErrorCode]
    ERROR_CODE_POLICY_DENIED: _ClassVar[ErrorCode]
    ERROR_CODE_INTERNAL: _ClassVar[ErrorCode]

class TaskState(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    TASK_STATE_UNSPECIFIED: _ClassVar[TaskState]
    TASK_STATE_PLANNED: _ClassVar[TaskState]
    TASK_STATE_ADMITTED: _ClassVar[TaskState]
    TASK_STATE_RUNNING: _ClassVar[TaskState]
    TASK_STATE_SUCCEEDED: _ClassVar[TaskState]
    TASK_STATE_RETRYABLE: _ClassVar[TaskState]
    TASK_STATE_FAILED: _ClassVar[TaskState]
    TASK_STATE_CANCELLED: _ClassVar[TaskState]

class JobState(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    JOB_STATE_UNSPECIFIED: _ClassVar[JobState]
    JOB_STATE_PLANNED: _ClassVar[JobState]
    JOB_STATE_RUNNING: _ClassVar[JobState]
    JOB_STATE_SUCCEEDED: _ClassVar[JobState]
    JOB_STATE_FAILED: _ClassVar[JobState]
    JOB_STATE_CANCEL_REQUESTED: _ClassVar[JobState]
    JOB_STATE_CANCELLED: _ClassVar[JobState]
ERROR_CODE_UNSPECIFIED: ErrorCode
ERROR_CODE_INVALID_ARGUMENT: ErrorCode
ERROR_CODE_NOT_FOUND: ErrorCode
ERROR_CODE_CONFLICT: ErrorCode
ERROR_CODE_UNAVAILABLE: ErrorCode
ERROR_CODE_POLICY_DENIED: ErrorCode
ERROR_CODE_INTERNAL: ErrorCode
TASK_STATE_UNSPECIFIED: TaskState
TASK_STATE_PLANNED: TaskState
TASK_STATE_ADMITTED: TaskState
TASK_STATE_RUNNING: TaskState
TASK_STATE_SUCCEEDED: TaskState
TASK_STATE_RETRYABLE: TaskState
TASK_STATE_FAILED: TaskState
TASK_STATE_CANCELLED: TaskState
JOB_STATE_UNSPECIFIED: JobState
JOB_STATE_PLANNED: JobState
JOB_STATE_RUNNING: JobState
JOB_STATE_SUCCEEDED: JobState
JOB_STATE_FAILED: JobState
JOB_STATE_CANCEL_REQUESTED: JobState
JOB_STATE_CANCELLED: JobState

class Request(_message.Message):
    __slots__ = ("request_id", "ping", "health", "create_project", "get_project", "list_projects", "delete_project", "submit_job", "subscribe_task_events", "get_device_profile", "get_job", "list_jobs", "cancel_job", "register_source", "get_source", "list_sources", "create_edit_doc", "apply_edit_command", "get_edit_doc", "snapshot_edit_doc")
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    PING_FIELD_NUMBER: _ClassVar[int]
    HEALTH_FIELD_NUMBER: _ClassVar[int]
    CREATE_PROJECT_FIELD_NUMBER: _ClassVar[int]
    GET_PROJECT_FIELD_NUMBER: _ClassVar[int]
    LIST_PROJECTS_FIELD_NUMBER: _ClassVar[int]
    DELETE_PROJECT_FIELD_NUMBER: _ClassVar[int]
    SUBMIT_JOB_FIELD_NUMBER: _ClassVar[int]
    SUBSCRIBE_TASK_EVENTS_FIELD_NUMBER: _ClassVar[int]
    GET_DEVICE_PROFILE_FIELD_NUMBER: _ClassVar[int]
    GET_JOB_FIELD_NUMBER: _ClassVar[int]
    LIST_JOBS_FIELD_NUMBER: _ClassVar[int]
    CANCEL_JOB_FIELD_NUMBER: _ClassVar[int]
    REGISTER_SOURCE_FIELD_NUMBER: _ClassVar[int]
    GET_SOURCE_FIELD_NUMBER: _ClassVar[int]
    LIST_SOURCES_FIELD_NUMBER: _ClassVar[int]
    CREATE_EDIT_DOC_FIELD_NUMBER: _ClassVar[int]
    APPLY_EDIT_COMMAND_FIELD_NUMBER: _ClassVar[int]
    GET_EDIT_DOC_FIELD_NUMBER: _ClassVar[int]
    SNAPSHOT_EDIT_DOC_FIELD_NUMBER: _ClassVar[int]
    request_id: str
    ping: _ping_pb2.PingRequest
    health: HealthRequest
    create_project: CreateProjectRequest
    get_project: GetProjectRequest
    list_projects: ListProjectsRequest
    delete_project: DeleteProjectRequest
    submit_job: SubmitJobRequest
    subscribe_task_events: SubscribeTaskEventsRequest
    get_device_profile: GetDeviceProfileRequest
    get_job: GetJobRequest
    list_jobs: ListJobsRequest
    cancel_job: CancelJobRequest
    register_source: RegisterSourceRequest
    get_source: GetSourceRequest
    list_sources: ListSourcesRequest
    create_edit_doc: CreateEditDocRequest
    apply_edit_command: ApplyEditCommandRequest
    get_edit_doc: GetEditDocRequest
    snapshot_edit_doc: SnapshotEditDocRequest
    def __init__(self, request_id: _Optional[str] = ..., ping: _Optional[_Union[_ping_pb2.PingRequest, _Mapping]] = ..., health: _Optional[_Union[HealthRequest, _Mapping]] = ..., create_project: _Optional[_Union[CreateProjectRequest, _Mapping]] = ..., get_project: _Optional[_Union[GetProjectRequest, _Mapping]] = ..., list_projects: _Optional[_Union[ListProjectsRequest, _Mapping]] = ..., delete_project: _Optional[_Union[DeleteProjectRequest, _Mapping]] = ..., submit_job: _Optional[_Union[SubmitJobRequest, _Mapping]] = ..., subscribe_task_events: _Optional[_Union[SubscribeTaskEventsRequest, _Mapping]] = ..., get_device_profile: _Optional[_Union[GetDeviceProfileRequest, _Mapping]] = ..., get_job: _Optional[_Union[GetJobRequest, _Mapping]] = ..., list_jobs: _Optional[_Union[ListJobsRequest, _Mapping]] = ..., cancel_job: _Optional[_Union[CancelJobRequest, _Mapping]] = ..., register_source: _Optional[_Union[RegisterSourceRequest, _Mapping]] = ..., get_source: _Optional[_Union[GetSourceRequest, _Mapping]] = ..., list_sources: _Optional[_Union[ListSourcesRequest, _Mapping]] = ..., create_edit_doc: _Optional[_Union[CreateEditDocRequest, _Mapping]] = ..., apply_edit_command: _Optional[_Union[ApplyEditCommandRequest, _Mapping]] = ..., get_edit_doc: _Optional[_Union[GetEditDocRequest, _Mapping]] = ..., snapshot_edit_doc: _Optional[_Union[SnapshotEditDocRequest, _Mapping]] = ...) -> None: ...

class Response(_message.Message):
    __slots__ = ("request_id", "error", "ping", "health", "create_project", "get_project", "list_projects", "delete_project", "submit_job", "task_event", "get_device_profile", "get_job", "list_jobs", "cancel_job", "subscribe_task_events", "register_source", "get_source", "list_sources", "create_edit_doc", "apply_edit_command", "get_edit_doc", "snapshot_edit_doc")
    REQUEST_ID_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    PING_FIELD_NUMBER: _ClassVar[int]
    HEALTH_FIELD_NUMBER: _ClassVar[int]
    CREATE_PROJECT_FIELD_NUMBER: _ClassVar[int]
    GET_PROJECT_FIELD_NUMBER: _ClassVar[int]
    LIST_PROJECTS_FIELD_NUMBER: _ClassVar[int]
    DELETE_PROJECT_FIELD_NUMBER: _ClassVar[int]
    SUBMIT_JOB_FIELD_NUMBER: _ClassVar[int]
    TASK_EVENT_FIELD_NUMBER: _ClassVar[int]
    GET_DEVICE_PROFILE_FIELD_NUMBER: _ClassVar[int]
    GET_JOB_FIELD_NUMBER: _ClassVar[int]
    LIST_JOBS_FIELD_NUMBER: _ClassVar[int]
    CANCEL_JOB_FIELD_NUMBER: _ClassVar[int]
    SUBSCRIBE_TASK_EVENTS_FIELD_NUMBER: _ClassVar[int]
    REGISTER_SOURCE_FIELD_NUMBER: _ClassVar[int]
    GET_SOURCE_FIELD_NUMBER: _ClassVar[int]
    LIST_SOURCES_FIELD_NUMBER: _ClassVar[int]
    CREATE_EDIT_DOC_FIELD_NUMBER: _ClassVar[int]
    APPLY_EDIT_COMMAND_FIELD_NUMBER: _ClassVar[int]
    GET_EDIT_DOC_FIELD_NUMBER: _ClassVar[int]
    SNAPSHOT_EDIT_DOC_FIELD_NUMBER: _ClassVar[int]
    request_id: str
    error: Error
    ping: _ping_pb2.PingResponse
    health: HealthResponse
    create_project: CreateProjectResponse
    get_project: GetProjectResponse
    list_projects: ListProjectsResponse
    delete_project: DeleteProjectResponse
    submit_job: SubmitJobResponse
    task_event: TaskEvent
    get_device_profile: GetDeviceProfileResponse
    get_job: GetJobResponse
    list_jobs: ListJobsResponse
    cancel_job: CancelJobResponse
    subscribe_task_events: SubscribeTaskEventsResponse
    register_source: RegisterSourceResponse
    get_source: GetSourceResponse
    list_sources: ListSourcesResponse
    create_edit_doc: CreateEditDocResponse
    apply_edit_command: ApplyEditCommandResponse
    get_edit_doc: GetEditDocResponse
    snapshot_edit_doc: SnapshotEditDocResponse
    def __init__(self, request_id: _Optional[str] = ..., error: _Optional[_Union[Error, _Mapping]] = ..., ping: _Optional[_Union[_ping_pb2.PingResponse, _Mapping]] = ..., health: _Optional[_Union[HealthResponse, _Mapping]] = ..., create_project: _Optional[_Union[CreateProjectResponse, _Mapping]] = ..., get_project: _Optional[_Union[GetProjectResponse, _Mapping]] = ..., list_projects: _Optional[_Union[ListProjectsResponse, _Mapping]] = ..., delete_project: _Optional[_Union[DeleteProjectResponse, _Mapping]] = ..., submit_job: _Optional[_Union[SubmitJobResponse, _Mapping]] = ..., task_event: _Optional[_Union[TaskEvent, _Mapping]] = ..., get_device_profile: _Optional[_Union[GetDeviceProfileResponse, _Mapping]] = ..., get_job: _Optional[_Union[GetJobResponse, _Mapping]] = ..., list_jobs: _Optional[_Union[ListJobsResponse, _Mapping]] = ..., cancel_job: _Optional[_Union[CancelJobResponse, _Mapping]] = ..., subscribe_task_events: _Optional[_Union[SubscribeTaskEventsResponse, _Mapping]] = ..., register_source: _Optional[_Union[RegisterSourceResponse, _Mapping]] = ..., get_source: _Optional[_Union[GetSourceResponse, _Mapping]] = ..., list_sources: _Optional[_Union[ListSourcesResponse, _Mapping]] = ..., create_edit_doc: _Optional[_Union[CreateEditDocResponse, _Mapping]] = ..., apply_edit_command: _Optional[_Union[ApplyEditCommandResponse, _Mapping]] = ..., get_edit_doc: _Optional[_Union[GetEditDocResponse, _Mapping]] = ..., snapshot_edit_doc: _Optional[_Union[SnapshotEditDocResponse, _Mapping]] = ...) -> None: ...

class Error(_message.Message):
    __slots__ = ("code", "message")
    CODE_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    code: ErrorCode
    message: str
    def __init__(self, code: _Optional[_Union[ErrorCode, str]] = ..., message: _Optional[str] = ...) -> None: ...

class HealthRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class HealthResponse(_message.Message):
    __slots__ = ("daemon_version", "started_unix_millis", "local_lock")
    DAEMON_VERSION_FIELD_NUMBER: _ClassVar[int]
    STARTED_UNIX_MILLIS_FIELD_NUMBER: _ClassVar[int]
    LOCAL_LOCK_FIELD_NUMBER: _ClassVar[int]
    daemon_version: str
    started_unix_millis: int
    local_lock: bool
    def __init__(self, daemon_version: _Optional[str] = ..., started_unix_millis: _Optional[int] = ..., local_lock: _Optional[bool] = ...) -> None: ...

class Project(_message.Message):
    __slots__ = ("project_id", "name", "created_unix_millis")
    PROJECT_ID_FIELD_NUMBER: _ClassVar[int]
    NAME_FIELD_NUMBER: _ClassVar[int]
    CREATED_UNIX_MILLIS_FIELD_NUMBER: _ClassVar[int]
    project_id: str
    name: str
    created_unix_millis: int
    def __init__(self, project_id: _Optional[str] = ..., name: _Optional[str] = ..., created_unix_millis: _Optional[int] = ...) -> None: ...

class CreateProjectRequest(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class CreateProjectResponse(_message.Message):
    __slots__ = ("project",)
    PROJECT_FIELD_NUMBER: _ClassVar[int]
    project: Project
    def __init__(self, project: _Optional[_Union[Project, _Mapping]] = ...) -> None: ...

class GetProjectRequest(_message.Message):
    __slots__ = ("project_id",)
    PROJECT_ID_FIELD_NUMBER: _ClassVar[int]
    project_id: str
    def __init__(self, project_id: _Optional[str] = ...) -> None: ...

class GetProjectResponse(_message.Message):
    __slots__ = ("project",)
    PROJECT_FIELD_NUMBER: _ClassVar[int]
    project: Project
    def __init__(self, project: _Optional[_Union[Project, _Mapping]] = ...) -> None: ...

class ListProjectsRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ListProjectsResponse(_message.Message):
    __slots__ = ("projects",)
    PROJECTS_FIELD_NUMBER: _ClassVar[int]
    projects: _containers.RepeatedCompositeFieldContainer[Project]
    def __init__(self, projects: _Optional[_Iterable[_Union[Project, _Mapping]]] = ...) -> None: ...

class DeleteProjectRequest(_message.Message):
    __slots__ = ("project_id",)
    PROJECT_ID_FIELD_NUMBER: _ClassVar[int]
    project_id: str
    def __init__(self, project_id: _Optional[str] = ...) -> None: ...

class DeleteProjectResponse(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SubmitJobRequest(_message.Message):
    __slots__ = ("project_id", "kind", "payload")
    PROJECT_ID_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    project_id: str
    kind: str
    payload: bytes
    def __init__(self, project_id: _Optional[str] = ..., kind: _Optional[str] = ..., payload: _Optional[bytes] = ...) -> None: ...

class DemoDagPayloadV1(_message.Message):
    __slots__ = ("key_version", "seed")
    KEY_VERSION_FIELD_NUMBER: _ClassVar[int]
    SEED_FIELD_NUMBER: _ClassVar[int]
    key_version: str
    seed: bytes
    def __init__(self, key_version: _Optional[str] = ..., seed: _Optional[bytes] = ...) -> None: ...

class ProbeSourcePayloadV1(_message.Message):
    __slots__ = ("key_version", "source_id")
    KEY_VERSION_FIELD_NUMBER: _ClassVar[int]
    SOURCE_ID_FIELD_NUMBER: _ClassVar[int]
    key_version: str
    source_id: str
    def __init__(self, key_version: _Optional[str] = ..., source_id: _Optional[str] = ...) -> None: ...

class IngestSourcePayloadV1(_message.Message):
    __slots__ = ("key_version", "source_id")
    KEY_VERSION_FIELD_NUMBER: _ClassVar[int]
    SOURCE_ID_FIELD_NUMBER: _ClassVar[int]
    key_version: str
    source_id: str
    def __init__(self, key_version: _Optional[str] = ..., source_id: _Optional[str] = ...) -> None: ...

class RenderClipPayloadV1(_message.Message):
    __slots__ = ("key_version", "doc_id", "ir_artifact_id", "source_attestation", "gates_passed", "ai_assistance")
    KEY_VERSION_FIELD_NUMBER: _ClassVar[int]
    DOC_ID_FIELD_NUMBER: _ClassVar[int]
    IR_ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    SOURCE_ATTESTATION_FIELD_NUMBER: _ClassVar[int]
    GATES_PASSED_FIELD_NUMBER: _ClassVar[int]
    AI_ASSISTANCE_FIELD_NUMBER: _ClassVar[int]
    key_version: str
    doc_id: str
    ir_artifact_id: str
    source_attestation: str
    gates_passed: _containers.RepeatedScalarFieldContainer[str]
    ai_assistance: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, key_version: _Optional[str] = ..., doc_id: _Optional[str] = ..., ir_artifact_id: _Optional[str] = ..., source_attestation: _Optional[str] = ..., gates_passed: _Optional[_Iterable[str]] = ..., ai_assistance: _Optional[_Iterable[str]] = ...) -> None: ...

class TranscribeSourcePayloadV1(_message.Message):
    __slots__ = ("key_version", "source_id", "language", "detection")
    KEY_VERSION_FIELD_NUMBER: _ClassVar[int]
    SOURCE_ID_FIELD_NUMBER: _ClassVar[int]
    LANGUAGE_FIELD_NUMBER: _ClassVar[int]
    DETECTION_FIELD_NUMBER: _ClassVar[int]
    key_version: str
    source_id: str
    language: str
    detection: SpeechDetectionV1
    def __init__(self, key_version: _Optional[str] = ..., source_id: _Optional[str] = ..., language: _Optional[str] = ..., detection: _Optional[_Union[SpeechDetectionV1, _Mapping]] = ...) -> None: ...

class SpeechDetectionV1(_message.Message):
    __slots__ = ("threshold", "min_speech_ticks", "min_silence_ticks", "speech_pad_ticks")
    THRESHOLD_FIELD_NUMBER: _ClassVar[int]
    MIN_SPEECH_TICKS_FIELD_NUMBER: _ClassVar[int]
    MIN_SILENCE_TICKS_FIELD_NUMBER: _ClassVar[int]
    SPEECH_PAD_TICKS_FIELD_NUMBER: _ClassVar[int]
    threshold: float
    min_speech_ticks: int
    min_silence_ticks: int
    speech_pad_ticks: int
    def __init__(self, threshold: _Optional[float] = ..., min_speech_ticks: _Optional[int] = ..., min_silence_ticks: _Optional[int] = ..., speech_pad_ticks: _Optional[int] = ...) -> None: ...

class SpeechStagePayloadV1(_message.Message):
    __slots__ = ("key_version", "stage", "source_fingerprint", "audio_artifact_id", "detection", "recognition", "alignment")
    KEY_VERSION_FIELD_NUMBER: _ClassVar[int]
    STAGE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FINGERPRINT_FIELD_NUMBER: _ClassVar[int]
    AUDIO_ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    DETECTION_FIELD_NUMBER: _ClassVar[int]
    RECOGNITION_FIELD_NUMBER: _ClassVar[int]
    ALIGNMENT_FIELD_NUMBER: _ClassVar[int]
    key_version: str
    stage: str
    source_fingerprint: str
    audio_artifact_id: str
    detection: SpeechDetectionV1
    recognition: SpeechRecognitionV1
    alignment: SpeechAlignmentV1
    def __init__(self, key_version: _Optional[str] = ..., stage: _Optional[str] = ..., source_fingerprint: _Optional[str] = ..., audio_artifact_id: _Optional[str] = ..., detection: _Optional[_Union[SpeechDetectionV1, _Mapping]] = ..., recognition: _Optional[_Union[SpeechRecognitionV1, _Mapping]] = ..., alignment: _Optional[_Union[SpeechAlignmentV1, _Mapping]] = ...) -> None: ...

class SpeechRecognitionV1(_message.Message):
    __slots__ = ("language", "conditioned_on_previous")
    LANGUAGE_FIELD_NUMBER: _ClassVar[int]
    CONDITIONED_ON_PREVIOUS_FIELD_NUMBER: _ClassVar[int]
    language: str
    conditioned_on_previous: bool
    def __init__(self, language: _Optional[str] = ..., conditioned_on_previous: _Optional[bool] = ...) -> None: ...

class SpeechAlignmentV1(_message.Message):
    __slots__ = ("min_score",)
    MIN_SCORE_FIELD_NUMBER: _ClassVar[int]
    min_score: float
    def __init__(self, min_score: _Optional[float] = ...) -> None: ...

class DetectShotsPayloadV1(_message.Message):
    __slots__ = ("key_version", "source_id", "detection")
    KEY_VERSION_FIELD_NUMBER: _ClassVar[int]
    SOURCE_ID_FIELD_NUMBER: _ClassVar[int]
    DETECTION_FIELD_NUMBER: _ClassVar[int]
    key_version: str
    source_id: str
    detection: ShotDetectionV1
    def __init__(self, key_version: _Optional[str] = ..., source_id: _Optional[str] = ..., detection: _Optional[_Union[ShotDetectionV1, _Mapping]] = ...) -> None: ...

class ShotDetectionV1(_message.Message):
    __slots__ = ("threshold", "min_shot_ticks", "analysis_height")
    THRESHOLD_FIELD_NUMBER: _ClassVar[int]
    MIN_SHOT_TICKS_FIELD_NUMBER: _ClassVar[int]
    ANALYSIS_HEIGHT_FIELD_NUMBER: _ClassVar[int]
    threshold: float
    min_shot_ticks: int
    analysis_height: int
    def __init__(self, threshold: _Optional[float] = ..., min_shot_ticks: _Optional[int] = ..., analysis_height: _Optional[int] = ...) -> None: ...

class ShotsStagePayloadV1(_message.Message):
    __slots__ = ("key_version", "stage", "source_fingerprint", "proxy_artifact_id", "detection", "decoder_bom")
    KEY_VERSION_FIELD_NUMBER: _ClassVar[int]
    STAGE_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FINGERPRINT_FIELD_NUMBER: _ClassVar[int]
    PROXY_ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    DETECTION_FIELD_NUMBER: _ClassVar[int]
    DECODER_BOM_FIELD_NUMBER: _ClassVar[int]
    key_version: str
    stage: str
    source_fingerprint: str
    proxy_artifact_id: str
    detection: ShotDetectionV1
    decoder_bom: str
    def __init__(self, key_version: _Optional[str] = ..., stage: _Optional[str] = ..., source_fingerprint: _Optional[str] = ..., proxy_artifact_id: _Optional[str] = ..., detection: _Optional[_Union[ShotDetectionV1, _Mapping]] = ..., decoder_bom: _Optional[str] = ...) -> None: ...

class IndexTranscriptPayloadV1(_message.Message):
    __slots__ = ("key_version", "source_id")
    KEY_VERSION_FIELD_NUMBER: _ClassVar[int]
    SOURCE_ID_FIELD_NUMBER: _ClassVar[int]
    key_version: str
    source_id: str
    def __init__(self, key_version: _Optional[str] = ..., source_id: _Optional[str] = ...) -> None: ...

class IndexStagePayloadV1(_message.Message):
    __slots__ = ("key_version", "stage", "transcript_artifact_id", "shots_artifact_id")
    KEY_VERSION_FIELD_NUMBER: _ClassVar[int]
    STAGE_FIELD_NUMBER: _ClassVar[int]
    TRANSCRIPT_ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    SHOTS_ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    key_version: str
    stage: str
    transcript_artifact_id: str
    shots_artifact_id: str
    def __init__(self, key_version: _Optional[str] = ..., stage: _Optional[str] = ..., transcript_artifact_id: _Optional[str] = ..., shots_artifact_id: _Optional[str] = ...) -> None: ...

class DiscoverCandidatesPayloadV1(_message.Message):
    __slots__ = ("key_version", "source_id", "duration")
    KEY_VERSION_FIELD_NUMBER: _ClassVar[int]
    SOURCE_ID_FIELD_NUMBER: _ClassVar[int]
    DURATION_FIELD_NUMBER: _ClassVar[int]
    key_version: str
    source_id: str
    duration: ClipDurationV1
    def __init__(self, key_version: _Optional[str] = ..., source_id: _Optional[str] = ..., duration: _Optional[_Union[ClipDurationV1, _Mapping]] = ...) -> None: ...

class ClipDurationV1(_message.Message):
    __slots__ = ("min_ticks", "max_ticks")
    MIN_TICKS_FIELD_NUMBER: _ClassVar[int]
    MAX_TICKS_FIELD_NUMBER: _ClassVar[int]
    min_ticks: int
    max_ticks: int
    def __init__(self, min_ticks: _Optional[int] = ..., max_ticks: _Optional[int] = ...) -> None: ...

class DiscoverStagePayloadV1(_message.Message):
    __slots__ = ("key_version", "stage", "index_artifact_id", "transcript_artifact_id", "loudness_artifact_id", "duration", "exploration_floor")
    KEY_VERSION_FIELD_NUMBER: _ClassVar[int]
    STAGE_FIELD_NUMBER: _ClassVar[int]
    INDEX_ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    TRANSCRIPT_ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    LOUDNESS_ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    DURATION_FIELD_NUMBER: _ClassVar[int]
    EXPLORATION_FLOOR_FIELD_NUMBER: _ClassVar[int]
    key_version: str
    stage: str
    index_artifact_id: str
    transcript_artifact_id: str
    loudness_artifact_id: str
    duration: ClipDurationV1
    exploration_floor: int
    def __init__(self, key_version: _Optional[str] = ..., stage: _Optional[str] = ..., index_artifact_id: _Optional[str] = ..., transcript_artifact_id: _Optional[str] = ..., loudness_artifact_id: _Optional[str] = ..., duration: _Optional[_Union[ClipDurationV1, _Mapping]] = ..., exploration_floor: _Optional[int] = ...) -> None: ...

class DeviceProfilePayloadV1(_message.Message):
    __slots__ = ("key_version", "hardware_fingerprint", "measurement_generation")
    KEY_VERSION_FIELD_NUMBER: _ClassVar[int]
    HARDWARE_FINGERPRINT_FIELD_NUMBER: _ClassVar[int]
    MEASUREMENT_GENERATION_FIELD_NUMBER: _ClassVar[int]
    key_version: str
    hardware_fingerprint: str
    measurement_generation: int
    def __init__(self, key_version: _Optional[str] = ..., hardware_fingerprint: _Optional[str] = ..., measurement_generation: _Optional[int] = ...) -> None: ...

class SubmitJobResponse(_message.Message):
    __slots__ = ("job_id", "job")
    JOB_ID_FIELD_NUMBER: _ClassVar[int]
    JOB_FIELD_NUMBER: _ClassVar[int]
    job_id: str
    job: Job
    def __init__(self, job_id: _Optional[str] = ..., job: _Optional[_Union[Job, _Mapping]] = ...) -> None: ...

class SubscribeTaskEventsRequest(_message.Message):
    __slots__ = ("project_id", "job_id", "after_event_id")
    PROJECT_ID_FIELD_NUMBER: _ClassVar[int]
    JOB_ID_FIELD_NUMBER: _ClassVar[int]
    AFTER_EVENT_ID_FIELD_NUMBER: _ClassVar[int]
    project_id: str
    job_id: str
    after_event_id: int
    def __init__(self, project_id: _Optional[str] = ..., job_id: _Optional[str] = ..., after_event_id: _Optional[int] = ...) -> None: ...

class SubscribeTaskEventsResponse(_message.Message):
    __slots__ = ("current_event_id",)
    CURRENT_EVENT_ID_FIELD_NUMBER: _ClassVar[int]
    current_event_id: int
    def __init__(self, current_event_id: _Optional[int] = ...) -> None: ...

class TaskEvent(_message.Message):
    __slots__ = ("job_id", "task_id", "state", "progress", "wait_reason", "at_unix_millis", "event_id", "attempt", "failure_class")
    JOB_ID_FIELD_NUMBER: _ClassVar[int]
    TASK_ID_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    PROGRESS_FIELD_NUMBER: _ClassVar[int]
    WAIT_REASON_FIELD_NUMBER: _ClassVar[int]
    AT_UNIX_MILLIS_FIELD_NUMBER: _ClassVar[int]
    EVENT_ID_FIELD_NUMBER: _ClassVar[int]
    ATTEMPT_FIELD_NUMBER: _ClassVar[int]
    FAILURE_CLASS_FIELD_NUMBER: _ClassVar[int]
    job_id: str
    task_id: str
    state: TaskState
    progress: _worker_pb2.ProgressUnits
    wait_reason: str
    at_unix_millis: int
    event_id: int
    attempt: int
    failure_class: _worker_pb2.FailureClass
    def __init__(self, job_id: _Optional[str] = ..., task_id: _Optional[str] = ..., state: _Optional[_Union[TaskState, str]] = ..., progress: _Optional[_Union[_worker_pb2.ProgressUnits, _Mapping]] = ..., wait_reason: _Optional[str] = ..., at_unix_millis: _Optional[int] = ..., event_id: _Optional[int] = ..., attempt: _Optional[int] = ..., failure_class: _Optional[_Union[_worker_pb2.FailureClass, str]] = ...) -> None: ...

class Job(_message.Message):
    __slots__ = ("job_id", "project_id", "kind", "state", "created_unix_millis", "updated_unix_millis", "tasks", "output_artifact_ids", "failure_class", "failure_detail")
    JOB_ID_FIELD_NUMBER: _ClassVar[int]
    PROJECT_ID_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    CREATED_UNIX_MILLIS_FIELD_NUMBER: _ClassVar[int]
    UPDATED_UNIX_MILLIS_FIELD_NUMBER: _ClassVar[int]
    TASKS_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_ARTIFACT_IDS_FIELD_NUMBER: _ClassVar[int]
    FAILURE_CLASS_FIELD_NUMBER: _ClassVar[int]
    FAILURE_DETAIL_FIELD_NUMBER: _ClassVar[int]
    job_id: str
    project_id: str
    kind: str
    state: JobState
    created_unix_millis: int
    updated_unix_millis: int
    tasks: _containers.RepeatedCompositeFieldContainer[Task]
    output_artifact_ids: _containers.RepeatedScalarFieldContainer[str]
    failure_class: _worker_pb2.FailureClass
    failure_detail: str
    def __init__(self, job_id: _Optional[str] = ..., project_id: _Optional[str] = ..., kind: _Optional[str] = ..., state: _Optional[_Union[JobState, str]] = ..., created_unix_millis: _Optional[int] = ..., updated_unix_millis: _Optional[int] = ..., tasks: _Optional[_Iterable[_Union[Task, _Mapping]]] = ..., output_artifact_ids: _Optional[_Iterable[str]] = ..., failure_class: _Optional[_Union[_worker_pb2.FailureClass, str]] = ..., failure_detail: _Optional[str] = ...) -> None: ...

class Task(_message.Message):
    __slots__ = ("task_id", "kind", "state", "attempt", "max_attempts", "progress", "wait_reason", "output_artifact_id")
    TASK_ID_FIELD_NUMBER: _ClassVar[int]
    KIND_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    ATTEMPT_FIELD_NUMBER: _ClassVar[int]
    MAX_ATTEMPTS_FIELD_NUMBER: _ClassVar[int]
    PROGRESS_FIELD_NUMBER: _ClassVar[int]
    WAIT_REASON_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    task_id: str
    kind: str
    state: TaskState
    attempt: int
    max_attempts: int
    progress: _worker_pb2.ProgressUnits
    wait_reason: str
    output_artifact_id: str
    def __init__(self, task_id: _Optional[str] = ..., kind: _Optional[str] = ..., state: _Optional[_Union[TaskState, str]] = ..., attempt: _Optional[int] = ..., max_attempts: _Optional[int] = ..., progress: _Optional[_Union[_worker_pb2.ProgressUnits, _Mapping]] = ..., wait_reason: _Optional[str] = ..., output_artifact_id: _Optional[str] = ...) -> None: ...

class GetJobRequest(_message.Message):
    __slots__ = ("job_id",)
    JOB_ID_FIELD_NUMBER: _ClassVar[int]
    job_id: str
    def __init__(self, job_id: _Optional[str] = ...) -> None: ...

class GetJobResponse(_message.Message):
    __slots__ = ("job",)
    JOB_FIELD_NUMBER: _ClassVar[int]
    job: Job
    def __init__(self, job: _Optional[_Union[Job, _Mapping]] = ...) -> None: ...

class ListJobsRequest(_message.Message):
    __slots__ = ("project_id",)
    PROJECT_ID_FIELD_NUMBER: _ClassVar[int]
    project_id: str
    def __init__(self, project_id: _Optional[str] = ...) -> None: ...

class ListJobsResponse(_message.Message):
    __slots__ = ("jobs",)
    JOBS_FIELD_NUMBER: _ClassVar[int]
    jobs: _containers.RepeatedCompositeFieldContainer[Job]
    def __init__(self, jobs: _Optional[_Iterable[_Union[Job, _Mapping]]] = ...) -> None: ...

class CancelJobRequest(_message.Message):
    __slots__ = ("job_id",)
    JOB_ID_FIELD_NUMBER: _ClassVar[int]
    job_id: str
    def __init__(self, job_id: _Optional[str] = ...) -> None: ...

class CancelJobResponse(_message.Message):
    __slots__ = ("job",)
    JOB_FIELD_NUMBER: _ClassVar[int]
    job: Job
    def __init__(self, job: _Optional[_Union[Job, _Mapping]] = ...) -> None: ...

class Source(_message.Message):
    __slots__ = ("source_id", "project_id", "absolute_path", "byte_size", "sample_sha256", "source_fingerprint", "source_map_artifact_id", "created_unix_millis")
    SOURCE_ID_FIELD_NUMBER: _ClassVar[int]
    PROJECT_ID_FIELD_NUMBER: _ClassVar[int]
    ABSOLUTE_PATH_FIELD_NUMBER: _ClassVar[int]
    BYTE_SIZE_FIELD_NUMBER: _ClassVar[int]
    SAMPLE_SHA256_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FINGERPRINT_FIELD_NUMBER: _ClassVar[int]
    SOURCE_MAP_ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    CREATED_UNIX_MILLIS_FIELD_NUMBER: _ClassVar[int]
    source_id: str
    project_id: str
    absolute_path: str
    byte_size: int
    sample_sha256: str
    source_fingerprint: str
    source_map_artifact_id: str
    created_unix_millis: int
    def __init__(self, source_id: _Optional[str] = ..., project_id: _Optional[str] = ..., absolute_path: _Optional[str] = ..., byte_size: _Optional[int] = ..., sample_sha256: _Optional[str] = ..., source_fingerprint: _Optional[str] = ..., source_map_artifact_id: _Optional[str] = ..., created_unix_millis: _Optional[int] = ...) -> None: ...

class RegisterSourceRequest(_message.Message):
    __slots__ = ("project_id", "absolute_path")
    PROJECT_ID_FIELD_NUMBER: _ClassVar[int]
    ABSOLUTE_PATH_FIELD_NUMBER: _ClassVar[int]
    project_id: str
    absolute_path: str
    def __init__(self, project_id: _Optional[str] = ..., absolute_path: _Optional[str] = ...) -> None: ...

class RegisterSourceResponse(_message.Message):
    __slots__ = ("source", "observation_cache_hit")
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    OBSERVATION_CACHE_HIT_FIELD_NUMBER: _ClassVar[int]
    source: Source
    observation_cache_hit: bool
    def __init__(self, source: _Optional[_Union[Source, _Mapping]] = ..., observation_cache_hit: _Optional[bool] = ...) -> None: ...

class GetSourceRequest(_message.Message):
    __slots__ = ("source_id",)
    SOURCE_ID_FIELD_NUMBER: _ClassVar[int]
    source_id: str
    def __init__(self, source_id: _Optional[str] = ...) -> None: ...

class GetSourceResponse(_message.Message):
    __slots__ = ("source",)
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    source: Source
    def __init__(self, source: _Optional[_Union[Source, _Mapping]] = ...) -> None: ...

class ListSourcesRequest(_message.Message):
    __slots__ = ("project_id",)
    PROJECT_ID_FIELD_NUMBER: _ClassVar[int]
    project_id: str
    def __init__(self, project_id: _Optional[str] = ...) -> None: ...

class ListSourcesResponse(_message.Message):
    __slots__ = ("sources",)
    SOURCES_FIELD_NUMBER: _ClassVar[int]
    sources: _containers.RepeatedCompositeFieldContainer[Source]
    def __init__(self, sources: _Optional[_Iterable[_Union[Source, _Mapping]]] = ...) -> None: ...

class GetDeviceProfileRequest(_message.Message):
    __slots__ = ("remeasure",)
    REMEASURE_FIELD_NUMBER: _ClassVar[int]
    remeasure: bool
    def __init__(self, remeasure: _Optional[bool] = ...) -> None: ...

class GetDeviceProfileResponse(_message.Message):
    __slots__ = ("artifact_id", "profile_json")
    ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    PROFILE_JSON_FIELD_NUMBER: _ClassVar[int]
    artifact_id: str
    profile_json: str
    def __init__(self, artifact_id: _Optional[str] = ..., profile_json: _Optional[str] = ...) -> None: ...

class EditDoc(_message.Message):
    __slots__ = ("doc_id", "project_id", "revision", "document_json", "created_unix_millis", "updated_unix_millis")
    DOC_ID_FIELD_NUMBER: _ClassVar[int]
    PROJECT_ID_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    DOCUMENT_JSON_FIELD_NUMBER: _ClassVar[int]
    CREATED_UNIX_MILLIS_FIELD_NUMBER: _ClassVar[int]
    UPDATED_UNIX_MILLIS_FIELD_NUMBER: _ClassVar[int]
    doc_id: str
    project_id: str
    revision: int
    document_json: str
    created_unix_millis: int
    updated_unix_millis: int
    def __init__(self, doc_id: _Optional[str] = ..., project_id: _Optional[str] = ..., revision: _Optional[int] = ..., document_json: _Optional[str] = ..., created_unix_millis: _Optional[int] = ..., updated_unix_millis: _Optional[int] = ...) -> None: ...

class CreateEditDocRequest(_message.Message):
    __slots__ = ("project_id", "document_json")
    PROJECT_ID_FIELD_NUMBER: _ClassVar[int]
    DOCUMENT_JSON_FIELD_NUMBER: _ClassVar[int]
    project_id: str
    document_json: str
    def __init__(self, project_id: _Optional[str] = ..., document_json: _Optional[str] = ...) -> None: ...

class CreateEditDocResponse(_message.Message):
    __slots__ = ("doc",)
    DOC_FIELD_NUMBER: _ClassVar[int]
    doc: EditDoc
    def __init__(self, doc: _Optional[_Union[EditDoc, _Mapping]] = ...) -> None: ...

class ApplyEditCommandRequest(_message.Message):
    __slots__ = ("doc_id", "expected_revision", "command_json")
    DOC_ID_FIELD_NUMBER: _ClassVar[int]
    EXPECTED_REVISION_FIELD_NUMBER: _ClassVar[int]
    COMMAND_JSON_FIELD_NUMBER: _ClassVar[int]
    doc_id: str
    expected_revision: int
    command_json: str
    def __init__(self, doc_id: _Optional[str] = ..., expected_revision: _Optional[int] = ..., command_json: _Optional[str] = ...) -> None: ...

class ApplyEditCommandResponse(_message.Message):
    __slots__ = ("doc", "inverse_command_json")
    DOC_FIELD_NUMBER: _ClassVar[int]
    INVERSE_COMMAND_JSON_FIELD_NUMBER: _ClassVar[int]
    doc: EditDoc
    inverse_command_json: str
    def __init__(self, doc: _Optional[_Union[EditDoc, _Mapping]] = ..., inverse_command_json: _Optional[str] = ...) -> None: ...

class GetEditDocRequest(_message.Message):
    __slots__ = ("doc_id",)
    DOC_ID_FIELD_NUMBER: _ClassVar[int]
    doc_id: str
    def __init__(self, doc_id: _Optional[str] = ...) -> None: ...

class GetEditDocResponse(_message.Message):
    __slots__ = ("doc",)
    DOC_FIELD_NUMBER: _ClassVar[int]
    doc: EditDoc
    def __init__(self, doc: _Optional[_Union[EditDoc, _Mapping]] = ...) -> None: ...

class SnapshotEditDocRequest(_message.Message):
    __slots__ = ("doc_id",)
    DOC_ID_FIELD_NUMBER: _ClassVar[int]
    doc_id: str
    def __init__(self, doc_id: _Optional[str] = ...) -> None: ...

class SnapshotEditDocResponse(_message.Message):
    __slots__ = ("artifact_id", "revision")
    ARTIFACT_ID_FIELD_NUMBER: _ClassVar[int]
    REVISION_FIELD_NUMBER: _ClassVar[int]
    artifact_id: str
    revision: int
    def __init__(self, artifact_id: _Optional[str] = ..., revision: _Optional[int] = ...) -> None: ...
