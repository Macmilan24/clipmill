use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Write,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clipmill_artifacts::{
    ArtifactPath, ArtifactRecipe, NetworkPolicy, PrepareOutcome, Producer, RecipeSpec, Timebase,
};
use clipmill_contracts::proto::{
    ipc::v1::{
        self, AnalysisStagePayloadV1, AnalyzeSourcePayloadV1, ClipDurationV1, DetectShotsPayloadV1,
        DeviceProfilePayloadV1, DiscoverCandidatesPayloadV1, DiscoverStagePayloadV1,
        IndexStagePayloadV1, IndexTranscriptPayloadV1, IngestSourcePayloadV1, JobState,
        ProbeSourcePayloadV1, RankCandidatesPayloadV1, RankStagePayloadV1, ShotsStagePayloadV1,
        SkippedStageV1, SpeechAlignmentV1, SpeechDetectionV1, SpeechRecognitionV1,
        SpeechStagePayloadV1, TranscribeSourcePayloadV1,
    },
    worker::v1::{FailureClass, ProgressUnits},
};
use clipmill_core::{ArtifactId, JobId, LeaseId, ProjectId, Sha256Digest, TaskId};
use prost::Message;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use tokio::{
    sync::{Notify, broadcast, oneshot},
    task::{JoinHandle, JoinSet},
    time::{MissedTickBehavior, interval},
};

use crate::{
    analysis,
    artifacts::ArtifactHandle,
    db::{DbHandle, StoreError},
    device::{DeviceProfiler, VerifiedDeviceProfile, verify_profile},
    discovery, evidence,
    media::{self, MediaRunner, ProgressSlot},
    ranking,
    render::{self, RenderContext},
    sources::{SourceInspector, SourceProbeError},
    speech,
};

/// Key version every speech stage payload carries, so a worker can refuse a
/// payload the daemon never meant for it.
pub(crate) const SPEECH_STAGE_KEY_VERSION: &str = "clipmill.speech-stage.v1";

/// Key version the shot-detection stage payload carries, for the same reason.
pub(crate) const SHOTS_STAGE_KEY_VERSION: &str = "clipmill.shots-stage.v1";

/// Key version the evidence-index stage payload carries.
pub(crate) const INDEX_STAGE_KEY_VERSION: &str = "clipmill.index-stage.v1";

/// Key version the discovery stage payload carries.
pub(crate) const DISCOVER_STAGE_KEY_VERSION: &str = "clipmill.discover-stage.v1";

/// Key version the ranking stage payload carries.
pub(crate) const RANK_STAGE_KEY_VERSION: &str = "clipmill.rank-stage.v1";

/// Key version the analysis fan-in payload carries.
pub(crate) const ANALYSIS_STAGE_KEY_VERSION: &str = "clipmill.analysis-stage.v1";

/// Key versions of the two job requests the analyze DAG re-submits internally.
/// They live here rather than beside the other job kinds' because the analyze
/// plan writes these payloads itself: the probe and the ingest fan-out inside a
/// DAG have to be byte-identical to the ones a standalone job would submit, or
/// they would not share a cache entry with them.
pub(crate) const PROBE_SOURCE_KEY_VERSION: &str = "clipmill.probe-source.v1";
pub(crate) const INGEST_SOURCE_KEY_VERSION: &str = "clipmill.ingest-source.v1";

/// The one implementation of shot detection. Unlike the speech stages there is
/// nothing to select between: the stage runs no model, so there is no cost to
/// measure and no accelerated candidate to prefer. Recorded on the task all the
/// same, because `producer.implementation` is what makes two producers' output
/// distinguishable if a second one ever appears.
pub(crate) const SHOTS_IMPLEMENTATION: &str = "clipmill-worker-shots@0.1.0+pyscenedetect-content";

pub(crate) const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
pub(crate) const LEASE_TTL: Duration = Duration::from_secs(15);
pub(crate) const DEVICE_PROFILE_KEY_VERSION: &str = "clipmill.device-profile.v1";
pub(crate) const SYSTEM_PROJECT_ID: &str = "prj_00000000000000000000000000";
const SCHEDULER_TICK: Duration = Duration::from_millis(100);
const MAX_BUILTIN_TASKS: usize = 4;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResourceDeclaration {
    pub cpu_threads: u32,
    pub ram_bytes: u64,
    pub accelerator_class: String,
    pub vram_bytes: u64,
    pub disk_bytes: u64,
    pub network_policy: String,
    pub thermal_class: String,
    pub determinism_class: String,
    pub checkpoint_support: bool,
    pub preemption_cost: u32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ResourceCapacity {
    pub cpu_threads: u32,
    pub ram_bytes: u64,
    pub disk_bytes: u64,
    pub accelerator_mask: u32,
    pub vram_bytes: u64,
}

impl ResourceCapacity {
    #[cfg(test)]
    pub(crate) fn w4_builtin() -> Self {
        let cpu_threads = std::thread::available_parallelism()
            .map_or(1, std::num::NonZeroUsize::get)
            .min(MAX_BUILTIN_TASKS);
        Self {
            cpu_threads: u32::try_from(cpu_threads).unwrap_or(u32::MAX),
            ram_bytes: 512 * 1024 * 1024,
            disk_bytes: 512 * 1024 * 1024,
            accelerator_mask: 0,
            vram_bytes: 0,
        }
    }

    pub(crate) fn measured(logical_cores: u32, available_memory_bytes: u64) -> Self {
        let maximum_threads = u32::try_from(MAX_BUILTIN_TASKS).unwrap_or(u32::MAX);
        Self {
            cpu_threads: logical_cores.clamp(1, maximum_threads),
            ram_bytes: available_memory_bytes
                .saturating_mul(3)
                .checked_div(4)
                .unwrap_or(available_memory_bytes)
                .max(64 * 1024 * 1024),
            disk_bytes: 512 * 1024 * 1024,
            accelerator_mask: 0,
            vram_bytes: 0,
        }
    }

    pub(crate) fn with_available_backends(mut self, backends: &BTreeSet<String>) -> Self {
        self.accelerator_mask = backends
            .iter()
            .filter_map(|backend| accelerator_bit(backend))
            .fold(0, std::ops::BitOr::bitor);
        self
    }

    fn reserve(&mut self, resources: &ResourceDeclaration) -> bool {
        let accelerator_available = if resources.accelerator_class.is_empty() {
            resources.vram_bytes == 0
        } else {
            accelerator_bit(&resources.accelerator_class)
                .is_some_and(|bit| self.accelerator_mask & bit != 0)
                && resources.vram_bytes <= self.vram_bytes
        };
        if accelerator_available
            && resources.network_policy == "local-lock"
            && resources.cpu_threads <= self.cpu_threads
            && resources.ram_bytes <= self.ram_bytes
            && resources.disk_bytes <= self.disk_bytes
        {
            self.cpu_threads -= resources.cpu_threads;
            self.ram_bytes -= resources.ram_bytes;
            self.disk_bytes -= resources.disk_bytes;
            self.vram_bytes -= resources.vram_bytes;
            true
        } else {
            false
        }
    }

    fn release(&mut self, resources: &ResourceDeclaration) {
        self.cpu_threads = self.cpu_threads.saturating_add(resources.cpu_threads);
        self.ram_bytes = self.ram_bytes.saturating_add(resources.ram_bytes);
        self.disk_bytes = self.disk_bytes.saturating_add(resources.disk_bytes);
        self.vram_bytes = self.vram_bytes.saturating_add(resources.vram_bytes);
    }
}

pub(crate) fn accelerator_bit(backend: &str) -> Option<u32> {
    match backend {
        "videotoolbox" => Some(1 << 0),
        "vaapi" => Some(1 << 1),
        "cuda" => Some(1 << 2),
        "vulkan" => Some(1 << 3),
        "metal" => Some(1 << 4),
        _ => None,
    }
}

impl ResourceDeclaration {
    fn demo() -> Self {
        Self {
            cpu_threads: 1,
            ram_bytes: 1024 * 1024,
            accelerator_class: String::new(),
            vram_bytes: 0,
            disk_bytes: 1024 * 1024,
            network_policy: "local-lock".to_owned(),
            thermal_class: "light".to_owned(),
            determinism_class: "deterministic".to_owned(),
            checkpoint_support: false,
            preemption_cost: 1,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TaskSpec {
    pub task_id: String,
    pub ordinal: u32,
    pub kind: String,
    pub input_kinds: Vec<String>,
    pub output_kind: String,
    pub payload: Vec<u8>,
    pub dependencies: Vec<String>,
    /// Artifacts this task reads that no task in this plan produced, named as
    /// content addresses.
    ///
    /// A task's inputs are ordinarily the outputs of the tasks it depends on,
    /// which is the whole of the story for a stage in the middle of a DAG. A
    /// stage submitted on its own is the other case: what it reads was
    /// published by an earlier job, so there is no dependency to carry it. The
    /// plan says so here, and the daemon delivers both together.
    ///
    /// This exists rather than the address travelling in the stage payload for
    /// two reasons. A worker may only open what its lease names, so an address
    /// known only to the payload named an artifact the worker was forbidden to
    /// read. And the payload is hashed into the artifact key: an address
    /// present standalone and necessarily absent inside a DAG — where the
    /// artifact does not exist yet when the plan is written — would give one
    /// observation two addresses.
    ///
    /// Delivered before the dependency outputs, in this order, because input
    /// order is part of the key. A stage reached by both routes therefore has
    /// to declare them so the two agree, which
    /// `the_two_routes_deliver_one_input_order` holds it to.
    pub input_artifact_ids: Vec<String>,
    pub resources: ResourceDeclaration,
    pub implementation: String,
    pub max_attempts: u32,
    pub is_final: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct JobPlan {
    pub job_id: String,
    pub project_id: String,
    pub kind: String,
    pub source_id: Option<String>,
    pub payload: Vec<u8>,
    pub created_unix_millis: u64,
    pub tasks: Vec<TaskSpec>,
}

impl JobPlan {
    pub(crate) fn demo(project_id: &ProjectId, payload: Vec<u8>, now: u64) -> Self {
        let job_id = JobId::new().to_string();
        let seed = TaskId::new().to_string();
        let left = TaskId::new().to_string();
        let right = TaskId::new().to_string();
        let join = TaskId::new().to_string();
        let task_payload = payload.clone();
        let task = |task_id: String,
                    ordinal: u32,
                    kind: &str,
                    input_kinds: Vec<&str>,
                    output_kind: &str,
                    dependencies: Vec<String>,
                    is_final: bool| TaskSpec {
            task_id,
            ordinal,
            kind: kind.to_owned(),
            input_kinds: input_kinds.into_iter().map(str::to_owned).collect(),
            output_kind: output_kind.to_owned(),
            payload: task_payload.clone(),
            dependencies,
            input_artifact_ids: Vec::new(),
            resources: ResourceDeclaration::demo(),
            implementation: "builtin-demo@1.0.0".to_owned(),
            max_attempts: 3,
            is_final,
        };
        Self {
            job_id,
            project_id: project_id.to_string(),
            kind: "demo-dag".to_owned(),
            source_id: None,
            payload,
            created_unix_millis: now,
            tasks: vec![
                task(
                    seed.clone(),
                    0,
                    "demo-seed",
                    Vec::new(),
                    "evidence.demo.seed.v1",
                    Vec::new(),
                    false,
                ),
                task(
                    left.clone(),
                    1,
                    "demo-left",
                    vec!["evidence.demo.seed.v1"],
                    "evidence.demo.left.v1",
                    vec![seed.clone()],
                    false,
                ),
                task(
                    right.clone(),
                    2,
                    "demo-right",
                    vec!["evidence.demo.seed.v1"],
                    "evidence.demo.right.v1",
                    vec![seed],
                    false,
                ),
                task(
                    join,
                    3,
                    "demo-join",
                    vec!["evidence.demo.left.v1", "evidence.demo.right.v1"],
                    "evidence.demo.final.v1",
                    vec![left, right],
                    true,
                ),
            ],
        }
    }

    pub(crate) fn probe_source(
        project_id: &ProjectId,
        source_id: String,
        payload: Vec<u8>,
        now: u64,
    ) -> Self {
        let job_id = JobId::new().to_string();
        Self {
            job_id,
            project_id: project_id.to_string(),
            kind: "probe-source".to_owned(),
            source_id: Some(source_id),
            payload: payload.clone(),
            created_unix_millis: now,
            tasks: vec![TaskSpec {
                task_id: TaskId::new().to_string(),
                ordinal: 0,
                kind: "probe-source".to_owned(),
                input_kinds: Vec::new(),
                output_kind: "evidence.source_map.v1".to_owned(),
                payload,
                dependencies: Vec::new(),
                input_artifact_ids: Vec::new(),
                resources: ResourceDeclaration {
                    cpu_threads: 1,
                    ram_bytes: 64 * 1024 * 1024,
                    accelerator_class: String::new(),
                    vram_bytes: 0,
                    disk_bytes: 32 * 1024 * 1024,
                    network_policy: "local-lock".to_owned(),
                    thermal_class: "light".to_owned(),
                    determinism_class: "deterministic".to_owned(),
                    checkpoint_support: false,
                    preemption_cost: 1,
                },
                implementation: "ffprobe-8.1.2+clipmill-map-v1".to_owned(),
                max_attempts: 3,
                is_final: true,
            }],
        }
    }

    /// The W11 ingest fan-out (book ch. 12): decode the source video once
    /// into the proxy, render the PCM diets straight from the source's audio,
    /// then hang every other derivative off an already-committed artifact.
    /// The single final task is the fan-in manifest whose recipe inputs are
    /// all children, because the job store roots exactly one artifact per job
    /// and garbage collection walks recipe inputs from the roots.
    pub(crate) fn ingest_source(
        project_id: &ProjectId,
        source_id: String,
        payload: Vec<u8>,
        has_video: bool,
        has_audio: bool,
        now: u64,
    ) -> Result<Self, &'static str> {
        let mut builder = IngestPlanBuilder::new(payload);
        ingest_fan_out(&mut builder, has_video, has_audio)?;
        let (payload, tasks, _, _) = builder.finish_with_manifest(true);
        Ok(Self {
            job_id: JobId::new().to_string(),
            project_id: project_id.to_string(),
            kind: "ingest-source".to_owned(),
            source_id: Some(source_id),
            payload,
            created_unix_millis: now,
            tasks,
        })
    }
}

/// What an analysis needs to know about its source before it can be planned.
///
/// The stream inventory is here rather than measured inside because it is a
/// measured fact: the fan-out's shape depends on it, and the probe that
/// establishes it has to have run already.
#[derive(Clone, Copy, Debug)]
pub(crate) struct AnalyzeSource<'a> {
    pub source_id: &'a str,
    pub source_fingerprint: &'a str,
    pub has_video: bool,
    pub has_audio: bool,
}

/// One shot-detection task, shared by the standalone job and the analyze DAG so
/// the two key identically.
fn shots_task(task_id: String, payload: &ShotsStagePayloadV1) -> TaskSpec {
    TaskSpec {
        task_id,
        ordinal: 0,
        kind: "detect-shots".to_owned(),
        input_kinds: Vec::new(),
        output_kind: "evidence.shots.v1".to_owned(),
        payload: payload.encode_to_vec(),
        dependencies: Vec::new(),
        input_artifact_ids: Vec::new(),
        resources: ResourceDeclaration {
            // One decoder process beside the detector, which is why this is not
            // the single thread the speech stages declare.
            cpu_threads: 2,
            ram_bytes: 512 * 1024 * 1024,
            // Nothing to accelerate: the work is a decode and some array
            // arithmetic, so any machine will do.
            accelerator_class: String::new(),
            vram_bytes: 0,
            disk_bytes: 128 * 1024 * 1024,
            network_policy: "local-lock".to_owned(),
            thermal_class: "sustained".to_owned(),
            determinism_class: "deterministic".to_owned(),
            checkpoint_support: false,
            preemption_cost: 2,
        },
        implementation: SHOTS_IMPLEMENTATION.to_owned(),
        max_attempts: 3,
        is_final: false,
    }
}

/// One model-free builtin stage inside a larger plan.
///
/// The three of them differ in what they read and how much they hold in memory
/// at once, and in nothing else: no model, no accelerator, no network, and one
/// thread. What varies is a field here; what does not is the same for all three.
struct Builtin<'a> {
    kind: &'a str,
    output_kind: &'a str,
    implementation: &'a str,
    input_kinds: Vec<String>,
    dependencies: Vec<String>,
    payload: Vec<u8>,
    ram_mib: u64,
}

fn builtin_task(task_id: String, builtin: Builtin<'_>) -> TaskSpec {
    let Builtin {
        kind,
        output_kind,
        implementation,
        input_kinds,
        dependencies,
        payload,
        ram_mib,
    } = builtin;
    TaskSpec {
        task_id,
        ordinal: 0,
        kind: kind.to_owned(),
        input_kinds,
        output_kind: output_kind.to_owned(),
        payload,
        dependencies,
        input_artifact_ids: Vec::new(),
        resources: ResourceDeclaration {
            cpu_threads: 1,
            ram_bytes: ram_mib * 1024 * 1024,
            accelerator_class: String::new(),
            vram_bytes: 0,
            disk_bytes: 128 * 1024 * 1024,
            network_policy: "local-lock".to_owned(),
            thermal_class: "light".to_owned(),
            determinism_class: "deterministic".to_owned(),
            checkpoint_support: false,
            preemption_cost: 1,
        },
        implementation: implementation.to_owned(),
        max_attempts: 3,
        is_final: false,
    }
}

/// Where the speech chain finds the rendition it reads.
///
/// Two routes, both real, and the one thing that must hold across them is that
/// each stage ends up with the same lease inputs in the same order — because
/// that order is part of the artifact key, and a transcript derived by one route
/// has to be the same artifact as one derived by the other.
#[derive(Clone, Copy)]
enum SpeechAudioRoute<'a> {
    /// A rendition an earlier job published, declared by content address.
    Published(&'a str),
    /// A rendition a task in this same plan will produce.
    Planned(&'a str),
}

/// Everything the speech chain needs that is not the device's answer.
struct SpeechChain<'a> {
    audio: SpeechAudioRoute<'a>,
    source_fingerprint: &'a str,
    language: &'a str,
    detection: Option<SpeechDetectionV1>,
    /// False when the chain is the middle of a longer plan.
    transcript_is_final: bool,
}

/// The four speech tasks and the id of the one that publishes the transcript.
struct PlannedChain {
    tasks: Vec<TaskSpec>,
    transcript_task_id: String,
}

/// The W15 chain (book ch. 13): voice activity, then recognition, then forced
/// alignment, then the assembly that fuses them.
///
/// Strictly serial, and not for want of trying: each stage's input is the
/// previous stage's output. What the split buys is not parallelism but blast
/// radius — re-pinning the recognizer invalidates transcripts and leaves voice
/// activity alone, and a failed alignment costs word timing without costing
/// anyone the text.
///
/// Each stage carries only the parameters it reads, because the payload is
/// hashed into the artifact key: a recognizer payload carrying the voice
/// activity threshold would make re-tuning voice activity invalidate every
/// cached transcript, including ones whose inputs never changed. For the same
/// reason the audio's address is not in the payload at all — it would be
/// present on one route and necessarily absent on the other.
#[allow(
    clippy::too_many_lines,
    reason = "four task specifications, which is what the chain is"
)]
fn speech_chain(
    chain: &SpeechChain<'_>,
    models: &crate::models::ModelRegistry,
    bindings: &crate::selection::Bindings,
) -> PlannedChain {
    let stage_payload = |stage: &str, fill: &dyn Fn(&mut SpeechStagePayloadV1)| {
        let mut payload = SpeechStagePayloadV1 {
            key_version: SPEECH_STAGE_KEY_VERSION.to_owned(),
            stage: stage.to_owned(),
            source_fingerprint: chain.source_fingerprint.to_owned(),
            ..SpeechStagePayloadV1::default()
        };
        fill(&mut payload);
        payload.encode_to_vec()
    };

    let vad = TaskId::new().to_string();
    let asr = TaskId::new().to_string();
    let align = TaskId::new().to_string();
    let transcript = TaskId::new().to_string();

    // The audio, first in every leased stage's input list on both routes.
    let (audio_declared, audio_dependency) = match chain.audio {
        SpeechAudioRoute::Published(artifact_id) => (vec![artifact_id.to_owned()], Vec::new()),
        SpeechAudioRoute::Planned(task_id) => (Vec::new(), vec![task_id.to_owned()]),
    };
    let audio_input_kinds = audio_dependency
        .iter()
        .map(|_| "media.audio_16k.v1".to_owned())
        .collect::<Vec<_>>();

    // The device's answer, frozen into the plan. Every leased stage below
    // records the implementation this machine chose, which is what its artifact
    // key is computed from and what the scheduler routes it by. Re-measuring the
    // device later moves the next plan and nothing already published.
    let leased = |task_id: String,
                  ordinal: u32,
                  kind: &str,
                  previous: Option<(&str, &str)>,
                  output_kind: &str,
                  payload: Vec<u8>| {
        let implementation = speech_implementation(kind, bindings);
        let mut input_kinds = audio_input_kinds.clone();
        let mut dependencies = audio_dependency.clone();
        if let Some((previous_task, previous_kind)) = previous {
            input_kinds.push(previous_kind.to_owned());
            dependencies.push(previous_task.to_owned());
        }
        TaskSpec {
            task_id,
            ordinal,
            kind: kind.to_owned(),
            input_kinds,
            output_kind: output_kind.to_owned(),
            payload,
            dependencies,
            input_artifact_ids: audio_declared.clone(),
            resources: speech_resources(implementation, models, 1),
            implementation: implementation.name.to_owned(),
            max_attempts: 3,
            is_final: false,
        }
    };

    let detection = chain.detection;
    let language = chain.language.to_owned();
    let tasks = vec![
        leased(
            vad.clone(),
            0,
            "speech-vad",
            None,
            "speech.vad.v1",
            stage_payload("speech-vad", &|payload| {
                payload.detection = detection;
            }),
        ),
        leased(
            asr.clone(),
            1,
            "speech-asr",
            Some((vad.as_str(), "speech.vad.v1")),
            "speech.asr.v1",
            stage_payload("speech-asr", &|payload| {
                payload.recognition = Some(SpeechRecognitionV1 {
                    language: language.clone(),
                    conditioned_on_previous: false,
                });
            }),
        ),
        leased(
            align.clone(),
            2,
            "speech-align",
            Some((asr.as_str(), "speech.asr.v1")),
            "speech.alignment.v1",
            stage_payload("speech-align", &|payload| {
                payload.alignment = Some(SpeechAlignmentV1 { min_score: 0.0 });
            }),
        ),
        TaskSpec {
            task_id: transcript.clone(),
            ordinal: 3,
            kind: speech::KIND_TRANSCRIPT.to_owned(),
            // Named in dependency order, but assembly matches its inputs by the
            // kind each artifact declares rather than by position. It reads no
            // audio: the three documents already carry everything it fuses.
            input_kinds: vec![
                "speech.vad.v1".to_owned(),
                "speech.asr.v1".to_owned(),
                "speech.alignment.v1".to_owned(),
            ],
            output_kind: "speech.transcript.v1".to_owned(),
            payload: stage_payload(speech::KIND_TRANSCRIPT, &|_| {}),
            dependencies: vec![vad, asr, align],
            input_artifact_ids: Vec::new(),
            resources: ResourceDeclaration {
                cpu_threads: 1,
                ram_bytes: 128 * 1024 * 1024,
                accelerator_class: String::new(),
                vram_bytes: 0,
                disk_bytes: 64 * 1024 * 1024,
                network_policy: "local-lock".to_owned(),
                thermal_class: "light".to_owned(),
                determinism_class: "deterministic".to_owned(),
                checkpoint_support: false,
                preemption_cost: 1,
            },
            implementation: speech::IMPLEMENTATION.to_owned(),
            max_attempts: 3,
            is_final: chain.transcript_is_final,
        },
    ];
    PlannedChain {
        tasks,
        transcript_task_id: transcript,
    }
}

/// The ingest derivatives a later stage in the same plan reads.
///
/// Each is absent for the same reason the derivative is: a source with no video
/// has no proxy, and a stage that needed one is skipped with that stated rather
/// than planned against nothing.
#[derive(Clone)]
struct IngestHandles {
    proxy: Option<DerivativeHandle>,
    audio_16k: Option<DerivativeHandle>,
    loudness: Option<DerivativeHandle>,
}

/// The W11 fan-out itself, shared by the ingest job and the analyze DAG.
///
/// Shared rather than reimplemented: every task this adds is keyed from its
/// kind, its payload, and its inputs, so the two callers produce byte-identical
/// keys and a source already ingested costs an analysis nothing. A second copy
/// of this list would be a second set of keys the first day somebody edited one
/// of them.
fn ingest_fan_out(
    builder: &mut IngestPlanBuilder,
    has_video: bool,
    has_audio: bool,
) -> Result<IngestHandles, &'static str> {
    if !has_video && !has_audio {
        return Err("source carries neither video nor audio");
    }
    {
        let proxy = has_video.then(|| {
            builder.derivative(
                media::KIND_PROXY,
                "media.proxy.v1",
                "ffmpeg-8.1.2+clipmill-proxy-v1",
                ingest_resources(2, 256, 512),
                &[],
            )
        });
        let audio_16k = has_audio.then(|| {
            builder.derivative(
                media::KIND_AUDIO_16K,
                "media.audio_16k.v1",
                "ffmpeg-8.1.2+clipmill-audio-v1",
                ingest_resources(1, 64, 128),
                &[],
            )
        });
        let audio_48k = has_audio.then(|| {
            builder.derivative(
                media::KIND_AUDIO_48K,
                "media.audio_48k.v1",
                "ffmpeg-8.1.2+clipmill-audio-v1",
                ingest_resources(1, 64, 256),
                &[],
            )
        });
        builder.derivative(
            media::KIND_REFERENCE_INDEX,
            "media.reference_index.v1",
            "ffprobe-8.1.2+clipmill-refindex-v1",
            ingest_resources(1, 128, 64),
            &[],
        );
        let loudness = audio_48k.as_ref().map(|audio_48k| {
            builder.derivative(
                media::KIND_LOUDNESS,
                "media.loudness_envelope.v1",
                "ffmpeg-8.1.2+clipmill-loudness-v1",
                ingest_resources(1, 64, 32),
                std::slice::from_ref(audio_48k),
            )
        });
        if let Some(audio_16k) = &audio_16k {
            builder.derivative(
                media::KIND_AUDIO_PEAKS,
                "media.audio_peaks.v1",
                "clipmill-audio-peaks@1.0.0",
                ingest_resources(1, 64, 32),
                std::slice::from_ref(audio_16k),
            );
        }
        if let Some(proxy) = &proxy {
            builder.derivative(
                media::KIND_FILMSTRIP,
                "media.filmstrip.v1",
                "ffmpeg-8.1.2+clipmill-filmstrip-v1",
                ingest_resources(1, 128, 128),
                std::slice::from_ref(proxy),
            );
            builder.derivative(
                media::KIND_FRAMES,
                "media.frames.v1",
                "ffmpeg-8.1.2+clipmill-frames-v1",
                ingest_resources(1, 128, 256),
                std::slice::from_ref(proxy),
            );
        }
        Ok(IngestHandles {
            proxy,
            audio_16k,
            loudness,
        })
    }
}

impl JobPlan {
    /// The W13 render (book ch. 17). One task, because the encode is one
    /// FFmpeg graph and splitting it would mean a joiner that has to prove it
    /// preserved timestamps, colour, and audio continuity — a Phase 2 trade
    /// worth making only when profiling asks for it.
    ///
    /// The document arrives as an immutable snapshot artifact rather than a
    /// document id, so the render is pinned to the revision the user approved
    /// and an edit in flight cannot change what was asked for.
    pub(crate) fn render_clip(project_id: &ProjectId, payload: Vec<u8>, now: u64) -> Self {
        Self {
            job_id: JobId::new().to_string(),
            project_id: project_id.to_string(),
            kind: "render-clip".to_owned(),
            source_id: None,
            payload: payload.clone(),
            created_unix_millis: now,
            tasks: vec![TaskSpec {
                task_id: TaskId::new().to_string(),
                ordinal: 0,
                kind: render::KIND_RENDER_CLIP.to_owned(),
                input_kinds: Vec::new(),
                output_kind: "render.clip.v1".to_owned(),
                payload,
                dependencies: Vec::new(),
                input_artifact_ids: Vec::new(),
                resources: ResourceDeclaration {
                    cpu_threads: 2,
                    ram_bytes: 512 * 1024 * 1024,
                    accelerator_class: String::new(),
                    vram_bytes: 0,
                    disk_bytes: 512 * 1024 * 1024,
                    network_policy: "local-lock".to_owned(),
                    thermal_class: "sustained".to_owned(),
                    determinism_class: "deterministic".to_owned(),
                    checkpoint_support: false,
                    preemption_cost: 4,
                },
                implementation: "ffmpeg-8.1.2+clipmill-render-v1".to_owned(),
                max_attempts: 3,
                is_final: true,
            }],
        }
    }

    /// The W15 speech chain (book ch. 13): voice activity, then recognition,
    /// then forced alignment, then the assembly that fuses them.
    ///
    /// Strictly serial, and not for want of trying: each stage's input is the
    /// previous stage's output. What the split buys is not parallelism but
    /// blast radius — re-pinning the recognizer invalidates transcripts and
    /// leaves voice activity alone, and a failed alignment costs word timing
    /// without costing anyone the text.
    ///
    /// Each stage carries only the parameters it reads, because the payload is
    /// hashed into the artifact key: a recognizer payload carrying the voice
    /// activity threshold would make re-tuning voice activity invalidate every
    /// cached transcript, including ones whose inputs never changed.
    pub(crate) fn transcribe_source(
        project_id: &ProjectId,
        source_id: String,
        audio: SpeechAudio<'_>,
        request: &TranscribeSourcePayloadV1,
        models: &crate::models::ModelRegistry,
        bindings: &crate::selection::Bindings,
        now: u64,
    ) -> Self {
        let chain = speech_chain(
            &SpeechChain {
                audio: SpeechAudioRoute::Published(audio.artifact_id),
                source_fingerprint: audio.source_fingerprint,
                language: request.language.as_str(),
                detection: request.detection,
                transcript_is_final: true,
            },
            models,
            bindings,
        );
        let tasks = chain.tasks;

        Self {
            job_id: JobId::new().to_string(),
            project_id: project_id.to_string(),
            kind: "transcribe-source".to_owned(),
            source_id: Some(source_id),
            payload: request.encode_to_vec(),
            created_unix_millis: now,
            tasks,
        }
    }

    /// Shot detection over the proxy ingest already derived (book ch. 13).
    ///
    /// One task, not a chain. There is nothing to fan out: the stage reads one
    /// artifact, runs no model, and publishes one document.
    ///
    /// The decoder's build identity is written into the payload rather than
    /// left to the worker, because the payload is hashed into the artifact key
    /// and the decoder is a versioned input to the result. Re-pinning FFmpeg
    /// therefore invalidates shot detections and nothing else — which is the
    /// correct blast radius, since a different build can hand the detector
    /// different pixels.
    pub(crate) fn detect_shots(
        project_id: &ProjectId,
        source_id: String,
        proxy: ShotsProxy<'_>,
        request: &DetectShotsPayloadV1,
        decoder_bom: &str,
        now: u64,
    ) -> Self {
        let mut task = shots_task(
            TaskId::new().to_string(),
            &ShotsStagePayloadV1 {
                key_version: SHOTS_STAGE_KEY_VERSION.to_owned(),
                stage: "detect-shots".to_owned(),
                source_fingerprint: proxy.source_fingerprint.to_owned(),
                detection: request.detection,
                decoder_bom: decoder_bom.to_owned(),
            },
        );
        // The proxy was published by the ingest job, so no dependency in this
        // plan carries it and the plan declares the address instead.
        task.input_artifact_ids = vec![proxy.artifact_id.to_owned()];
        task.is_final = true;
        Self {
            job_id: JobId::new().to_string(),
            project_id: project_id.to_string(),
            kind: "detect-shots".to_owned(),
            source_id: Some(source_id),
            payload: request.encode_to_vec(),
            created_unix_millis: now,
            tasks: vec![task],
        }
    }

    /// The evidence index over a published transcript (book ch. 14).
    ///
    /// One builtin task, and it names its inputs in its payload rather than
    /// through dependencies. A task's input artifacts are the outputs of the
    /// tasks it depends on, and this job depends on nothing — both documents
    /// were published by earlier jobs. Content addresses are safe to hash into
    /// a key: unlike a path, the same address means the same bytes anywhere.
    pub(crate) fn index_transcript(
        project_id: &ProjectId,
        source_id: String,
        evidence_inputs: EvidenceInputs<'_>,
        request: &IndexTranscriptPayloadV1,
        now: u64,
    ) -> Self {
        let mut declared = vec![evidence_inputs.transcript.to_owned()];
        if let Some(shots) = evidence_inputs.shots {
            declared.push(shots.to_owned());
        }
        let payload = IndexStagePayloadV1 {
            key_version: INDEX_STAGE_KEY_VERSION.to_owned(),
            stage: evidence::KIND_INDEX.to_owned(),
        };
        Self {
            job_id: JobId::new().to_string(),
            project_id: project_id.to_string(),
            kind: evidence::KIND_INDEX.to_owned(),
            source_id: Some(source_id),
            payload: request.encode_to_vec(),
            created_unix_millis: now,
            tasks: vec![TaskSpec {
                task_id: TaskId::new().to_string(),
                ordinal: 0,
                kind: evidence::KIND_INDEX.to_owned(),
                // Empty because nothing in this plan produces them: the list
                // describes dependencies, and the addresses below are what the
                // stage actually reads.
                input_kinds: Vec::new(),
                output_kind: "index.transcript.v1".to_owned(),
                payload: payload.encode_to_vec(),
                dependencies: Vec::new(),
                input_artifact_ids: declared,
                resources: ResourceDeclaration {
                    cpu_threads: 1,
                    ram_bytes: 256 * 1024 * 1024,
                    accelerator_class: String::new(),
                    vram_bytes: 0,
                    disk_bytes: 64 * 1024 * 1024,
                    network_policy: "local-lock".to_owned(),
                    thermal_class: "light".to_owned(),
                    determinism_class: "deterministic".to_owned(),
                    checkpoint_support: false,
                    preemption_cost: 1,
                },
                implementation: evidence::IMPLEMENTATION.to_owned(),
                max_attempts: 3,
                is_final: true,
            }],
        }
    }

    /// The proposer mesh over a published index (book ch. 15).
    ///
    /// One builtin task, naming its three documents in its payload for the
    /// same reason the index does: all of them were published by earlier jobs,
    /// and a task's inputs are the outputs of the tasks it depends on.
    pub(crate) fn discover_candidates(
        project_id: &ProjectId,
        source_id: String,
        discovery_inputs: DiscoveryInputs<'_>,
        request: &DiscoverCandidatesPayloadV1,
        now: u64,
    ) -> Self {
        let mut declared = vec![
            discovery_inputs.index.to_owned(),
            discovery_inputs.transcript.to_owned(),
        ];
        if let Some(loudness) = discovery_inputs.loudness {
            declared.push(loudness.to_owned());
        }
        let payload = DiscoverStagePayloadV1 {
            key_version: DISCOVER_STAGE_KEY_VERSION.to_owned(),
            stage: discovery::KIND_DISCOVER.to_owned(),
            duration: request.duration.or(Some(ClipDurationV1 {
                min_ticks: 0,
                max_ticks: 0,
            })),
            // Zero means the daemon's default, so a caller with no opinion
            // does not have to have one.
            exploration_floor: 0,
        };
        Self {
            job_id: JobId::new().to_string(),
            project_id: project_id.to_string(),
            kind: discovery::KIND_DISCOVER.to_owned(),
            source_id: Some(source_id),
            payload: request.encode_to_vec(),
            created_unix_millis: now,
            tasks: vec![TaskSpec {
                task_id: TaskId::new().to_string(),
                ordinal: 0,
                kind: discovery::KIND_DISCOVER.to_owned(),
                input_kinds: Vec::new(),
                output_kind: "discovery.candidates.v1".to_owned(),
                payload: payload.encode_to_vec(),
                dependencies: Vec::new(),
                input_artifact_ids: declared,
                resources: ResourceDeclaration {
                    cpu_threads: 1,
                    ram_bytes: 512 * 1024 * 1024,
                    accelerator_class: String::new(),
                    vram_bytes: 0,
                    disk_bytes: 128 * 1024 * 1024,
                    network_policy: "local-lock".to_owned(),
                    thermal_class: "light".to_owned(),
                    determinism_class: "deterministic".to_owned(),
                    checkpoint_support: false,
                    preemption_cost: 1,
                },
                implementation: discovery::IMPLEMENTATION.to_owned(),
                max_attempts: 3,
                is_final: true,
            }],
        }
    }

    /// The ranking baseline over a searched cohort (book ch. 16).
    ///
    /// One builtin task naming its three documents in its payload, for the
    /// same reason the two stages before it do: all of them were published by
    /// earlier jobs, and a task's inputs are the outputs of the tasks it
    /// depends on.
    pub(crate) fn rank_candidates(
        project_id: &ProjectId,
        source_id: String,
        ranking_inputs: RankingJobInputs<'_>,
        request: &RankCandidatesPayloadV1,
        now: u64,
    ) -> Self {
        let payload = RankStagePayloadV1 {
            key_version: RANK_STAGE_KEY_VERSION.to_owned(),
            stage: ranking::KIND_RANK.to_owned(),
            count: request.count,
            diversity_milli: request.diversity_milli,
        };
        Self {
            job_id: JobId::new().to_string(),
            project_id: project_id.to_string(),
            kind: ranking::KIND_RANK.to_owned(),
            source_id: Some(source_id),
            payload: request.encode_to_vec(),
            created_unix_millis: now,
            tasks: vec![TaskSpec {
                task_id: TaskId::new().to_string(),
                ordinal: 0,
                kind: ranking::KIND_RANK.to_owned(),
                input_kinds: Vec::new(),
                output_kind: "ranking.set.v1".to_owned(),
                payload: payload.encode_to_vec(),
                dependencies: Vec::new(),
                input_artifact_ids: vec![
                    ranking_inputs.candidates.to_owned(),
                    ranking_inputs.index.to_owned(),
                    ranking_inputs.transcript.to_owned(),
                ],
                resources: rank_resources(),
                implementation: ranking::IMPLEMENTATION.to_owned(),
                max_attempts: 3,
                is_final: true,
            }],
        }
    }

    /// The whole analysis as one job (book ch. 12–16).
    ///
    /// Probe, ingest, the speech chain, shot detection, the evidence index,
    /// discovery, and ranking, planned together and rooted by a single fan-in
    /// manifest. This is what a new project submits and what evaluation runs.
    ///
    /// Every task here is keyed exactly as the standalone job that runs the same
    /// stage — the ingest fan-out and the speech chain are the same code, and the
    /// three model-free builtins take their inputs from dependencies instead of
    /// from their payload without that reaching the key. So an analysis over a
    /// source somebody already ingested and transcribed derives nothing twice,
    /// and re-running it is a walk through the cache.
    ///
    /// What it will not do is plan around a source nobody has probed. The ingest
    /// fan-out is shaped by which streams the file has, and that is a measured
    /// fact rather than a guess: a plan that assumed audio would fan out four
    /// tasks that each fail to find it. The probe still runs here — as this job's
    /// first task, so the observation is part of the analysis and reachable from
    /// its root — but it has to have run at least once before.
    #[allow(
        clippy::too_many_lines,
        reason = "ten stages, their skip reasons, and one fan-in: the DAG is the function"
    )]
    pub(crate) fn analyze_source(
        project_id: &ProjectId,
        source: AnalyzeSource<'_>,
        request: &AnalyzeSourcePayloadV1,
        models: &crate::models::ModelRegistry,
        bindings: &crate::selection::Bindings,
        decoder_bom: &str,
        now: u64,
    ) -> Result<Self, &'static str> {
        let mut tasks = Vec::new();
        // Every stage that publishes something, named by the artifact kind it
        // publishes, so the fan-in can depend on all of them and say what each
        // one was.
        let mut stages: Vec<(String, String)> = Vec::new();
        let mut skipped: Vec<SkippedStageV1> = Vec::new();
        let mut skip = |kind: &str, reason: &str| {
            skipped.push(SkippedStageV1 {
                kind: kind.to_owned(),
                reason: reason.to_owned(),
            });
        };

        // The probe, re-derived or served from cache. It depends on nothing and
        // nothing depends on it: the fan-out below was shaped from the answer it
        // already gave, and hanging ingest off it would put the source map in
        // every derivative's key and cost them the cache they share with a
        // standalone ingest.
        let probe = TaskId::new().to_string();
        tasks.push(TaskSpec {
            task_id: probe.clone(),
            ordinal: 0,
            kind: "probe-source".to_owned(),
            input_kinds: Vec::new(),
            output_kind: "evidence.source_map.v1".to_owned(),
            payload: ProbeSourcePayloadV1 {
                key_version: PROBE_SOURCE_KEY_VERSION.to_owned(),
                source_id: source.source_id.to_owned(),
            }
            .encode_to_vec(),
            dependencies: Vec::new(),
            input_artifact_ids: Vec::new(),
            resources: ResourceDeclaration {
                cpu_threads: 1,
                ram_bytes: 64 * 1024 * 1024,
                accelerator_class: String::new(),
                vram_bytes: 0,
                disk_bytes: 32 * 1024 * 1024,
                network_policy: "local-lock".to_owned(),
                thermal_class: "light".to_owned(),
                determinism_class: "deterministic".to_owned(),
                checkpoint_support: false,
                preemption_cost: 1,
            },
            implementation: "ffprobe-8.1.2+clipmill-map-v1".to_owned(),
            max_attempts: 3,
            is_final: false,
        });
        stages.push(("evidence.source_map.v1".to_owned(), probe.clone()));

        // The ingest fan-out, shared verbatim with the ingest job.
        let mut ingest = IngestPlanBuilder::new(
            IngestSourcePayloadV1 {
                key_version: INGEST_SOURCE_KEY_VERSION.to_owned(),
                source_id: source.source_id.to_owned(),
            }
            .encode_to_vec(),
        );
        let handles = ingest_fan_out(&mut ingest, source.has_video, source.has_audio)?;
        let (_, ingest_tasks, _, ingest_manifest) = ingest.finish_with_manifest(false);
        tasks.extend(ingest_tasks);
        stages.push((
            "media.ingest_manifest.v1".to_owned(),
            ingest_manifest.task_id.clone(),
        ));

        // The speech chain, reading the rendition this plan is about to derive.
        let transcript = match &handles.audio_16k {
            None => {
                for kind in [
                    "speech.vad.v1",
                    "speech.asr.v1",
                    "speech.alignment.v1",
                    "speech.transcript.v1",
                ] {
                    skip(kind, "no_audio");
                }
                None
            }
            Some(audio) => {
                let chain = speech_chain(
                    &SpeechChain {
                        audio: SpeechAudioRoute::Planned(audio.task_id.as_str()),
                        source_fingerprint: source.source_fingerprint,
                        language: request.language.as_str(),
                        detection: None,
                        transcript_is_final: false,
                    },
                    models,
                    bindings,
                );
                for task in &chain.tasks {
                    stages.push((task.output_kind.clone(), task.task_id.clone()));
                }
                tasks.extend(chain.tasks);
                Some(chain.transcript_task_id)
            }
        };

        // Shot detection over the proxy this plan is about to derive.
        let shots = match &handles.proxy {
            None => {
                skip("evidence.shots.v1", "no_video");
                None
            }
            Some(proxy) => {
                let shots = TaskId::new().to_string();
                let mut spec = shots_task(
                    shots.clone(),
                    &ShotsStagePayloadV1 {
                        key_version: SHOTS_STAGE_KEY_VERSION.to_owned(),
                        stage: "detect-shots".to_owned(),
                        source_fingerprint: source.source_fingerprint.to_owned(),
                        // The daemon's thresholds. An analysis is the "find me
                        // clips" request, and it does not carry detector knobs:
                        // somebody re-tuning shot detection is running the
                        // standalone stage and looking at the result.
                        detection: None,
                        decoder_bom: decoder_bom.to_owned(),
                    },
                );
                spec.input_kinds = vec!["media.proxy.v1".to_owned()];
                spec.dependencies = vec![proxy.task_id.clone()];
                tasks.push(spec);
                stages.push(("evidence.shots.v1".to_owned(), shots.clone()));
                Some(shots)
            }
        };

        // Everything downstream of the transcript. Without one there is nothing
        // to index, nothing to search, and nothing to rank — which the manifest
        // states rather than the plan pretending it ran them and found nothing.
        let ranked = match &transcript {
            None => {
                for kind in [
                    "index.transcript.v1",
                    "discovery.candidates.v1",
                    "ranking.set.v1",
                ] {
                    skip(kind, "no_audio");
                }
                None
            }
            Some(transcript) => {
                let index = TaskId::new().to_string();
                let mut index_dependencies = vec![transcript.clone()];
                let mut index_kinds = vec!["speech.transcript.v1".to_owned()];
                if let Some(shots) = &shots {
                    index_dependencies.push(shots.clone());
                    index_kinds.push("evidence.shots.v1".to_owned());
                }
                tasks.push(builtin_task(
                    index.clone(),
                    Builtin {
                        kind: evidence::KIND_INDEX,
                        output_kind: "index.transcript.v1",
                        implementation: evidence::IMPLEMENTATION,
                        input_kinds: index_kinds,
                        dependencies: index_dependencies,
                        payload: IndexStagePayloadV1 {
                            key_version: INDEX_STAGE_KEY_VERSION.to_owned(),
                            stage: evidence::KIND_INDEX.to_owned(),
                        }
                        .encode_to_vec(),
                        ram_mib: 256,
                    },
                ));
                stages.push(("index.transcript.v1".to_owned(), index.clone()));

                let discover = TaskId::new().to_string();
                let mut discover_dependencies = vec![index.clone(), transcript.clone()];
                let mut discover_kinds = vec![
                    "index.transcript.v1".to_owned(),
                    "speech.transcript.v1".to_owned(),
                ];
                if let Some(loudness) = &handles.loudness {
                    discover_dependencies.push(loudness.task_id.clone());
                    discover_kinds.push("media.loudness_envelope.v1".to_owned());
                }
                tasks.push(builtin_task(
                    discover.clone(),
                    Builtin {
                        kind: discovery::KIND_DISCOVER,
                        output_kind: "discovery.candidates.v1",
                        implementation: discovery::IMPLEMENTATION,
                        input_kinds: discover_kinds,
                        dependencies: discover_dependencies,
                        payload: DiscoverStagePayloadV1 {
                            key_version: DISCOVER_STAGE_KEY_VERSION.to_owned(),
                            stage: discovery::KIND_DISCOVER.to_owned(),
                            duration: request.duration.or(Some(ClipDurationV1 {
                                min_ticks: 0,
                                max_ticks: 0,
                            })),
                            exploration_floor: 0,
                        }
                        .encode_to_vec(),
                        ram_mib: 512,
                    },
                ));
                stages.push(("discovery.candidates.v1".to_owned(), discover.clone()));

                let rank = TaskId::new().to_string();
                tasks.push(builtin_task(
                    rank.clone(),
                    Builtin {
                        kind: ranking::KIND_RANK,
                        output_kind: "ranking.set.v1",
                        implementation: ranking::IMPLEMENTATION,
                        input_kinds: vec![
                            "discovery.candidates.v1".to_owned(),
                            "index.transcript.v1".to_owned(),
                            "speech.transcript.v1".to_owned(),
                        ],
                        dependencies: vec![discover, index, transcript.clone()],
                        payload: RankStagePayloadV1 {
                            key_version: RANK_STAGE_KEY_VERSION.to_owned(),
                            stage: ranking::KIND_RANK.to_owned(),
                            count: request.count,
                            diversity_milli: request.diversity_milli,
                        }
                        .encode_to_vec(),
                        ram_mib: 512,
                    },
                ));
                stages.push(("ranking.set.v1".to_owned(), rank.clone()));
                Some(rank)
            }
        };
        // Named so the compiler holds the intent: the ranked set is the point of
        // the job, and it is a dependency of the fan-in like everything else.
        let _ = &ranked;

        // The fan-in. Every stage above is a dependency, which is what makes the
        // whole analysis reachable from one root: the job store roots a job's
        // single final artifact, and garbage collection walks recipe inputs.
        tasks.push(TaskSpec {
            task_id: TaskId::new().to_string(),
            ordinal: 0,
            kind: analysis::KIND_MANIFEST.to_owned(),
            input_kinds: stages.iter().map(|(kind, _)| kind.clone()).collect(),
            output_kind: "analysis.manifest.v1".to_owned(),
            payload: AnalysisStagePayloadV1 {
                key_version: ANALYSIS_STAGE_KEY_VERSION.to_owned(),
                stage: analysis::KIND_MANIFEST.to_owned(),
                source_fingerprint: source.source_fingerprint.to_owned(),
                skipped,
            }
            .encode_to_vec(),
            dependencies: stages.into_iter().map(|(_, task_id)| task_id).collect(),
            input_artifact_ids: Vec::new(),
            resources: ResourceDeclaration {
                cpu_threads: 1,
                ram_bytes: 64 * 1024 * 1024,
                accelerator_class: String::new(),
                vram_bytes: 0,
                disk_bytes: 16 * 1024 * 1024,
                network_policy: "local-lock".to_owned(),
                thermal_class: "light".to_owned(),
                determinism_class: "deterministic".to_owned(),
                checkpoint_support: false,
                preemption_cost: 1,
            },
            implementation: analysis::IMPLEMENTATION.to_owned(),
            max_attempts: 3,
            is_final: true,
        });

        // Ordinals last, because three builders contributed tasks and each
        // numbered from zero. They have to be unique within a job; what they
        // order is the scheduler's tie-break among tasks already runnable.
        for (ordinal, task) in tasks.iter_mut().enumerate() {
            task.ordinal = u32::try_from(ordinal).unwrap_or(u32::MAX);
        }
        Ok(Self {
            job_id: JobId::new().to_string(),
            project_id: project_id.to_string(),
            kind: "analyze-source".to_owned(),
            source_id: Some(source.source_id.to_owned()),
            payload: request.encode_to_vec(),
            created_unix_millis: now,
            tasks,
        })
    }

    pub(crate) fn device_profile(
        hardware_fingerprint: String,
        measurement_generation: u64,
        now: u64,
    ) -> Self {
        let payload = DeviceProfilePayloadV1 {
            key_version: DEVICE_PROFILE_KEY_VERSION.to_owned(),
            hardware_fingerprint,
            measurement_generation,
        }
        .encode_to_vec();
        Self {
            job_id: JobId::new().to_string(),
            project_id: SYSTEM_PROJECT_ID.to_owned(),
            kind: "device-profile".to_owned(),
            source_id: None,
            payload: payload.clone(),
            created_unix_millis: now,
            tasks: vec![TaskSpec {
                task_id: TaskId::new().to_string(),
                ordinal: 0,
                kind: "device-profile".to_owned(),
                input_kinds: Vec::new(),
                output_kind: "evidence.device_profile.v1".to_owned(),
                payload,
                dependencies: Vec::new(),
                input_artifact_ids: Vec::new(),
                resources: ResourceDeclaration {
                    cpu_threads: 1,
                    ram_bytes: 64 * 1024 * 1024,
                    accelerator_class: String::new(),
                    vram_bytes: 0,
                    disk_bytes: 32 * 1024 * 1024,
                    network_policy: "local-lock".to_owned(),
                    thermal_class: "light".to_owned(),
                    determinism_class: "generation-scoped".to_owned(),
                    checkpoint_support: false,
                    preemption_cost: 1,
                },
                implementation: "clipmill-device-profiler@1.0.0".to_owned(),
                max_attempts: 3,
                is_final: true,
            }],
        }
    }
}

/// The rendition the speech chain reads, and the source behind it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SpeechAudio<'a> {
    pub artifact_id: &'a str,
    pub source_fingerprint: &'a str,
}

/// The proxy shot detection reads, and the source behind it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ShotsProxy<'a> {
    pub artifact_id: &'a str,
    pub source_fingerprint: &'a str,
}

/// The published documents the evidence index reads.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EvidenceInputs<'a> {
    pub transcript: &'a str,
    /// Absent for a source with no video.
    pub shots: Option<&'a str>,
}

/// The published documents ranking reads.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RankingJobInputs<'a> {
    pub candidates: &'a str,
    pub index: &'a str,
    pub transcript: &'a str,
}

/// What the ranking stage costs. Exhaustive over a lattice the design keeps
/// under a few hundred pairs, so this is arithmetic rather than search.
fn rank_resources() -> ResourceDeclaration {
    ResourceDeclaration {
        cpu_threads: 1,
        ram_bytes: 512 * 1024 * 1024,
        accelerator_class: String::new(),
        vram_bytes: 0,
        disk_bytes: 128 * 1024 * 1024,
        network_policy: "local-lock".to_owned(),
        thermal_class: "light".to_owned(),
        determinism_class: "deterministic".to_owned(),
        checkpoint_support: false,
        preemption_cost: 1,
    }
}

/// The published documents discovery reads.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DiscoveryInputs<'a> {
    pub index: &'a str,
    pub transcript: &'a str,
    /// Absent for a source with no audio.
    pub loudness: Option<&'a str>,
}

/// What one leased speech stage needs, taken from the model it will load.
///
/// Sized from the pinned manifest rather than guessed: the registry already
/// states each model's weights and its runtime's allowance, and admission
/// checks the sum. Guessing here would either starve the stage or admit it
/// onto a machine that cannot hold it, and the registry is the only place
/// that knows which.
/// The implementation this device bound a speech stage to.
///
/// Falls back to the portable candidate when the profile has nothing to say,
/// which is the state of a daemon that has not measured its device yet. The
/// fallback is a real registered implementation rather than a synthesized one,
/// so a task planned before the first measurement is keyed exactly like a task
/// planned after a measurement that chose the same thing.
fn speech_implementation(
    kind: &str,
    bindings: &crate::selection::Bindings,
) -> &'static crate::implementations::Implementation {
    bindings
        .for_stage(kind)
        .and_then(|binding| crate::implementations::lookup(&binding.implementation))
        .or_else(|| crate::implementations::portable_for_stage(kind))
        .unwrap_or_else(|| unreachable!("{kind} is planned without a registered implementation"))
}

/// What a stage costs, taken from the model it was actually bound to.
///
/// The accelerator class comes from the implementation, so the scheduler's
/// existing capability match does the routing: an MLX task declares `metal`
/// and only reaches a worker whose verified device profile reports one, while
/// the portable candidate declares nothing and reaches anybody.
fn speech_resources(
    implementation: &crate::implementations::Implementation,
    models: &crate::models::ModelRegistry,
    cpu_threads: u32,
) -> ResourceDeclaration {
    let resident = models
        .get(implementation.model)
        .map_or(512 * 1024 * 1024, |manifest| {
            manifest.memory.resident_bytes()
        });
    ResourceDeclaration {
        cpu_threads,
        ram_bytes: resident,
        accelerator_class: implementation.accelerator_class.to_owned(),
        // Unified memory on Apple silicon: the weights are already counted in
        // RAM above, and counting them again as VRAM would refuse a machine
        // that can run the model comfortably.
        vram_bytes: 0,
        disk_bytes: 128 * 1024 * 1024,
        network_policy: "local-lock".to_owned(),
        thermal_class: "sustained".to_owned(),
        determinism_class: "deterministic".to_owned(),
        checkpoint_support: false,
        preemption_cost: 3,
    }
}

fn ingest_resources(
    cpu_threads: u32,
    ram_mebibytes: u64,
    disk_mebibytes: u64,
) -> ResourceDeclaration {
    ResourceDeclaration {
        cpu_threads,
        ram_bytes: ram_mebibytes * 1024 * 1024,
        accelerator_class: String::new(),
        vram_bytes: 0,
        disk_bytes: disk_mebibytes * 1024 * 1024,
        network_policy: "local-lock".to_owned(),
        thermal_class: "sustained".to_owned(),
        determinism_class: "deterministic".to_owned(),
        checkpoint_support: false,
        preemption_cost: 2,
    }
}

/// A derivative already added to an ingest plan, usable as a dependency.
#[derive(Clone)]
struct DerivativeHandle {
    task_id: String,
    output_kind: String,
}

/// Accumulates the ingest fan-out and closes it with the fan-in manifest,
/// keeping ordinals dense and dependency/input-kind lists aligned (the plan
/// validator requires one input kind per dependency).
struct IngestPlanBuilder {
    payload: Vec<u8>,
    tasks: Vec<TaskSpec>,
    children: Vec<DerivativeHandle>,
}

impl IngestPlanBuilder {
    fn new(payload: Vec<u8>) -> Self {
        Self {
            payload,
            tasks: Vec::new(),
            children: Vec::new(),
        }
    }

    fn derivative(
        &mut self,
        kind: &str,
        output_kind: &str,
        implementation: &str,
        resources: ResourceDeclaration,
        dependencies: &[DerivativeHandle],
    ) -> DerivativeHandle {
        let handle = DerivativeHandle {
            task_id: TaskId::new().to_string(),
            output_kind: output_kind.to_owned(),
        };
        self.tasks.push(TaskSpec {
            task_id: handle.task_id.clone(),
            ordinal: u32::try_from(self.tasks.len()).unwrap_or(u32::MAX),
            kind: kind.to_owned(),
            input_kinds: dependencies
                .iter()
                .map(|dependency| dependency.output_kind.clone())
                .collect(),
            output_kind: output_kind.to_owned(),
            payload: self.payload.clone(),
            dependencies: dependencies
                .iter()
                .map(|dependency| dependency.task_id.clone())
                .collect(),
            // Every ingest derivative reads the source file by path, or a
            // sibling this plan produced. Neither is a content address.
            input_artifact_ids: Vec::new(),
            resources,
            implementation: implementation.to_owned(),
            max_attempts: 3,
            is_final: false,
        });
        self.children.push(handle.clone());
        handle
    }

    /// Close the fan-out with the manifest that names its children.
    ///
    /// `is_final` is false when this fan-out is the front of a longer plan: the
    /// job store roots exactly one artifact per job, and inside the analyze DAG
    /// that artifact is the analysis, not the ingest. Nothing else about the
    /// manifest changes, and nothing about the derivatives does — which is what
    /// lets a source ingested on its own and a source ingested as part of an
    /// analysis share every artifact rather than deriving the same proxy twice.
    fn finish_with_manifest(
        mut self,
        is_final: bool,
    ) -> (
        Vec<u8>,
        Vec<TaskSpec>,
        Vec<DerivativeHandle>,
        DerivativeHandle,
    ) {
        let manifest = TaskSpec {
            task_id: TaskId::new().to_string(),
            ordinal: u32::try_from(self.tasks.len()).unwrap_or(u32::MAX),
            kind: media::KIND_MANIFEST.to_owned(),
            input_kinds: self
                .children
                .iter()
                .map(|child| child.output_kind.clone())
                .collect(),
            output_kind: "media.ingest_manifest.v1".to_owned(),
            payload: self.payload.clone(),
            dependencies: self
                .children
                .iter()
                .map(|child| child.task_id.clone())
                .collect(),
            input_artifact_ids: Vec::new(),
            resources: ResourceDeclaration {
                cpu_threads: 1,
                ram_bytes: 32 * 1024 * 1024,
                accelerator_class: String::new(),
                vram_bytes: 0,
                disk_bytes: 8 * 1024 * 1024,
                network_policy: "local-lock".to_owned(),
                thermal_class: "light".to_owned(),
                determinism_class: "deterministic".to_owned(),
                checkpoint_support: false,
                preemption_cost: 1,
            },
            implementation: "clipmill-ingest-manifest@1.0.0".to_owned(),
            max_attempts: 3,
            is_final,
        };
        let handle = DerivativeHandle {
            task_id: manifest.task_id.clone(),
            output_kind: manifest.output_kind.clone(),
        };
        self.tasks.push(manifest);
        (self.payload, self.tasks, self.children, handle)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TaskRecord {
    pub task_id: String,
    pub kind: String,
    pub state: i32,
    pub attempt: u32,
    pub max_attempts: u32,
    pub progress_unit: String,
    pub progress_done: u64,
    pub progress_total: u64,
    pub wait_reason: String,
    pub output_artifact_id: String,
}

impl From<TaskRecord> for v1::Task {
    fn from(value: TaskRecord) -> Self {
        let progress = (!value.progress_unit.is_empty()).then_some(ProgressUnits {
            unit: value.progress_unit,
            done: value.progress_done,
            total: value.progress_total,
        });
        Self {
            task_id: value.task_id,
            kind: value.kind,
            state: value.state,
            attempt: value.attempt,
            max_attempts: value.max_attempts,
            progress,
            wait_reason: value.wait_reason,
            output_artifact_id: value.output_artifact_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct JobRecord {
    pub job_id: String,
    pub project_id: String,
    pub kind: String,
    pub state: i32,
    pub created_unix_millis: u64,
    pub updated_unix_millis: u64,
    pub tasks: Vec<TaskRecord>,
    pub output_artifact_ids: Vec<String>,
    pub failure_class: i32,
    pub failure_detail: String,
}

impl From<JobRecord> for v1::Job {
    fn from(value: JobRecord) -> Self {
        Self {
            job_id: value.job_id,
            project_id: value.project_id,
            kind: value.kind,
            state: value.state,
            created_unix_millis: value.created_unix_millis,
            updated_unix_millis: value.updated_unix_millis,
            tasks: value.tasks.into_iter().map(Into::into).collect(),
            output_artifact_ids: value.output_artifact_ids,
            failure_class: value.failure_class,
            failure_detail: value.failure_detail,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TaskEventRecord {
    pub event_id: u64,
    pub project_id: String,
    pub job_id: String,
    pub task_id: String,
    pub state: i32,
    pub attempt: u32,
    pub progress_unit: String,
    pub progress_done: u64,
    pub progress_total: u64,
    pub wait_reason: String,
    pub failure_class: i32,
    pub at_unix_millis: u64,
}

impl TaskEventRecord {
    #[must_use]
    pub(crate) fn as_proto(&self) -> v1::TaskEvent {
        let progress = (!self.progress_unit.is_empty()).then_some(ProgressUnits {
            unit: self.progress_unit.clone(),
            done: self.progress_done,
            total: self.progress_total,
        });
        v1::TaskEvent {
            job_id: self.job_id.clone(),
            task_id: self.task_id.clone(),
            state: self.state,
            progress,
            wait_reason: self.wait_reason.clone(),
            at_unix_millis: self.at_unix_millis,
            event_id: self.event_id,
            attempt: self.attempt,
            failure_class: self.failure_class,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct EventFilter {
    pub project_id: Option<String>,
    pub job_id: Option<String>,
}

impl EventFilter {
    #[must_use]
    pub(crate) fn matches(&self, event: &TaskEventRecord) -> bool {
        self.project_id
            .as_ref()
            .is_none_or(|project_id| project_id == &event.project_id)
            && self
                .job_id
                .as_ref()
                .is_none_or(|job_id| job_id == &event.job_id)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct EventHub {
    sender: broadcast::Sender<TaskEventRecord>,
}

impl EventHub {
    #[must_use]
    pub(crate) fn new() -> Self {
        let (sender, _receiver) = broadcast::channel(1024);
        Self { sender }
    }

    pub(crate) fn publish_all(&self, events: impl IntoIterator<Item = TaskEventRecord>) {
        for event in events {
            let _receivers = self.sender.send(event);
        }
    }

    pub(crate) fn subscribe(&self) -> broadcast::Receiver<TaskEventRecord> {
        self.sender.subscribe()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LeasedTask {
    pub project_id: String,
    pub job_id: String,
    pub source_id: Option<String>,
    pub task_id: String,
    pub lease_id: String,
    pub kind: String,
    pub output_kind: String,
    pub payload: Vec<u8>,
    pub implementation: String,
    pub attempt: u32,
    pub input_artifact_ids: Vec<ArtifactId>,
    pub resources: ResourceDeclaration,
}

#[derive(Clone, Debug)]
pub(crate) struct LeaseSelection {
    pub task: Option<LeasedTask>,
    pub events: Vec<TaskEventRecord>,
}

#[derive(Clone, Debug)]
pub(crate) struct LeaseRequest {
    pub lease_id: String,
    pub daemon_epoch: String,
    pub now_unix_millis: u64,
    pub expires_unix_millis: u64,
    pub capacity: ResourceCapacity,
    pub worker_id: String,
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct TaskCompletion {
    pub response: Vec<u8>,
    pub events: Vec<TaskEventRecord>,
}

#[derive(Debug)]
pub(crate) struct Scheduler {
    handle: SchedulerHandle,
    stopped: Option<oneshot::Sender<()>>,
    task: JoinHandle<()>,
}

#[derive(Clone, Debug)]
pub(crate) struct SchedulerHandle {
    notify: Arc<Notify>,
    capacity_update: Arc<Mutex<Option<ResourceCapacity>>>,
    capacity_limit: ResourceCapacity,
    /// Which implementation each speech stage is bound to, as last verified
    /// (D19). Read when a job is planned, never when a task is leased: a
    /// re-measurement that arrives mid-job must not change what a task already
    /// in flight means.
    bindings: Arc<Mutex<crate::selection::Bindings>>,
}

impl SchedulerHandle {
    pub(crate) fn notify(&self) {
        self.notify.notify_one();
    }

    pub(crate) fn apply_device_profile(&self, profile: &VerifiedDeviceProfile) {
        let measured =
            ResourceCapacity::measured(profile.logical_cores, profile.available_memory_bytes)
                .with_available_backends(&profile.available_backends);
        let capacity = ResourceCapacity {
            cpu_threads: measured
                .cpu_threads
                .min(self.capacity_limit.cpu_threads)
                .max(1),
            ram_bytes: measured.ram_bytes.min(self.capacity_limit.ram_bytes),
            disk_bytes: measured.disk_bytes.min(self.capacity_limit.disk_bytes),
            accelerator_mask: measured.accelerator_mask,
            vram_bytes: measured.vram_bytes,
        };
        if let Ok(mut pending) = self.capacity_update.lock() {
            *pending = Some(capacity);
        }
        // A profile predating selection carries no bindings. Keeping the
        // portable defaults is the only safe reading of that: it is a profile
        // that says nothing about implementations, not one that says every
        // capability has none.
        if !profile.bindings.is_empty()
            && let Ok(mut bindings) = self.bindings.lock()
        {
            for binding in profile.bindings.iter() {
                // Said differently on purpose. An operator wondering why a
                // machine with an accelerator is running the CPU path should
                // find the answer in the log rather than in the profile JSON.
                let note = if binding.was_measured() {
                    "capability bound to the implementation a benchmark chose"
                } else {
                    "capability bound to its portable implementation; no benchmark covers this device"
                };
                tracing::info!(
                    capability = binding.capability,
                    implementation = binding.implementation,
                    model = binding.model,
                    backend = binding.backend,
                    selected_by = binding.selected_by,
                    note
                );
            }
            bindings.clone_from(&profile.bindings);
        }
        self.notify.notify_one();
    }

    /// What this machine actually has, as last verified. Falls back to the
    /// boot-time limit until a device profile has been verified, so admission
    /// is never checked against nothing.
    pub(crate) fn machine_capacity(&self) -> ResourceCapacity {
        self.capacity_update
            .lock()
            .ok()
            .and_then(|pending| *pending)
            .unwrap_or(self.capacity_limit)
    }

    /// The bindings a new plan should be built against.
    pub(crate) fn bindings(&self) -> crate::selection::Bindings {
        self.bindings
            .lock()
            .map(|bindings| bindings.clone())
            .unwrap_or_default()
    }
}

impl Scheduler {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn start(
        database: DbHandle,
        artifacts: ArtifactHandle,
        events: EventHub,
        daemon_epoch: String,
        sources: SourceInspector,
        device_profiler: DeviceProfiler,
        media: MediaRunner,
        fonts_dir: std::path::PathBuf,
        models: Arc<crate::models::ModelRegistry>,
        capacity: ResourceCapacity,
        builtin_fixture_executor: bool,
    ) -> Self {
        debug_assert!(LEASE_TTL >= HEARTBEAT_INTERVAL.saturating_mul(3));
        let notify = Arc::new(Notify::new());
        let capacity_update = Arc::new(Mutex::new(None));
        let handle = SchedulerHandle {
            notify: Arc::clone(&notify),
            capacity_update: Arc::clone(&capacity_update),
            capacity_limit: capacity,
            // Until a profile is verified, every stage takes the candidate
            // that runs anywhere — the honest default for a device nobody has
            // measured.
            bindings: Arc::new(Mutex::new(crate::selection::Bindings::portable())),
        };
        let (stopped, stop) = oneshot::channel();
        let task = tokio::spawn(run_scheduler(
            database,
            artifacts,
            events,
            daemon_epoch,
            sources,
            device_profiler,
            media,
            fonts_dir,
            models,
            capacity,
            capacity_update,
            builtin_fixture_executor,
            notify,
            stop,
        ));
        handle.notify();
        Self {
            handle,
            stopped: Some(stopped),
            task,
        }
    }

    #[must_use]
    pub(crate) fn handle(&self) -> SchedulerHandle {
        self.handle.clone()
    }

    pub(crate) async fn shutdown(mut self) {
        if let Some(stopped) = self.stopped.take() {
            let _sent = stopped.send(());
        }
        let _joined = self.task.await;
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_scheduler(
    database: DbHandle,
    artifacts: ArtifactHandle,
    events: EventHub,
    daemon_epoch: String,
    sources: SourceInspector,
    device_profiler: DeviceProfiler,
    media: MediaRunner,
    fonts_dir: std::path::PathBuf,
    models: Arc<crate::models::ModelRegistry>,
    capacity: ResourceCapacity,
    capacity_update: Arc<Mutex<Option<ResourceCapacity>>>,
    builtin_fixture_executor: bool,
    notify: Arc<Notify>,
    mut stop: oneshot::Receiver<()>,
) {
    let executors = BuiltinExecutors {
        database: database.clone(),
        artifacts,
        sources,
        device_profiler,
        media,
        fonts_dir,
        models,
    };
    let mut schedule = interval(SCHEDULER_TICK);
    schedule.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut running = JoinSet::new();
    let mut available_capacity = capacity;
    let builtin_capabilities = builtin_capabilities(builtin_fixture_executor);
    loop {
        tokio::select! {
            biased;
            _ = &mut stop => break,
            joined = running.join_next(), if !running.is_empty() => {
                match joined {
                    Some(Ok(resources)) => available_capacity.release(&resources),
                    Some(Err(error)) => {
                        tracing::warn!(%error, "built-in task executor stopped unexpectedly");
                    }
                    None => {}
                }
            }
            () = notify.notified() => {}
            _ = schedule.tick() => {}
        }

        if let Ok(expired) = database
            .expire_task_leases(now_millis(), &daemon_epoch)
            .await
        {
            events.publish_all(expired);
        }
        if running.is_empty()
            && let Ok(mut pending) = capacity_update.lock()
            && let Some(measured) = pending.take()
        {
            available_capacity = measured;
            tracing::info!(
                cpu_threads = measured.cpu_threads,
                ram_bytes = measured.ram_bytes,
                "scheduler applied verified device profile capacity"
            );
        }
        while running.len() < MAX_BUILTIN_TASKS && available_capacity.cpu_threads > 0 {
            let now = now_millis();
            let lease = LeaseId::new().to_string();
            let leased = database
                .lease_next_task(LeaseRequest {
                    lease_id: lease,
                    daemon_epoch: daemon_epoch.clone(),
                    now_unix_millis: now,
                    expires_unix_millis: now.saturating_add(duration_millis(LEASE_TTL)),
                    capacity: available_capacity,
                    worker_id: "builtin-fixture".to_owned(),
                    capabilities: builtin_capabilities.clone(),
                })
                .await;
            let Ok(selection) = leased else {
                break;
            };
            events.publish_all(selection.events);
            let Some(task) = selection.task else {
                break;
            };
            let resources = task.resources.clone();
            if !available_capacity.reserve(&resources) {
                tracing::error!(
                    task_id = task.task_id,
                    "database admitted an over-capacity task"
                );
                break;
            }
            let executors = executors.clone();
            let events = events.clone();
            let notify = Arc::clone(&notify);
            running.spawn(async move {
                execute_task(executors, events, task).await;
                notify.notify_one();
                resources
            });
        }
    }

    running.abort_all();
    while running.join_next().await.is_some() {}
}

#[allow(clippy::too_many_lines)]
/// Everything the daemon's own executors need. Grouped so that adding a
/// stage does not mean widening three signatures.
#[derive(Clone)]
struct BuiltinExecutors {
    database: DbHandle,
    artifacts: ArtifactHandle,
    sources: SourceInspector,
    device_profiler: DeviceProfiler,
    media: MediaRunner,
    fonts_dir: std::path::PathBuf,
    models: std::sync::Arc<crate::models::ModelRegistry>,
}

impl BuiltinExecutors {
    async fn run(
        &self,
        task: &LeasedTask,
        progress: &ProgressSlot,
    ) -> Result<ArtifactId, TaskExecutionError> {
        match task.kind.as_str() {
            "probe-source" => {
                execute_probe_artifact(&self.database, &self.artifacts, &self.sources, task).await
            }
            "device-profile" => {
                execute_device_artifact(
                    &self.database,
                    &self.artifacts,
                    &self.device_profiler,
                    task,
                )
                .await
            }
            kind if media::is_ingest_kind(kind) => {
                media::execute_ingest_task(
                    &self.database,
                    &self.artifacts,
                    &self.media,
                    &self.sources,
                    task,
                    progress,
                )
                .await
            }
            speech::KIND_TRANSCRIPT => {
                speech::execute_transcript_task(&self.artifacts, task, progress).await
            }
            evidence::KIND_INDEX => {
                evidence::execute_index_task(&self.artifacts, task, progress).await
            }
            discovery::KIND_DISCOVER => {
                discovery::execute_discover_task(&self.artifacts, task, progress).await
            }
            ranking::KIND_RANK => ranking::execute_rank_task(&self.artifacts, task, progress).await,
            analysis::KIND_MANIFEST => {
                analysis::execute_manifest_task(&self.artifacts, task, progress).await
            }
            kind if render::is_render_kind(kind) => {
                render::execute_render_task(
                    &RenderContext {
                        database: &self.database,
                        artifacts: &self.artifacts,
                        media: &self.media,
                        sources: &self.sources,
                        fonts_dir: &self.fonts_dir,
                    },
                    task,
                    progress,
                )
                .await
            }
            _ => execute_demo_artifact(&self.artifacts, &self.models, task)
                .await
                .map_err(TaskExecutionError::transient),
        }
    }
}

/// Task kinds the daemon executes itself, rather than leasing to a worker.
fn builtin_capabilities(builtin_fixture_executor: bool) -> Vec<String> {
    let mut kinds = vec!["probe-source".to_owned(), "device-profile".to_owned()];
    kinds.extend(media::INGEST_TASK_KINDS.map(str::to_owned));
    kinds.push(render::KIND_RENDER_CLIP.to_owned());
    kinds.push(speech::KIND_TRANSCRIPT.to_owned());
    kinds.push(evidence::KIND_INDEX.to_owned());
    kinds.push(discovery::KIND_DISCOVER.to_owned());
    kinds.push(ranking::KIND_RANK.to_owned());
    kinds.push(analysis::KIND_MANIFEST.to_owned());
    if builtin_fixture_executor {
        kinds.extend(["demo-seed", "demo-left", "demo-right", "demo-join"].map(str::to_owned));
    }
    kinds
}

async fn execute_task(executors: BuiltinExecutors, events: EventHub, task: LeasedTask) {
    tracing::debug!(
        project_id = task.project_id,
        job_id = task.job_id,
        task_id = task.task_id,
        attempt = task.attempt,
        "executing built-in durable task"
    );
    let database = executors.database.clone();
    let lease_id = task.lease_id.clone();
    let progress = ProgressSlot::default();
    let work = async {
        if let Ok(delay) = std::env::var("CLIPMILL_W4_STEP_DELAY_MS")
            && let Ok(delay) = delay.parse::<u64>()
        {
            tokio::time::sleep(Duration::from_millis(delay.min(30_000))).await;
        }
        executors.run(&task, &progress).await
    };
    tokio::pin!(work);
    let mut heartbeat = interval(HEARTBEAT_INTERVAL);
    heartbeat.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let outcome = loop {
        tokio::select! {
            result = &mut work => break Some(result),
            _ = heartbeat.tick() => {
                let now = now_millis();
                if let Ok(task_events) = database
                    .heartbeat_task(
                        lease_id.clone(),
                        now,
                        now.saturating_add(duration_millis(LEASE_TTL)),
                        progress.take(),
                    )
                    .await
                {
                    events.publish_all(task_events);
                } else {
                    tracing::debug!(task_id = task.task_id, "task lease heartbeat was rejected");
                    break None;
                }
            }
        }
    };
    let Some(outcome) = outcome else {
        return;
    };
    match outcome {
        Ok(artifact_id) => {
            let response = artifact_id.to_string().into_bytes();
            let expected_response = response.clone();
            match database
                .complete_task(
                    lease_id.clone(),
                    artifact_id,
                    Sha256::digest(&response).into(),
                    response,
                    now_millis(),
                )
                .await
            {
                Ok(completion) => {
                    if completion.response == expected_response {
                        events.publish_all(completion.events);
                    } else {
                        tracing::error!(
                            task_id = task.task_id,
                            "durable task completion returned an inconsistent response"
                        );
                    }
                }
                Err(StoreError::Conflict | StoreError::NotFound) => {
                    tracing::debug!(task_id = task.task_id, "discarded stale task completion");
                }
                Err(error) => {
                    tracing::warn!(task_id = task.task_id, %error, "task completion failed");
                }
            }
        }
        Err(failure) => match database
            .fail_task(
                lease_id,
                failure.classification as i32,
                failure.detail,
                now_millis(),
            )
            .await
        {
            Ok(task_events) => events.publish_all(task_events),
            Err(error) => {
                tracing::warn!(task_id = task.task_id, %error, "task failure could not be persisted");
            }
        },
    }
}

#[derive(Debug)]
pub(crate) struct TaskExecutionError {
    classification: FailureClass,
    detail: String,
}

impl TaskExecutionError {
    pub(crate) fn transient(detail: String) -> Self {
        Self {
            classification: FailureClass::Transient,
            detail,
        }
    }

    pub(crate) fn deterministic(detail: impl Into<String>) -> Self {
        Self {
            classification: FailureClass::Deterministic,
            detail: detail.into(),
        }
    }
}

async fn execute_probe_artifact(
    database: &DbHandle,
    artifacts: &ArtifactHandle,
    sources: &SourceInspector,
    task: &LeasedTask,
) -> Result<ArtifactId, TaskExecutionError> {
    let source_id = task
        .source_id
        .as_ref()
        .ok_or_else(|| TaskExecutionError::deterministic("probe task omitted source id"))?;
    let source = database
        .get_source(source_id.clone())
        .await
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    sources
        .verify(&source.observation)
        .await
        .map_err(|error| match error {
            SourceProbeError::SourceChanged => TaskExecutionError::deterministic("SOURCE_CHANGED"),
            SourceProbeError::InvalidPath(_) | SourceProbeError::ProbeFailed(_) => {
                TaskExecutionError::deterministic(error.to_string())
            }
            _ => TaskExecutionError::transient(error.to_string()),
        })?;
    let digest = source
        .source_fingerprint
        .strip_prefix("sha256:")
        .ok_or_else(|| TaskExecutionError::deterministic("source fingerprint is invalid"))?
        .parse::<Sha256Digest>()
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    let mut config = Map::new();
    config.insert(
        "ffmpeg_bom".to_owned(),
        Value::String(media::FFMPEG_BOM.to_owned()),
    );
    config.insert(
        "probe_algorithm".to_owned(),
        Value::String("clipmill.ffprobe.normalize.v1".to_owned()),
    );
    config.insert(
        "mapping_algorithm".to_owned(),
        Value::String("clipmill.source-map.mapping.v1".to_owned()),
    );
    let recipe = ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: task.output_kind.clone(),
        source_fingerprint: digest,
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: task.kind.clone(),
            implementation: task.implementation.clone(),
            model_digest: None,
        },
        inputs: Vec::new(),
        policy: NetworkPolicy::LocalLock,
        config,
        semantic_version: "clipmill.source_map.v1".to_owned(),
    })
    .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    match artifacts
        .prepare(recipe)
        .await
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?
    {
        PrepareOutcome::Hit(lease) => Ok(lease.artifact_id()),
        PrepareOutcome::InFlight { .. } => Err(TaskExecutionError::transient(
            "source-map artifact key is already in flight".to_owned(),
        )),
        PrepareOutcome::Miss(staging) => {
            let path = "source-map.json"
                .parse::<ArtifactPath>()
                .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
            let mut file = staging
                .create_file(&path)
                .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
            write_and_sync(&mut file, &source.source_map_json)
                .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
            drop(file);
            let lease = artifacts
                .commit(staging.id().clone(), vec![path], BTreeMap::new())
                .await
                .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
            Ok(lease.artifact_id())
        }
    }
}

#[allow(clippy::too_many_lines)]
async fn execute_device_artifact(
    database: &DbHandle,
    artifacts: &ArtifactHandle,
    profiler: &DeviceProfiler,
    task: &LeasedTask,
) -> Result<ArtifactId, TaskExecutionError> {
    let payload = DeviceProfilePayloadV1::decode(task.payload.as_slice())
        .map_err(|_| TaskExecutionError::deterministic("device profile payload is invalid"))?;
    if payload.key_version != DEVICE_PROFILE_KEY_VERSION || payload.measurement_generation == 0 {
        return Err(TaskExecutionError::deterministic(
            "device profile payload version or generation is invalid",
        ));
    }
    payload
        .hardware_fingerprint
        .parse::<ArtifactId>()
        .map_err(|_| TaskExecutionError::deterministic("device fingerprint is invalid"))?;
    let record = database
        .device_profile_for_job(task.job_id.clone())
        .await
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
    if record.hardware_fingerprint != payload.hardware_fingerprint
        || record.measurement_generation != payload.measurement_generation
    {
        return Err(TaskExecutionError::deterministic(
            "device job does not match its durable generation",
        ));
    }
    let profile_json = if let Some(profile_json) = record.profile_json {
        profile_json
    } else {
        let measured = profiler
            .measure(
                &payload.hardware_fingerprint,
                payload.measurement_generation,
            )
            .await
            .map_err(|error| match error {
                crate::device::DeviceProfileError::FingerprintMismatch => {
                    TaskExecutionError::deterministic("DEVICE_CHANGED")
                }
                _ => TaskExecutionError::transient(error.to_string()),
            })?;
        database
            .store_device_profile_json(task.job_id.clone(), measured, now_millis())
            .await
            .map_err(|error| TaskExecutionError::transient(error.to_string()))?
            .profile_json
            .ok_or_else(|| {
                TaskExecutionError::deterministic("stored device measurement is missing")
            })?
    };
    verify_profile(&profile_json, Some(&payload.hardware_fingerprint))
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    let digest = payload
        .hardware_fingerprint
        .strip_prefix("sha256:")
        .ok_or_else(|| TaskExecutionError::deterministic("device fingerprint is invalid"))?
        .parse::<Sha256Digest>()
        .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    let mut config = Map::new();
    config.insert(
        "measurement_generation".to_owned(),
        Value::Number(payload.measurement_generation.into()),
    );
    config.insert(
        "profile_algorithm".to_owned(),
        Value::String("clipmill.device-profile.measure.v1".to_owned()),
    );
    let recipe = ArtifactRecipe::try_from_spec(RecipeSpec {
        kind: task.output_kind.clone(),
        source_fingerprint: digest,
        timebase: Timebase {
            num: 1,
            den: 90_000,
        },
        producer: Producer {
            stage: task.kind.clone(),
            implementation: task.implementation.clone(),
            model_digest: None,
        },
        inputs: Vec::new(),
        policy: NetworkPolicy::LocalLock,
        config,
        semantic_version: "clipmill.device_profile.v1".to_owned(),
    })
    .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
    match artifacts
        .prepare(recipe)
        .await
        .map_err(|error| TaskExecutionError::transient(error.to_string()))?
    {
        PrepareOutcome::Hit(lease) => Ok(lease.artifact_id()),
        PrepareOutcome::InFlight { .. } => Err(TaskExecutionError::transient(
            "device profile artifact key is already in flight".to_owned(),
        )),
        PrepareOutcome::Miss(staging) => {
            let path = "profile.json"
                .parse::<ArtifactPath>()
                .map_err(|error| TaskExecutionError::deterministic(error.to_string()))?;
            let mut file = staging
                .create_file(&path)
                .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
            write_and_sync(&mut file, profile_json.as_bytes())
                .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
            drop(file);
            let lease = artifacts
                .commit(staging.id().clone(), vec![path], BTreeMap::new())
                .await
                .map_err(|error| TaskExecutionError::transient(error.to_string()))?;
            Ok(lease.artifact_id())
        }
    }
}

async fn execute_demo_artifact(
    artifacts: &ArtifactHandle,
    models: &crate::models::ModelRegistry,
    task: &LeasedTask,
) -> Result<ArtifactId, String> {
    let recipe = crate::recipes::worker_recipe(task, models).map_err(|error| error.to_string())?;
    match artifacts
        .prepare(recipe)
        .await
        .map_err(|error| error.to_string())?
    {
        PrepareOutcome::Hit(lease) => Ok(lease.artifact_id()),
        PrepareOutcome::InFlight { .. } => Err("artifact key is already in flight".to_owned()),
        PrepareOutcome::Miss(staging) => {
            let path = "result.json"
                .parse::<ArtifactPath>()
                .map_err(|error| error.to_string())?;
            let output = demo_output(task)?;
            let mut file = staging
                .create_file(&path)
                .map_err(|error| error.to_string())?;
            write_and_sync(&mut file, &output).map_err(|error| error.to_string())?;
            drop(file);
            let lease = artifacts
                .commit(staging.id().clone(), vec![path], BTreeMap::new())
                .await
                .map_err(|error| error.to_string())?;
            Ok(lease.artifact_id())
        }
    }
}

pub(crate) fn demo_output(task: &LeasedTask) -> Result<Vec<u8>, String> {
    serde_json_canonicalizer::to_vec(&json!({
        "inputs": task.input_artifact_ids.iter().map(ToString::to_string).collect::<Vec<_>>(),
        "kind": task.kind,
        "payload_sha256": format!("sha256:{}", Sha256Digest::from_bytes(Sha256::digest(&task.payload).into())),
    }))
    .map_err(|error| error.to_string())
}

fn write_and_sync(file: &mut File, bytes: &[u8]) -> std::io::Result<()> {
    file.write_all(bytes)?;
    file.sync_all()
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
        })
}

fn duration_millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

pub(crate) fn is_terminal_job(state: i32) -> bool {
    state == JobState::Succeeded as i32
        || state == JobState::Failed as i32
        || state == JobState::Cancelled as i32
}

#[cfg(test)]
mod ingest_plan_tests {
    #![allow(clippy::expect_used)]

    use clipmill_core::ProjectId;

    use super::JobPlan;
    use crate::media;

    fn kinds(plan: &JobPlan) -> Vec<&str> {
        plan.tasks.iter().map(|task| task.kind.as_str()).collect()
    }

    #[test]
    fn full_source_plan_decodes_video_once_and_fans_into_one_manifest() {
        let plan = JobPlan::ingest_source(
            &ProjectId::new(),
            "src_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_owned(),
            b"payload".to_vec(),
            true,
            true,
            7,
        )
        .expect("plan");
        assert_eq!(plan.kind, "ingest-source");
        assert_eq!(plan.tasks.len(), 9);
        let finals = plan
            .tasks
            .iter()
            .filter(|task| task.is_final)
            .collect::<Vec<_>>();
        assert_eq!(finals.len(), 1);
        let manifest = finals[0];
        assert_eq!(manifest.kind, media::KIND_MANIFEST);
        assert_eq!(
            manifest.dependencies.len(),
            plan.tasks.len() - 1,
            "the fan-in manifest depends on every derivative"
        );
        assert_eq!(manifest.input_kinds.len(), manifest.dependencies.len());
        for task in &plan.tasks {
            assert_eq!(
                task.input_kinds.len(),
                task.dependencies.len(),
                "{} must declare one input kind per dependency",
                task.kind
            );
        }
        let ordinals = plan
            .tasks
            .iter()
            .map(|task| task.ordinal)
            .collect::<Vec<_>>();
        assert_eq!(ordinals, (0..9).collect::<Vec<_>>());
        let decode_source = plan
            .tasks
            .iter()
            .filter(|task| task.dependencies.is_empty() && task.kind != media::KIND_MANIFEST)
            .map(|task| task.kind.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            decode_source,
            vec![
                media::KIND_PROXY,
                media::KIND_AUDIO_16K,
                media::KIND_AUDIO_48K,
                media::KIND_REFERENCE_INDEX
            ],
            "only the proxy decodes source video; everything else chains off artifacts"
        );
    }

    #[test]
    fn audio_only_sources_skip_the_video_chain() {
        let plan = JobPlan::ingest_source(
            &ProjectId::new(),
            "src_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_owned(),
            Vec::new(),
            false,
            true,
            7,
        )
        .expect("plan");
        let kinds = kinds(&plan);
        assert!(!kinds.contains(&media::KIND_PROXY));
        assert!(!kinds.contains(&media::KIND_FILMSTRIP));
        assert!(!kinds.contains(&media::KIND_FRAMES));
        assert!(kinds.contains(&media::KIND_AUDIO_16K));
        assert!(kinds.contains(&media::KIND_LOUDNESS));
        assert!(kinds.contains(&media::KIND_AUDIO_PEAKS));
        assert!(kinds.contains(&media::KIND_REFERENCE_INDEX));
        assert!(kinds.contains(&media::KIND_MANIFEST));
    }

    #[test]
    fn video_only_sources_skip_the_audio_chain() {
        let plan = JobPlan::ingest_source(
            &ProjectId::new(),
            "src_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_owned(),
            Vec::new(),
            true,
            false,
            7,
        )
        .expect("plan");
        let kinds = kinds(&plan);
        assert!(kinds.contains(&media::KIND_PROXY));
        assert!(kinds.contains(&media::KIND_FILMSTRIP));
        assert!(kinds.contains(&media::KIND_FRAMES));
        assert!(!kinds.contains(&media::KIND_AUDIO_16K));
        assert!(!kinds.contains(&media::KIND_AUDIO_48K));
        assert!(!kinds.contains(&media::KIND_LOUDNESS));
        assert!(!kinds.contains(&media::KIND_AUDIO_PEAKS));
    }

    #[test]
    fn streamless_sources_are_rejected_at_planning() {
        let rejected = JobPlan::ingest_source(
            &ProjectId::new(),
            "src_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_owned(),
            Vec::new(),
            false,
            false,
            7,
        );
        assert!(rejected.is_err());
    }
}

#[cfg(test)]
mod resource_tests {
    use std::collections::BTreeSet;

    use super::{ResourceCapacity, ResourceDeclaration};

    #[test]
    fn measured_backend_availability_controls_accelerator_admission() {
        let backends = BTreeSet::from(["videotoolbox".to_owned()]);
        let mut capacity =
            ResourceCapacity::measured(4, 1024 * 1024 * 1024).with_available_backends(&backends);
        let mut resources = ResourceDeclaration::demo();
        resources.accelerator_class = "vaapi".to_owned();
        assert!(!capacity.reserve(&resources));
        resources.accelerator_class = "videotoolbox".to_owned();
        assert!(capacity.reserve(&resources));
        capacity.release(&resources);
        assert!(capacity.reserve(&resources));
    }
}

#[cfg(test)]
mod shots_tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use clipmill_contracts::proto::ipc::v1::{
        DetectShotsPayloadV1, ShotDetectionV1, ShotsStagePayloadV1,
    };
    use clipmill_core::ProjectId;
    use prost::Message;

    use super::{JobPlan, SHOTS_IMPLEMENTATION, SHOTS_STAGE_KEY_VERSION, ShotsProxy};

    const PROXY: &str = "sha256:9c0f000000000000000000000000000000000000000000000000000000000031";
    const FINGERPRINT: &str =
        "sha256:31ab000000000000000000000000000000000000000000000000000000000007";
    const BOM: &str = "ffmpeg-8.1.2-btb-n8.1.2";

    fn plan(request: &DetectShotsPayloadV1) -> JobPlan {
        JobPlan::detect_shots(
            &ProjectId::new(),
            "src_0123456789abcdefghijklmnop".to_owned(),
            ShotsProxy {
                artifact_id: PROXY,
                source_fingerprint: FINGERPRINT,
            },
            request,
            BOM,
            1_700_000_000_000,
        )
    }

    fn request() -> DetectShotsPayloadV1 {
        DetectShotsPayloadV1 {
            key_version: "clipmill.detect-shots.v1".to_owned(),
            source_id: "src_0123456789abcdefghijklmnop".to_owned(),
            detection: None,
        }
    }

    fn stage_payload(plan: &JobPlan) -> ShotsStagePayloadV1 {
        ShotsStagePayloadV1::decode(plan.tasks[0].payload.as_slice()).expect("a shots payload")
    }

    /// One task, and it declares the proxy it reads. A plan that asked for
    /// nothing would be leased before ingest finished.
    #[test]
    fn the_plan_is_one_task_over_the_proxy() {
        let plan = plan(&request());
        assert_eq!(plan.kind, "detect-shots");
        assert_eq!(plan.tasks.len(), 1);
        let task = &plan.tasks[0];
        assert_eq!(task.kind, "detect-shots");
        assert_eq!(task.input_artifact_ids, [PROXY]);
        // Empty because nothing in this plan produces the proxy. It was one
        // entry with no dependency behind it, which the plan validator rejects —
        // so this job could not be submitted at all until the proxy moved to the
        // declared list.
        assert!(task.input_kinds.is_empty());
        assert_eq!(task.output_kind, "evidence.shots.v1");
        assert_eq!(task.implementation, SHOTS_IMPLEMENTATION);
        assert!(task.is_final);
        // Nothing to accelerate, so nothing is demanded of the machine that
        // takes it. A class here would refuse workers that can do this fine.
        assert!(task.resources.accelerator_class.is_empty());
        assert_eq!(task.resources.vram_bytes, 0);
    }

    /// The payload is hashed into the artifact key, so what it contains is what
    /// re-running is sensitive to. A path in here would give the same footage
    /// two addresses on two machines.
    #[test]
    fn the_stage_payload_names_the_decoder_build_and_no_path() {
        let payload = stage_payload(&plan(&request()));
        assert_eq!(payload.key_version, SHOTS_STAGE_KEY_VERSION);
        assert_eq!(payload.stage, "detect-shots");
        assert_eq!(payload.source_fingerprint, FINGERPRINT);
        assert_eq!(payload.decoder_bom, BOM);
        let encoded = String::from_utf8_lossy(&plan(&request()).tasks[0].payload).into_owned();
        assert!(
            !encoded.contains('/'),
            "the keyed payload carries a path: {encoded}"
        );
    }

    /// A caller with no opinion leaves the knobs at zero and the worker
    /// resolves the defaults. Writing the daemon's defaults into the payload
    /// instead would make a later change to them invalidate every cached
    /// detection, including the ones nobody re-tuned.
    #[test]
    fn an_unopinionated_request_keys_without_numbers() {
        let payload = stage_payload(&plan(&request()));
        assert!(
            payload.detection.is_none(),
            "the daemon's defaults reached the key, so changing one would \
             invalidate every cached detection"
        );
        let mut deliberate = request();
        deliberate.detection = Some(ShotDetectionV1::default());
        // An explicit all-zero block is the same request said out loud, and
        // must key the same way — otherwise a caller who filled the struct in
        // gets a cache miss against a caller who left it out.
        assert_eq!(
            stage_payload(&plan(&deliberate))
                .detection
                .unwrap_or_default(),
            ShotDetectionV1::default()
        );
    }

    /// Re-tuning is a different observation, not a correction of this one.
    #[test]
    fn a_retuned_threshold_changes_the_keyed_payload() {
        let mut tuned = request();
        tuned.detection = Some(ShotDetectionV1 {
            threshold: 31.5,
            min_shot_ticks: 45_045,
            analysis_height: 180,
        });
        assert_ne!(
            plan(&request()).tasks[0].payload,
            plan(&tuned).tasks[0].payload
        );
    }

    /// Re-pinning the decoder invalidates shot detections, because a different
    /// build can hand the detector different pixels. Nothing else moves.
    #[test]
    fn a_repinned_decoder_changes_the_keyed_payload() {
        let request = request();
        let here = plan(&request);
        let elsewhere = JobPlan::detect_shots(
            &ProjectId::new(),
            "src_0123456789abcdefghijklmnop".to_owned(),
            ShotsProxy {
                artifact_id: PROXY,
                source_fingerprint: FINGERPRINT,
            },
            &request,
            "ffmpeg-9.0.0-btb-n9.0.0",
            1_700_000_000_000,
        );
        assert_ne!(here.tasks[0].payload, elsewhere.tasks[0].payload);
    }

    /// Two plans for the same source at different times must key identically,
    /// or a warm run is never a cache hit.
    #[test]
    fn the_same_request_keys_identically_whenever_it_is_planned() {
        let request = request();
        let early = plan(&request);
        let late = JobPlan::detect_shots(
            &ProjectId::new(),
            "src_0123456789abcdefghijklmnop".to_owned(),
            ShotsProxy {
                artifact_id: PROXY,
                source_fingerprint: FINGERPRINT,
            },
            &request,
            BOM,
            1_900_000_000_000,
        );
        assert_eq!(early.tasks[0].payload, late.tasks[0].payload);
    }
}

#[cfg(test)]
mod index_tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use clipmill_contracts::proto::ipc::v1::{IndexStagePayloadV1, IndexTranscriptPayloadV1};
    use clipmill_core::ProjectId;
    use prost::Message;

    use super::{EvidenceInputs, INDEX_STAGE_KEY_VERSION, JobPlan, evidence};

    const TRANSCRIPT: &str =
        "sha256:7a11000000000000000000000000000000000000000000000000000000000042";
    const SHOTS: &str = "sha256:9c0f000000000000000000000000000000000000000000000000000000000031";

    fn plan(shots: Option<&str>) -> JobPlan {
        JobPlan::index_transcript(
            &ProjectId::new(),
            "src_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_owned(),
            EvidenceInputs {
                transcript: TRANSCRIPT,
                shots,
            },
            &IndexTranscriptPayloadV1 {
                key_version: "clipmill.index-transcript.v1".to_owned(),
                source_id: "src_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_owned(),
            },
            7,
        )
    }

    fn payload(plan: &JobPlan) -> IndexStagePayloadV1 {
        IndexStagePayloadV1::decode(plan.tasks[0].payload.as_slice()).expect("a stage payload")
    }

    #[test]
    fn the_index_is_one_builtin_task_that_names_what_it_reads() {
        let plan = plan(None);
        assert_eq!(plan.tasks.len(), 1);
        let task = &plan.tasks[0];
        assert_eq!(task.kind, evidence::KIND_INDEX);
        assert_eq!(task.output_kind, "index.transcript.v1");
        assert_eq!(task.implementation, evidence::IMPLEMENTATION);
        assert!(task.is_final);
        // Nothing to depend on: both documents were published by earlier jobs.
        assert!(task.dependencies.is_empty());
        let payload = payload(&plan);
        assert_eq!(payload.key_version, INDEX_STAGE_KEY_VERSION);
        assert_eq!(payload.stage, evidence::KIND_INDEX);
        assert_eq!(task.input_artifact_ids, [TRANSCRIPT]);
        // The list describes dependencies, and there are none: an entry here
        // with nothing behind it would fail the plan validator.
        assert!(task.input_kinds.is_empty());
    }

    /// A source with no video reads one document; a source whose cuts were found
    /// reads two, and the second reaches the key through the declared inputs
    /// rather than through the payload — which is what lets an index built inside
    /// an analysis be the same artifact as one built on its own.
    #[test]
    fn shot_cuts_change_the_declared_inputs_and_not_the_payload() {
        let without = plan(None);
        let with = plan(Some(SHOTS));
        assert_eq!(without.tasks[0].input_artifact_ids, [TRANSCRIPT]);
        assert_eq!(with.tasks[0].input_artifact_ids, [TRANSCRIPT, SHOTS]);
        assert_eq!(
            without.tasks[0].payload, with.tasks[0].payload,
            "the payload says which stage this is, and that has not changed"
        );
    }

    /// The index runs no model and needs no accelerator, so it must not declare
    /// one — a task that asked for Metal here would sit unscheduled on a
    /// machine that has none.
    #[test]
    fn the_index_asks_for_nothing_it_does_not_use() {
        let plan = plan(None);
        let resources = &plan.tasks[0].resources;
        assert!(resources.accelerator_class.is_empty());
        assert_eq!(resources.vram_bytes, 0);
        assert_eq!(resources.network_policy, "local-lock");
        assert_eq!(resources.determinism_class, "deterministic");
    }

    /// Paths are machine-specific and content addresses are not. The payload is
    /// hashed into the key, so the difference decides whether the same
    /// transcript indexes to one address everywhere or to a different one on
    /// every machine.
    #[test]
    fn the_keyed_payload_carries_no_path() {
        let encoded = plan(Some(SHOTS)).tasks[0].payload.clone();
        let text = String::from_utf8_lossy(&encoded);
        assert!(!text.contains('/'), "a path reached the artifact key");
        assert!(!text.contains(std::env::temp_dir().to_string_lossy().as_ref()));
    }
}

#[cfg(test)]
mod discovery_tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use clipmill_contracts::proto::ipc::v1::{
        ClipDurationV1, DiscoverCandidatesPayloadV1, DiscoverStagePayloadV1,
    };
    use clipmill_core::ProjectId;
    use prost::Message;

    use super::{DISCOVER_STAGE_KEY_VERSION, DiscoveryInputs, JobPlan, discovery};

    const INDEX: &str = "sha256:1de0000000000000000000000000000000000000000000000000000000000011";
    const TRANSCRIPT: &str =
        "sha256:7a11000000000000000000000000000000000000000000000000000000000042";
    const LOUDNESS: &str =
        "sha256:10ad000000000000000000000000000000000000000000000000000000000099";

    fn plan(loudness: Option<&str>, duration: Option<ClipDurationV1>) -> JobPlan {
        JobPlan::discover_candidates(
            &ProjectId::new(),
            "src_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_owned(),
            DiscoveryInputs {
                index: INDEX,
                transcript: TRANSCRIPT,
                loudness,
            },
            &DiscoverCandidatesPayloadV1 {
                key_version: "clipmill.discover-candidates.v1".to_owned(),
                source_id: "src_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_owned(),
                duration,
            },
            7,
        )
    }

    fn payload(plan: &JobPlan) -> DiscoverStagePayloadV1 {
        DiscoverStagePayloadV1::decode(plan.tasks[0].payload.as_slice()).expect("a stage payload")
    }

    #[test]
    fn discovery_is_one_builtin_task_that_names_what_it_reads() {
        let plan = plan(None, None);
        assert_eq!(plan.tasks.len(), 1);
        let task = &plan.tasks[0];
        assert_eq!(task.kind, discovery::KIND_DISCOVER);
        assert_eq!(task.output_kind, "discovery.candidates.v1");
        assert_eq!(task.implementation, discovery::IMPLEMENTATION);
        assert!(task.is_final);
        assert!(task.dependencies.is_empty());
        let payload = payload(&plan);
        assert_eq!(payload.key_version, DISCOVER_STAGE_KEY_VERSION);
        assert_eq!(task.input_artifact_ids, [INDEX, TRANSCRIPT]);
        assert!(task.input_kinds.is_empty());
    }

    /// A source with no audio reads two documents; one whose loudness was
    /// measured reads three, and the third reaches the key through the declared
    /// inputs. The two searches are still different observations, because the
    /// recipe covers what was read.
    #[test]
    fn prosody_changes_the_declared_inputs_and_not_the_payload() {
        let silent = plan(None, None);
        let heard = plan(Some(LOUDNESS), None);
        assert_eq!(silent.tasks[0].input_artifact_ids, [INDEX, TRANSCRIPT]);
        assert_eq!(
            heard.tasks[0].input_artifact_ids,
            [INDEX, TRANSCRIPT, LOUDNESS]
        );
        assert_eq!(silent.tasks[0].payload, heard.tasks[0].payload);
    }

    /// Asking for a different clip length is a different search, so it must
    /// reach the key rather than filtering a shared result.
    #[test]
    fn the_requested_length_reaches_the_keyed_payload() {
        let default = plan(None, None);
        let asked = plan(
            None,
            Some(ClipDurationV1 {
                min_ticks: 30 * 90_000,
                max_ticks: 60 * 90_000,
            }),
        );
        assert_ne!(default.tasks[0].payload, asked.tasks[0].payload);
        let duration = payload(&asked).duration.expect("a stated range");
        assert_eq!(duration.min_ticks, 30 * 90_000);
    }

    #[test]
    fn discovery_asks_for_nothing_it_does_not_use() {
        let resources = &plan(None, None).tasks[0].resources;
        assert!(resources.accelerator_class.is_empty());
        assert_eq!(resources.vram_bytes, 0);
        assert_eq!(resources.network_policy, "local-lock");
        assert_eq!(resources.determinism_class, "deterministic");
    }

    #[test]
    fn the_keyed_payload_carries_no_path() {
        let encoded = plan(Some(LOUDNESS), None).tasks[0].payload.clone();
        let text = String::from_utf8_lossy(&encoded);
        assert!(!text.contains('/'), "a path reached the artifact key");
    }
}

#[cfg(test)]
mod analyze_tests {
    #![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

    use std::collections::BTreeSet;
    use std::path::Path;

    use clipmill_contracts::proto::ipc::v1::AnalyzeSourcePayloadV1;
    use clipmill_core::ProjectId;

    use super::{
        AnalyzeSource, JobPlan, SpeechAudio, TaskSpec, TranscribeSourcePayloadV1, analysis,
    };

    const SOURCE: &str = "src_01ARZ3NDEKTSV4RRFFQ69G5FAV";
    const FINGERPRINT: &str =
        "sha256:1111111111111111111111111111111111111111111111111111111111111111";
    const AUDIO: &str = "sha256:a0d1000000000000000000000000000000000000000000000000000000000001";
    const BOM: &str = "ffmpeg-8.1.2-btb-n8.1.2";

    fn models() -> crate::models::ModelRegistry {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../models/registry");
        crate::models::ModelRegistry::load(&path).expect("the published registry loads")
    }

    fn request() -> AnalyzeSourcePayloadV1 {
        AnalyzeSourcePayloadV1 {
            key_version: "clipmill.analyze-source.v1".to_owned(),
            source_id: SOURCE.to_owned(),
            language: "en".to_owned(),
            duration: None,
            count: 0,
            diversity_milli: 0,
        }
    }

    fn plan(has_video: bool, has_audio: bool) -> JobPlan {
        JobPlan::analyze_source(
            &ProjectId::new(),
            AnalyzeSource {
                source_id: SOURCE,
                source_fingerprint: FINGERPRINT,
                has_video,
                has_audio,
            },
            &request(),
            &models(),
            &crate::selection::Bindings::portable(),
            BOM,
            7,
        )
        .expect("a source with streams plans")
    }

    fn task<'a>(plan: &'a JobPlan, kind: &str) -> &'a TaskSpec {
        plan.tasks
            .iter()
            .find(|task| task.kind == kind)
            .unwrap_or_else(|| panic!("no {kind} task"))
    }

    /// Every stage from the probe to the ranked set, in one job with one root.
    #[test]
    fn the_dag_runs_every_stage_and_roots_exactly_one_artifact() {
        let plan = plan(true, true);
        assert_eq!(plan.kind, "analyze-source");
        let kinds = plan
            .tasks
            .iter()
            .map(|task| task.kind.as_str())
            .collect::<BTreeSet<_>>();
        for expected in [
            "probe-source",
            "ingest-proxy",
            "ingest-audio-16k",
            "ingest-manifest",
            "speech-vad",
            "speech-asr",
            "speech-align",
            "speech-transcript",
            "detect-shots",
            "index-transcript",
            "discover-candidates",
            "rank-candidates",
            analysis::KIND_MANIFEST,
        ] {
            assert!(kinds.contains(expected), "the DAG has no {expected}");
        }
        let finals = plan
            .tasks
            .iter()
            .filter(|task| task.is_final)
            .collect::<Vec<_>>();
        assert_eq!(finals.len(), 1, "a job roots exactly one artifact");
        assert_eq!(finals[0].kind, analysis::KIND_MANIFEST);
    }

    /// The fan-in depends on every stage, because that is what makes the whole
    /// analysis reachable from the one artifact the job roots.
    #[test]
    fn the_fan_in_depends_on_every_stage_that_published_something() {
        let plan = plan(true, true);
        let manifest = task(&plan, analysis::KIND_MANIFEST);
        assert_eq!(manifest.dependencies.len(), 10);
        assert_eq!(manifest.input_kinds.len(), manifest.dependencies.len());
        // Nothing declared: every input is a task in this plan.
        assert!(manifest.input_artifact_ids.is_empty());
        for kind in [
            "evidence.source_map.v1",
            "media.ingest_manifest.v1",
            "speech.vad.v1",
            "speech.asr.v1",
            "speech.alignment.v1",
            "speech.transcript.v1",
            "evidence.shots.v1",
            "index.transcript.v1",
            "discovery.candidates.v1",
            "ranking.set.v1",
        ] {
            assert!(
                manifest.input_kinds.iter().any(|named| named == kind),
                "the fan-in does not name {kind}"
            );
        }
    }

    /// The property the whole declared-input mechanism exists for: a leased stage
    /// reached by both routes must be handed the same inputs in the same order,
    /// because that order is part of its artifact key. If this drifts, one
    /// transcript gets two content addresses and the cache silently doubles.
    #[test]
    fn the_two_routes_deliver_one_input_order() {
        let standalone = JobPlan::transcribe_source(
            &ProjectId::new(),
            SOURCE.to_owned(),
            SpeechAudio {
                artifact_id: AUDIO,
                source_fingerprint: FINGERPRINT,
            },
            &TranscribeSourcePayloadV1 {
                key_version: "clipmill.transcribe-source.v1".to_owned(),
                source_id: SOURCE.to_owned(),
                language: "en".to_owned(),
                detection: None,
            },
            &models(),
            &crate::selection::Bindings::portable(),
            7,
        );
        let in_dag = plan(true, true);
        let audio_task = task(&in_dag, "ingest-audio-16k").task_id.clone();

        for stage in ["speech-vad", "speech-asr", "speech-align"] {
            let alone = task(&standalone, stage);
            let inside = task(&in_dag, stage);
            // The payload is hashed into the key, so it has to be byte-identical:
            // an address present on one route and absent on the other would be
            // two keys for one observation.
            assert_eq!(
                alone.payload, inside.payload,
                "{stage} encodes different payloads on the two routes"
            );
            // What the lease will deliver: declared first, then dependency
            // outputs in order. The audio has to land first on both.
            let alone_order = [alone.input_artifact_ids.len(), alone.dependencies.len()];
            let inside_order = [inside.input_artifact_ids.len(), inside.dependencies.len()];
            assert_eq!(
                alone_order.iter().sum::<usize>(),
                inside_order.iter().sum::<usize>(),
                "{stage} reads a different number of inputs on the two routes"
            );
            assert_eq!(
                alone.input_artifact_ids,
                [AUDIO],
                "{stage} standalone must declare the audio it reads"
            );
            assert_eq!(
                inside.dependencies.first(),
                Some(&audio_task),
                "{stage} inside the DAG must take the audio as its first dependency"
            );
            assert!(
                inside.input_artifact_ids.is_empty(),
                "{stage} inside the DAG declares nothing: the plan produces it"
            );
        }
    }

    /// A source with no video has no shot cuts, and the difference between that
    /// and nobody looking is what the skip list carries.
    #[test]
    fn a_source_with_no_video_skips_shot_detection_and_says_so() {
        let plan = plan(false, true);
        assert!(plan.tasks.iter().all(|task| task.kind != "detect-shots"));
        let skipped = skipped_of(&plan);
        assert_eq!(
            skipped,
            vec![("evidence.shots.v1".to_owned(), "no_video".to_owned())]
        );
        // Everything the transcript feeds still runs.
        for kind in ["index-transcript", "discover-candidates", "rank-candidates"] {
            assert!(plan.tasks.iter().any(|task| task.kind == kind));
        }
    }

    /// A source with no audio has no transcript, so the four speech stages and
    /// the three that read a transcript are all absent — each with the reason.
    #[test]
    fn a_source_with_no_audio_skips_everything_that_needs_speech() {
        let plan = plan(true, false);
        let skipped = skipped_of(&plan);
        assert_eq!(
            skipped
                .iter()
                .map(|(kind, _)| kind.as_str())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([
                "speech.vad.v1",
                "speech.asr.v1",
                "speech.alignment.v1",
                "speech.transcript.v1",
                "index.transcript.v1",
                "discovery.candidates.v1",
                "ranking.set.v1",
            ])
        );
        assert!(skipped.iter().all(|(_, reason)| reason == "no_audio"));
        assert!(plan.tasks.iter().any(|task| task.kind == "detect-shots"));
    }

    fn skipped_of(plan: &JobPlan) -> Vec<(String, String)> {
        use prost::Message;
        let manifest = plan
            .tasks
            .iter()
            .find(|task| task.kind == analysis::KIND_MANIFEST)
            .expect("a fan-in");
        let payload =
            super::AnalysisStagePayloadV1::decode(manifest.payload.as_slice()).expect("a payload");
        payload
            .skipped
            .iter()
            .map(|stage| (stage.kind.clone(), stage.reason.clone()))
            .collect()
    }

    /// A source with neither is not an analysis anyone can plan, and saying so
    /// beats fanning out tasks that each fail to find their input.
    #[test]
    fn a_source_with_no_streams_is_refused() {
        let refused = JobPlan::analyze_source(
            &ProjectId::new(),
            AnalyzeSource {
                source_id: SOURCE,
                source_fingerprint: FINGERPRINT,
                has_video: false,
                has_audio: false,
            },
            &request(),
            &models(),
            &crate::selection::Bindings::portable(),
            BOM,
            7,
        );
        assert!(refused.is_err());
    }

    /// Ordinals come from three builders that each number from zero, so the last
    /// pass has to make them unique — the store rejects a plan where they are not.
    #[test]
    fn ordinals_are_unique_across_every_builder_that_contributed() {
        for (has_video, has_audio) in [(true, true), (true, false), (false, true)] {
            let plan = plan(has_video, has_audio);
            let ordinals = plan
                .tasks
                .iter()
                .map(|task| task.ordinal)
                .collect::<BTreeSet<_>>();
            assert_eq!(ordinals.len(), plan.tasks.len());
        }
    }
}
