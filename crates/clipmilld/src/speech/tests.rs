//! Assembly, against the published fixtures.
//!
//! The four speech fixtures were authored as one story — one recording seen by
//! four stages — so assembling the first three must reproduce the fourth. That
//! makes the fixture set an executable specification rather than four
//! documents that happen to look consistent, and it is why the interesting
//! assertions below are about what happens when they are made inconsistent.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use clipmill_contracts::schemas::{
    speech_alignment::SpeechAlignment, speech_asr::SpeechAsr, speech_transcript as transcript,
    speech_vad::SpeechVad,
};

use super::{AssemblyError, Inputs, assemble};

const VAD_ID: &str = "sha256:5ead000000000000000000000000000000000000000000000000000000000001";
const ASR_ID: &str = "sha256:a52a000000000000000000000000000000000000000000000000000000000002";
const ALIGN_ID: &str = "sha256:a11a000000000000000000000000000000000000000000000000000000000003";
const ASSEMBLER: &str = "clipmill-transcript-assembly@1.0.0";

fn fixture(relative: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/fixtures")
        .join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

fn activity() -> SpeechVad {
    serde_json::from_str(&fixture("speech.vad/valid/two_utterances.json")).expect("valid fixture")
}

fn recognized() -> SpeechAsr {
    serde_json::from_str(&fixture("speech.asr/valid/two_utterances.json")).expect("valid fixture")
}

fn alignment() -> SpeechAlignment {
    serde_json::from_str(&fixture("speech.alignment/valid/ten_words.json")).expect("valid fixture")
}

fn inputs() -> Inputs<'static> {
    Inputs {
        vad: VAD_ID,
        asr: ASR_ID,
        alignment: ALIGN_ID,
    }
}

fn canonical(document: &transcript::SpeechTranscript) -> String {
    let value = serde_json::to_value(document).expect("serializes");
    let mut text = serde_json::to_string_pretty(&value).expect("pretty");
    text.push('\n');
    text
}

/// The load-bearing test: the three published inputs assemble into exactly the
/// published output, byte for byte.
#[test]
fn the_three_inputs_assemble_into_the_published_transcript() {
    let assembled = assemble(
        &activity(),
        &recognized(),
        &alignment(),
        inputs(),
        ASSEMBLER,
    )
    .expect("the fixtures describe one recording");
    assert_eq!(
        canonical(&assembled.document),
        fixture("speech.transcript/valid/ten_words.json")
    );
}

/// Every model that touched the transcript is named in it, with its digest.
/// A render manifest's AI-use disclosure is built from this list, so a stage
/// that went missing here would go missing from what a creator publishes.
#[test]
fn the_transcript_names_every_stage_that_produced_it() {
    let assembled = assemble(
        &activity(),
        &recognized(),
        &alignment(),
        inputs(),
        ASSEMBLER,
    )
    .expect("assembles");
    let stages = assembled
        .document
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
    // The three that ran a model carry its digest; assembly, which runs none,
    // does not pretend to.
    assert_eq!(
        assembled
            .document
            .producers
            .iter()
            .filter(|producer| producer.model_digest.is_some())
            .count(),
        3
    );
    assert!(assembled.document.producers[3].model_digest.is_none());
}

/// The check the generated type cannot make. `timing_authority` is a JSON
/// Schema `const`, and typify carries constants without validating them — so
/// a recognizer claiming its own token positions are authoritative parses
/// fine, and has to be refused here or nowhere.
#[test]
fn a_recognizer_claiming_its_own_timing_cannot_be_assembled() {
    let mut smuggled = recognized();
    smuggled.timing_authority = serde_json::json!("decoder_hint");
    let refused = assemble(&activity(), &smuggled, &alignment(), inputs(), ASSEMBLER);
    assert!(matches!(
        refused,
        Err(AssemblyError::TimingAuthority { .. })
    ));
}

/// A pass that never ran is not a recording with nothing in it. Assembling one
/// would publish an empty transcript that reads as "nobody spoke".
#[test]
fn a_stage_that_never_analyzed_the_audio_is_refused() {
    let mut unexamined = activity();
    unexamined.coverage.analyzed = false;
    assert!(matches!(
        assemble(
            &unexamined,
            &recognized(),
            &alignment(),
            inputs(),
            ASSEMBLER
        ),
        Err(AssemblyError::NotAnalyzed { .. })
    ));

    let mut undecoded = recognized();
    undecoded.coverage.analyzed = false;
    assert!(matches!(
        assemble(&activity(), &undecoded, &alignment(), inputs(), ASSEMBLER),
        Err(AssemblyError::NotAnalyzed { .. })
    ));
}

/// Three artifacts about three different recordings would fuse into a document
/// whose every claim was about a different file.
#[test]
fn inputs_describing_different_sources_are_refused() {
    let mut elsewhere = alignment();
    elsewhere.source_fingerprint = format!("sha256:{}", "f".repeat(64))
        .parse()
        .expect("a well-formed digest");
    assert!(matches!(
        assemble(&activity(), &recognized(), &elsewhere, inputs(), ASSEMBLER),
        Err(AssemblyError::MismatchedSources)
    ));
}

#[test]
fn alignment_referring_to_an_utterance_nobody_recognized_is_refused() {
    let mut stray = alignment();
    stray.words[0].segment_index = 99;
    assert!(matches!(
        assemble(&activity(), &recognized(), &stray, inputs(), ASSEMBLER),
        Err(AssemblyError::UnknownSegment { segment: 99 })
    ));
}

/// The decision this module exists for. A word the aligner would not place is
/// carried with spread timing, labelled, and its span declared invalid — so
/// the boundary optimizer refuses to cut inside it and nothing downstream can
/// mistake the guess for a measurement.
#[test]
fn an_unplaced_utterance_is_carried_with_spread_timing_and_declared_invalid() {
    let partial: SpeechAlignment = serde_json::from_str(&fixture(
        "speech.alignment/valid/one_segment_unaligned.json",
    ))
    .expect("valid fixture");
    let assembled =
        assemble(&activity(), &recognized(), &partial, inputs(), ASSEMBLER).expect("assembles");

    let spread = assembled
        .document
        .words
        .iter()
        .filter(|word| matches!(word.timing, transcript::WordTiming::Interpolated))
        .collect::<Vec<_>>();
    assert!(!spread.is_empty(), "the words were kept, not dropped");
    // Every one of them is inside a span the document says it does not vouch
    // for.
    for word in &spread {
        assert!(
            assembled.document.invalid_regions.iter().any(|region| {
                region.start_ticks <= word.start_ticks && word.end_ticks <= region.end_ticks
            }),
            "{} is spread timing that nothing declares",
            *word.text
        );
    }
    // And the coverage says how much of the speech actually has measured
    // timing, which is the number a consumer checks before trusting a cut.
    assert!(assembled.document.coverage.aligned_ticks < assembled.document.coverage.speech_ticks);
}

/// A single out-of-vocabulary word goes back between its neighbours rather
/// than at the end, or the transcript's word order stops matching what was
/// said.
#[test]
fn a_single_unplaced_word_lands_between_the_words_around_it() {
    let mut recognized = recognized();
    // "is" becomes a numeral, which the CTC alphabet cannot spell.
    recognized.segments[1].text = "Every timestamp 101 an integer tick.".to_owned();
    let mut alignment = alignment();
    alignment.words.retain(|word| *word.text != "is");
    for (position, word) in alignment.words.iter_mut().enumerate() {
        word.index = u64::try_from(position).expect("fits");
    }
    alignment.unaligned.push(
        clipmill_contracts::schemas::speech_alignment::UnalignedSpan {
            segment_index: 1,
            word_index: Some(2),
            text: "101".to_owned(),
            reason:
                clipmill_contracts::schemas::speech_alignment::UnalignedSpanReason::OutOfVocabulary,
            detail: None,
        },
    );

    let assembled =
        assemble(&activity(), &recognized, &alignment, inputs(), ASSEMBLER).expect("assembles");
    let spoken = assembled
        .document
        .words
        .iter()
        .filter(|word| word.segment_index == 1)
        .map(|word| word.text.to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        spoken,
        ["Every", "timestamp", "101", "an", "integer", "tick"],
        "the replacement sits where the word it replaced was"
    );
}

/// The transcript's own confidence answers "is this text safe to quote", which
/// is a question about recognition. Word confidence in this document is timing
/// confidence, so aggregating that instead would report a transcript as
/// worthless whenever alignment had a hard time.
#[test]
fn document_confidence_summarizes_recognition_rather_than_timing() {
    let assembled = assemble(
        &activity(),
        &recognized(),
        &alignment(),
        inputs(),
        ASSEMBLER,
    )
    .expect("assembles");
    let segment_medians = assembled
        .document
        .segments
        .iter()
        .map(|segment| segment.confidence.p50)
        .collect::<Vec<_>>();
    assert!(segment_medians.contains(&assembled.document.confidence.p50));
    assert!(assembled.document.confidence.p10 <= assembled.document.confidence.p50);
}

/// Segments index into the word list rather than restating it, so the two
/// cannot drift. That only holds if the indices actually resolve.
#[test]
fn segment_word_ranges_resolve_into_the_word_list() {
    let assembled = assemble(
        &activity(),
        &recognized(),
        &alignment(),
        inputs(),
        ASSEMBLER,
    )
    .expect("assembles");
    for segment in &assembled.document.segments {
        let first = usize::try_from(segment.first_word_index).expect("fits");
        let count = usize::try_from(segment.word_count).expect("fits");
        let members = &assembled.document.words[first..first + count];
        assert!(
            members
                .iter()
                .all(|word| word.segment_index == segment.index)
        );
        assert_eq!(segment.start_ticks, members[0].start_ticks);
        assert_eq!(segment.end_ticks, members[members.len() - 1].end_ticks);
    }
    // And every word is reachable from exactly one segment.
    let claimed: usize = assembled
        .document
        .segments
        .iter()
        .map(|segment| usize::try_from(segment.word_count).expect("fits"))
        .sum();
    assert_eq!(claimed, assembled.document.words.len());
}

/// The quantile rank is pinned rather than left to a language's rounding mode:
/// Rust rounds halves away from zero and Python rounds them to even, and the
/// workers compute this same summary over the same numbers.
#[test]
fn the_median_of_an_even_list_does_not_depend_on_the_rounding_mode() {
    // Half-up: the rank for p50 over two values is index one, not index zero.
    let (p50, p10) = super::distribution(&[0.1, 0.9]);
    assert!((p50 - 0.9).abs() < f64::EPSILON);
    assert!((p10 - 0.1).abs() < f64::EPSILON);
    let (p50, _) = super::distribution(&[0.2, 0.4, 0.6, 0.8]);
    assert!((p50 - 0.6).abs() < f64::EPSILON);
}
