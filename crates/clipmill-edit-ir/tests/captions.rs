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
    // Gain automation moved with the words, and the envelope the kept audio
    // had is the envelope it keeps: the ramp from 0 dB at 0s to 2 dB at 3s
    // stood at two thirds of a decibel one second in, so that is what the
    // new opening carries, rising to 2 dB at 2s. The point at 0s is gone with
    // the second it was in.
    assert_eq!(before_gain.len(), 2);
    let curve = &document.audio.gain_curve;
    assert_eq!(curve.len(), 2, "{curve:?}");
    assert_eq!(curve[0].t_ticks, 0);
    assert!((curve[0].gain_db - 2.0 / 3.0).abs() < 1e-9, "{curve:?}");
    assert_eq!(curve[1].t_ticks, 270_000 - second);
    assert!((curve[1].gain_db - 2.0).abs() < 1e-9);

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
/// the moment its last word ended. Now the cut side moves to the cut and
/// the other side stays where it was.
#[test]
fn a_cue_cut_through_keeps_its_hold_on_the_side_that_was_not_cut() {
    let second = TICKS_PER_SECOND;
    // The first segment now begins at source 2.3s, program 0.3s. "the" is
    // gone; "first slice" (0.5338s–1.2012s) remains, readable in the window
    // it keeps: cue_1 starts with the picture and ends where it always did,
    // three tenths of program earlier.
    let mut document = fixture();
    EditCommand::Trim {
        segment_id: "seg_open".to_owned(),
        in_ticks: 2 * second + second * 3 / 10,
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
        ["first", "slice"]
    );
    assert_eq!(first.start_ticks, 0, "the cut cue starts with the picture");
    assert_eq!(first.end_ticks, 108_108 - second * 3 / 10);
    assert_eq!(
        document.captions.cues.len(),
        6,
        "nothing was folded together"
    );
    document.validate().expect("valid after the head trim");
}

/// A cue the cut leaves too brief to read is folded into the cue beside it
/// and the pair is broken again, by the caption engine's own segmenter.
///
/// "slice" alone, on screen for six tenths of a second, is a caption no
/// profile calls readable and the export strip refuses for the sidecar.
/// The word is still said, so it is not dropped; it joins the next cue.
#[test]
fn a_fragment_too_brief_to_read_is_folded_into_the_next_cue() {
    let second = TICKS_PER_SECOND;
    let mut document = fixture();
    let inverse = EditCommand::Trim {
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
        ["slice", "renders", "from", "the", "edit", "document"],
        "the fragment and its neighbour are one cue"
    );
    // As every cue the engine makes, it begins with its first word.
    assert_eq!(first.start_ticks, first.words().next().unwrap().start_ticks);
    assert!(first.end_ticks > 108_108 - second * 6 / 10);
    assert_eq!(document.captions.cues.len(), 5, "one cue fewer");
    assert_eq!(document.captions.cues[1].cue_id, "cue_3");
    assert!(
        first.words().all(|word| word.word_id.is_some()),
        "every word keeps its identity"
    );
    document.validate().expect("valid after the fold");

    // The fold is part of the trim, so the undo is the whole arrangement
    // and it is exact.
    inverse.apply(&mut document).expect("undo");
    assert_eq!(document, fixture());
}

/// The same at the tail: "so" alone for three tenths of a second joins the
/// cue before it.
#[test]
fn a_fragment_too_brief_to_read_at_the_tail_is_folded_into_the_cue_before() {
    let second = TICKS_PER_SECOND;
    let mut document = fixture();
    EditCommand::Trim {
        segment_id: "seg_close".to_owned(),
        in_ticks: 10 * second,
        out_ticks: 11 * second + second / 2,
    }
    .apply(&mut document)
    .expect("the tail trim applies");
    let last = document.captions.cues.last().expect("cues remain");
    assert_eq!(
        last.words()
            .map(|word| word.text.as_str())
            .collect::<Vec<_>>(),
        ["and", "stored", "here", "so"]
    );
    assert_eq!(last.cue_id, "cue_5");
    assert!(last.end_ticks <= 5 * second + second / 2);
    assert_eq!(document.captions.cues.len(), 5);
    document.validate().expect("valid after the fold");
}

/// A camera-cut document whose edge cue can be reduced to the exact 8,700-tick
/// one-word fragment seen by the native whole-program trim gate.
fn edge_ripple_fixture() -> EditDocument {
    let mut document = fixture();
    let model = document.captions.cues[0].clone();
    document.captions.cues = [
        (
            "opening",
            0,
            108_700,
            vec![
                ("one", 0, 45_000),
                ("two", 45_000, 90_000),
                ("and", 100_000, 108_700),
            ],
        ),
        (
            "next",
            108_700,
            270_000,
            vec![
                ("that", 108_700, 150_000),
                ("is", 150_000, 180_000),
                ("enough", 180_000, 240_000),
            ],
        ),
        (
            "keep",
            280_000,
            405_000,
            vec![("Keep", 280_000, 310_000), ("this.", 310_000, 350_000)],
        ),
        (
            "closing",
            405_000,
            540_000,
            vec![
                ("and", 405_000, 413_700),
                ("final", 450_000, 490_000),
                ("words", 490_000, 530_000),
            ],
        ),
    ]
    .into_iter()
    .map(
        |(cue_id, start_ticks, end_ticks, tokens)| clipmill_edit_ir::CaptionCue {
            cue_id: cue_id.to_owned(),
            start_ticks,
            end_ticks,
            lines: vec![clipmill_edit_ir::CaptionLine {
                words: tokens
                    .into_iter()
                    .map(
                        |(text, start_ticks, end_ticks)| clipmill_edit_ir::CaptionWord {
                            text: text.to_owned(),
                            start_ticks,
                            end_ticks,
                            word_id: Some(format!("w@{start_ticks}")),
                        },
                    )
                    .collect(),
            }],
            ..model.clone()
        },
    )
    .collect();
    document
        .captions
        .burn_in
        .clone_from(&document.captions.cues);
    let first = document.video.segments[0].clone();
    let shots = [(0, 50_000), (50_000, 100_000), (100_000, 360_000)]
        .into_iter()
        .enumerate()
        .map(|(index, (start, end))| {
            let mut shot = first.clone();
            shot.segment_id = format!("shot_{index}");
            shot.in_ticks = first.in_ticks + start;
            shot.out_ticks = first.in_ticks + end;
            shot.layout.state = clipmill_edit_ir::LayoutState::Fit;
            shot.layout.crop_path.clear();
            shot.layout.secondary_crop_path.clear();
            shot
        });
    document.video.segments.splice(0..1, shots);
    document.validate().expect("multi-shot fragment fixture");
    document
}

fn assert_readable_edge(edge: &clipmill_edit_ir::CaptionCue, presentation: Presentation) {
    let profile = match presentation {
        Presentation::Reading => clipmill_captions::Profile::ACCESSIBILITY_EN,
        Presentation::BurnIn => clipmill_captions::Profile::BURN_IN_EN,
    };
    assert!(edge.end_ticks - edge.start_ticks >= profile.min_duration_ticks);
    let widths: Vec<_> = edge
        .lines
        .iter()
        .map(|line| {
            line.words
                .iter()
                .map(|word| word.text.chars().count())
                .sum::<usize>()
                + line.words.len().saturating_sub(1)
        })
        .collect();
    let violations = clipmill_captions::validate(
        &[clipmill_captions::CueFacts {
            cue_id: &edge.cue_id,
            start_ticks: edge.start_ticks,
            end_ticks: edge.end_ticks,
            speech_end_ticks: edge.words().map(|word| word.end_ticks).max(),
            lines: &widths,
        }],
        profile,
        &[],
    );
    assert!(
        !violations.iter().any(|violation| matches!(
            violation,
            clipmill_captions::Violation::TooBrief { .. }
                | clipmill_captions::Violation::ReadingRate { .. }
        )),
        "{presentation:?}: edge remains unreadable: {violations:?}"
    );
}

#[test]
fn edge_ripple_repairs_both_caption_presentations_and_undo_redo_is_exact() {
    for (start_ticks, end_ticks, edge_id, untouched_id) in [
        (0, 100_000, "opening", "keep"),
        (413_700, 540_000, "closing", "opening"),
    ] {
        let original = edge_ripple_fixture();
        let mut document = original.clone();
        let command = EditCommand::RippleDelete {
            start_ticks,
            end_ticks,
            reflow_edges: true,
        };
        let inverse = command.apply(&mut document).expect("edge ripple");
        for presentation in [Presentation::Reading, Presentation::BurnIn] {
            let before = original.captions.list(presentation);
            let after = document.captions.list(presentation);
            let edge = if start_ticks == 0 {
                after.first()
            } else {
                after.last()
            }
            .unwrap();
            assert!(
                edge.word_count() > 1,
                "{presentation:?}: the 0.0967s fragment must join its neighbour"
            );
            assert_readable_edge(edge, presentation);
            assert!(
                after
                    .iter()
                    .flat_map(clipmill_edit_ir::CaptionCue::words)
                    .all(|word| word.word_id.is_some())
            );
            let mut untouched = before
                .iter()
                .find(|cue| cue.cue_id == untouched_id)
                .unwrap()
                .clone();
            if start_ticks == 0 {
                untouched.start_ticks -= end_ticks;
                untouched.end_ticks -= end_ticks;
                for word in untouched.lines.iter_mut().flat_map(|line| &mut line.words) {
                    word.start_ticks -= end_ticks;
                    word.end_ticks -= end_ticks;
                }
            }
            assert_eq!(
                after.iter().find(|cue| cue.cue_id == untouched_id),
                Some(&untouched),
                "{edge_id}: unrelated user grouping is preserved"
            );
            let expected_words: Vec<_> = before
                .iter()
                .flat_map(clipmill_edit_ir::CaptionCue::words)
                .filter(|word| word.end_ticks <= start_ticks || word.start_ticks >= end_ticks)
                .map(|word| (&word.text, &word.word_id))
                .collect();
            let kept_words: Vec<_> = after
                .iter()
                .flat_map(clipmill_edit_ir::CaptionCue::words)
                .map(|word| (&word.text, &word.word_id))
                .collect();
            assert_eq!(
                kept_words, expected_words,
                "repair keeps every remaining word in order"
            );
        }
        document.validate().expect("valid repaired document");
        let repaired = document.clone();
        let redo = inverse.apply(&mut document).expect("undo edge ripple");
        assert_eq!(document, original);
        redo.apply(&mut document).expect("redo edge ripple");
        assert_eq!(document, repaired);
    }
}

#[test]
fn deleting_a_whole_edge_cue_preserves_the_next_grouping_even_if_brief() {
    let mut document = edge_ripple_fixture();
    // An intentionally short hand-grouped cue is not a fragment of this cut.
    for presentation in [Presentation::Reading, Presentation::BurnIn] {
        let cues = document.captions.list_mut(presentation);
        cues[1].lines[0].words.truncate(1);
        cues[1].end_ticks = 150_000;
    }
    let before = document.clone();
    EditCommand::RippleDelete {
        start_ticks: 0,
        end_ticks: 108_700,
        reflow_edges: true,
    }
    .apply(&mut document)
    .expect("remove entire opening cue");
    for presentation in [Presentation::Reading, Presentation::BurnIn] {
        let after = document.captions.list(presentation);
        assert_eq!(after.len(), before.captions.list(presentation).len() - 1);
        assert_eq!(after[0].cue_id, "next");
        assert_eq!(
            after[0].word_count(),
            1,
            "an unchanged cue is not automatically regrouped"
        );
    }
}

#[test]
fn legacy_ripple_json_preserves_its_shape_and_does_not_reflow_fragments() {
    for (start_ticks, end_ticks, head) in [(0, 100_000, true), (413_700, 540_000, false)] {
        let stored = serde_json::json!({
            "op": "ripple_delete",
            "start_ticks": start_ticks,
            "end_ticks": end_ticks,
        });
        let legacy: EditCommand = serde_json::from_value(stored.clone()).expect("old log command");
        assert_eq!(
            serde_json::to_value(&legacy).unwrap(),
            stored,
            "legacy command identity is unchanged"
        );
        let mut document = edge_ripple_fixture();
        let original = document.clone();
        let undo = legacy.apply(&mut document).expect("legacy replay");
        for presentation in [Presentation::Reading, Presentation::BurnIn] {
            let cues = document.captions.list(presentation);
            let edge = if head { cues.first() } else { cues.last() }.unwrap();
            assert_eq!(edge.word_count(), 1, "old ripple did not regroup captions");
            assert_eq!(edge.words().next().unwrap().text, "and");
            assert_eq!(edge.end_ticks - edge.start_ticks, 8_700);
        }
        let mut explicit_false = stored.clone();
        explicit_false["reflow_edges"] = serde_json::json!(false);
        let explicit: EditCommand = serde_json::from_value(explicit_false).unwrap();
        assert_eq!(serde_json::to_value(&explicit).unwrap(), stored);
        let mut replayed = original.clone();
        explicit
            .apply(&mut replayed)
            .expect("explicit legacy behavior");
        assert_eq!(
            replayed.to_canonical_json().unwrap(),
            document.to_canonical_json().unwrap()
        );
        undo.apply(&mut document).expect("undo legacy ripple");
        assert_eq!(document, original);
    }
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
