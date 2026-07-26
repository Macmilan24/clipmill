//! The speech chain's contracts, Rust leg.
//!
//! Parsing is the least of it. What these assert is the observation contract
//! itself (book ch. 13): that timing has a stated authority, that a pass which
//! ran and found nothing is distinguishable from one that never ran, and that
//! a word whose timing was guessed says so and has its span declared invalid.
//! Those are the properties every later stage will lean on without checking,
//! so they are checked once, here.
//!
//! Tests may panic; the workspace deny targets production code.
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use clipmill_contracts::schemas::{
    speech_alignment::SpeechAlignment, speech_asr::SpeechAsr, speech_transcript::SpeechTranscript,
    speech_vad::SpeechVad,
};

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

fn roundtrip<T>(rel: &str) -> T
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let raw = read(rel);
    let parsed: T = match serde_json::from_str(&raw) {
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
fn every_valid_speech_fixture_roundtrips_canonically() {
    roundtrip::<SpeechVad>("contracts/fixtures/speech.vad/valid/two_utterances.json");
    roundtrip::<SpeechVad>("contracts/fixtures/speech.vad/valid/analyzed_and_silent.json");
    roundtrip::<SpeechVad>("contracts/fixtures/speech.vad/valid/never_examined.json");
    roundtrip::<SpeechAsr>("contracts/fixtures/speech.asr/valid/two_utterances.json");
    roundtrip::<SpeechAsr>("contracts/fixtures/speech.asr/valid/one_segment_undecodable.json");
    roundtrip::<SpeechAlignment>("contracts/fixtures/speech.alignment/valid/ten_words.json");
    roundtrip::<SpeechAlignment>(
        "contracts/fixtures/speech.alignment/valid/one_segment_unaligned.json",
    );
    roundtrip::<SpeechTranscript>("contracts/fixtures/speech.transcript/valid/ten_words.json");
    roundtrip::<SpeechTranscript>(
        "contracts/fixtures/speech.transcript/valid/interpolated_timing.json",
    );
}

/// "A skipped pass, a below-threshold result, and a genuinely empty result are
/// three different facts." Two of them look identical if you only read the
/// segment list, which is exactly why coverage is not optional.
#[test]
fn silence_and_never_having_looked_are_different_documents() {
    let silent: SpeechVad =
        roundtrip("contracts/fixtures/speech.vad/valid/analyzed_and_silent.json");
    let unexamined: SpeechVad =
        roundtrip("contracts/fixtures/speech.vad/valid/never_examined.json");

    assert!(silent.segments.is_empty());
    assert!(unexamined.segments.is_empty());
    // Same emptiness, opposite meanings.
    assert!(silent.coverage.analyzed);
    assert!(!unexamined.coverage.analyzed);
    // The one that looked can say where the quiet was; the one that did not
    // declares the whole span unexamined instead of implying it was quiet.
    assert!(!silent.silences.is_empty());
    assert!(unexamined.silences.is_empty());
    assert!(!unexamined.invalid_regions.is_empty());
}

/// The recognizer's intervals are bookkeeping. If this constant were ever
/// widened, every word-snapped trim in the product would quietly start
/// trusting token positions.
#[test]
fn recognizer_output_names_alignment_as_the_timing_authority() {
    let asr: SpeechAsr = roundtrip("contracts/fixtures/speech.asr/valid/two_utterances.json");
    assert_eq!(asr.timing_authority, serde_json::json!("forced_alignment"));
    // Greedy at temperature zero is what makes a cached transcript worth
    // caching: the same audio must decode to the same text.
    assert!(matches!(
        asr.decoding.strategy,
        clipmill_contracts::schemas::speech_asr::SpeechAsrDecodingStrategy::Greedy
    ));
    assert!(asr.decoding.temperature.abs() < f64::EPSILON);
    assert!(!asr.decoding.conditioned_on_previous);
}

/// A `const` reaches Rust as an unvalidated `serde_json::Value`: typify turns
/// string patterns and enums into newtypes that refuse bad input, but a fixed
/// value is carried, not checked. That is a real gap on a field whose whole
/// job is to stop anything downstream from trusting decoder positions, so it
/// is pinned here rather than assumed away — the schema and the Python leg
/// refuse the document, and every Rust reader of this artifact has to check
/// the field itself.
#[test]
fn a_const_is_carried_by_the_rust_type_but_not_enforced_by_it() {
    let smuggled: SpeechAsr = serde_json::from_str(&read(
        "contracts/fixtures/speech.asr/invalid/decoder_claimed_as_timing_authority.json",
    ))
    .expect("typify does not enforce const, so this parses");
    assert_eq!(smuggled.timing_authority, serde_json::json!("decoder_hint"));
}

/// Speech that could not be decoded is a hole in the transcript, not silence.
/// A boundary optimizer that mistook one for the other would cut mid-sentence.
#[test]
fn undecodable_speech_is_declared_rather_than_omitted() {
    let partial: SpeechAsr =
        roundtrip("contracts/fixtures/speech.asr/valid/one_segment_undecodable.json");
    assert_eq!(partial.segments.len(), 1);
    assert_eq!(partial.invalid_regions.len(), 1);
    assert!(partial.coverage.analyzed);
    // The span it failed on is still speech, and is still named.
    assert!(partial.coverage.decoded_ticks < 675_000);
}

/// Word edges are only as precise as the stride that measured them, and every
/// edge must actually be on that grid — a "measurement" off the grid is an
/// interpolation wearing a measurement's label.
#[test]
fn word_edges_land_on_the_stride_that_measured_them() {
    let alignment: SpeechAlignment =
        roundtrip("contracts/fixtures/speech.alignment/valid/ten_words.json");
    // The generated type already refuses a zero stride, so the only thing left
    // to check is that the edges honour it.
    let frame = alignment.frame_ticks.get();
    for word in &alignment.words {
        assert_eq!(
            word.start_ticks % frame,
            0,
            "{} starts off-grid",
            *word.text
        );
        assert_eq!(word.end_ticks % frame, 0, "{} ends off-grid", *word.text);
        assert!(word.end_ticks > word.start_ticks);
    }
    // Ordered, and never overlapping: a trim that snaps to a word boundary
    // needs a total order to snap within.
    for pair in alignment.words.windows(2) {
        assert!(pair[1].start_ticks >= pair[0].end_ticks);
    }
}

#[test]
fn text_the_aligner_could_not_place_is_kept_and_named() {
    let partial: SpeechAlignment =
        roundtrip("contracts/fixtures/speech.alignment/valid/one_segment_unaligned.json");
    assert!(partial.words.iter().all(|word| word.segment_index == 0));
    assert_eq!(partial.unaligned.len(), 1);
    // Losing the words would lose what was said; publishing them with invented
    // timing would be worse. It is carried, and the span is declared invalid.
    assert!(!partial.unaligned[0].text.is_empty());
    assert_eq!(partial.invalid_regions.len(), 1);

    let rejected = serde_json::from_str::<SpeechAlignment>(&read(
        "contracts/fixtures/speech.alignment/invalid/unaligned_without_reason.json",
    ));
    assert!(rejected.is_err(), "unaligned text must say why");
}

/// The assembled observation is what every later stage actually reads, so the
/// invariants that make it trustworthy are asserted against it directly rather
/// than inferred from the three artifacts behind it.
#[test]
fn the_assembled_transcript_carries_what_an_observation_must() {
    use clipmill_contracts::schemas::speech_transcript::WordTiming;

    let transcript: SpeechTranscript =
        roundtrip("contracts/fixtures/speech.transcript/valid/ten_words.json");

    // Every model that touched this is named, with its digest.
    let stages = transcript
        .producers
        .iter()
        .map(|producer| producer.stage.to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        stages,
        [
            "speech-vad",
            "speech-asr",
            "speech-align",
            "speech-transcript"
        ]
    );
    assert_eq!(
        transcript
            .producers
            .iter()
            .filter(|producer| producer.model_digest.is_some())
            .count(),
        3,
        "each model-running stage must carry its model's digest"
    );

    // A distribution, not a scalar, and the low quantile is the lower one.
    assert!(transcript.confidence.p10 <= transcript.confidence.p50);

    // Segments index into the word list rather than restating it, so the two
    // cannot drift; check the indices actually resolve.
    for segment in &transcript.segments {
        let first = usize::try_from(segment.first_word_index).unwrap();
        let count = usize::try_from(segment.word_count).unwrap();
        assert!(count > 0);
        let members = &transcript.words[first..first + count];
        assert!(
            members
                .iter()
                .all(|word| word.segment_index == segment.index)
        );
        assert_eq!(segment.start_ticks, members[0].start_ticks);
        assert_eq!(segment.end_ticks, members[members.len() - 1].end_ticks);
    }

    assert!(
        transcript
            .words
            .iter()
            .all(|word| matches!(word.timing, WordTiming::Aligned))
    );
    assert!(transcript.invalid_regions.is_empty());
    assert!(transcript.coverage.aligned_ticks <= transcript.coverage.speech_ticks);
}

/// The case the `timing` field exists for. Interpolated words are usable —
/// captions still need to show them — but nothing may cut inside their span,
/// and the only way a later stage can know that is if the span is declared.
#[test]
fn interpolated_timing_is_labelled_and_its_span_declared_invalid() {
    use clipmill_contracts::schemas::speech_transcript::{InvalidRegionReason, WordTiming};

    let transcript: SpeechTranscript =
        roundtrip("contracts/fixtures/speech.transcript/valid/interpolated_timing.json");

    let guessed = transcript
        .words
        .iter()
        .filter(|word| matches!(word.timing, WordTiming::Interpolated))
        .collect::<Vec<_>>();
    assert!(!guessed.is_empty());
    assert!(transcript.coverage.aligned_ticks < transcript.coverage.speech_ticks);

    for word in guessed {
        let covered = transcript.invalid_regions.iter().any(|region| {
            matches!(region.reason, InvalidRegionReason::TimingInterpolated)
                && region.start_ticks <= word.start_ticks
                && word.end_ticks <= region.end_ticks
        });
        assert!(
            covered,
            "{} has guessed timing that no invalid region declares",
            *word.text
        );
    }
}

#[test]
fn invalid_speech_fixtures_are_rejected_at_the_type_level() {
    for (fixture, why) in [
        (
            "speech.vad/invalid/scalar_confidence.json",
            "a bare scalar is not a confidence distribution",
        ),
        (
            "speech.vad/invalid/float_seconds.json",
            "float seconds are not integer ticks",
        ),
        (
            "speech.asr/invalid/unknown_decoding_strategy.json",
            "an unlisted decoding strategy must not parse",
        ),
        (
            "speech.alignment/invalid/word_without_timing.json",
            "a word with no interval is not an alignment",
        ),
        (
            "speech.transcript/invalid/word_timing_unlabelled.json",
            "timing provenance is not optional",
        ),
        (
            "speech.transcript/invalid/unknown_timing_provenance.json",
            "'guessed' is not one of the two things timing can be",
        ),
        (
            "speech.transcript/invalid/no_producers.json",
            "an observation with no producer cannot be audited",
        ),
    ] {
        let raw = read(&format!("contracts/fixtures/{fixture}"));
        let rejected = if fixture.starts_with("speech.vad/") {
            serde_json::from_str::<SpeechVad>(&raw).err().is_some()
        } else if fixture.starts_with("speech.asr/") {
            serde_json::from_str::<SpeechAsr>(&raw).err().is_some()
        } else if fixture.starts_with("speech.alignment/") {
            serde_json::from_str::<SpeechAlignment>(&raw)
                .err()
                .is_some()
        } else {
            serde_json::from_str::<SpeechTranscript>(&raw)
                .err()
                .is_some()
        };
        assert!(rejected, "{fixture}: {why}");
    }
}
