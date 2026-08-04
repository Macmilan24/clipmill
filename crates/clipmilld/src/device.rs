use std::{
    collections::BTreeSet,
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use clipmill_contracts::schemas::device_profile::DeviceProfile;
use clipmill_core::{ArtifactId, Sha256Digest};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::Serialize;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::{process::Command, sync::OnceCell, time::timeout};
use ulid::Ulid;

use crate::{
    jobs::ResourceCapacity,
    models::ModelRegistry,
    selection::{Bindings, measure as measure_selection},
    shm::benchmark_shared_memory,
};

const PROFILE_SCHEMA: &str = "clipmill.device_profile.v1";
const FINGERPRINT_DOMAIN: &[u8] = b"clipmill.device.fingerprint.v1\0";
const PROFILE_SIGNING_DOMAIN: &[u8] = b"clipmill.device.attestation.v1\0";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(8);
const MAX_COMMAND_OUTPUT: usize = 64 * 1024;
const SHARED_MEMORY_SAMPLE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone)]
pub(crate) struct DeviceProfiler {
    inner: Arc<ProfilerInner>,
}

struct ProfilerInner {
    ffmpeg: PathBuf,
    scratch: PathBuf,
    /// Where `tools/bench/speech-benchmark.py` leaves what it measured. Inside
    /// the daemon's own 0700 state directory, so writing it already requires
    /// being the user the daemon runs as — the same standing the attestation
    /// key has.
    speech_benchmark: PathBuf,
    /// What the registry pins right now. A benchmark is believed only where it
    /// names these digests, so a re-pinned weight retires its own measurement.
    models: Arc<ModelRegistry>,
    signing_key: SigningKey,
    identity: OnceCell<DeviceIdentity>,
}

impl std::fmt::Debug for DeviceProfiler {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DeviceProfiler")
            .field("ffmpeg", &self.inner.ffmpeg)
            .field("scratch", &self.inner.scratch)
            .field("identity_cached", &self.inner.identity.initialized())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, Serialize)]
struct DeviceIdentity {
    platform: PlatformIdentity,
    cpu: CpuIdentity,
    total_memory_bytes: u64,
    accelerators: Vec<AcceleratorIdentity>,
    ffmpeg_identity: String,
    ffmpeg_available: bool,
    ffmpeg_hwaccels: Vec<String>,
    ffmpeg_encoders: Vec<String>,
    driver_identities: Vec<RuntimeIdentity>,
    hardware_fingerprint: String,
}

#[derive(Clone, Debug, Serialize)]
struct PlatformIdentity {
    os: String,
    arch: String,
    os_version: String,
}

#[derive(Clone, Debug, Serialize)]
struct CpuIdentity {
    model: String,
    logical_cores: u32,
    physical_cores: u32,
}

#[derive(Clone, Debug, Serialize)]
struct AcceleratorIdentity {
    kind: String,
    name: String,
}

#[derive(Clone, Debug, Serialize)]
struct RuntimeIdentity {
    kind: String,
    identity: String,
    available: bool,
}

#[derive(Clone, Debug)]
struct CodecMeasurements {
    codec: String,
    encode_fps: Option<f64>,
    decode_fps: Option<f64>,
}

#[derive(Clone, Debug)]
struct HardwareMeasurement {
    backend: String,
    milliseconds: Option<f64>,
    unavailable_reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedDeviceProfile {
    pub hardware_fingerprint: String,
    pub measurement_generation: u64,
    pub logical_cores: u32,
    pub available_memory_bytes: u64,
    pub available_backends: BTreeSet<String>,
    /// Which implementation each stage is bound to on this device. Read from
    /// the same signed bytes as everything else here: a binding that arrived
    /// by any other road is one nobody attested.
    pub(crate) bindings: Bindings,
}

#[derive(Debug, Error)]
pub enum DeviceProfileError {
    #[error("device profile I/O failed at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("device profile JSON is invalid: {0}")]
    Json(String),
    #[error("device attestation key is invalid")]
    InvalidKey,
    #[error("device profile signature is invalid")]
    InvalidSignature,
    #[error("device profile fingerprint does not match this runtime")]
    FingerprintMismatch,
    #[error("device measurement command timed out")]
    CommandTimeout,
    #[error("device measurement task stopped")]
    TaskStopped,
    #[error("device shared-memory measurement failed: {0}")]
    SharedMemory(String),
}

impl DeviceProfiler {
    pub(crate) fn new(
        ffprobe: &Path,
        attestation_key: &Path,
        scratch: &Path,
        speech_benchmark: &Path,
        models: Arc<ModelRegistry>,
    ) -> Result<Self, DeviceProfileError> {
        create_private_directory(scratch)?;
        let signing_key = load_or_create_signing_key(attestation_key)?;
        Ok(Self {
            inner: Arc::new(ProfilerInner {
                ffmpeg: ffmpeg_sibling(ffprobe),
                scratch: scratch.to_path_buf(),
                speech_benchmark: speech_benchmark.to_path_buf(),
                models,
                signing_key,
                identity: OnceCell::new(),
            }),
        })
    }

    pub(crate) async fn hardware_fingerprint(&self) -> Result<String, DeviceProfileError> {
        Ok(self.identity().await?.hardware_fingerprint.clone())
    }

    pub(crate) async fn scheduler_capacity(&self) -> Result<ResourceCapacity, DeviceProfileError> {
        let identity = self.identity().await?;
        let available_memory = measured_available_memory(identity.total_memory_bytes).await;
        Ok(ResourceCapacity::measured(
            identity.cpu.logical_cores,
            available_memory,
            measured_available_disk(&self.inner.scratch),
        ))
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) async fn measure(
        &self,
        expected_fingerprint: &str,
        measurement_generation: u64,
    ) -> Result<String, DeviceProfileError> {
        let identity = self.identity().await?.clone();
        if identity.hardware_fingerprint != expected_fingerprint {
            return Err(DeviceProfileError::FingerprintMismatch);
        }
        let available_memory = measured_available_memory(identity.total_memory_bytes).await;
        let shared_memory =
            tokio::task::spawn_blocking(|| benchmark_shared_memory(SHARED_MEMORY_SAMPLE_BYTES))
                .await
                .map_err(|_| DeviceProfileError::TaskStopped)?
                .map_err(|error| DeviceProfileError::SharedMemory(error.to_string()))?;
        let (codec, hardware) = self.measure_ffmpeg(&identity).await;

        let accelerators = identity
            .accelerators
            .iter()
            .map(|accelerator| {
                json!({
                    "kind": accelerator.kind,
                    "name": accelerator.name,
                })
            })
            .collect::<Vec<_>>();
        let decode = codec
            .decode_fps
            .map(|fps| {
                vec![json!({
                    "codec": codec.codec,
                    "fps_measured": fps,
                    "hardware": false,
                    "height": 90,
                })]
            })
            .unwrap_or_default();
        let encode = codec
            .encode_fps
            .map(|fps| {
                vec![json!({
                    "codec": codec.codec,
                    "fps_measured": fps,
                    "hardware": false,
                    "height": 90,
                })]
            })
            .unwrap_or_default();
        // Taken before the profile is built, because two of its fields depend
        // on it: the selection block itself, and the accelerators the
        // scheduler is willing to admit workers onto.
        let selection = measure_selection(
            &self.inner.speech_benchmark,
            &identity.hardware_fingerprint,
            &self.inner.models,
        );
        let cpu_available = codec.encode_fps.is_some() && codec.decode_fps.is_some();
        let hardware_available = hardware.milliseconds.is_some();
        let mut capability_results = vec![json!({
            "available": cpu_available,
            "backend": "cpu",
            "capability": "video-roundtrip",
            "detail": if cpu_available { "bounded synthetic round trip completed" } else { "FFmpeg CPU round trip unavailable" },
        })];
        capability_results.push(json!({
            "available": hardware_available,
            "backend": hardware.backend,
            "capability": "video-roundtrip",
            "detail": hardware.unavailable_reason.clone().unwrap_or_else(|| "bounded synthetic round trip completed".to_owned()),
        }));
        // An accelerator counts as available once a model has been measured
        // running on it. Nothing here probes for a device: a benchmark bound
        // to this hardware that reported a real-time factor is proof the
        // runtime works, where a probe would only be proof that a driver
        // answered.
        for class in &selection.proven_accelerators {
            capability_results.push(json!({
                "available": true,
                "backend": class,
                "capability": "model-inference",
                "detail": "an implementation was measured running on this accelerator",
            }));
        }
        let hardware_roundtrip = if let Some(milliseconds) = hardware.milliseconds {
            json!({
                "available": true,
                "backend": hardware.backend,
                "milliseconds": milliseconds,
            })
        } else {
            json!({
                "available": false,
                "backend": hardware.backend,
                "unavailable_reason": hardware.unavailable_reason.unwrap_or_else(|| "no measured hardware path".to_owned()),
            })
        };
        let mut profile = json!({
            "accelerators": accelerators,
            "cpu": {
                "logical_cores": identity.cpu.logical_cores,
                "model": identity.cpu.model,
                "physical_cores": identity.cpu.physical_cores,
            },
            "measured": {
                "decode": decode,
                "encode": encode,
                "ffmpeg_build": expected_ffmpeg_build(),
            },
            "memory": {
                "total_bytes": identity.total_memory_bytes,
                "unified": cfg!(all(target_os = "macos", target_arch = "aarch64")),
            },
            "phase0": {
                "available_memory_bytes": available_memory,
                "capability_results": capability_results,
                "hardware_fingerprint": identity.hardware_fingerprint,
                "hardware_roundtrip": hardware_roundtrip,
                "measurement_generation": measurement_generation,
                "runtime_identities": std::iter::once(RuntimeIdentity {
                    available: identity.ffmpeg_available,
                    identity: identity.ffmpeg_identity.clone(),
                    kind: "ffmpeg".to_owned(),
                }).chain(identity.driver_identities.clone()).collect::<Vec<_>>(),
                "shared_memory": {
                    "bytes_per_second": shared_memory.1,
                    "sample_bytes": shared_memory.0,
                },
            },
            "platform": {
                "arch": identity.platform.arch,
                "os": identity.platform.os,
                "os_version": identity.platform.os_version,
            },
            "schema_version": PROFILE_SCHEMA,
            // Which implementation each capability is bound to, and the
            // measurement behind it (D19). Signed with everything else, so a
            // binding cannot be edited into the profile after the fact.
            "selection": selection.value,
        });
        let unsigned = canonical_unsigned_profile(&profile)?;
        let mut signing_preimage =
            Vec::with_capacity(PROFILE_SIGNING_DOMAIN.len() + unsigned.len());
        signing_preimage.extend_from_slice(PROFILE_SIGNING_DOMAIN);
        signing_preimage.extend_from_slice(&unsigned);
        let signature = self.inner.signing_key.sign(&signing_preimage);
        let phase0 = profile
            .get_mut("phase0")
            .and_then(Value::as_object_mut)
            .ok_or_else(|| DeviceProfileError::Json("phase0 object is missing".to_owned()))?;
        phase0.insert(
            "attestation".to_owned(),
            json!({
                "algorithm": "ed25519",
                "public_key": hex::encode(self.inner.signing_key.verifying_key().to_bytes()),
                "signature": hex::encode(signature.to_bytes()),
            }),
        );
        let bytes = serde_json_canonicalizer::to_vec(&profile)
            .map_err(|error| DeviceProfileError::Json(error.to_string()))?;
        let text = String::from_utf8(bytes)
            .map_err(|error| DeviceProfileError::Json(error.to_string()))?;
        verify_profile(&text, Some(expected_fingerprint))?;
        Ok(text)
    }

    async fn identity(&self) -> Result<&DeviceIdentity, DeviceProfileError> {
        self.inner
            .identity
            .get_or_try_init(|| async { capture_identity(&self.inner.ffmpeg).await })
            .await
    }

    async fn measure_ffmpeg(
        &self,
        identity: &DeviceIdentity,
    ) -> (CodecMeasurements, HardwareMeasurement) {
        let run_dir = self
            .inner
            .scratch
            .join(format!("measurement-{}", Ulid::new()));
        let created = create_private_directory(&run_dir).is_ok();
        let result = if created {
            self.measure_ffmpeg_in(&run_dir, identity).await
        } else {
            None
        };
        if created {
            let _removed = fs::remove_dir_all(&run_dir);
        }
        result.unwrap_or_else(|| {
            (
                CodecMeasurements {
                    codec: "h264".to_owned(),
                    encode_fps: None,
                    decode_fps: None,
                },
                HardwareMeasurement {
                    backend: preferred_hardware_backend(identity),
                    milliseconds: None,
                    unavailable_reason: Some("bounded FFmpeg measurement unavailable".to_owned()),
                },
            )
        })
    }

    async fn measure_ffmpeg_in(
        &self,
        run_dir: &Path,
        identity: &DeviceIdentity,
    ) -> Option<(CodecMeasurements, HardwareMeasurement)> {
        if !identity.ffmpeg_available {
            return None;
        }
        let sample = run_dir.join("cpu-roundtrip.mkv");
        let (codec, encoder) = if identity
            .ffmpeg_encoders
            .iter()
            .any(|value| value == "libx264")
        {
            ("h264".to_owned(), "libx264")
        } else {
            ("mpeg4".to_owned(), "mpeg4")
        };
        let encode_args = strings([
            "-hide_banner",
            "-nostdin",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=160x90:rate=30",
            "-frames:v",
            "30",
            "-c:v",
            encoder,
            "-y",
        ])
        .into_iter()
        .chain([sample.as_os_str().to_os_string()])
        .collect::<Vec<_>>();
        let encode_started = Instant::now();
        let encode = run_command(&self.inner.ffmpeg, &encode_args, run_dir).await;
        let encode_fps = encode
            .ok()
            .filter(|output| output.status.success())
            .map(|_| frames_per_second(30, encode_started.elapsed()));

        let decode_args = vec![
            OsString::from("-hide_banner"),
            OsString::from("-nostdin"),
            OsString::from("-loglevel"),
            OsString::from("error"),
            OsString::from("-i"),
            sample.as_os_str().to_os_string(),
            OsString::from("-f"),
            OsString::from("null"),
            OsString::from("-"),
        ];
        let decode_started = Instant::now();
        let decode = run_command(&self.inner.ffmpeg, &decode_args, run_dir).await;
        let decode_fps = decode
            .ok()
            .filter(|output| output.status.success())
            .map(|_| frames_per_second(30, decode_started.elapsed()));
        let hardware = self.measure_hardware_roundtrip(run_dir, identity).await;
        Some((
            CodecMeasurements {
                codec,
                encode_fps,
                decode_fps,
            },
            hardware,
        ))
    }

    async fn measure_hardware_roundtrip(
        &self,
        run_dir: &Path,
        identity: &DeviceIdentity,
    ) -> HardwareMeasurement {
        let backend = preferred_hardware_backend(identity);
        let encoder = match backend.as_str() {
            "videotoolbox" => "h264_videotoolbox",
            "vaapi" => "h264_vaapi",
            _ => {
                return HardwareMeasurement {
                    backend,
                    milliseconds: None,
                    unavailable_reason: Some("no supported accelerator was measured".to_owned()),
                };
            }
        };
        if !identity
            .ffmpeg_encoders
            .iter()
            .any(|value| value == encoder)
        {
            return HardwareMeasurement {
                backend,
                milliseconds: None,
                unavailable_reason: Some(
                    "pinned FFmpeg build lacks the hardware encoder".to_owned(),
                ),
            };
        }
        if backend == "vaapi" && !Path::new("/dev/dri/renderD128").exists() {
            return HardwareMeasurement {
                backend,
                milliseconds: None,
                unavailable_reason: Some("VAAPI render device is unavailable".to_owned()),
            };
        }
        let output = run_dir.join("hardware-roundtrip.mp4");
        let mut encode_args = strings([
            "-hide_banner",
            "-nostdin",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=160x90:rate=30",
            "-frames:v",
            "8",
        ]);
        if backend == "vaapi" {
            encode_args.extend(strings([
                "-vaapi_device",
                "/dev/dri/renderD128",
                "-vf",
                "format=nv12,hwupload",
            ]));
        }
        encode_args.extend(strings(["-c:v", encoder, "-y"]));
        encode_args.push(output.as_os_str().to_os_string());
        let started = Instant::now();
        let encode_result = run_command(&self.inner.ffmpeg, &encode_args, run_dir).await;
        if !encode_result.is_ok_and(|output| output.status.success()) {
            return HardwareMeasurement {
                backend,
                milliseconds: None,
                unavailable_reason: Some("hardware encode probe failed".to_owned()),
            };
        }
        let mut decode_args = strings(["-hide_banner", "-nostdin", "-loglevel", "error"]);
        decode_args.extend(strings(["-hwaccel", backend.as_str(), "-i"]));
        decode_args.push(output.as_os_str().to_os_string());
        decode_args.extend(strings(["-f", "null", "-"]));
        let decoded = run_command(&self.inner.ffmpeg, &decode_args, run_dir).await;
        if !decoded.is_ok_and(|output| output.status.success()) {
            return HardwareMeasurement {
                backend,
                milliseconds: None,
                unavailable_reason: Some("hardware decode probe failed".to_owned()),
            };
        }
        HardwareMeasurement {
            backend,
            milliseconds: Some(duration_millis_f64(started.elapsed())),
            unavailable_reason: None,
        }
    }
}

pub fn verify_profile(
    profile_json: &str,
    expected_fingerprint: Option<&str>,
) -> Result<VerifiedDeviceProfile, DeviceProfileError> {
    let mut value: Value = serde_json::from_str(profile_json)
        .map_err(|error| DeviceProfileError::Json(error.to_string()))?;
    serde_json::from_value::<DeviceProfile>(value.clone())
        .map_err(|error| DeviceProfileError::Json(error.to_string()))?;
    // Read before the attestation object is removed, but from the same bytes
    // the signature covers — a binding that is not inside what was signed is
    // not a binding this daemon attested.
    let bindings = Bindings::from_profile(&value);
    let canonical = serde_json_canonicalizer::to_vec(&value)
        .map_err(|error| DeviceProfileError::Json(error.to_string()))?;
    if canonical != profile_json.as_bytes() {
        return Err(DeviceProfileError::Json(
            "profile is not RFC 8785 canonical JSON".to_owned(),
        ));
    }
    let logical_cores = value
        .get("cpu")
        .and_then(|cpu| cpu.get("logical_cores"))
        .and_then(Value::as_u64)
        .and_then(|cores| u32::try_from(cores).ok())
        .filter(|cores| *cores > 0)
        .ok_or_else(|| DeviceProfileError::Json("logical CPU count is invalid".to_owned()))?;
    let phase0 = value
        .get_mut("phase0")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| DeviceProfileError::Json("Phase 0 extension is missing".to_owned()))?;
    let attestation = phase0
        .remove("attestation")
        .and_then(|value| value.as_object().cloned())
        .ok_or_else(|| DeviceProfileError::Json("attestation is missing".to_owned()))?;
    let hardware_fingerprint = phase0
        .get("hardware_fingerprint")
        .and_then(Value::as_str)
        .ok_or_else(|| DeviceProfileError::Json("hardware fingerprint is missing".to_owned()))?
        .to_owned();
    hardware_fingerprint
        .parse::<ArtifactId>()
        .map_err(|_| DeviceProfileError::Json("hardware fingerprint is invalid".to_owned()))?;
    if expected_fingerprint.is_some_and(|expected| expected != hardware_fingerprint) {
        return Err(DeviceProfileError::FingerprintMismatch);
    }
    let measurement_generation = phase0
        .get("measurement_generation")
        .and_then(Value::as_u64)
        .filter(|generation| *generation > 0)
        .ok_or_else(|| DeviceProfileError::Json("measurement generation is invalid".to_owned()))?;
    let available_memory_bytes = phase0
        .get("available_memory_bytes")
        .and_then(Value::as_u64)
        .ok_or_else(|| DeviceProfileError::Json("available memory is invalid".to_owned()))?;
    let available_backends = phase0
        .get("capability_results")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|capability| capability.get("available").and_then(Value::as_bool) == Some(true))
        .filter_map(|capability| capability.get("backend").and_then(Value::as_str))
        .map(str::to_owned)
        .collect();
    let public_key = hex_field(&attestation, "public_key", 32)?;
    let signature = hex_field(&attestation, "signature", 64)?;
    let public_key: [u8; 32] = public_key
        .try_into()
        .map_err(|_| DeviceProfileError::InvalidSignature)?;
    let verifying_key =
        VerifyingKey::from_bytes(&public_key).map_err(|_| DeviceProfileError::InvalidSignature)?;
    let signature =
        Signature::from_slice(&signature).map_err(|_| DeviceProfileError::InvalidSignature)?;
    let unsigned = serde_json_canonicalizer::to_vec(&value)
        .map_err(|error| DeviceProfileError::Json(error.to_string()))?;
    let mut preimage = Vec::with_capacity(PROFILE_SIGNING_DOMAIN.len() + unsigned.len());
    preimage.extend_from_slice(PROFILE_SIGNING_DOMAIN);
    preimage.extend_from_slice(&unsigned);
    verifying_key
        .verify(&preimage, &signature)
        .map_err(|_| DeviceProfileError::InvalidSignature)?;
    Ok(VerifiedDeviceProfile {
        hardware_fingerprint,
        measurement_generation,
        logical_cores,
        available_memory_bytes,
        available_backends,
        bindings,
    })
}

async fn capture_identity(ffmpeg: &Path) -> Result<DeviceIdentity, DeviceProfileError> {
    let platform = platform_identity().await;
    let cpu = cpu_identity().await;
    let total_memory_bytes = measured_total_memory().await.max(1);
    let version = run_command(ffmpeg, &strings(["-version"]), Path::new("/"))
        .await
        .ok();
    let ffmpeg_available = version
        .as_ref()
        .is_some_and(|output| output.status.success());
    let ffmpeg_identity = version
        .as_ref()
        .map(|output| first_line(&output.stdout))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "ffmpeg unavailable".to_owned());
    let hwaccels = run_command(
        ffmpeg,
        &strings(["-hide_banner", "-hwaccels"]),
        Path::new("/"),
    )
    .await
    .ok()
    .filter(|output| output.status.success())
    .map_or_else(Vec::new, |output| parse_hwaccels(&output.stdout));
    let encoders = run_command(
        ffmpeg,
        &strings(["-hide_banner", "-encoders"]),
        Path::new("/"),
    )
    .await
    .ok()
    .filter(|output| output.status.success())
    .map_or_else(Vec::new, |output| parse_encoders(&output.stdout));
    let driver_identities = capture_driver_identities(&platform, &hwaccels);
    let mut accelerators = vec![AcceleratorIdentity {
        kind: "cpu".to_owned(),
        name: cpu.model.clone(),
    }];
    for driver in driver_identities.iter().filter(|driver| driver.available) {
        if let Some(name) = accelerator_name(&driver.kind) {
            accelerators.push(AcceleratorIdentity {
                kind: driver.kind.clone(),
                name: name.to_owned(),
            });
        }
    }
    let identity_value = json!({
        "accelerators": accelerators,
        "cpu": cpu,
        "ffmpeg": {
            "available": ffmpeg_available,
            "identity": ffmpeg_identity,
            "hwaccels": hwaccels,
        },
        "drivers": driver_identities,
        "memory": { "total_bytes": total_memory_bytes },
        "platform": platform,
    });
    let canonical = serde_json_canonicalizer::to_vec(&identity_value)
        .map_err(|error| DeviceProfileError::Json(error.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(FINGERPRINT_DOMAIN);
    hasher.update(canonical);
    let hardware_fingerprint = format!(
        "sha256:{}",
        Sha256Digest::from_bytes(hasher.finalize().into())
    );
    Ok(DeviceIdentity {
        platform,
        cpu,
        total_memory_bytes,
        accelerators,
        ffmpeg_identity,
        ffmpeg_available,
        ffmpeg_hwaccels: hwaccels,
        ffmpeg_encoders: encoders,
        driver_identities,
        hardware_fingerprint,
    })
}

async fn platform_identity() -> PlatformIdentity {
    let os = if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    };
    let arch = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x86_64"
    };
    let os_version = if cfg!(target_os = "macos") {
        run_command(
            Path::new("/usr/bin/sw_vers"),
            &strings(["-productVersion"]),
            Path::new("/"),
        )
        .await
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_owned())
    } else {
        linux_os_version()
    };
    PlatformIdentity {
        os: os.to_owned(),
        arch: arch.to_owned(),
        os_version,
    }
}

async fn cpu_identity() -> CpuIdentity {
    let logical = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
    let logical_cores = u32::try_from(logical).unwrap_or(u32::MAX).max(1);
    if cfg!(target_os = "macos") {
        let model = sysctl_text("machdep.cpu.brand_string")
            .await
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| std::env::consts::ARCH.to_owned());
        let physical_cores = sysctl_text("hw.physicalcpu")
            .await
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(logical_cores)
            .max(1);
        CpuIdentity {
            model,
            logical_cores,
            physical_cores,
        }
    } else {
        CpuIdentity {
            model: linux_cpu_model(),
            logical_cores,
            physical_cores: linux_physical_cores().unwrap_or(logical_cores).max(1),
        }
    }
}

async fn sysctl_text(key: &str) -> Option<String> {
    run_command(
        Path::new("/usr/sbin/sysctl"),
        &strings(["-n", key]),
        Path::new("/"),
    )
    .await
    .ok()
    .filter(|output| output.status.success())
    .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn canonical_unsigned_profile(profile: &Value) -> Result<Vec<u8>, DeviceProfileError> {
    serde_json_canonicalizer::to_vec(profile)
        .map_err(|error| DeviceProfileError::Json(error.to_string()))
}

fn hex_field(
    object: &Map<String, Value>,
    field: &'static str,
    bytes: usize,
) -> Result<Vec<u8>, DeviceProfileError> {
    let encoded = object
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| DeviceProfileError::Json(format!("attestation {field} is missing")))?;
    let decoded = hex::decode(encoded).map_err(|_| DeviceProfileError::InvalidSignature)?;
    if decoded.len() != bytes {
        return Err(DeviceProfileError::InvalidSignature);
    }
    Ok(decoded)
}

async fn run_command(
    executable: &Path,
    args: &[OsString],
    working_directory: &Path,
) -> Result<std::process::Output, DeviceProfileError> {
    let mut command = Command::new(executable);
    command
        .args(args)
        .current_dir(working_directory)
        .env_clear()
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .env(
            "PATH",
            "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin",
        )
        .kill_on_drop(true);
    let mut output = timeout(COMMAND_TIMEOUT, command.output())
        .await
        .map_err(|_| DeviceProfileError::CommandTimeout)?
        .map_err(|source| DeviceProfileError::Io {
            path: executable.to_path_buf(),
            source,
        })?;
    output.stdout.truncate(MAX_COMMAND_OUTPUT);
    output.stderr.truncate(MAX_COMMAND_OUTPUT);
    Ok(output)
}

fn strings<const N: usize>(values: [&str; N]) -> Vec<OsString> {
    values.into_iter().map(OsString::from).collect()
}

fn first_line(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .chars()
        .take(512)
        .collect()
}

fn parse_hwaccels(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.ends_with(':'))
        .filter(|line| {
            line.chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
        })
        .map(str::to_owned)
        .collect()
}

fn parse_encoders(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let flags = fields.next()?;
            let name = fields.next()?;
            (flags.len() >= 6 && flags.starts_with('V')).then(|| name.to_owned())
        })
        .collect()
}

fn preferred_hardware_backend(identity: &DeviceIdentity) -> String {
    ["videotoolbox", "vaapi", "cuda", "vulkan"]
        .into_iter()
        .find(|candidate| {
            identity
                .accelerators
                .iter()
                .any(|accelerator| accelerator.kind == *candidate)
        })
        .unwrap_or("none")
        .to_owned()
}

fn accelerator_name(kind: &str) -> Option<&'static str> {
    match kind {
        "videotoolbox" => Some("Apple VideoToolbox"),
        "vaapi" => Some("VAAPI"),
        "cuda" => Some("CUDA"),
        "vulkan" => Some("Vulkan"),
        _ => None,
    }
}

fn capture_driver_identities(
    platform: &PlatformIdentity,
    ffmpeg_hwaccels: &[String],
) -> Vec<RuntimeIdentity> {
    let mut identities = Vec::new();
    for kind in ["videotoolbox", "vaapi", "cuda", "vulkan"] {
        if !ffmpeg_hwaccels.iter().any(|value| value == kind) {
            continue;
        }
        let (available, identity) = match kind {
            "videotoolbox" => (
                platform.os == "macos",
                if platform.os == "macos" {
                    format!("macOS {} integrated media driver", platform.os_version)
                } else {
                    "VideoToolbox is unavailable on this platform".to_owned()
                },
            ),
            "vaapi" => linux_render_driver_identity("VAAPI"),
            "cuda" => linux_cuda_driver_identity(),
            "vulkan" => linux_render_driver_identity("Vulkan"),
            _ => unreachable!(),
        };
        identities.push(RuntimeIdentity {
            kind: kind.to_owned(),
            identity: identity.chars().take(512).collect(),
            available,
        });
    }
    identities
}

fn linux_render_driver_identity(label: &str) -> (bool, String) {
    let render = Path::new("/dev/dri/renderD128");
    if !cfg!(target_os = "linux") || !render.exists() {
        return (false, format!("{label} render device unavailable"));
    }
    let driver = fs::read_link("/sys/class/drm/renderD128/device/driver")
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "unknown".to_owned());
    let version = fs::read_to_string("/sys/class/drm/renderD128/device/driver/module/version")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "kernel-managed".to_owned());
    (true, format!("{label} driver {driver} {version}"))
}

fn linux_cuda_driver_identity() -> (bool, String) {
    if !cfg!(target_os = "linux") || !Path::new("/dev/nvidiactl").exists() {
        return (false, "CUDA driver device unavailable".to_owned());
    }
    let identity = fs::read_to_string("/proc/driver/nvidia/version")
        .ok()
        .and_then(|value| value.lines().next().map(str::trim).map(str::to_owned))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "NVIDIA driver version unavailable".to_owned());
    (true, identity)
}

fn frames_per_second(frames: u32, duration: Duration) -> f64 {
    let seconds = duration.as_secs_f64().max(0.000_001);
    (f64::from(frames) / seconds).max(0.000_001)
}

fn duration_millis_f64(duration: Duration) -> f64 {
    (duration.as_secs_f64() * 1_000.0).max(0.0)
}

fn ffmpeg_sibling(ffprobe: &Path) -> PathBuf {
    let mut ffmpeg = ffprobe.to_path_buf();
    let replacement = ffprobe
        .file_name()
        .and_then(|name| name.to_str())
        .map_or_else(
            || OsString::from("ffmpeg"),
            |name| OsString::from(name.replacen("ffprobe", "ffmpeg", 1)),
        );
    ffmpeg.set_file_name(replacement);
    ffmpeg
}

fn expected_ffmpeg_build() -> &'static str {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "1783011502_8.1.2"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "1783011670_8.1.2"
    } else {
        "unsupported-phase0-platform"
    }
}

fn linux_os_version() -> String {
    fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.strip_prefix("PRETTY_NAME=")
                    .map(|value| value.trim_matches('"').to_owned())
            })
        })
        .unwrap_or_else(|| "unknown".to_owned())
}

fn linux_cpu_model() -> String {
    fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                let (key, value) = line.split_once(':')?;
                matches!(key.trim(), "model name" | "Hardware").then(|| value.trim().to_owned())
            })
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| std::env::consts::ARCH.to_owned())
}

fn linux_physical_cores() -> Option<u32> {
    let text = fs::read_to_string("/proc/cpuinfo").ok()?;
    let mut pairs = BTreeSet::new();
    let mut physical = None;
    let mut core = None;
    for line in text.lines().chain([""]) {
        if line.trim().is_empty() {
            if let (Some(physical), Some(core)) = (physical.take(), core.take()) {
                pairs.insert((physical, core));
            }
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        match key.trim() {
            "physical id" => physical = value.trim().parse::<u32>().ok(),
            "core id" => core = value.trim().parse::<u32>().ok(),
            _ => {}
        }
    }
    u32::try_from(pairs.len()).ok().filter(|count| *count > 0)
}

async fn measured_total_memory() -> u64 {
    if cfg!(target_os = "macos") {
        sysctl_text("hw.memsize")
            .await
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(8 * 1024 * 1024 * 1024)
    } else {
        linux_memory_value("MemTotal").unwrap_or(8 * 1024 * 1024 * 1024)
    }
}

/// Free space on the volume a task's scratch actually lands on.
///
/// Measured rather than assumed, and measured *here* rather than read from the
/// device profile, because free space is the one figure in a capacity that
/// changes between two runs on the same machine. A profile that carried it
/// would be stale the moment anything was written.
///
/// The fallback is the old constant. A machine whose free space cannot be read
/// is one this should be conservative about, not optimistic — but a stage that
/// wants more than half a gigabyte of scratch will then wait, so the reason is
/// logged rather than left to be deduced from a task that never runs.
fn measured_available_disk(scratch: &std::path::Path) -> u64 {
    const FALLBACK: u64 = 512 * 1024 * 1024;
    match fs2::available_space(scratch) {
        Ok(bytes) => bytes,
        Err(error) => {
            tracing::warn!(
                path = %scratch.display(),
                %error,
                "cannot measure free space for task scratch; assuming half a gigabyte"
            );
            FALLBACK
        }
    }
}

async fn measured_available_memory(total: u64) -> u64 {
    if cfg!(target_os = "macos") {
        run_command(Path::new("/usr/bin/vm_stat"), &[], Path::new("/"))
            .await
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| parse_vm_stat(&output.stdout))
            .unwrap_or(total / 2)
    } else {
        linux_memory_value("MemAvailable").unwrap_or(total / 2)
    }
}

fn parse_vm_stat(bytes: &[u8]) -> Option<u64> {
    let text = String::from_utf8_lossy(bytes);
    let page_size = text
        .lines()
        .next()?
        .split_whitespace()
        .find_map(|field| field.parse::<u64>().ok())?;
    let available_pages = text.lines().skip(1).filter_map(|line| {
        let (label, value) = line.split_once(':')?;
        matches!(
            label,
            "Pages free" | "Pages inactive" | "Pages speculative" | "Pages purgeable"
        )
        .then(|| value.trim().trim_end_matches('.').parse::<u64>().ok())
        .flatten()
    });
    Some(
        available_pages
            .fold(0_u64, u64::saturating_add)
            .saturating_mul(page_size),
    )
}

fn linux_memory_value(key: &str) -> Option<u64> {
    let text = fs::read_to_string("/proc/meminfo").ok()?;
    let line = text.lines().find(|line| line.starts_with(key))?;
    let kibibytes = line.split_whitespace().nth(1)?.parse::<u64>().ok()?;
    kibibytes.checked_mul(1024)
}

fn load_or_create_signing_key(path: &Path) -> Result<SigningKey, DeviceProfileError> {
    if path.exists() {
        return load_signing_key(path);
    }
    let parent = path.parent().ok_or(DeviceProfileError::InvalidKey)?;
    create_private_directory(parent)?;
    let mut seed = [0_u8; 32];
    getrandom::fill(&mut seed).map_err(|_| DeviceProfileError::InvalidKey)?;
    write_new_private(path, &seed)?;
    sync_directory(parent)?;
    Ok(SigningKey::from_bytes(&seed))
}

fn load_signing_key(path: &Path) -> Result<SigningKey, DeviceProfileError> {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    let metadata = fs::symlink_metadata(path).map_err(|source| DeviceProfileError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    #[cfg(unix)]
    let private = metadata.permissions().mode().trailing_zeros() >= 6;
    #[cfg(not(unix))]
    let private = true;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() || !private {
        return Err(DeviceProfileError::InvalidKey);
    }
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|mut file| file.read_to_end(&mut bytes))
        .map_err(|source| DeviceProfileError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    let seed: [u8; 32] = bytes
        .try_into()
        .map_err(|_| DeviceProfileError::InvalidKey)?;
    Ok(SigningKey::from_bytes(&seed))
}

fn create_private_directory(path: &Path) -> Result<(), DeviceProfileError> {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    fs::create_dir_all(path).map_err(|source| DeviceProfileError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|source| {
        DeviceProfileError::Io {
            path: path.to_path_buf(),
            source,
        }
    })?;
    Ok(())
}

fn write_new_private(path: &Path, bytes: &[u8]) -> Result<(), DeviceProfileError> {
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;

    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
        .open(path)
        .map_err(|source| DeviceProfileError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|source| DeviceProfileError::Io {
            path: path.to_path_buf(),
            source,
        })
}

fn sync_directory(path: &Path) -> Result<(), DeviceProfileError> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|source| DeviceProfileError::Io {
            path: path.to_path_buf(),
            source,
        })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use std::{fs, path::Path, sync::Arc};

    use serde_json::{Value, json};
    use tempfile::TempDir;

    use super::{DeviceProfiler, verify_profile};
    use crate::{models::ModelRegistry, selection::Bindings};

    fn registry() -> Arc<ModelRegistry> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../models/registry");
        Arc::new(ModelRegistry::load(&path).expect("the published registry loads"))
    }

    fn profiler(temp: &TempDir) -> DeviceProfiler {
        DeviceProfiler::new(
            temp.path().join("missing-ffprobe").as_path(),
            temp.path().join("device.key").as_path(),
            temp.path().join("scratch").as_path(),
            temp.path().join("speech-benchmark.json").as_path(),
            registry(),
        )
        .expect("profiler")
    }

    #[tokio::test]
    async fn measured_profile_is_canonical_signed_and_generation_scoped() {
        let temp = TempDir::new().expect("temp");
        let profiler = profiler(&temp);
        let fingerprint = profiler.hardware_fingerprint().await.expect("fingerprint");
        let profile = profiler.measure(&fingerprint, 7).await.expect("profile");
        let verified = verify_profile(&profile, Some(&fingerprint)).expect("verify");
        assert_eq!(verified.measurement_generation, 7);
        assert_eq!(verified.hardware_fingerprint, fingerprint);
        assert!(verified.available_memory_bytes > 0);
        serde_json::from_str::<Value>(&profile).expect("JSON");
        assert_eq!(
            fs::metadata(temp.path().join("device.key"))
                .expect("key")
                .len(),
            32
        );
    }

    /// A machine nobody has benchmarked still publishes a binding for every
    /// capability, and every one of them says it was not measured. Selection
    /// that silently produced nothing would leave the plan factory guessing.
    #[tokio::test]
    async fn an_unbenchmarked_device_still_attests_a_complete_binding() {
        let temp = TempDir::new().expect("temp");
        let profiler = profiler(&temp);
        let fingerprint = profiler.hardware_fingerprint().await.expect("fingerprint");
        let profile = profiler.measure(&fingerprint, 1).await.expect("profile");
        let verified = verify_profile(&profile, Some(&fingerprint)).expect("verify");

        assert_eq!(verified.bindings, Bindings::portable());
        assert!(
            !verified
                .bindings
                .iter()
                .any(crate::selection::Binding::was_measured)
        );
        // And no accelerator is admissible, because nothing has been measured
        // running on one. A machine that claimed Metal here on the strength of
        // being a Mac would be asserting a static platform default.
        assert!(
            !verified.available_backends.contains("metal"),
            "an unbenchmarked device must not advertise an accelerator"
        );
    }

    /// The benchmark decides the binding, and the signature covers it. An
    /// attacker who could edit the choice after the fact would be choosing
    /// which model produced somebody's transcript.
    #[tokio::test]
    async fn a_benchmark_moves_the_binding_and_the_signature_covers_it() {
        let temp = TempDir::new().expect("temp");
        let models = registry();
        let profiler = profiler(&temp);
        let fingerprint = profiler.hardware_fingerprint().await.expect("fingerprint");
        let digest = format!(
            "sha256:{}",
            models.get("qwen3-asr-mlx").expect("pinned").digest()
        );
        fs::write(
            temp.path().join("speech-benchmark.json"),
            serde_json::to_vec(&json!({
                "schema_version": "clipmill.speech_benchmark.v1",
                "hardware_fingerprint": fingerprint,
                "measurements": [{
                    "implementation": "clipmill-worker-speech-mlx@0.1.0/asr",
                    "model_digest": digest,
                    "runnable": true,
                    "real_time_factor": 22.5,
                    "peak_resident_bytes": 3_400_000_000_u64,
                }],
            }))
            .expect("json"),
        )
        .expect("write");

        let profile = profiler.measure(&fingerprint, 2).await.expect("profile");
        let verified = verify_profile(&profile, Some(&fingerprint)).expect("verify");
        let asr = verified
            .bindings
            .for_stage("speech-asr")
            .expect("a binding for recognition");
        assert_eq!(asr.model, "qwen3-asr-mlx");
        assert!(asr.was_measured());
        // Having run a model on it is the only evidence this daemon accepts
        // that an accelerator works. Without it the scheduler would decline
        // the very worker the binding just chose, and the task would starve.
        assert!(
            verified.available_backends.contains("metal"),
            "a measured MLX run must make Metal admissible: {:?}",
            verified.available_backends
        );

        let rebound = profile.replacen("qwen3-asr-mlx", "whisper-base00", 1);
        assert_ne!(rebound, profile, "the substitution has to have happened");
        assert!(
            verify_profile(&rebound, Some(&fingerprint)).is_err(),
            "a binding edited after signing is not one this device attested"
        );
    }

    #[tokio::test]
    async fn signature_rejects_tampering() {
        let temp = TempDir::new().expect("temp");
        let profiler = profiler(&temp);
        let fingerprint = profiler.hardware_fingerprint().await.expect("fingerprint");
        let profile = profiler.measure(&fingerprint, 1).await.expect("profile");
        let tampered = profile.replacen(
            "\"measurement_generation\":1",
            "\"measurement_generation\":2",
            1,
        );
        assert!(verify_profile(&tampered, Some(&fingerprint)).is_err());
    }
}
