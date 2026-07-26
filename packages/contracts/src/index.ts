export { canonicalJson } from './canonical.js';

// Artifact contracts (JSON Schema).
export type { ArtifactManifest } from './gen/schemas/artifact-manifest.js';
export type { SourceMap } from './gen/schemas/source-map.js';
export type { DeviceProfile } from './gen/schemas/device-profile.js';
export type { EditIr } from './gen/schemas/edit-ir.js';
export type { MediaProxy } from './gen/schemas/media-proxy.js';
export type { MediaAudio } from './gen/schemas/media-audio.js';
export type { MediaLoudnessEnvelope } from './gen/schemas/media-loudness-envelope.js';
export type { MediaReferenceIndex } from './gen/schemas/media-reference-index.js';
export type { MediaFilmstrip } from './gen/schemas/media-filmstrip.js';
export type { MediaAudioPeaks } from './gen/schemas/media-audio-peaks.js';
export type { MediaFrames } from './gen/schemas/media-frames.js';
export type { MediaIngestManifest } from './gen/schemas/media-ingest-manifest.js';
export type { RenderClipManifest } from './gen/schemas/render-clip.js';
export type { SpeechVad } from './gen/schemas/speech-vad.js';
export type { SpeechAsr } from './gen/schemas/speech-asr.js';
export type { SpeechAlignment } from './gen/schemas/speech-alignment.js';
export type { SpeechTranscript } from './gen/schemas/speech-transcript.js';

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
  IngestSourcePayloadV1Schema,
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
