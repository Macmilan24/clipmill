//! The model library against a temporary weights directory and, where a
//! download is exercised, a loopback server. Nothing leaves the machine.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

use clipmill_hub::Hub;
use tempfile::TempDir;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

use super::{LibraryPaths, ModelLibrary, Refusal, memory_fit};
use crate::{
    implementations,
    models::{
        HUGGING_FACE, ModelCatalog, ModelFile, ModelLicense, ModelManifest, ModelMemory,
        ModelOrigin, ModelRegistry, ModelSource,
    },
    policy::LocalLockPolicy,
    selection::Bindings,
};

const COMMIT: &str = "5359861c739e955e79d9a303bcbc70fb988958b1";
const GIB: u64 = 1024 * 1024 * 1024;

struct Fixture {
    _root: TempDir,
    paths: LibraryPaths,
    registry: Arc<ModelRegistry>,
    policy: Arc<LocalLockPolicy>,
}

impl Fixture {
    fn new() -> Self {
        let root = TempDir::new().expect("temp");
        let paths = LibraryPaths {
            weights: root.path().join("weights"),
            custom: root.path().join("state/models"),
            choices: root.path().join("state/model-choices.json"),
        };
        fs::create_dir_all(root.path().join("state/models")).unwrap();
        let bundled = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../models/registry");
        let registry =
            Arc::new(ModelRegistry::load_with_custom(&bundled, &paths.custom).expect("registry"));
        Self {
            _root: root,
            paths,
            registry,
            policy: Arc::default(),
        }
    }

    fn library(&self, hub: Option<Hub>) -> Arc<ModelLibrary> {
        ModelLibrary::start_with_hub(
            Arc::clone(&self.registry),
            self.paths.clone(),
            None,
            Arc::clone(&self.policy),
            24 * GIB,
            hub,
        )
    }

    /// Put a model's pinned files on disk at their pinned sizes. Sparse, so a
    /// gigabyte of weights costs nothing; the size check is what install
    /// state reads, and nothing here hashes them.
    fn place(&self, name: &str) {
        let manifest = self.registry.get(name).expect("pinned");
        for file in &manifest.files {
            let path = self.paths.weights.join(name).join(&file.path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::File::create(&path)
                .unwrap()
                .set_len(file.bytes)
                .unwrap();
        }
    }
}

/// A person's model pinned to bytes this test serves.
fn tiny_manifest(name: &str, body: &[u8]) -> ModelManifest {
    ModelManifest {
        name: name.to_owned(),
        capability: "asr".to_owned(),
        family: "whisper.cpp".to_owned(),
        runtime: "whisper.cpp".to_owned(),
        backend: "cpu".to_owned(),
        quantization: "ggml-f16".to_owned(),
        source: ModelSource {
            provider: HUGGING_FACE.to_owned(),
            repo: "ggerganov/whisper.cpp".to_owned(),
            revision: COMMIT.to_owned(),
        },
        license: ModelLicense {
            source: "repo-metadata".to_owned(),
            spdx: "MIT".to_owned(),
            class: "permissive".to_owned(),
        },
        memory: ModelMemory {
            weights_bytes: body.len() as u64,
            runtime_overhead_bytes: 256 * 1024 * 1024,
        },
        files: vec![ModelFile {
            path: "ggml-tiny.bin".to_owned(),
            sha256: clipmill_hub::sha256_hex(body),
            bytes: body.len() as u64,
        }],
        catalog: ModelCatalog {
            title: "Whisper tiny".to_owned(),
            summary: "Added in a test.".to_owned(),
            recommended: false,
        },
        origin: ModelOrigin::Custom,
    }
}

fn register(fixture: &Fixture, manifest: ModelManifest) {
    implementations::register_custom(&manifest.name, &manifest.capability).expect("plannable");
    fixture.registry.register(manifest).expect("registered");
}

/// Serves fixed bodies by path; anything else is a 404.
async fn serve(routes: HashMap<String, Vec<u8>>) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let base = format!("http://{}/", listener.local_addr().unwrap());
    let seen = Arc::new(Mutex::new(Vec::new()));
    let recorded = Arc::clone(&seen);
    let routes = Arc::new(routes);
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let routes = Arc::clone(&routes);
            let recorded = Arc::clone(&recorded);
            tokio::spawn(async move {
                let mut head = Vec::new();
                let mut buffer = [0_u8; 1024];
                while !head.windows(4).any(|window| window == b"\r\n\r\n") {
                    match stream.read(&mut buffer).await {
                        Ok(0) | Err(_) => return,
                        Ok(count) => head.extend_from_slice(&buffer[..count]),
                    }
                }
                let text = String::from_utf8_lossy(&head).into_owned();
                let target = text
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or_default()
                    .to_owned();
                recorded.lock().unwrap().push(target.clone());
                let reply = match routes.get(&target) {
                    Some(body) => {
                        let mut reply = format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        )
                        .into_bytes();
                        reply.extend_from_slice(body);
                        reply
                    }
                    None => {
                        b"HTTP/1.1 404 Missing\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                            .to_vec()
                    }
                };
                let _ = stream.write_all(&reply).await;
                let _ = stream.shutdown().await;
            });
        }
    });
    (base, seen)
}

async fn settle(library: &ModelLibrary, name: &str) -> clipmill_contracts::proto::ipc::v1::ModelV1 {
    for _ in 0..200 {
        let listed = library.list(&Bindings::portable(), 16 * GIB);
        let model = listed
            .models
            .into_iter()
            .find(|model| model.name == name)
            .expect("listed");
        let busy = model.download.as_ref().is_some_and(|download| {
            matches!(
                download.state.as_str(),
                "queued" | "downloading" | "verifying"
            )
        });
        if !busy {
            return model;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("{name} never settled");
}

#[tokio::test]
async fn a_fresh_install_lists_everything_and_says_what_to_download() {
    let fixture = Fixture::new();
    let library = fixture.library(None);

    let listed = library.list(&Bindings::portable(), 18 * GIB);

    assert_eq!(listed.models.len(), fixture.registry.len());
    assert!(
        listed
            .models
            .iter()
            .all(|model| model.install_state == "missing")
    );
    for portable in [
        "silero-vad",
        "whisper-base",
        "wav2vec2-ctc-en",
        "yunet-face",
    ] {
        assert!(
            listed
                .recommended_missing
                .iter()
                .any(|name| name == portable),
            "{portable} is not offered to a fresh install"
        );
    }
    assert!(
        !listed
            .recommended_missing
            .iter()
            .any(|name| name == "whisper-large-v3-turbo"),
        "an optional model is not part of the first download"
    );
    let transcription = listed
        .jobs
        .iter()
        .find(|job| job.capability == "asr")
        .expect("transcription");
    assert_eq!(transcription.models[0], "whisper-base", "recommended first");
    assert!(
        transcription
            .models
            .iter()
            .any(|name| name == "whisper-large-v3-turbo")
    );
    assert_eq!(transcription.model, "whisper-base");
    assert!(listed.recommended_missing_bytes > 0);
    assert_eq!(listed.memory_total_bytes, 24 * GIB);
    // Every model is listed on every device; fit is a warning, never a filter.
    let editorial = listed
        .models
        .iter()
        .find(|model| model.name == "qwen3-5-editorial-mlx")
        .expect("listed everywhere");
    assert_eq!(editorial.memory_fit, "fits");
    assert_eq!(
        editorial.supported,
        cfg!(all(target_os = "macos", target_arch = "aarch64"))
    );
}

/// A computer that can run a more accurate recognizer than the one planned
/// is told so, and offered it, but nothing is switched or downloaded for it:
/// the floor stays until the person chooses.
#[tokio::test]
async fn a_more_accurate_recognizer_that_fits_is_offered_never_imposed() {
    let fixture = Fixture::new();
    let library = fixture.library(None);
    fixture.place("whisper-base");
    let asr = |listed: &clipmill_contracts::proto::ipc::v1::ListModelsResponse| {
        listed
            .jobs
            .iter()
            .find(|job| job.capability == "asr")
            .expect("transcription")
            .clone()
    };

    let listed = library.list(&Bindings::portable(), 18 * GIB);

    // Apple silicon runs the MLX recognizer, which measurement may choose;
    // anywhere else the larger Whisper is the more accurate one it can run.
    let better = if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "qwen3-asr-mlx"
    } else {
        "whisper-large-v3-turbo"
    };
    let transcription = asr(&listed);
    assert_eq!(
        transcription.model, "whisper-base",
        "the floor is still planned"
    );
    assert_eq!(transcription.more_accurate, better);
    assert!(
        listed
            .models
            .iter()
            .any(|model| model.name == better && model.recommended),
        "its row says it is recommended here"
    );
    assert!(
        !listed.recommended_missing.iter().any(|name| name == better),
        "an upgrade is not part of what analysis needs"
    );
    // The other jobs' candidates are equals, so none is offered.
    assert!(
        listed
            .jobs
            .iter()
            .filter(|job| job.capability != "asr")
            .all(|job| job.more_accurate.is_empty())
    );

    // A model the analysis budget cannot hold is not offered, only listed.
    assert!(
        asr(&library.list(&Bindings::portable(), GIB / 2))
            .more_accurate
            .is_empty()
    );

    // Once a recognizer as accurate is chosen, nothing more is offered.
    fixture.place("whisper-large-v3-turbo");
    library
        .set_choice("asr", "whisper-large-v3-turbo")
        .expect("chosen");
    assert!(
        asr(&library.list(&Bindings::portable(), 18 * GIB))
            .more_accurate
            .is_empty()
    );
}

#[test]
fn memory_fit_warns_on_every_scale_and_never_hides() {
    assert_eq!(memory_fit(4 * GIB, 12 * GIB, 16 * GIB), "fits");
    assert_eq!(memory_fit(14 * GIB, 12 * GIB, 16 * GIB), "tight");
    assert_eq!(memory_fit(26 * GIB, 12 * GIB, 16 * GIB), "too_large");
    assert_eq!(memory_fit(1, 0, 0), "unknown");
}

/// Removing the default must not strand the next analysis: a candidate whose
/// weights are here is planned instead, and says it is a fallback.
#[tokio::test]
async fn a_missing_default_yields_to_an_installed_candidate() {
    let fixture = Fixture::new();
    let library = fixture.library(None);
    fixture.place("whisper-large-v3-turbo");

    let effective = library.effective_bindings(&Bindings::portable());

    let transcription = effective.for_stage("speech-asr").expect("bound");
    assert_eq!(transcription.model, "whisper-large-v3-turbo");
    assert_eq!(transcription.selected_by, "installed_fallback");
    assert_eq!(
        transcription.implementation,
        "clipmill-worker-asr@0.1.0/whisper-large-v3-turbo"
    );
    // With nothing installed at all the default stays, for readiness to name.
    let alignment = effective.for_stage("speech-align").expect("bound");
    assert_eq!(alignment.model, "wav2vec2-ctc-en");

    fixture.place("whisper-base");
    let restored = library.effective_bindings(&Bindings::portable());
    assert_eq!(
        restored.for_stage("speech-asr").expect("bound").model,
        "whisper-base",
        "the installed default wins again"
    );
}

/// A choice needs the weights first, outlives a restart, and drops back to
/// automatic when its model is removed.
#[tokio::test]
async fn a_choice_is_kept_until_its_model_is_removed() {
    let fixture = Fixture::new();
    let library = fixture.library(None);
    fixture.place("whisper-base");

    assert!(matches!(
        library.set_choice("asr", "whisper-large-v3-turbo"),
        Err(Refusal::Conflict(_))
    ));
    assert!(matches!(
        library.set_choice("asr", "wav2vec2-ctc-en"),
        Err(Refusal::Invalid(_))
    ));
    assert!(matches!(
        library.set_choice("not-a-job", ""),
        Err(Refusal::Invalid(_))
    ));

    fixture.place("whisper-large-v3-turbo");
    library
        .set_choice("asr", "whisper-large-v3-turbo")
        .expect("installed, so choosable");
    let chosen = library.effective_bindings(&Bindings::portable());
    assert_eq!(
        chosen.for_stage("speech-asr").expect("bound").selected_by,
        "chosen"
    );
    let job = library
        .list(&Bindings::portable(), 16 * GIB)
        .jobs
        .into_iter()
        .find(|job| job.capability == "asr")
        .expect("listed");
    assert_eq!(job.choice, "whisper-large-v3-turbo");
    assert_eq!(job.selected_by, "chosen");

    // A second library over the same state reads the choice back.
    library.stop().await;
    let restarted = fixture.library(None);
    assert_eq!(
        restarted
            .effective_bindings(&Bindings::portable())
            .for_stage("speech-asr")
            .expect("bound")
            .model,
        "whisper-large-v3-turbo"
    );

    let freed = restarted
        .remove("whisper-large-v3-turbo")
        .await
        .expect("removable");
    assert!(freed > 0);
    assert!(
        !fixture
            .paths
            .weights
            .join("whisper-large-v3-turbo")
            .exists()
    );
    let after = restarted.effective_bindings(&Bindings::portable());
    let transcription = after.for_stage("speech-asr").expect("bound");
    assert_eq!(transcription.model, "whisper-base");
    assert_ne!(transcription.selected_by, "chosen");
    assert!(matches!(
        restarted.remove("not-a-model").await,
        Err(Refusal::NotFound(_))
    ));
}

/// Editorial has no benchmark: the bundled model is the default, a person's
/// installed choice replaces it for every editorial stage and for titles.
#[tokio::test]
async fn an_editorial_choice_covers_every_stage_the_model_serves() {
    let fixture = Fixture::new();
    let library = fixture.library(None);
    let body = b"not really mlx weights".to_vec();
    let mut manifest = tiny_manifest("test-editorial-choice", &body);
    manifest.capability = "editorial".to_owned();
    manifest.runtime = "mlx".to_owned();
    manifest.backend = "mlx".to_owned();
    manifest.files[0].path = "model.safetensors".to_owned();
    register(&fixture, manifest);
    fs::create_dir_all(fixture.paths.weights.join("test-editorial-choice")).unwrap();
    fs::write(
        fixture
            .paths
            .weights
            .join("test-editorial-choice/model.safetensors"),
        &body,
    )
    .unwrap();

    let supported = cfg!(all(target_os = "macos", target_arch = "aarch64"));
    let fallback = library.effective_bindings(&Bindings::portable());
    let proposing = fallback.for_stage("editorial-propose").expect("bound");
    if supported {
        // The bundled model is not installed and this one is: it stands in.
        assert_eq!(proposing.model, "test-editorial-choice");
        assert_eq!(proposing.selected_by, "installed_fallback");
    } else {
        assert_eq!(proposing.model, "qwen3-5-editorial-mlx");
    }

    fixture.place("qwen3-5-editorial-mlx");
    let before = library.effective_bindings(&Bindings::portable());
    assert_eq!(
        before.for_stage("editorial-propose").expect("bound").model,
        "qwen3-5-editorial-mlx",
        "the installed default comes back"
    );

    let choice = library.set_choice("editorial", "test-editorial-choice");
    if !supported {
        assert!(
            matches!(choice, Err(Refusal::Invalid(_))),
            "MLX needs Apple silicon"
        );
        return;
    }
    choice.expect("an installed MLX model is choosable");
    let after = library.effective_bindings(&Bindings::portable());
    for stage in [
        "editorial-propose",
        "editorial-review",
        "editorial-look",
        "youtube-metadata",
    ] {
        let binding = after.for_stage(stage).expect("bound");
        assert_eq!(binding.model, "test-editorial-choice", "{stage}");
        assert_eq!(
            binding.implementation,
            format!("{}/custom/test-editorial-choice", {
                implementations::for_stage_and_model(stage, "qwen3-5-editorial-mlx")
                    .expect("bundled")
                    .name
            })
        );
    }
}

/// The whole path: queued, fetched over the transport, verified, installed
/// under its final name, staging cleared, and the Lock told.
#[tokio::test]
async fn a_download_installs_verified_weights_and_is_counted_by_the_lock() {
    let fixture = Fixture::new();
    let body = (0..40_000_u32)
        .flat_map(u32::to_le_bytes)
        .collect::<Vec<_>>();
    register(&fixture, tiny_manifest("test-download-tiny", &body));
    let (base, seen) = serve(HashMap::from([(
        format!("/ggerganov/whisper.cpp/resolve/{COMMIT}/ggml-tiny.bin"),
        body.clone(),
    )]))
    .await;
    let library = fixture.library(Some(Hub::with_test_endpoint(&base).unwrap()));
    let egress_before = fixture.policy.status().egress_attempts;

    library
        .download(&["test-download-tiny".to_owned()])
        .expect("queued");
    let model = settle(&library, "test-download-tiny").await;

    assert_eq!(model.install_state, "installed", "{:?}", model.download);
    assert!(
        model.download.is_none(),
        "a finished download leaves no status"
    );
    assert_eq!(
        fs::read(
            fixture
                .paths
                .weights
                .join("test-download-tiny/ggml-tiny.bin")
        )
        .unwrap(),
        body
    );
    assert!(
        !fixture
            .paths
            .weights
            .join(".partial/test-download-tiny")
            .exists()
    );
    assert_eq!(seen.lock().unwrap().len(), 1);
    assert!(
        fixture.policy.status().egress_attempts > egress_before,
        "a download is a network operation"
    );
    assert!(!fixture.policy.status().engaged);

    // Downloading an installed model re-checks it and fetches nothing.
    library
        .download(&["test-download-tiny".to_owned()])
        .expect("queued again");
    let again = settle(&library, "test-download-tiny").await;
    assert_eq!(again.install_state, "installed");
    assert_eq!(
        seen.lock().unwrap().len(),
        1,
        "verified in place, not refetched"
    );
}

#[tokio::test]
async fn bytes_that_miss_their_pin_are_never_installed() {
    let fixture = Fixture::new();
    let pinned = b"the bytes that were pinned".to_vec();
    register(&fixture, tiny_manifest("test-download-wrong", &pinned));
    let (base, _) = serve(HashMap::from([(
        format!("/ggerganov/whisper.cpp/resolve/{COMMIT}/ggml-tiny.bin"),
        b"other bytes, same length!!".to_vec(),
    )]))
    .await;
    let library = fixture.library(Some(Hub::with_test_endpoint(&base).unwrap()));

    library
        .download(&["test-download-wrong".to_owned()])
        .expect("queued");
    let model = settle(&library, "test-download-wrong").await;

    let download = model.download.expect("a failure is reported");
    assert_eq!(download.state, "failed");
    assert!(download.error.contains("SHA-256"), "{}", download.error);
    assert_ne!(model.install_state, "installed");
    assert!(
        !fixture
            .paths
            .weights
            .join("test-download-wrong/ggml-tiny.bin")
            .exists()
    );
}

#[tokio::test]
async fn verify_removes_a_file_that_no_longer_matches() {
    let fixture = Fixture::new();
    let body = b"weights as pinned".to_vec();
    register(&fixture, tiny_manifest("test-verify", &body));
    let library = fixture.library(None);
    let installed = fixture.paths.weights.join("test-verify/ggml-tiny.bin");
    fs::create_dir_all(installed.parent().unwrap()).unwrap();
    fs::write(&installed, &body).unwrap();

    library.verify("test-verify").expect("queued");
    let good = settle(&library, "test-verify").await;
    assert_eq!(good.install_state, "installed");
    assert!(good.download.is_none());

    fs::write(&installed, b"weights as swapped").unwrap();
    library.verify("test-verify").expect("queued");
    let bad = settle(&library, "test-verify").await;
    let download = bad.download.expect("the damage is reported");
    assert_eq!(download.state, "failed");
    assert!(
        download.error.contains("ggml-tiny.bin"),
        "{}",
        download.error
    );
    assert!(
        !installed.exists(),
        "the swapped file is removed for repair"
    );
}

#[tokio::test]
async fn downloads_are_refused_without_a_transport_and_bundled_models_are_not_forgotten() {
    let fixture = Fixture::new();
    let library = fixture.library(None);

    assert!(matches!(
        library.download(&["whisper-base".to_owned()]),
        Err(Refusal::Unavailable(_))
    ));
    assert!(matches!(
        library.forget("whisper-base").await,
        Err(Refusal::Invalid(_))
    ));

    let body = b"x".to_vec();
    let manifest = tiny_manifest("test-forget", &body);
    fs::write(
        fixture.paths.custom.join("test-forget.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    register(&fixture, manifest);
    library.forget("test-forget").await.expect("forgotten");
    assert!(fixture.registry.get("test-forget").is_none());
    assert!(!fixture.paths.custom.join("test-forget.json").exists());
}

#[tokio::test]
async fn a_queued_download_can_be_cancelled_before_it_starts() {
    let fixture = Fixture::new();
    let body = b"never fetched".to_vec();
    register(&fixture, tiny_manifest("test-cancel-a", &body));
    register(&fixture, tiny_manifest("test-cancel-b", &body));
    // A server that never answers keeps the first download busy.
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let base = format!("http://{}/", listener.local_addr().unwrap());
    let _held = tokio::spawn(async move {
        let mut open = Vec::new();
        while let Ok((stream, _)) = listener.accept().await {
            open.push(stream);
        }
    });
    let library = fixture.library(Some(Hub::with_test_endpoint(&base).unwrap()));

    library
        .download(&["test-cancel-a".to_owned(), "test-cancel-b".to_owned()])
        .expect("queued");
    library.cancel("test-cancel-b");
    let listed = library.list(&Bindings::portable(), 16 * GIB);
    let queued = listed
        .models
        .iter()
        .find(|model| model.name == "test-cancel-b")
        .and_then(|model| model.download.clone())
        .expect("status kept");
    assert_eq!(queued.state, "cancelled");
    assert!(matches!(
        library.remove("test-cancel-a").await,
        Err(Refusal::Conflict(_))
    ));

    library.cancel("test-cancel-a");
    let running = settle(&library, "test-cancel-a").await;
    assert_eq!(running.download.expect("status kept").state, "cancelled");
    library.stop().await;
}
