//! Pinning a person's own model from a repository.
//!
//! The bundled registry's rules, applied at the moment of pinning: one commit,
//! every file's SHA-256, a licence on the allowlist. Weights kept in the
//! repository's large-file storage carry a SHA-256 of their own. Small files
//! kept in git carry only a git object id, so they are fetched whole here — at
//! the pinned commit, over HTTPS — and the digest of what arrived becomes the
//! pin. From then on the model is handled exactly like a bundled one.

use clipmill_contracts::proto::ipc::v1::{HubModelFileV1, InspectHubModelResponse};
use clipmill_hub::{Hub, Repository, RepositoryFile, SMALL_FILE_LIMIT};

use crate::{
    implementations::{self, CustomRuntime},
    models::{
        HUGGING_FACE, ModelCatalog, ModelFile, ModelLicense, ModelManifest, ModelMemory,
        ModelOrigin, ModelRegistry, ModelSource, is_commit, valid_name, valid_relative_path,
    },
};

/// Licence identifiers as the hub spells them, and their SPDX form: the same
/// allowlist `check-models.py` holds the bundled registry to. Terms that let a
/// creator sell what the model helped make; anything else is refused.
const PERMISSIVE: &[(&str, &str)] = &[
    ("apache-2.0", "Apache-2.0"),
    ("mit", "MIT"),
    ("bsd-3-clause", "BSD-3-Clause"),
    ("bsd-2-clause", "BSD-2-Clause"),
    ("cc0-1.0", "CC0-1.0"),
    ("isc", "ISC"),
];

/// A model directory is bounded: a repository holding more than this is not
/// one model.
const MAX_FILES: usize = 256;

/// What adding a model asks for, as the request carried it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct AddRequest<'a> {
    pub repo: &'a str,
    pub commit: &'a str,
    pub capability: &'a str,
    pub weights_file: &'a str,
    pub name: &'a str,
    pub title: &'a str,
}

/// A pinned manifest, and the small files that were fetched to pin it.
#[derive(Debug)]
pub(crate) struct Pinned {
    pub manifest: ModelManifest,
    pub fetched: Vec<(String, Vec<u8>)>,
}

pub(crate) fn refused(repo: &str, capability: &str, problem: &str) -> InspectHubModelResponse {
    InspectHubModelResponse {
        repo: repo.to_owned(),
        capability: capability.to_owned(),
        problem: problem.to_owned(),
        ..InspectHubModelResponse::default()
    }
}

/// Read a repository and say what pinning it would pin, or why it cannot.
pub(crate) async fn inspect(
    hub: &Hub,
    registry: &ModelRegistry,
    repo: &str,
    revision: &str,
    capability: &str,
) -> InspectHubModelResponse {
    let repo = repo.trim();
    let revision = match revision.trim() {
        "" => "main",
        revision => revision,
    };
    let Some(runtime) = implementations::custom_runtime(capability) else {
        return refused(
            repo,
            capability,
            "Only transcription and editorial models can be added from Hugging Face.",
        );
    };
    let repository = match hub.repository(repo, revision).await {
        Ok(repository) => repository,
        Err(error) => return refused(repo, capability, &error.to_string()),
    };
    let license = repository.license.clone().unwrap_or_default();
    let spdx = permissive_spdx(repository.license.as_deref());
    let mut answer = InspectHubModelResponse {
        repo: repo.to_owned(),
        commit: repository.commit.clone(),
        capability: capability.to_owned(),
        license,
        license_spdx: spdx.unwrap_or_default().to_owned(),
        license_allowed: spdx.is_some(),
        ..InspectHubModelResponse::default()
    };
    let selection = select(&repository, runtime);
    match &selection {
        Ok(Selection::Weights {
            choices,
            companions,
        }) => {
            answer.weight_choices = choices
                .iter()
                .map(|file| {
                    let (name, title) = if runtime.runtime == "llama.cpp" {
                        gguf_names(registry, &file.path)
                    } else {
                        whisper_names(registry, &file.path)
                    };
                    HubModelFileV1 {
                        path: file.path.clone(),
                        bytes: file.bytes,
                        suggested_name: name,
                        suggested_title: title,
                    }
                })
                .collect();
            // What is pinned beside whichever file is chosen: a GGUF
            // model's vision projector.
            answer.files = companions
                .iter()
                .map(|file| HubModelFileV1 {
                    path: file.path.clone(),
                    bytes: file.bytes,
                    ..HubModelFileV1::default()
                })
                .collect();
            if let Some(first) = answer.weight_choices.first() {
                answer.suggested_name.clone_from(&first.suggested_name);
                answer.suggested_title.clone_from(&first.suggested_title);
            }
        }
        Ok(Selection::Directory(files)) => {
            answer.files = files
                .iter()
                .map(|file| HubModelFileV1 {
                    path: file.path.clone(),
                    bytes: file.bytes,
                    ..HubModelFileV1::default()
                })
                .collect();
            let (name, title) = directory_names(registry, repo);
            answer.suggested_name = name;
            answer.suggested_title = title;
        }
        Err(_) => {}
    }
    answer.problem = if repository.gated || repository.private {
        "This repository requires signing in to Hugging Face, which ClipMill does not do."
            .to_owned()
    } else if spdx.is_none() {
        format!(
            "Its licence ({}) is not one ClipMill permits. Models must allow using their output in work you publish and sell: {}.",
            if answer.license.is_empty() {
                "none stated"
            } else {
                &answer.license
            },
            PERMISSIVE
                .iter()
                .map(|(_, spdx)| *spdx)
                .collect::<Vec<_>>()
                .join(", ")
        )
    } else {
        selection.err().unwrap_or_default()
    };
    answer
}

/// Pin the model an inspection described, fetching and hashing its small
/// files. Nothing is registered or written here.
pub(crate) async fn pin(hub: &Hub, request: &AddRequest<'_>) -> Result<Pinned, String> {
    let runtime = implementations::custom_runtime(request.capability).ok_or_else(|| {
        "Only transcription and editorial models can be added from Hugging Face.".to_owned()
    })?;
    if !valid_name(request.name) {
        return Err(
            "Use lowercase letters, digits, hyphens and dots for the model's name.".to_owned(),
        );
    }
    if !is_commit(request.commit) {
        return Err("Inspect the repository again before adding it.".to_owned());
    }
    let repository = hub
        .repository(request.repo, request.commit)
        .await
        .map_err(|error| error.to_string())?;
    if repository.gated || repository.private {
        return Err(
            "This repository requires signing in to Hugging Face, which ClipMill does not do."
                .to_owned(),
        );
    }
    let spdx = permissive_spdx(repository.license.as_deref())
        .ok_or_else(|| "Its licence is not one ClipMill permits.".to_owned())?;
    let chosen: Vec<RepositoryFile> = match select(&repository, runtime)? {
        Selection::Weights {
            choices,
            companions,
        } => {
            let weights = choices
                .into_iter()
                .find(|file| file.path == request.weights_file)
                .ok_or_else(|| {
                    format!(
                        "{} is not a weight file in this repository.",
                        request.weights_file
                    )
                })?;
            std::iter::once(weights).chain(companions).collect()
        }
        Selection::Directory(files) => files,
    };
    let (files, fetched) = pin_files(hub, request, chosen).await?;
    let weights_bytes = files
        .iter()
        .fold(0_u64, |total, file| total.saturating_add(file.bytes));
    let quantization = match runtime.runtime {
        "whisper.cpp" => whisper_quantization(request.weights_file),
        "llama.cpp" => gguf_quantization(request.weights_file),
        _ => fetched
            .iter()
            .find(|(path, _)| path == "config.json")
            .map_or_else(|| "none".to_owned(), |(_, bytes)| mlx_quantization(bytes)),
    };
    let title = match request.title.trim() {
        "" => request.name.to_owned(),
        title => title.chars().take(64).collect(),
    };
    let manifest = ModelManifest {
        name: request.name.to_owned(),
        capability: request.capability.to_owned(),
        family: runtime.family.to_owned(),
        runtime: runtime.runtime.to_owned(),
        backend: runtime.backend.to_owned(),
        quantization,
        source: ModelSource {
            provider: HUGGING_FACE.to_owned(),
            repo: request.repo.to_owned(),
            revision: repository.commit.clone(),
        },
        license: ModelLicense {
            source: "repo-metadata".to_owned(),
            spdx: spdx.to_owned(),
            class: "permissive".to_owned(),
        },
        memory: ModelMemory {
            weights_bytes,
            runtime_overhead_bytes: runtime_overhead(request.capability, weights_bytes),
        },
        files,
        catalog: ModelCatalog {
            title,
            summary: format!(
                "Added from {} at {}.",
                request.repo,
                &repository.commit[..7]
            ),
            recommended: false,
        },
        origin: ModelOrigin::Custom,
    };
    manifest.validate()?;
    Ok(Pinned { manifest, fetched })
}

/// Every chosen file's pin: the repository's own SHA-256 for weights, and
/// the digest of the bytes fetched here for small files kept in git.
async fn pin_files(
    hub: &Hub,
    request: &AddRequest<'_>,
    chosen: Vec<RepositoryFile>,
) -> Result<(Vec<ModelFile>, Vec<(String, Vec<u8>)>), String> {
    let mut files = Vec::with_capacity(chosen.len());
    let mut fetched = Vec::new();
    for file in chosen {
        let sha256 = if let Some(sha256) = file.sha256 {
            sha256
        } else {
            let bytes = hub
                .fetch_small(request.repo, request.commit, &file.path, file.bytes)
                .await
                .map_err(|error| error.to_string())?;
            let digest = clipmill_hub::sha256_hex(&bytes);
            fetched.push((file.path.clone(), bytes));
            digest
        };
        files.push(ModelFile {
            path: file.path,
            sha256,
            bytes: file.bytes,
        });
    }
    Ok((files, fetched))
}

enum Selection {
    /// Files a person chooses one of, pinned with the companions it needs:
    /// whisper.cpp's GGML weights need none; a GGUF model needs the vision
    /// projector (mmproj) llama.cpp looks at frames through.
    Weights {
        choices: Vec<RepositoryFile>,
        companions: Vec<RepositoryFile>,
    },
    /// MLX: every file the model directory needs, pinned together.
    Directory(Vec<RepositoryFile>),
}

fn select(repository: &Repository, runtime: CustomRuntime) -> Result<Selection, String> {
    if runtime.runtime == "whisper.cpp" {
        let choices = repository
            .files
            .iter()
            .filter(|file| {
                !file.path.contains('/')
                    && has_extension(&file.path, "bin")
                    && file.sha256.is_some()
                    && pinnable(&file.path)
            })
            .cloned()
            .collect::<Vec<_>>();
        if choices.is_empty() {
            return Err(
                "This repository has no whisper.cpp weight file (a GGML .bin kept in large-file storage)."
                    .to_owned(),
            );
        }
        return Ok(Selection::Weights {
            choices,
            companions: Vec::new(),
        });
    }
    if runtime.runtime == "llama.cpp" {
        return gguf_selection(repository);
    }
    let files = repository
        .files
        .iter()
        .filter(|file| !incidental(&file.path))
        .cloned()
        .collect::<Vec<_>>();
    if files.len() > MAX_FILES {
        return Err("This repository holds more files than one model directory.".to_owned());
    }
    if let Some(file) = files.iter().find(|file| !pinnable(&file.path)) {
        return Err(format!(
            "{} has a name ClipMill cannot pin safely.",
            file.path
        ));
    }
    if let Some(file) = files
        .iter()
        .find(|file| file.sha256.is_none() && file.bytes > SMALL_FILE_LIMIT)
    {
        return Err(format!(
            "{} is too large to pin without a SHA-256 from the repository.",
            file.path
        ));
    }
    if !files.iter().any(|file| file.path == "config.json") {
        return Err("This is not an MLX model directory: it has no config.json.".to_owned());
    }
    if !files.iter().any(|file| file.path.ends_with(".safetensors")) {
        return Err(
            "This is not an MLX model directory: it has no .safetensors weights.".to_owned(),
        );
    }
    Ok(Selection::Directory(files))
}

/// A GGUF repository: the model files to choose between, the 4-bit `Q4_K_M`
/// build first where there is one, then the smaller, and the vision
/// projector pinned beside the choice, F16 where there is one. A split
/// model is left out: its parts are one model, and one part alone loads
/// nothing.
fn gguf_selection(repository: &Repository) -> Result<Selection, String> {
    let (projectors, mut choices): (Vec<RepositoryFile>, Vec<RepositoryFile>) = repository
        .files
        .iter()
        .filter(|file| {
            !file.path.contains('/')
                && has_extension(&file.path, "gguf")
                && file.sha256.is_some()
                && pinnable(&file.path)
                && !file.path.contains("-of-")
        })
        .cloned()
        .partition(|file| file.path.to_ascii_lowercase().starts_with("mmproj"));
    if choices.is_empty() {
        return Err(
            "This repository has no GGUF model file (a .gguf kept in large-file storage)."
                .to_owned(),
        );
    }
    let projector = ["mmproj-f16.gguf", "mmproj-bf16.gguf", "mmproj-f32.gguf"]
        .iter()
        .find_map(|preferred| {
            projectors
                .iter()
                .find(|file| file.path.eq_ignore_ascii_case(preferred))
        })
        .or_else(|| projectors.first())
        .cloned()
        .ok_or_else(|| {
            "This repository has no vision projector (an mmproj .gguf file), which the editorial model needs to look at frames."
                .to_owned()
        })?;
    choices.sort_by_key(|file| {
        (
            !file.path.to_ascii_lowercase().contains("q4_k_m"),
            file.bytes,
        )
    });
    Ok(Selection::Weights {
        choices,
        companions: vec![projector],
    })
}

/// Files a model directory never loads: documentation, images, git metadata.
fn incidental(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    let name = lower.rsplit('/').next().unwrap_or(&lower);
    name.starts_with('.')
        || name.starts_with("license")
        || name.starts_with("notice")
        || [
            ".md", ".png", ".jpg", ".jpeg", ".gif", ".webp", ".pdf", ".svg",
        ]
        .iter()
        .any(|suffix| name.ends_with(suffix))
}

fn has_extension(path: &str, extension: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .is_some_and(|found| found.eq_ignore_ascii_case(extension))
}

fn pinnable(path: &str) -> bool {
    valid_relative_path(path) && clipmill_hub::check_path(path).is_ok()
}

fn permissive_spdx(license: Option<&str>) -> Option<&'static str> {
    let license = license?.trim().to_ascii_lowercase();
    PERMISSIVE
        .iter()
        .find(|(hub, _)| *hub == license)
        .map(|(_, spdx)| *spdx)
}

/// `ggml-small.en.bin` → `whisper-small.en`, "Whisper small.en".
fn whisper_names(registry: &ModelRegistry, path: &str) -> (String, String) {
    let stem = path.trim_end_matches(".bin");
    let stem = stem.strip_prefix("ggml-").unwrap_or(stem);
    let base = if stem.to_ascii_lowercase().starts_with("whisper") {
        sanitize(stem)
    } else {
        sanitize(&format!("whisper-{stem}"))
    };
    let title = if stem.to_ascii_lowercase().starts_with("whisper") {
        stem.to_owned()
    } else {
        format!("Whisper {stem}")
    };
    (unique(registry, &base), title.chars().take(64).collect())
}

/// `Qwen3.5-9B-Q4_K_M.gguf` → `qwen3.5-9b-q4-k-m`, "Qwen3.5-9B-Q4_K_M".
fn gguf_names(registry: &ModelRegistry, path: &str) -> (String, String) {
    let stem = path.rsplit_once('.').map_or(path, |(stem, _)| stem);
    (
        unique(registry, &sanitize(stem)),
        stem.chars().take(64).collect(),
    )
}

/// The quantization a GGUF file's name states (`…-Q4_K_M.gguf` is
/// `q4_k_m`), or plain `gguf` where it states none.
fn gguf_quantization(file: &str) -> String {
    let stem = file.rsplit_once('.').map_or(file, |(stem, _)| stem);
    let tail = stem
        .rsplit(['-', '.'])
        .next()
        .unwrap_or(stem)
        .to_ascii_lowercase();
    let stated = tail.starts_with('q')
        || tail.starts_with("iq")
        || matches!(tail.as_str(), "f16" | "bf16" | "f32");
    if stated { tail } else { "gguf".to_owned() }
}

/// `mlx-community/Qwen3.5-4B-4bit` → `qwen3.5-4b-4bit`, "Qwen3.5-4B-4bit".
fn directory_names(registry: &ModelRegistry, repo: &str) -> (String, String) {
    let tail = repo.rsplit('/').next().unwrap_or(repo);
    (
        unique(registry, &sanitize(tail)),
        tail.chars().take(64).collect(),
    )
}

/// Lowercase words joined by hyphens or dots, as the registry requires.
fn sanitize(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for character in raw.chars().flat_map(char::to_lowercase) {
        let character = if character.is_ascii_alphanumeric() || character == '.' {
            character
        } else {
            '-'
        };
        let previous = out.chars().last();
        if matches!(character, '-' | '.') && matches!(previous, None | Some('-' | '.')) {
            continue;
        }
        out.push(character);
    }
    let trimmed = out.trim_end_matches(['-', '.']);
    let limited = trimmed.chars().take(56).collect::<String>();
    let limited = limited.trim_end_matches(['-', '.']).to_owned();
    if limited.is_empty() {
        "model".to_owned()
    } else {
        limited
    }
}

fn unique(registry: &ModelRegistry, base: &str) -> String {
    if registry.get(base).is_none() {
        return base.to_owned();
    }
    (2..1000)
        .map(|suffix| format!("{base}-{suffix}"))
        .find(|name| registry.get(name).is_none())
        .unwrap_or_else(|| format!("{base}-custom"))
}

fn whisper_quantization(file: &str) -> String {
    let lower = file.to_ascii_lowercase();
    ["q4_0", "q4_1", "q5_0", "q5_1", "q8_0"]
        .iter()
        .find(|level| lower.contains(*level))
        .map_or_else(|| "ggml-f16".to_owned(), |level| format!("ggml-{level}"))
}

/// The bit width an MLX config states, as the registry spells it.
fn mlx_quantization(config: &[u8]) -> String {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(config) else {
        return "none".to_owned();
    };
    ["quantization", "quantization_config"]
        .iter()
        .find_map(|key| value.get(key).and_then(|table| table.get("bits")))
        .and_then(serde_json::Value::as_u64)
        .map_or_else(|| "none".to_owned(), |bits| format!("int{bits}"))
}

/// What a runtime needs beside the weights, estimated from the bundled pins:
/// whisper.cpp's compute buffers grow with the model, and an MLX language
/// model needs room for its context and image inputs. An estimate, which is
/// why a model near the device's limit is warned about rather than refused.
fn runtime_overhead(capability: &str, weights: u64) -> u64 {
    const MIB: u64 = 1024 * 1024;
    if capability == "asr" {
        (weights / 3 * 2).clamp(256 * MIB, 2048 * MIB)
    } else {
        (weights / 3).max(2048 * MIB)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

    use std::path::Path;

    use clipmill_hub::{Repository, RepositoryFile};

    const GIB: u64 = 1024 * 1024 * 1024;

    use super::{
        Selection, directory_names, gguf_names, gguf_quantization, incidental, mlx_quantization,
        permissive_spdx, runtime_overhead, sanitize, select, whisper_names, whisper_quantization,
    };
    use crate::{
        implementations::{CustomRuntime, custom_runtime},
        models::ModelRegistry,
    };

    // Each editorial runtime by name, whatever this machine runs itself.
    const MLX: CustomRuntime = CustomRuntime {
        family: "mlx-vlm",
        runtime: "mlx",
        backend: "mlx",
        quantization: "mlx",
        worker: "editorial",
    };
    const LLAMA: CustomRuntime = CustomRuntime {
        family: "gguf",
        runtime: "llama.cpp",
        backend: "llama.cpp",
        quantization: "gguf",
        worker: "editorial",
    };

    fn registry() -> ModelRegistry {
        ModelRegistry::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../models/registry"))
            .expect("registry")
    }

    fn file(path: &str, bytes: u64, lfs: bool) -> RepositoryFile {
        RepositoryFile {
            path: path.to_owned(),
            bytes,
            sha256: lfs.then(|| "a".repeat(64)),
        }
    }

    fn repository(files: Vec<RepositoryFile>) -> Repository {
        Repository {
            id: "owner/name".to_owned(),
            commit: "b".repeat(40),
            license: Some("apache-2.0".to_owned()),
            gated: false,
            private: false,
            library: None,
            pipeline: None,
            tags: Vec::new(),
            files,
        }
    }

    #[test]
    fn only_licences_that_permit_selling_the_output_are_pinned() {
        assert_eq!(permissive_spdx(Some("apache-2.0")), Some("Apache-2.0"));
        assert_eq!(permissive_spdx(Some("MIT")), Some("MIT"));
        for refused in ["cc-by-nc-4.0", "llama3", "gemma", "other", "openrail", ""] {
            assert_eq!(permissive_spdx(Some(refused)), None, "{refused}");
        }
        assert_eq!(permissive_spdx(None), None);
    }

    #[test]
    fn a_whisper_repository_offers_its_ggml_weights_to_choose_from() {
        let listing = repository(vec![
            file(".gitattributes", 1477, false),
            file("README.md", 3196, false),
            file("ggml-base.bin", 147_951_465, true),
            file("ggml-small.en-q5_1.bin", 190_085_487, true),
            file("ggml-base-encoder.mlmodelc.zip", 37_922_638, true),
            file("models/nested.bin", 10, true),
            file("unhashed.bin", 10, false),
        ]);
        let Ok(Selection::Weights {
            choices,
            companions,
        }) = select(&listing, custom_runtime("asr").expect("whisper.cpp"))
        else {
            panic!("expected weight choices");
        };
        assert!(companions.is_empty());
        let paths = choices
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>();
        assert_eq!(paths, ["ggml-base.bin", "ggml-small.en-q5_1.bin"]);
        assert_eq!(whisper_quantization("ggml-small.en-q5_1.bin"), "ggml-q5_1");
        assert_eq!(whisper_quantization("ggml-large-v3.bin"), "ggml-f16");
    }

    #[test]
    fn an_mlx_repository_pins_its_whole_directory_but_not_its_readme() {
        let listing = repository(vec![
            file(".gitattributes", 1570, false),
            file("README.md", 666, false),
            file("LICENSE", 11_000, false),
            file("config.json", 3331, false),
            file("model-00001-of-00002.safetensors", 5_349_771_222, true),
            file("model-00002-of-00002.safetensors", 600_449_850, true),
            file("model.safetensors.index.json", 123_592, false),
            file("tokenizer.json", 19_989_343, true),
            file("vocab.json", 6_722_759, false),
            file("figure.png", 10, false),
        ]);
        let Ok(Selection::Directory(files)) = select(&listing, MLX) else {
            panic!("expected a directory");
        };
        let paths = files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            paths,
            [
                "config.json",
                "model-00001-of-00002.safetensors",
                "model-00002-of-00002.safetensors",
                "model.safetensors.index.json",
                "tokenizer.json",
                "vocab.json"
            ]
        );
        assert!(incidental("docs/diagram.PNG"));
        assert!(!incidental("chat_template.jinja"));
    }

    #[test]
    fn a_directory_without_weights_or_with_unsafe_names_is_refused() {
        let runtime = MLX;
        let no_config = repository(vec![file("model.safetensors", 10, true)]);
        assert!(select(&no_config, runtime).is_err());
        let no_weights = repository(vec![file("config.json", 10, false)]);
        assert!(select(&no_weights, runtime).is_err());
        let spaced = repository(vec![
            file("config.json", 10, false),
            file("model.safetensors", 10, true),
            file("weird name.json", 10, false),
        ]);
        assert!(select(&spaced, runtime).is_err());
        let huge_unhashed = repository(vec![
            file("config.json", 10, false),
            file("model.safetensors", 10, true),
            file("merges.txt", 128 * 1024 * 1024, false),
        ]);
        assert!(select(&huge_unhashed, runtime).is_err());
    }

    #[test]
    fn suggested_names_follow_the_registry_rules_and_never_collide() {
        let registry = registry();
        let (name, title) = whisper_names(&registry, "ggml-small.en.bin");
        assert_eq!(name, "whisper-small.en");
        assert_eq!(title, "Whisper small.en");
        // The bundled pin already owns this name.
        let (name, _) = whisper_names(&registry, "ggml-base.bin");
        assert_eq!(name, "whisper-base-2");
        let (name, title) = directory_names(&registry, "mlx-community/Qwen3.5-4B-4bit");
        assert_eq!(name, "qwen3.5-4b-4bit");
        assert_eq!(title, "Qwen3.5-4B-4bit");
        assert_eq!(sanitize("--Weird__Name!!v2.."), "weird-name-v2");
        assert_eq!(sanitize("!!!"), "model");
        for raw in ["Gemma 4 12B (it)", "a..b", "x-", "UPPER.case"] {
            assert!(crate::models::valid_name(&sanitize(raw)), "{raw}");
        }
    }

    #[test]
    fn quantization_and_overhead_are_read_or_estimated() {
        assert_eq!(
            mlx_quantization(br#"{"quantization": {"group_size": 64, "bits": 4}}"#),
            "int4"
        );
        assert_eq!(mlx_quantization(br#"{"model_type": "qwen"}"#), "none");
        assert_eq!(runtime_overhead("editorial", 6 * GIB), 2 * GIB);
        assert_eq!(runtime_overhead("editorial", 15 * GIB), 5 * GIB);
        assert_eq!(
            runtime_overhead("asr", 100 * 1024 * 1024),
            256 * 1024 * 1024
        );
        assert_eq!(runtime_overhead("asr", 3 * GIB), 2 * GIB);
    }

    #[test]
    fn a_gguf_repository_offers_its_models_and_pins_the_projector_beside_them() {
        let listing = repository(vec![
            file("README.md", 3196, false),
            file("Qwen3.5-9B-Q8_0.gguf", 9_530_000_000, true),
            file("Qwen3.5-9B-Q4_K_M.gguf", 5_680_522_464, true),
            file("Qwen3.5-9B-Q2_K.gguf", 3_600_000_000, true),
            file("Qwen3.5-9B-BF16-00001-of-00002.gguf", 9_000_000_000, true),
            file("mmproj-BF16.gguf", 918_000_000, true),
            file("mmproj-F16.gguf", 918_166_080, true),
            file("unhashed.gguf", 10, false),
        ]);
        let Ok(Selection::Weights {
            choices,
            companions,
        }) = select(&listing, LLAMA)
        else {
            panic!("expected model choices");
        };
        let paths = choices
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>();
        // The usual 4-bit build first, then the smaller; no split parts.
        assert_eq!(
            paths,
            [
                "Qwen3.5-9B-Q4_K_M.gguf",
                "Qwen3.5-9B-Q2_K.gguf",
                "Qwen3.5-9B-Q8_0.gguf"
            ]
        );
        let projector = companions
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>();
        assert_eq!(projector, ["mmproj-F16.gguf"]);
    }

    #[test]
    fn a_gguf_repository_without_a_vision_projector_is_refused() {
        let listing = repository(vec![file("model-Q4_K_M.gguf", 10, true)]);
        let Err(problem) = select(&listing, LLAMA) else {
            panic!("a model that cannot see frames was accepted");
        };
        assert!(problem.contains("vision projector"), "{problem}");
        let Err(problem) = select(&repository(vec![file("mmproj-F16.gguf", 10, true)]), LLAMA)
        else {
            panic!("a projector alone was accepted");
        };
        assert!(problem.contains("no GGUF model file"), "{problem}");
    }

    #[test]
    fn gguf_names_and_quantization_come_from_the_file() {
        let registry = registry();
        let (name, title) = gguf_names(&registry, "Qwen3.5-9B-Q4_K_M.gguf");
        assert_eq!(name, "qwen3.5-9b-q4-k-m");
        assert_eq!(title, "Qwen3.5-9B-Q4_K_M");
        assert_eq!(gguf_quantization("Qwen3.5-9B-Q4_K_M.gguf"), "q4_k_m");
        assert_eq!(gguf_quantization("model.IQ3_XS.gguf"), "iq3_xs");
        assert_eq!(gguf_quantization("Model-BF16.gguf"), "bf16");
        assert_eq!(gguf_quantization("model.gguf"), "gguf");
    }
}
