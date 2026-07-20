export { canonicalJson } from './canonical.js';

// Artifact contracts (JSON Schema).
export type { ArtifactManifest } from './gen/schemas/artifact-manifest.js';
export type { SourceMap } from './gen/schemas/source-map.js';
export type { DeviceProfile } from './gen/schemas/device-profile.js';

// IPC control plane.
export { PingRequestSchema, PingResponseSchema } from './gen/proto/clipmill/ipc/v1/ping_pb.js';
export {
  RequestSchema,
  ResponseSchema,
  ErrorSchema,
  HealthRequestSchema,
  HealthResponseSchema,
  ProjectSchema,
  CreateProjectRequestSchema,
  CreateProjectResponseSchema,
  GetProjectRequestSchema,
  GetProjectResponseSchema,
  ListProjectsRequestSchema,
  ListProjectsResponseSchema,
  DeleteProjectRequestSchema,
  DeleteProjectResponseSchema,
  SubmitJobRequestSchema,
  SubmitJobResponseSchema,
  DemoDagPayloadV1Schema,
  ProbeSourcePayloadV1Schema,
  DeviceProfilePayloadV1Schema,
  SubscribeTaskEventsRequestSchema,
  SubscribeTaskEventsResponseSchema,
  TaskEventSchema,
  TaskState,
  TaskStateSchema,
  JobSchema,
  JobState,
  JobStateSchema,
  TaskSchema,
  GetJobRequestSchema,
  GetJobResponseSchema,
  ListJobsRequestSchema,
  ListJobsResponseSchema,
  CancelJobRequestSchema,
  CancelJobResponseSchema,
  SourceSchema,
  RegisterSourceRequestSchema,
  RegisterSourceResponseSchema,
  GetSourceRequestSchema,
  GetSourceResponseSchema,
  ListSourcesRequestSchema,
  ListSourcesResponseSchema,
  GetDeviceProfileRequestSchema,
  GetDeviceProfileResponseSchema,
} from './gen/proto/clipmill/ipc/v1/daemon_pb.js';

// Worker protocol.
export {
  CapabilityDescriptorSchema,
  TaskLeaseSchema,
  ProgressUnitsSchema,
  HeartbeatSchema,
  DeclineSchema,
  CompleteSchema,
  StagedOutputSchema,
  RegistrationChallengeSchema,
  RegisterWorkerSchema,
  RegistrationAckSchema,
  WorkRequestSchema,
  NoWorkSchema,
  LeaseAcceptanceSchema,
  HeartbeatAckSchema,
  CancelLeaseSchema,
  CompletionAckSchema,
  ProtocolErrorSchema,
  WorkerRequestSchema,
  WorkerResponseSchema,
  DeclineReason,
  TaskOutcome,
  FailureClass,
} from './gen/proto/clipmill/worker/v1/worker_pb.js';

// Shared-memory descriptors.
export {
  BufferDescriptorSchema,
  MapRequestSchema,
  MapAcknowledgementSchema,
  DataType,
  TransportType,
} from './gen/proto/clipmill/shm/v1/shm_pb.js';

// Time primitives.
export {
  TimebaseSchema,
  TicksSchema,
  IntervalSchema,
} from './gen/proto/clipmill/time/v1/time_pb.js';
