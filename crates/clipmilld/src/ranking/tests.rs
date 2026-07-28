//! What the daemon settles before a ranked set gets an address.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::float_cmp)]

use clipmill_contracts::proto::ipc::v1::RankStagePayloadV1;
use prost::Message;

use super::{KIND_RANK, request_of};
use crate::jobs::RANK_STAGE_KEY_VERSION;

fn payload() -> RankStagePayloadV1 {
    RankStagePayloadV1 {
        key_version: RANK_STAGE_KEY_VERSION.to_owned(),
        stage: KIND_RANK.to_owned(),
        count: 0,
        diversity_milli: 0,
    }
}

/// Zero means "no opinion" on the wire, so a caller who does not care how many
/// clips to cut does not have to know what the daemon would pick.
#[test]
fn an_unset_request_takes_the_daemon_defaults() {
    assert_eq!(request_of(&payload()), clipmill_discovery::Request::DEFAULT);
}

#[test]
fn a_stated_request_is_honoured_exactly() {
    let asked = RankStagePayloadV1 {
        count: 4,
        diversity_milli: 250,
        ..payload()
    };
    let request = request_of(&asked);
    assert_eq!(request.count, 4);
    assert_eq!(request.diversity, 0.25);
}

/// Diversity travels as thousandths so the value reaching an artifact key is an
/// integer: a float would make two runs that agree to fifteen decimal places
/// disagree about a content address.
#[test]
fn diversity_is_carried_as_an_integer_and_clamped() {
    assert_eq!(
        request_of(&RankStagePayloadV1 {
            diversity_milli: 1_000,
            ..payload()
        })
        .diversity,
        1.0
    );
    assert_eq!(
        request_of(&RankStagePayloadV1 {
            diversity_milli: 9_999,
            ..payload()
        })
        .diversity,
        1.0,
        "a nonsensical value is clamped rather than producing a lambda above one"
    );
}

/// Asking for a different set is a different answer, not a filter over this
/// one, so both knobs must change the bytes the key is computed from.
#[test]
fn the_request_reaches_the_keyed_payload() {
    let default = payload().encode_to_vec();
    assert_ne!(
        default,
        RankStagePayloadV1 {
            count: 3,
            ..payload()
        }
        .encode_to_vec()
    );
    assert_ne!(
        default,
        RankStagePayloadV1 {
            diversity_milli: 500,
            ..payload()
        }
        .encode_to_vec()
    );
}

/// A payload from another stage decodes into this shape without complaint —
/// protobuf field numbers carry no meaning — so the stage name is checked.
#[test]
fn a_payload_from_another_stage_is_named_as_such() {
    let borrowed = RankStagePayloadV1 {
        stage: "discover-candidates".to_owned(),
        ..payload()
    };
    let decoded = RankStagePayloadV1::decode(borrowed.encode_to_vec().as_slice())
        .expect("it decodes, which is the point");
    assert_ne!(decoded.stage, KIND_RANK);
}

#[test]
fn the_keyed_payload_carries_no_path() {
    let text = String::from_utf8_lossy(&payload().encode_to_vec()).into_owned();
    assert!(!text.contains('/'), "a path reached the artifact key");
}
