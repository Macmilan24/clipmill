//! What the daemon checks before a candidate set gets an address.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use clipmill_contracts::proto::ipc::v1::{ClipDurationV1, DiscoverStagePayloadV1};

use super::{KIND_DISCOVER, inputs_for, parameters_of};
use crate::jobs::DISCOVER_STAGE_KEY_VERSION;

const INDEX: &str = "sha256:1de0000000000000000000000000000000000000000000000000000000000011";
const TRANSCRIPT: &str = "sha256:7a11000000000000000000000000000000000000000000000000000000000042";
const LOUDNESS: &str = "sha256:10ad000000000000000000000000000000000000000000000000000000000099";

fn payload() -> DiscoverStagePayloadV1 {
    DiscoverStagePayloadV1 {
        key_version: DISCOVER_STAGE_KEY_VERSION.to_owned(),
        stage: KIND_DISCOVER.to_owned(),
        index_artifact_id: INDEX.to_owned(),
        transcript_artifact_id: TRANSCRIPT.to_owned(),
        loudness_artifact_id: String::new(),
        duration: None,
        exploration_floor: 0,
    }
}

/// The key must cover every document the search read, or one recording's
/// candidate set would be served for another's loudness.
#[test]
fn the_key_covers_every_document_that_was_read() {
    assert_eq!(inputs_for(&payload()).expect("addresses").len(), 2);
    let with_audio = DiscoverStagePayloadV1 {
        loudness_artifact_id: LOUDNESS.to_owned(),
        ..payload()
    };
    let inputs = inputs_for(&with_audio).expect("addresses");
    assert_eq!(inputs.len(), 3);
    assert_eq!(inputs[0].to_string(), INDEX);
    assert_eq!(inputs[2].to_string(), LOUDNESS);
}

#[test]
fn an_address_that_is_not_an_address_is_refused() {
    let malformed = DiscoverStagePayloadV1 {
        index_artifact_id: "/var/folders/index.json".to_owned(),
        ..payload()
    };
    assert!(inputs_for(&malformed).is_err());
}

/// Zero means "no opinion" on the wire, so a caller who does not care about
/// clip length does not have to know what the daemon would pick.
#[test]
fn an_unset_length_takes_the_daemon_default() {
    let parameters = parameters_of(&payload());
    assert_eq!(parameters, clipmill_discovery::Parameters::DEFAULT);

    let zeroed = DiscoverStagePayloadV1 {
        duration: Some(ClipDurationV1 {
            min_ticks: 0,
            max_ticks: 0,
        }),
        ..payload()
    };
    assert_eq!(
        parameters_of(&zeroed),
        clipmill_discovery::Parameters::DEFAULT
    );
}

#[test]
fn a_stated_length_is_honoured_exactly() {
    let asked = DiscoverStagePayloadV1 {
        duration: Some(ClipDurationV1 {
            min_ticks: 30 * 90_000,
            max_ticks: 60 * 90_000,
        }),
        exploration_floor: 5,
        ..payload()
    };
    let parameters = parameters_of(&asked);
    assert_eq!(parameters.min_ticks, 30 * 90_000);
    assert_eq!(parameters.max_ticks, 60 * 90_000);
    assert_eq!(parameters.exploration_floor, 5);
}

/// One half stated and the other left alone is a real request: a caller who
/// wants clips no shorter than thirty seconds should not have to name a
/// ceiling as well.
#[test]
fn half_a_length_leaves_the_other_half_at_the_default() {
    let asked = DiscoverStagePayloadV1 {
        duration: Some(ClipDurationV1 {
            min_ticks: 30 * 90_000,
            max_ticks: 0,
        }),
        ..payload()
    };
    let parameters = parameters_of(&asked);
    assert_eq!(parameters.min_ticks, 30 * 90_000);
    assert_eq!(
        parameters.max_ticks,
        clipmill_discovery::Parameters::DEFAULT.max_ticks
    );
}
