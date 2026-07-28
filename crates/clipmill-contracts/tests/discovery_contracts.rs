//! The candidate contract, Rust leg.
//!
//! Everything downstream — ranking, judging, the edit director — consumes this
//! one object, so what these assert is the shape those stages code against and
//! the three promises they are allowed to rely on without rechecking: a
//! candidate can be explained, its boundaries are legal, and it is grouped.
//!
//! Tests may panic; the workspace deny targets production code.
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeSet;

use clipmill_contracts::proto::ipc::v1::{
    ClipDurationV1, DiscoverCandidatesPayloadV1, DiscoverStagePayloadV1,
};
use clipmill_contracts::schemas::discovery_candidates::DiscoveryCandidates;
use prost::Message;

fn read(rel: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => panic!("cannot read {}: {err}", path.display()),
    }
}

fn canonical(value: &serde_json::Value) -> String {
    let mut text = serde_json::to_string_pretty(value).unwrap_or_else(|err| panic!("{err}"));
    text.push('\n');
    text
}

fn roundtrip(rel: &str) -> DiscoveryCandidates {
    let raw = read(rel);
    let parsed: DiscoveryCandidates = match serde_json::from_str(&raw) {
        Ok(parsed) => parsed,
        Err(err) => panic!("valid fixture {rel} rejected: {err}"),
    };
    let reserialized =
        serde_json::to_value(&parsed).unwrap_or_else(|err| panic!("reserialize {rel}: {err}"));
    assert_eq!(
        canonical(&reserialized),
        raw,
        "canonical round-trip must be byte-identical for {rel}"
    );
    parsed
}

#[test]
fn every_valid_candidate_fixture_roundtrips_canonically() {
    roundtrip("contracts/fixtures/discovery.candidates/valid/interview.json");
    roundtrip("contracts/fixtures/discovery.candidates/valid/ten_words.json");
}

/// Discovery guarantees ranking never has to search. That guarantee is the
/// lattice, and a lattice with one point on each side would be discovery having
/// chosen — with less information than the stage that lives with the choice.
#[test]
fn a_candidate_arrives_with_boundaries_ranking_can_search() {
    let found = roundtrip("contracts/fixtures/discovery.candidates/valid/interview.json");
    assert!(!found.candidates.is_empty());
    assert!(
        found.candidates.iter().any(|candidate| {
            candidate.boundary_lattice.starts.len() > 1 || candidate.boundary_lattice.ends.len() > 1
        }),
        "no candidate offers an alternative boundary"
    );
    for candidate in &found.candidates {
        assert!(!candidate.boundary_lattice.starts.is_empty());
        assert!(!candidate.boundary_lattice.ends.is_empty());
    }
}

/// Rule 14.1, at the contract level: the type cannot express an unexplained
/// candidate, and the fixture does not contain one.
#[test]
fn no_candidate_can_be_published_without_evidence() {
    let found = roundtrip("contracts/fixtures/discovery.candidates/valid/interview.json");
    for candidate in &found.candidates {
        assert!(!candidate.evidence.is_empty());
    }
}

/// Every candidate in exactly one cluster, every cluster naming one of its own
/// members as representative. A diversity decision the interface cannot show is
/// a decision nobody can argue with.
#[test]
fn the_clusters_partition_the_candidates() {
    let found = roundtrip("contracts/fixtures/discovery.candidates/valid/interview.json");
    let ids = found
        .candidates
        .iter()
        .map(|candidate| candidate.id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        ids.len(),
        found.candidates.len(),
        "an id is published twice"
    );

    let mut clustered = BTreeSet::new();
    for cluster in &found.clusters {
        assert!(
            cluster
                .members
                .iter()
                .any(|member| member.as_str() == cluster.representative.as_str()),
            "a cluster's representative is not one of its members"
        );
        for member in &cluster.members {
            assert!(ids.contains(member.as_str()), "a cluster names a stranger");
            assert!(
                clustered.insert(member.as_str()),
                "a candidate is in two clusters"
            );
        }
    }
    assert_eq!(
        clustered.len(),
        ids.len(),
        "a candidate belongs to no cluster"
    );
}

/// The rubric is where the honesty lives. A proposer whose rubric repeated its
/// name would be telling a reader nothing about what it actually measured.
#[test]
fn every_proposer_names_the_approximation_it_made() {
    let found = roundtrip("contracts/fixtures/discovery.candidates/valid/interview.json");
    assert_eq!(found.proposers.len(), 3);
    for run in &found.proposers {
        assert_ne!(run.proposer.rubric.as_str(), run.proposer.name.as_str());
        assert!(run.candidates <= run.seeds);
    }
}

/// A recording with no clip long enough is a real answer, and a different one
/// from a recording nobody searched.
#[test]
fn a_recording_too_short_to_clip_still_publishes_its_search() {
    let found = roundtrip("contracts/fixtures/discovery.candidates/valid/ten_words.json");
    assert!(found.candidates.is_empty());
    assert!(found.clusters.is_empty());
    // The mesh still reports: every proposer ran, and the ones that nominated
    // something say so even though nothing survived expansion.
    assert_eq!(found.proposers.len(), 3);
    assert!(found.coverage.analyzed, "the recording was searched");
    assert!(found.proposers.iter().any(|run| run.seeds > 0));
}

#[test]
fn invalid_candidate_fixtures_are_rejected() {
    // The three empty-array fixtures belong to the schema and the Python leg:
    // typify enforces string patterns, enums, and required fields through
    // newtypes, but carries `minItems` as documentation. Asserting them here
    // would be testing a claim this type does not make.
    for (fixture, why) in [
        ("float_ticks", "float ticks must not parse (D06)"),
        ("unnamed_rubric", "a proposer must name its approximation"),
        (
            "counted_identity",
            "a counted id would be renamed by a reordering",
        ),
        (
            "unknown_phi_reason",
            "an unlisted rejection reason must not parse",
        ),
    ] {
        let rejected = serde_json::from_str::<DiscoveryCandidates>(&read(&format!(
            "contracts/fixtures/discovery.candidates/invalid/{fixture}.json"
        )));
        assert!(rejected.is_err(), "{why}");
    }
}

/// The stage payload carries the search parameters and no inputs: the three
/// documents arrive on the lease, so a search run inside an analysis encodes the
/// same bytes as the same search run on its own. A path here would give the same
/// recording two candidate sets on two machines.
#[test]
fn the_discovery_stage_payload_carries_nothing_machine_specific() {
    let message = DiscoverStagePayloadV1 {
        key_version: "clipmill.discover-stage.v1".to_owned(),
        stage: "discover-candidates".to_owned(),
        duration: None,
        exploration_floor: 0,
    };
    let decoded =
        DiscoverStagePayloadV1::decode(message.encode_to_vec().as_slice()).expect("round-trip");
    assert_eq!(decoded, message);
    let encoded = String::from_utf8_lossy(&message.encode_to_vec()).into_owned();
    assert!(!encoded.contains('/'), "the keyed payload carries a path");
    assert!(
        !encoded.contains("sha256:"),
        "an address in the payload would be present on one route and absent on \
         the other, which is one observation with two keys"
    );

    // A different clip length is a different search, so it must change the key.
    let longer = DiscoverStagePayloadV1 {
        duration: Some(ClipDurationV1 {
            min_ticks: 30 * 90_000,
            max_ticks: 60 * 90_000,
        }),
        ..message.clone()
    };
    assert_ne!(message.encode_to_vec(), longer.encode_to_vec());
}

/// Zero means "no opinion" on the wire, so a caller who does not care about
/// clip length does not have to know what the daemon would pick.
#[test]
fn a_discovery_request_with_no_opinion_round_trips_empty() {
    let message = DiscoverCandidatesPayloadV1 {
        key_version: "clipmill.discover-candidates.v1".to_owned(),
        source_id: "src_0123456789abcdef".to_owned(),
        duration: None,
    };
    let decoded = DiscoverCandidatesPayloadV1::decode(message.encode_to_vec().as_slice())
        .expect("round-trip");
    assert_eq!(decoded, message);
    assert!(decoded.duration.is_none());
}
