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
    __slots__ = ("request_id", "ping", "health", "create_project", "get_project", "list_projects", "delete_project", "submit_job", "subscribe_task_events", "get_device_profile", "get_job", "list_jobs", "cancel_job")
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
    def __init__(self, request_id: _Optional[str] = ..., ping: _Optional[_Union[_ping_pb2.PingRequest, _Mapping]] = ..., health: _Optional[_Union[HealthRequest, _Mapping]] = ..., create_project: _Optional[_Union[CreateProjectRequest, _Mapping]] = ..., get_project: _Optional[_Union[GetProjectRequest, _Mapping]] = ..., list_projects: _Optional[_Union[ListProjectsRequest, _Mapping]] = ..., delete_project: _Optional[_Union[DeleteProjectRequest, _Mapping]] = ..., submit_job: _Optional[_Union[SubmitJobRequest, _Mapping]] = ..., subscribe_task_events: _Optional[_Union[SubscribeTaskEventsRequest, _Mapping]] = ..., get_device_profile: _Optional[_Union[GetDeviceProfileRequest, _Mapping]] = ..., get_job: _Optional[_Union[GetJobRequest, _Mapping]] = ..., list_jobs: _Optional[_Union[ListJobsRequest, _Mapping]] = ..., cancel_job: _Optional[_Union[CancelJobRequest, _Mapping]] = ...) -> None: ...

class Response(_message.Message):
    __slots__ = ("request_id", "error", "ping", "health", "create_project", "get_project", "list_projects", "delete_project", "submit_job", "task_event", "get_device_profile", "get_job", "list_jobs", "cancel_job", "subscribe_task_events")
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
    def __init__(self, request_id: _Optional[str] = ..., error: _Optional[_Union[Error, _Mapping]] = ..., ping: _Optional[_Union[_ping_pb2.PingResponse, _Mapping]] = ..., health: _Optional[_Union[HealthResponse, _Mapping]] = ..., create_project: _Optional[_Union[CreateProjectResponse, _Mapping]] = ..., get_project: _Optional[_Union[GetProjectResponse, _Mapping]] = ..., list_projects: _Optional[_Union[ListProjectsResponse, _Mapping]] = ..., delete_project: _Optional[_Union[DeleteProjectResponse, _Mapping]] = ..., submit_job: _Optional[_Union[SubmitJobResponse, _Mapping]] = ..., task_event: _Optional[_Union[TaskEvent, _Mapping]] = ..., get_device_profile: _Optional[_Union[GetDeviceProfileResponse, _Mapping]] = ..., get_job: _Optional[_Union[GetJobResponse, _Mapping]] = ..., list_jobs: _Optional[_Union[ListJobsResponse, _Mapping]] = ..., cancel_job: _Optional[_Union[CancelJobResponse, _Mapping]] = ..., subscribe_task_events: _Optional[_Union[SubscribeTaskEventsResponse, _Mapping]] = ...) -> None: ...

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
