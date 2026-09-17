//! Two presentations of one word list, kept as one.
//!
//! The reading cues and the burned-in cues are two groupings of the same
//! words. A correction belongs to the word and must land in both; a trim
//! removes words that no longer play from both; and the two may never read
//! differently under one word id. These run over the published
//! two-intents fixture, which carries both groupings and a gain curve.
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use clipmill_edit_ir::{EditCommand, EditDocument, Presentation, TICKS_PER_SECOND};

fn fixture() -> EditDocument {
    let raw = include_str!("../../../contracts/fixtures/edit_ir/valid/two_caption_intents.json");
    let mut document =
        EditDocument::from_canonical_json(raw.as_bytes()).expect("the fixture parses");
    // The fixture predates word ids; this is the migration every stored
    // document goes through.
    assert!(document.assign_word_ids());
    document.validate().expect("migrated fixture is valid");
    document
}

fn words(document: &EditDocument, presentation: Presentation) -> Vec<(String, i64)> {
    document
        .captions
        .list(presentation)
        .iter()
        .flat_map(clipmill_edit_ir::CaptionCue::words)
        .map(|word| (word.text.clone(), word.start_ticks))
        .collect()
}

fn word_id_of(document: &EditDocument, presentation: Presentation, text: &str) -> String {
    document
        .captions
        .list(presentation)
        .iter()
        .flat_map(clipmill_edit_ir::CaptionCue::words)
        .find(|word| word.text == text)
        .and_then(|word| word.word_id.clone())
        .expect("the word carries an id")
}

#[test]
fn a_word_gets_one_id_in_both_presentations_and_keeps_it() {
    let document = fixture();
    let reading = word_id_of(&document, Presentation::Reading, "timestamp");
    let burned = word_id_of(&document, Presentation::BurnIn, "timestamp");
    assert_eq!(reading, burned, "the same word carries the same id in each");
    // Assigning again renames nothing: an id is a name, and a correction
    // somebody addressed to it must still find it.
    let mut again = document.clone();
    assert!(!again.assign_word_ids());
    assert_eq!(again, document);
    assert!(document.words_are_identified());
}

#[test]
fn a_correction_addressed_to_the_word_lands_in_both_presentations() {
    let mut document = fixture();
    let word_id = word_id_of(&document, Presentation::Reading, "timestamp");
    let inverse = EditCommand::SetWordText {
        word_id: word_id.clone(),
        text: "timecode".to_owned(),
    }
    .apply(&mut document)
    .expect("the correction applies");
    assert_eq!(document.word_text(&word_id), Some("timecode"));
    assert!(
        words(&document, Presentation::Reading)
            .iter()
            .any(|(text, _)| text == "timecode")
    );
    assert!(
        words(&document, Presentation::BurnIn)
            .iter()
            .any(|(text, _)| text == "timecode")
    );
    assert!(
        !words(&document, Presentation::BurnIn)
            .iter()
            .any(|(text, _)| text == "timestamp")
    );

    // And back, in both.
    assert_eq!(
        inverse,
        EditCommand::SetWordText {
            word_id: word_id.clone(),
            text: "timestamp".to_owned(),
        }
    );
    inverse.apply(&mut document).expect("the inverse applies");
    assert_eq!(document, fixture());

    // A word nobody has is refused, and refused before anything moved.
    let untouched = document.clone();
    assert!(
        EditCommand::SetWordText {
            word_id: "w@nowhere".to_owned(),
            text: "x".to_owned(),
        }
        .apply(&mut document)
        .is_err()
    );
    assert_eq!(document, untouched);
}

#[test]
fn a_correction_addressed_to_the_burned_cue_on_screen_corrects_that_word_and_its_twin() {
    // The burned-in cue `hot_10` holds "timestamp" alone; the reading list
    // has no cue by that name, and its `cue_2` — which the old lookup would
    // have reached for any `cue_2` — holds different words. The command
    // names the presentation it saw, so the word corrected is the word shown.
    let mut document = fixture();
    let inverse = EditCommand::EditCaptionText {
        cue_id: "hot_10".to_owned(),
        word_index: 0,
        text: "timecode".to_owned(),
        presentation: Presentation::BurnIn,
    }
    .apply(&mut document)
    .expect("the burned cue is found in its own list");
    assert!(
        words(&document, Presentation::BurnIn)
            .iter()
            .any(|(text, _)| text == "timecode")
    );
    assert!(
        words(&document, Presentation::Reading)
            .iter()
            .any(|(text, _)| text == "timecode"),
        "the reading twin was corrected too"
    );
    inverse.apply(&mut document).expect("undo");
    assert_eq!(document, fixture());

    // Without the presentation named, `hot_10` is looked for among the
    // reading cues and is not there.
    assert!(
        EditCommand::EditCaptionText {
            cue_id: "hot_10".to_owned(),
            word_index: 0,
            text: "timecode".to_owned(),
            presentation: Presentation::Reading,
        }
        .apply(&mut document)
        .is_err()
    );
}

#[test]
fn a_document_without_word_ids_is_corrected_one_occurrence_at_a_time() {
    // Nothing links the twins, so nothing can find the other one; the
    // command does what it always did and no more.
    let raw = include_str!("../../../contracts/fixtures/edit_ir/valid/two_caption_intents.json");
    let mut document = EditDocument::from_canonical_json(raw.as_bytes()).expect("parses");
    EditCommand::EditCaptionText {
        cue_id: "cue_3".to_owned(),
        word_index: 1,
        text: "timecode".to_owned(),
        presentation: Presentation::Reading,
    }
    .apply(&mut document)
    .expect("applies");
    assert!(
        words(&document, Presentation::Reading)
            .iter()
            .any(|(text, _)| text == "timecode")
    );
    assert!(
        words(&document, Presentation::BurnIn)
            .iter()
            .any(|(text, _)| text == "timestamp")
    );
}

#[test]
fn the_two_presentations_may_not_read_differently_under_one_id() {
    let mut document = fixture();
    let word_id = word_id_of(&document, Presentation::Reading, "timestamp");
    for cue in &mut document.captions.burn_in {
        for word in cue.lines.iter_mut().flat_map(|line| line.words.iter_mut()) {
            if word.word_id.as_deref() == Some(word_id.as_str()) {
                "elsewhere".clone_into(&mut word.text);
            }
        }
    }
    let error = document
        .validate()
        .expect_err("a divergent word is refused");
    assert!(error.to_string().contains("reads differently"), "{error}");
}

#[test]
fn trimming_the_head_removes_the_opening_words_from_both_presentations() {
    // The first segment plays source 2s–6s; its captions begin at program
    // 0.2s with "the first slice". Advancing the in point by one second cuts
    // the first program second: "the" (0.2–0.53s) and "first" (0.53–0.87s)
    // are gone from both lists, "slice" (0.87–1.2s) is gone too because it
    // is touched, and what remains moves a second earlier.
    let mut document = fixture();
    let before_gain = document.audio.gain_curve.clone();
    let second = TICKS_PER_SECOND;
    let inverse = EditCommand::Trim {
        segment_id: "seg_open".to_owned(),
        in_ticks: 3 * second,
        out_ticks: 6 * second,
    }
    .apply(&mut document)
    .expect("the head trim applies");

    let fixture_words = |presentation| words(&fixture(), presentation).len();
    for presentation in [Presentation::Reading, Presentation::BurnIn] {
        let remaining = words(&document, presentation);
        assert_eq!(
            remaining.len(),
            fixture_words(presentation) - 3,
            "{}: the three opening words are gone: {remaining:?}",
            presentation.as_str()
        );
        let (first_text, first_start) = remaining.first().expect("words remain");
        assert_eq!(first_text, "renders", "{}", presentation.as_str());
        // "renders" began at program 1.2012s (108,108 ticks); a second earlier.
        assert_eq!(*first_start, 108_108 - second, "{}", presentation.as_str());
    }
    // The burned cue on screen ends inside the program that remains.
    let program = document.program_duration_ticks();
    assert!(
        document
            .captions
            .burned()
            .iter()
            .all(|cue| cue.end_ticks <= program),
        "no burned cue runs past the program's end"
    );
    // Gain automation moved with the words: the point at 0s was in the cut
    // second and is gone; the point at 3s is now at 2s.
    assert_eq!(before_gain.len(), 2);
    assert_eq!(document.audio.gain_curve.len(), 1);
    assert_eq!(document.audio.gain_curve[0].t_ticks, 270_000 - second);

    // Material was lost, so the inverse is the whole prior arrangement,
    // burned-in cues included — and it restores the fixture exactly.
    assert!(matches!(
        inverse,
        EditCommand::RestoreArrangement {
            burn_in: Some(_),
            ..
        }
    ));
    inverse.apply(&mut document).expect("undo");
    assert_eq!(document, fixture());
}

/// A cue cut through keeps its window on the side that was not cut.
///
/// The reading cues are held past their words so they can be read. A head
/// trim that lands inside a cue used to shrink what remained to its words'
/// own bounds — a caption that appeared a beat after the picture and left
/// the moment its last word ended, too brief to read and refused at export.
/// Now the cut side moves to the cut and the other side stays where it was.
#[test]
fn a_cue_cut_through_keeps_its_hold_on_the_side_that_was_not_cut() {
    let second = TICKS_PER_SECOND;
    // Head: the first segment now begins at source 2.6s, program 0.6s. "the"
    // and "first" are gone; "slice" (0.8675s–1.2012s) remains, and cue_1
    // starts with the picture and ends where it always did, a second's
    // worth of program earlier.
    let mut document = fixture();
    EditCommand::Trim {
        segment_id: "seg_open".to_owned(),
        in_ticks: 2 * second + second * 6 / 10,
        out_ticks: 6 * second,
    }
    .apply(&mut document)
    .expect("the head trim applies");
    let first = &document.captions.cues[0];
    assert_eq!(first.cue_id, "cue_1");
    assert_eq!(
        first
            .words()
            .map(|word| word.text.as_str())
            .collect::<Vec<_>>(),
        ["slice"]
    );
    assert_eq!(first.start_ticks, 0, "the cut cue starts with the picture");
    assert_eq!(first.end_ticks, 108_108 - second * 6 / 10);
    document.validate().expect("valid after the head trim");

    // Tail: the second segment now ends at source 11.5s, program 5.5s.
    // "preview" onward is gone; "so" remains, and cue_6 is held to the end
    // of the program rather than leaving the moment "so" ends.
    let mut document = fixture();
    EditCommand::Trim {
        segment_id: "seg_close".to_owned(),
        in_ticks: 10 * second,
        out_ticks: 11 * second + second / 2,
    }
    .apply(&mut document)
    .expect("the tail trim applies");
    let last = document.captions.cues.last().expect("cues remain");
    assert_eq!(last.cue_id, "cue_6");
    assert_eq!(
        last.words()
            .map(|word| word.text.as_str())
            .collect::<Vec<_>>(),
        ["so"]
    );
    assert_eq!(last.start_ticks, 468_468, "the start was not touched");
    assert_eq!(
        last.end_ticks,
        5 * second + second / 2,
        "held to the program's end"
    );
    assert_eq!(document.program_duration_ticks(), last.end_ticks);
    document.validate().expect("valid after the tail trim");
}

#[test]
fn trimming_the_tail_removes_the_closing_words_from_both_presentations() {
    // The second segment plays source 10s–12s, program 4s–6s, holding "so
    // preview and render agree" from program 5.2s. Ending it a second early
    // cuts everything from program 5s: those words are gone from both.
    let mut document = fixture();
    let second = TICKS_PER_SECOND;
    let inverse = EditCommand::Trim {
        segment_id: "seg_close".to_owned(),
        in_ticks: 10 * second,
        out_ticks: 11 * second,
    }
    .apply(&mut document)
    .expect("the tail trim applies");
    for presentation in [Presentation::Reading, Presentation::BurnIn] {
        let remaining = words(&document, presentation);
        assert!(
            !remaining.iter().any(|(text, _)| text == "agree"),
            "{}: the closing word is gone",
            presentation.as_str()
        );
        // Nothing before the cut moved.
        assert_eq!(
            remaining[0],
            ("the".to_owned(), 18_018),
            "{}",
            presentation.as_str()
        );
    }
    inverse.apply(&mut document).expect("undo");
    assert_eq!(document, fixture());
}

#[test]
fn a_restore_written_before_burned_cues_were_captured_leaves_them_as_they_stand() {
    let mut document = fixture();
    let burned_before = document.captions.burn_in.clone();
    let old_log = serde_json::json!({
        "op": "restore_arrangement",
        "segments": document.video.segments,
        "cues": [],
        "gain_curve": [],
    })
    .to_string();
    let restore = EditCommand::from_canonical_json(old_log.as_bytes()).expect("an old log line");
    restore.apply(&mut document).expect("applies");
    assert!(document.captions.cues.is_empty());
    assert_eq!(document.captions.burn_in, burned_before);
}
