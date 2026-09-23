use super::{Error, Publication, PublishingCommand, Service, YoutubeVideoMetadataV1};
use clipmill_artifacts::{ArtifactLease, ArtifactPath};
use clipmill_contracts::proto::ipc::v1::{
    DraftYoutubeMetadataRequest, DraftYoutubeMetadataResponse, JobState, TaskState,
};
use clipmill_core::ArtifactId;
use clipmill_render::{CLIP_FILE, MANIFEST_FILE, RenderManifest};
use sha2::{Digest, Sha256};
use std::io::{Read, Seek, SeekFrom};

pub(super) struct VerifiedExport {
    pub project_id: String,
    pub doc_id: String,
    pub ir_id: String,
    pub render_id: String,
    pub bytes: u64,
    pub sha256: String,
    pub lease: ArtifactLease,
    pub sources: Vec<String>,
}

impl Service {
    pub(super) async fn verified_youtube_export(
        &self,
        job_id: &str,
        revision: u64,
    ) -> Result<VerifiedExport, Error> {
        let job = self
            .database
            .get_job(job_id.into())
            .await
            .map_err(|_| Error::Invalid("Choose a completed export."))?;
        let summary = job.export.ok_or(Error::Invalid(
            "Choose an export job, not an analysis or preview.",
        ))?;
        if job.state != JobState::Succeeded as i32 || summary.revision != revision {
            return Err(Error::Invalid(
                "The selected export is not complete at the reviewed revision.",
            ));
        }
        let mut renders = job
            .tasks
            .iter()
            .filter(|task| task.kind == "render-clip" && task.state == TaskState::Succeeded as i32);
        let render = renders
            .next()
            .ok_or(Error::Invalid("This export has no completed render."))?;
        if renders.next().is_some() {
            return Err(Error::Protocol);
        }
        let render_id = render
            .output_artifact_id
            .parse::<ArtifactId>()
            .map_err(|_| Error::Protocol)?;
        let artifacts = self
            .artifacts
            .as_ref()
            .ok_or(Error::Invalid("Media storage is unavailable."))?;
        let lease = artifacts.open(render_id).await.map_err(|_| {
            Error::Invalid("The exported video is missing. Export it again before uploading.")
        })?;
        let expected_ir = summary.ir_artifact_id.clone();
        let (lease, manifest, bytes, sha256) = tokio::task::spawn_blocking(move || {
            let manifest = verified_manifest(&lease, &expected_ir)?;
            let (bytes, sha256, _) = verified_clip(&lease, &manifest)?;
            Ok::<_, Error>((lease, manifest, bytes, sha256))
        })
        .await
        .map_err(|_| Error::Protocol)??;
        Ok(VerifiedExport {
            project_id: job.project_id,
            doc_id: summary.doc_id,
            ir_id: summary.ir_artifact_id,
            render_id: render_id.to_string(),
            bytes,
            sha256,
            lease,
            sources: manifest.input_source_fingerprints,
        })
    }
    pub(super) async fn verified_publication_file(
        &self,
        record: &Publication,
    ) -> Result<(ArtifactLease, tokio::fs::File), Error> {
        let id = record
            .view
            .render_artifact_id
            .parse::<ArtifactId>()
            .map_err(|_| Error::Protocol)?;
        let lease=self.artifacts.as_ref().ok_or(Error::Protocol)?.open(id).await.map_err(|_|Error::Invalid("The approved rendered video is unavailable. Restore its local media before resuming."))?;
        let expected = record.clone();
        let (lease, file) = tokio::task::spawn_blocking(move || {
            let manifest = verified_manifest(&lease, &expected.view.ir_artifact_id)?;
            let (bytes, digest, file) = verified_clip(&lease, &manifest)?;
            if bytes != expected.view.total_bytes || digest != expected.sha256 {
                return Err(Error::Invalid(
                    "The approved video changed. This upload will not send different bytes.",
                ));
            }
            Ok((lease, file))
        })
        .await
        .map_err(|_| Error::Protocol)??;
        Ok((lease, tokio::fs::File::from_std(file)))
    }
    pub(super) async fn youtube_metadata_draft(
        &self,
        request: &DraftYoutubeMetadataRequest,
    ) -> Result<DraftYoutubeMetadataResponse, Error> {
        let export = self
            .verified_youtube_export(&request.export_job_id, request.expected_revision)
            .await?;
        let stored = self
            .database
            .publishing(PublishingCommand::ExportPayload {
                job_id: request.export_job_id.clone(),
            })
            .await
            .map_err(|_| Error::Protocol)?
            .export_payload
            .ok_or(Error::Protocol)?;
        let title = stored
            .request
            .map_or_else(String::new, |request| request.title);
        let ir_id = export.ir_id.parse().map_err(|_| Error::Protocol)?;
        let lease = self
            .artifacts
            .as_ref()
            .ok_or(Error::Protocol)?
            .open(ir_id)
            .await
            .map_err(|_| Error::Invalid("The approved edit snapshot is unavailable."))?;
        let excerpt = tokio::task::spawn_blocking(move || {
            let mut file = lease
                .open_verified(&"edit-ir.json".parse().map_err(|_| Error::Protocol)?)
                .map_err(|_| Error::Protocol)?;
            let mut bytes = Vec::new();
            file.by_ref()
                .take(16 * 1024 * 1024 + 1)
                .read_to_end(&mut bytes)
                .map_err(|_| Error::Protocol)?;
            if bytes.len() > 16 * 1024 * 1024 {
                return Err(Error::Protocol);
            }
            let document: clipmill_edit_ir::EditDocument =
                serde_json::from_slice(&bytes).map_err(|_| Error::Protocol)?;
            Ok::<_, Error>(
                document
                    .captions
                    .cues
                    .iter()
                    .flat_map(|cue| cue.lines.iter())
                    .flat_map(|line| &line.words)
                    .map(|word| word.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        })
        .await
        .map_err(|_| Error::Protocol)??;
        let mut source_link = None;
        let imports = self
            .database
            .youtube_imports(crate::db::YoutubeCommand::List {
                project_id: export.project_id.clone(),
            })
            .await
            .map_err(|_| Error::Protocol)?;
        for imported in imports
            .into_iter()
            .filter(|record| record.state == "completed")
        {
            if let Ok(source) = self.database.get_source(imported.source_id).await
                && export.sources.contains(&source.source_fingerprint)
            {
                source_link = Some(imported.canonical_url);
                break;
            }
        }
        let metadata = grounded_draft(&title, &excerpt, source_link.as_deref());
        Ok(DraftYoutubeMetadataResponse {
            metadata: Some(metadata),
            render_artifact_id: export.render_id,
            revision: request.expected_revision,
            transcript_excerpt: clean(&excerpt, 2000),
        })
    }
}

fn verified_manifest(lease: &ArtifactLease, expected_ir: &str) -> Result<RenderManifest, Error> {
    let path = MANIFEST_FILE
        .parse::<ArtifactPath>()
        .map_err(|_| Error::Protocol)?;
    let mut file = lease
        .open_verified(&path)
        .map_err(|_| Error::Invalid("The approved render manifest failed its integrity check."))?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::Protocol)?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err(Error::Protocol);
    }
    let manifest: RenderManifest = serde_json::from_slice(&bytes).map_err(|_| Error::Protocol)?;
    if manifest.ir_artifact_id != expected_ir {
        return Err(Error::Invalid(
            "The rendered video does not match the approved edit snapshot.",
        ));
    }
    Ok(manifest)
}
fn verified_clip(
    lease: &ArtifactLease,
    manifest: &RenderManifest,
) -> Result<(u64, String, std::fs::File), Error> {
    let output = manifest
        .outputs
        .iter()
        .find(|output| output.path == CLIP_FILE)
        .ok_or(Error::Protocol)?;
    let mut file = lease
        .open_verified(&CLIP_FILE.parse().map_err(|_| Error::Protocol)?)
        .map_err(|_| {
            Error::Invalid("The approved video failed its integrity check. Export it again.")
        })?;
    let bytes = file.metadata().map_err(|_| Error::Protocol)?.len();
    let mut hash = Sha256::new();
    let mut buffer = vec![0_u8; 65536];
    loop {
        let count = file.read(&mut buffer).map_err(|_| Error::Protocol)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    let digest = format!("sha256:{}", hex::encode(hash.finalize()));
    if bytes == 0 || bytes != output.bytes || digest != output.sha256 {
        return Err(Error::Invalid(
            "The video does not match its approved render manifest.",
        ));
    }
    file.seek(SeekFrom::Start(0)).map_err(|_| Error::Protocol)?;
    Ok((bytes, digest, file))
}
fn clean(text: &str, limit: usize) -> String {
    text.chars()
        .filter(|ch| !ch.is_control() && !matches!(ch, '<' | '>'))
        .take(limit)
        .collect()
}
fn grounded_draft(title: &str, excerpt: &str, source: Option<&str>) -> YoutubeVideoMetadataV1 {
    let fallback = excerpt
        .split_whitespace()
        .take(12)
        .collect::<Vec<_>>()
        .join(" ");
    let chosen = if title.trim().is_empty() {
        if fallback.is_empty() {
            "Untitled clip"
        } else {
            &fallback
        }
    } else {
        title
    };
    let auto_prefix = excerpt
        .split_whitespace()
        .take(6)
        .collect::<Vec<_>>()
        .join(" ");
    let chosen = if chosen.trim() == auto_prefix && !excerpt.trim().is_empty() {
        excerpt
            .split_inclusive(['.', '!', '?'])
            .next()
            .unwrap_or(excerpt)
            .trim()
    } else {
        chosen
    };
    let title = clean(chosen, 100);
    let mut description = if excerpt.trim().is_empty() {
        "Excerpt from the approved clip.".into()
    } else {
        format!("From the clip:\n“{}”", clean(excerpt, 2000))
    };
    if let Some(source) = source {
        description.push_str("\n\nSource: ");
        description.push_str(source);
    }
    let mut tags = Vec::new();
    for word in title.split(|ch: char| !ch.is_alphanumeric()) {
        let normalized = word.to_lowercase();
        if word.chars().count() >= 4
            && !matches!(
                normalized.as_str(),
                "this"
                    | "that"
                    | "with"
                    | "from"
                    | "have"
                    | "what"
                    | "when"
                    | "your"
                    | "they"
                    | "about"
                    | "clip"
                    | "untitled"
            )
            && !tags.contains(&normalized)
        {
            tags.push(normalized);
        }
        if tags.len() == 3 {
            break;
        }
    }
    if !tags.is_empty() {
        description.push_str("\n\n");
        description.push_str(
            &tags
                .iter()
                .map(|tag| format!("#{tag}"))
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
    YoutubeVideoMetadataV1 {
        title,
        description,
        tags,
        made_for_kids: false,
        contains_synthetic_media: false,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::panic)]
    use super::*;
    use std::io::Write;
    #[test]
    fn draft_quotes_the_frozen_clip_and_never_invents_a_summary() {
        let draft = grounded_draft(
            "Why SQLite works",
            "We chose SQLite because the files stay local.",
            Some("https://www.youtube.com/watch?v=abcdefghijk"),
        );
        assert_eq!(draft.title, "Why SQLite works");
        assert!(
            draft
                .description
                .contains("We chose SQLite because the files stay local.")
        );
        assert!(
            draft
                .description
                .contains("Source: https://www.youtube.com/watch?v=abcdefghijk")
        );
        assert_eq!(draft.tags, vec!["sqlite", "works"]);
    }
    #[test]
    fn draft_without_a_title_uses_only_spoken_words_and_empty_clips_stay_honest() {
        assert_eq!(
            grounded_draft("", "The name is Ada Lovelace.", None).title,
            "The name is Ada Lovelace."
        );
        assert_eq!(grounded_draft("", "", None).title, "Untitled clip");
        assert!(grounded_draft("", "", None).tags.is_empty());
    }
    #[test]
    fn automatic_caption_prefix_expands_to_the_opening_sentence_and_grounded_hashtags() {
        let draft = grounded_draft(
            "Why did we choose SQLite for",
            "Why did we choose SQLite for local storage? Because the files remain portable.",
            None,
        );
        assert_eq!(draft.title, "Why did we choose SQLite for local storage?");
        assert!(draft.description.ends_with("#choose #sqlite #local"));
        assert_eq!(
            grounded_draft(
                "My reviewed title",
                "Why did we choose SQLite for local storage?",
                None
            )
            .title,
            "My reviewed title"
        );
    }
    fn rendered_fixture(
        manifest_hash: &str,
    ) -> (
        tempfile::TempDir,
        clipmill_artifacts::ArtifactStore,
        ArtifactLease,
        RenderManifest,
    ) {
        use clipmill_artifacts::{
            ArtifactRecipe, ArtifactStore, NetworkPolicy, PrepareOutcome, Producer, RecipeSpec,
            Timebase,
        };
        let temp = tempfile::tempdir().unwrap();
        let (mut store, _) = ArtifactStore::initialize(temp.path()).unwrap();
        let recipe = ArtifactRecipe::try_from_spec(RecipeSpec {
            kind: "render.clip.v1".into(),
            source_fingerprint: clipmill_core::Sha256Digest::from_bytes([1; 32]),
            timebase: Timebase {
                num: 1,
                den: 90_000,
            },
            producer: Producer {
                stage: "render-clip".into(),
                implementation: "test@1".into(),
                model_digest: None,
            },
            inputs: vec![],
            policy: NetworkPolicy::LocalLock,
            config: serde_json::Map::new(),
            semantic_version: "1.0.0".into(),
        })
        .unwrap();
        let clip = b"immutable fixture bytes";
        let digest = format!("sha256:{}", hex::encode(Sha256::digest(clip)));
        let manifest:RenderManifest=serde_json::from_value(serde_json::json!({
            "schema_version":"clipmill.render.clip.v1","ir_hash":"ir-digest","ir_artifact_id":"frozen-ir","profile":clipmill_render::RenderProfile::default(),
            "engine":{"app":"fixture","ffmpeg":"fixture","font_sha256":"fixture","font_family":"Inter"},"determinism":"semantic",
            "ai_use_summary":{"assistance":[],"generated":[],"requires_youtube_ai_disclosure":false},"rights":{"source_attestation":"own_content","gates_passed":[]},"input_source_fingerprints":[],
            "program":{"duration_ticks":90000,"frame_count":30,"segments":[]},
            "loudness":{"target_lufs":-14.0,"target_true_peak_dbtp":-1.0,"measured_input":{"integrated_lufs":-14.0,"true_peak_dbtp":-1.0,"loudness_range_lu":1.0},"measured_output":{"integrated_lufs":-14.0,"true_peak_dbtp":-1.0,"loudness_range_lu":1.0}},
            "caption_windows":[],"outputs":[{"path":CLIP_FILE,"sha256":if manifest_hash.is_empty(){digest.as_str()}else{manifest_hash},"bytes":clip.len()}]
        })).unwrap();
        let PrepareOutcome::Miss(staging) = store.prepare(recipe).unwrap() else {
            panic!("new fixture")
        };
        let media_path = CLIP_FILE.parse().unwrap();
        let manifest_path = MANIFEST_FILE.parse().unwrap();
        staging
            .create_file(&media_path)
            .unwrap()
            .write_all(clip)
            .unwrap();
        staging
            .create_file(&manifest_path)
            .unwrap()
            .write_all(&serde_json::to_vec(&manifest).unwrap())
            .unwrap();
        let lease = store
            .commit(
                staging.id(),
                vec![media_path, manifest_path],
                std::collections::BTreeMap::new(),
            )
            .unwrap();
        (temp, store, lease, manifest)
    }
    #[test]
    fn approved_artifact_requires_matching_snapshot_manifest_and_exact_bytes() {
        let (_temp, _store, lease, manifest) = rendered_fixture("");
        assert!(verified_manifest(&lease, "another-ir").is_err());
        let saved = verified_manifest(&lease, "frozen-ir").unwrap();
        let (bytes, digest, mut file) = verified_clip(&lease, &saved).unwrap();
        assert_eq!(bytes, manifest.outputs[0].bytes);
        assert_eq!(digest, manifest.outputs[0].sha256);
        let mut content = Vec::new();
        file.read_to_end(&mut content).unwrap();
        assert_eq!(
            content, b"immutable fixture bytes",
            "verified handle rewinds before sending"
        );
        let (_temp, _store, lease, manifest) = rendered_fixture("sha256:wrong");
        assert!(
            verified_clip(&lease, &manifest).is_err(),
            "CAS membership does not replace render-manifest verification"
        );
    }
}
