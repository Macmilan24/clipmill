//! Everything the Inspector's two calls need, loaded once.
//!
//! Directing a clip reads six documents, three of which may legitimately not
//! exist. Gathering them is most of the work and none of the interest, so it
//! lives here rather than inside the request handler, which is then short
//! enough to read as what it is: a refusal ladder followed by one call into the
//! director.
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
}

impl LoadError {
    pub(crate) fn message(&self) -> String {
        match self {
            Self::Missing(what) => {
                format!("this source has no published {what} to direct a clip from")
            }
            Self::Unverified(what) => format!("the published {what} does not match its manifest"),
            Self::NoFrame => "the source map states no usable frame size".to_owned(),
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

pub(crate) async fn load(
    database: &DbHandle,
    artifacts: &ArtifactHandle,
    source_id: &str,
    source_map_json: &[u8],
) -> Result<Evidence, LoadError> {
    let candidates: DiscoveryCandidates =
        require(database, artifacts, source_id, REQUIRED[0]).await?;
    let ranking: RankingSet = require(database, artifacts, source_id, REQUIRED[1]).await?;
    let transcript: SpeechTranscript = require(database, artifacts, source_id, REQUIRED[2]).await?;

    let index: Option<IndexTranscript> = optional(
        database,
        artifacts,
        source_id,
        "index-transcript",
        "index.json",
    )
    .await;
    let shots: Option<EvidenceShots> =
        optional(database, artifacts, source_id, "detect-shots", "shots.json").await;
    let faces: Option<VisionFaceTrack> =
        optional(database, artifacts, source_id, "detect-faces", "faces.json").await;

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

async fn require<T: DeserializeOwned>(
    database: &DbHandle,
    artifacts: &ArtifactHandle,
    source_id: &str,
    (kind, file, name): (&'static str, &'static str, &'static str),
) -> Result<T, LoadError> {
    let Ok(Some(address)) = database
        .latest_source_task_artifact(source_id.to_owned(), kind.to_owned())
        .await
    else {
        return Err(LoadError::Missing(name));
    };
    read(artifacts, &address, file)
        .await
        .ok_or(LoadError::Unverified(name))
}

async fn optional<T: DeserializeOwned>(
    database: &DbHandle,
    artifacts: &ArtifactHandle,
    source_id: &str,
    kind: &str,
    file: &str,
) -> Option<T> {
    let address = database
        .latest_source_task_artifact(source_id.to_owned(), kind.to_owned())
        .await
        .ok()
        .flatten()?;
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
fn frame_of(source_map_json: &[u8]) -> Option<Frame> {
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

    use super::frame_of;

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
}
