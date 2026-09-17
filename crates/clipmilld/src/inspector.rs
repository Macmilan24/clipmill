//! Everything the Inspector's two calls need, loaded once.
//!
//! Directing a clip reads six documents, three of which may legitimately not
//! exist. Gathering them is most of the work and none of the interest, so it
//! lives here rather than inside the request handler, which is then short
//! enough to read as what it is: a refusal ladder followed by one call into the
//! director.
//!
//! The six are read as **one snapshot**. A candidate id is minted by a run, and
//! the boundaries, transcript and face tracks a clip is built from are that
//! run's; a document assembled from the newest publication of each stage would
//! mix a re-analysis that renumbered the candidates, or one still half
//! published, into a clip chosen from another. So a caller that names the run
//! gets that run's stages and nothing else, and a caller that does not gets
//! the newest of each — checked, either way, against what the documents say
//! about one another: the ranking names the candidate set and transcript it
//! was computed over, and a set that is not the one loaded is a refusal.
//!
//! The optional three are optional for different reasons and the distinction is
//! kept. No evidence index means captions break on punctuation alone. No shot
//! detection means nothing is known about where the picture changes. No face
//! tracks means nobody looked, which is a different sentence from "nobody earned
//! the frame" — and the director says which.

use clipmill_contracts::schemas::{
    discovery_candidates::DiscoveryCandidates, evidence_shots::EvidenceShots,
    index_transcript::IndexTranscript, ranking_set::RankingSet,
    speech_transcript::SpeechTranscript, vision_face_track::VisionFaceTrack,
};
use clipmill_core::ArtifactId;
use clipmill_director::Frame;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{artifacts::ArtifactHandle, db::DbHandle, speech};

/// The documents a directed clip is assembled from.
pub(crate) struct Evidence {
    pub candidates: DiscoveryCandidates,
    pub ranking: RankingSet,
    pub transcript: SpeechTranscript,
    pub index: Option<IndexTranscript>,
    pub shots: Option<EvidenceShots>,
    pub faces: Option<VisionFaceTrack>,
    pub frame: Frame,
}

/// Why a clip could not be directed, in the words the caller should hear.
pub(crate) enum LoadError {
    /// A document the director cannot work without.
    Missing(&'static str),
    /// Published, but it does not match its manifest.
    Unverified(&'static str),
    /// The source map states no usable frame, so there is nothing to crop in.
    NoFrame,
    /// The run the caller named does not exist, or is not this source's.
    NoSuchRun,
    /// The stages loaded do not describe one another: the ranking was
    /// computed over a different candidate set or transcript than the one
    /// published beside it.
    Incoherent(&'static str),
}

impl LoadError {
    pub(crate) fn message(&self) -> String {
        match self {
            Self::Missing(what) => {
                format!("this analysis has no published {what} to direct a clip from")
            }
            Self::Unverified(what) => format!("the published {what} does not match its manifest"),
            Self::NoFrame => "the source map states no usable frame size".to_owned(),
            Self::NoSuchRun => "the analysis run named is not one this source has".to_owned(),
            Self::Incoherent(what) => format!(
                "the published ranking was computed over a different {what} than the one \
                 published beside it; the analysis is not one snapshot"
            ),
        }
    }
}

/// Which stage publishes each document, and what the document is called inside
/// its artifact. Written out rather than derived, because a wrong guess here
/// reads as a missing artifact rather than as a typo.
///
/// These are **task** kinds, and the distinction is not pedantry: the assembly
/// that fuses voice activity, recognition and alignment runs as
/// `speech-transcript`, while `transcribe-source` is the *job* that carries it
/// when the speech chain is submitted on its own. Naming the job here found
/// nothing for every recording analyzed through the composite DAG, which is
/// every recording a user has.
const REQUIRED: [(&str, &str, &str); 3] = [
    ("discover-candidates", "candidates.json", "candidate set"),
    ("rank-candidates", "ranking.json", "ranking"),
    (speech::KIND_TRANSCRIPT, "transcript.json", "transcript"),
];

const OPTIONAL: [(&str, &str); 3] = [
    ("index-transcript", "index.json"),
    ("detect-shots", "shots.json"),
    ("detect-faces", "faces.json"),
];

/// Where each stage's artifact is found: one run's publications, or the newest
/// of each stage over the source.
enum Stages {
    Run(crate::db::RunArtifacts),
    Newest { source_id: String },
}

impl Stages {
    async fn address(&self, database: &DbHandle, task_kind: &str) -> Option<String> {
        match self {
            Self::Run(run) => run.by_task_kind.get(task_kind).cloned(),
            Self::Newest { source_id } => database
                .latest_source_task_artifact(source_id.clone(), task_kind.to_owned())
                .await
                .ok()
                .flatten(),
        }
    }
}

/// Load a clip's evidence: from the run named, or the newest of each stage.
pub(crate) async fn load(
    database: &DbHandle,
    artifacts: &ArtifactHandle,
    source_id: &str,
    source_map_json: &[u8],
    run: Option<&str>,
) -> Result<Evidence, LoadError> {
    let stages = match run {
        Some(job_id) => {
            let run = database
                .run_task_artifacts(job_id.to_owned())
                .await
                .map_err(|_| LoadError::NoSuchRun)?;
            if run.source_id.as_deref() != Some(source_id) {
                return Err(LoadError::NoSuchRun);
            }
            Stages::Run(run)
        }
        None => Stages::Newest {
            source_id: source_id.to_owned(),
        },
    };

    let (candidates_id, candidates): (String, DiscoveryCandidates) =
        require(database, artifacts, &stages, REQUIRED[0]).await?;
    let (_, ranking): (String, RankingSet) =
        require(database, artifacts, &stages, REQUIRED[1]).await?;
    let (transcript_id, transcript): (String, SpeechTranscript) =
        require(database, artifacts, &stages, REQUIRED[2]).await?;
    check_coherence(&ranking, &candidates_id, &transcript_id)?;

    let index: Option<IndexTranscript> = optional(database, artifacts, &stages, OPTIONAL[0]).await;
    let shots: Option<EvidenceShots> = optional(database, artifacts, &stages, OPTIONAL[1]).await;
    let faces: Option<VisionFaceTrack> = optional(database, artifacts, &stages, OPTIONAL[2]).await;

    Ok(Evidence {
        candidates,
        ranking,
        transcript,
        index,
        shots,
        faces,
        frame: frame_of(source_map_json).ok_or(LoadError::NoFrame)?,
    })
}

/// The ranking's own account of its inputs, held against what was loaded.
///
/// A deterministic check of citations, not of judgement: it establishes that
/// the three documents are one analysis, which is all a citation can
/// establish. The index is not checked because it is optional to load, and a
/// ranking computed with one is still a ranking of these candidates.
pub(crate) fn check_coherence(
    ranking: &RankingSet,
    candidates_id: &str,
    transcript_id: &str,
) -> Result<(), LoadError> {
    if ranking.inputs.candidates_artifact_id.as_str() != candidates_id {
        return Err(LoadError::Incoherent("candidate set"));
    }
    if ranking.inputs.transcript_artifact_id.as_str() != transcript_id {
        return Err(LoadError::Incoherent("transcript"));
    }
    Ok(())
}

async fn require<T: DeserializeOwned>(
    database: &DbHandle,
    artifacts: &ArtifactHandle,
    stages: &Stages,
    (kind, file, name): (&'static str, &'static str, &'static str),
) -> Result<(String, T), LoadError> {
    let Some(address) = stages.address(database, kind).await else {
        return Err(LoadError::Missing(name));
    };
    let document = read(artifacts, &address, file)
        .await
        .ok_or(LoadError::Unverified(name))?;
    Ok((address, document))
}

async fn optional<T: DeserializeOwned>(
    database: &DbHandle,
    artifacts: &ArtifactHandle,
    stages: &Stages,
    (kind, file): (&'static str, &'static str),
) -> Option<T> {
    let address = stages.address(database, kind).await?;
    read(artifacts, &address, file).await
}

/// One verified document, or nothing.
///
/// A document that fails verification is treated as absent for the optional
/// three and as an error for the required three, which is the same rule the
/// stages use: the store is the authority on whether bytes are what they claim.
async fn read<T: DeserializeOwned>(
    artifacts: &ArtifactHandle,
    address: &str,
    file: &str,
) -> Option<T> {
    let artifact_id = address.parse::<ArtifactId>().ok()?;
    let lease = artifacts.open(artifact_id).await.ok()?;
    crate::media::read_artifact_document(&lease, file).ok()
}

/// Display dimensions of the source's first video stream.
pub(crate) fn frame_of(source_map_json: &[u8]) -> Option<Frame> {
    let map: Value = serde_json::from_slice(source_map_json).ok()?;
    let streams = map.get("streams")?.as_array()?;
    let video = streams.iter().find(|stream| stream["kind"] == "video")?;
    let dimension = |primary: &str, fallback: &str| -> i64 {
        video["video"][primary]
            .as_i64()
            .or_else(|| video["video"][fallback].as_i64())
            .unwrap_or(0)
    };
    let width = dimension("display_width", "coded_width");
    let height = dimension("display_height", "coded_height");
    (width > 0 && height > 0).then_some(Frame { width, height })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use clipmill_contracts::schemas::ranking_set::RankingSet;

    use super::{LoadError, check_coherence, frame_of};

    #[test]
    fn the_display_dimensions_are_preferred_over_the_coded_ones() {
        // Anamorphic sources code a narrower frame than they display, and the
        // crop the user sees is measured in what is displayed.
        let map = br#"{"streams":[{"kind":"video","video":{
            "coded_width":1440,"coded_height":1080,
            "display_width":1920,"display_height":1080}}]}"#;
        assert_eq!(frame_of(map).map(|frame| frame.width), Some(1920));
    }

    #[test]
    fn a_source_with_only_coded_dimensions_still_answers() {
        let map = br#"{"streams":[{"kind":"video","video":{
            "coded_width":1280,"coded_height":720}}]}"#;
        let frame = frame_of(map).expect("a frame");
        assert_eq!((frame.width, frame.height), (1280, 720));
    }

    #[test]
    fn a_source_with_no_video_has_no_frame_to_crop_in() {
        assert!(frame_of(br#"{"streams":[{"kind":"audio"}]}"#).is_none());
        assert!(frame_of(br#"{"streams":[]}"#).is_none());
        assert!(frame_of(b"not json").is_none());
    }

    #[test]
    fn a_zero_dimension_is_no_frame_rather_than_a_frame_of_zero() {
        let map = br#"{"streams":[{"kind":"video","video":{
            "coded_width":0,"coded_height":720}}]}"#;
        assert!(frame_of(map).is_none());
    }

    /// The published interview ranking, whose inputs name the fixtures it was
    /// computed over.
    fn ranking() -> RankingSet {
        let raw = include_str!("../../../contracts/fixtures/ranking.set/valid/interview.json");
        serde_json::from_str(raw).expect("the interview ranking fixture parses")
    }

    #[test]
    fn a_ranking_is_coherent_with_the_candidates_and_transcript_it_names() {
        let ranking = ranking();
        let candidates = ranking.inputs.candidates_artifact_id.to_string();
        let transcript = ranking.inputs.transcript_artifact_id.to_string();
        assert!(check_coherence(&ranking, &candidates, &transcript).is_ok());
    }

    #[test]
    fn a_ranking_over_another_candidate_set_is_refused_as_not_one_snapshot() {
        // A re-analysis published a new candidate set beside an old ranking:
        // the newest of each stage, read together, is not an analysis.
        let ranking = ranking();
        let other = format!("sha256:{}", "ee".repeat(32));
        let transcript = ranking.inputs.transcript_artifact_id.to_string();
        assert!(matches!(
            check_coherence(&ranking, &other, &transcript),
            Err(LoadError::Incoherent("candidate set"))
        ));
        let candidates = ranking.inputs.candidates_artifact_id.to_string();
        assert!(matches!(
            check_coherence(&ranking, &candidates, &other),
            Err(LoadError::Incoherent("transcript"))
        ));
        assert!(
            LoadError::Incoherent("transcript")
                .message()
                .contains("not one snapshot")
        );
    }
}
