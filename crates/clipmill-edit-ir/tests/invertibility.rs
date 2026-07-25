//! The property the whole editing surface rests on: every command has an
//! exact inverse, and replaying a command log reconstructs the document
//! byte-for-byte. Undo that is merely approximate is undo that loses work.
//!
//! Randomised cases are driven by a fixed-seed generator rather than a
//! property-testing dependency: a failure here must be reproducible from the
//! seed printed in the assertion, not from a saved corpus file.
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use clipmill_edit_ir::{
    Asset, AudioTrack, CaptionAnimation, CaptionCue, CaptionLine, CaptionRegion, CaptionTrack,
    CaptionWord, CropKeyframe, CropRect, EditCommand, EditDocument, GainPoint, Layout, LayoutState,
    Rationale, VideoSegment, VideoTrack,
};

const FINGERPRINT: &str = "sha256:1111111111111111111111111111111111111111111111111111111111111111";
/// One 30000/1001 frame, the awkward interval float pipelines round wrong.
const FRAME_TICKS: i64 = 3003;

/// A small deterministic generator. Reproducing a failure needs only its seed.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 11
    }

    fn below(&mut self, bound: u64) -> u64 {
        if bound == 0 { 0 } else { self.next() % bound }
    }
}

fn word(text: &str, start: i64, end: i64) -> CaptionWord {
    CaptionWord {
        text: text.to_owned(),
        start_ticks: start,
        end_ticks: end,
    }
}

/// A document with several segments, cues, crop keyframes, and gain points —
/// enough structure that a command can plausibly disturb something it should
/// not.
fn sample_document() -> EditDocument {
    let segment = |id: &str, in_ticks: i64, out_ticks: i64, keyframes: Vec<i64>| VideoSegment {
        segment_id: id.to_owned(),
        source_fingerprint: FINGERPRINT.to_owned(),
        in_ticks,
        out_ticks,
        layout: Layout {
            state: LayoutState::SpeakerFill,
            crop_path: keyframes
                .into_iter()
                .map(|t_ticks| CropKeyframe {
                    t_ticks,
                    rect: CropRect {
                        x: 100,
                        y: 0,
                        width: 608,
                        height: 1080,
                    },
                })
                .collect(),
        },
    };
    let cue = |id: &str, start: i64, words: Vec<CaptionWord>| CaptionCue {
        cue_id: id.to_owned(),
        start_ticks: start,
        end_ticks: words.last().map_or(start + 1, |word| word.end_ticks),
        region: CaptionRegion::LowerSafe,
        anim: CaptionAnimation::Karaoke,
        lines: vec![CaptionLine { words }],
    };
    EditDocument {
        video: VideoTrack {
            segments: vec![
                segment("seg_a", 0, 90_000, vec![0, 45_000, 90_000]),
                segment("seg_b", 180_000, 270_000, vec![0, 90_000]),
                segment("seg_c", 900_000, 990_000, Vec::new()),
            ],
        },
        captions: CaptionTrack {
            style_ref: "clean".to_owned(),
            cues: vec![
                cue(
                    "cue_1",
                    0,
                    vec![
                        word("the", 0, 20_000),
                        word("whole", 20_000, 45_000),
                        word("point", 45_000, 80_000),
                    ],
                ),
                cue(
                    "cue_2",
                    100_000,
                    vec![
                        word("is", 100_000, 120_000),
                        word("exactness", 120_000, 175_000),
                    ],
                ),
                cue(
                    "cue_3",
                    200_000,
                    vec![
                        word("here", 200_000, 240_000),
                        word("too", 240_000, 265_000),
                    ],
                ),
            ],
        },
        audio: AudioTrack {
            target_lufs: -14.0,
            true_peak_dbtp: -1.0,
            gain_curve: vec![
                GainPoint {
                    t_ticks: 0,
                    gain_db: 0.0,
                },
                GainPoint {
                    t_ticks: 150_000,
                    gain_db: -3.5,
                },
                GainPoint {
                    t_ticks: 250_000,
                    gain_db: 1.5,
                },
            ],
        },
        assets: vec![Asset {
            hash: FINGERPRINT.to_owned(),
            license: "own_content".to_owned(),
        }],
        rationale: Some(Rationale {
            candidate_id: Some("cand_1".to_owned()),
            decisions: vec!["hook at 0".to_owned()],
        }),
        ..EditDocument::default()
    }
}

/// Commands spanning every variant that a user can reach directly.
fn candidate_commands(rng: &mut Rng, document: &EditDocument) -> Vec<EditCommand> {
    let duration = document.program_duration_ticks().max(1);
    let segment_ids = document
        .video
        .segments
        .iter()
        .map(|segment| segment.segment_id.clone())
        .collect::<Vec<_>>();
    let cue_ids = document
        .captions
        .cues
        .iter()
        .map(|cue| cue.cue_id.clone())
        .collect::<Vec<_>>();
    let pick = |rng: &mut Rng, values: &[String]| -> Option<String> {
        (!values.is_empty()).then(|| {
            let index = usize::try_from(rng.below(values.len() as u64)).unwrap_or(0);
            values[index].clone()
        })
    };
    let mut commands = Vec::new();
    if let Some(segment_id) = pick(rng, &segment_ids) {
        let index = document.segment_index(&segment_id).unwrap_or(0);
        let segment = &document.video.segments[index];
        let shrink = i64::try_from(rng.below(20)).unwrap_or(0) * FRAME_TICKS;
        commands.push(EditCommand::Trim {
            segment_id: segment_id.clone(),
            in_ticks: segment.in_ticks + shrink,
            out_ticks: segment.out_ticks,
        });
        commands.push(EditCommand::Trim {
            segment_id: segment_id.clone(),
            in_ticks: segment.in_ticks,
            out_ticks: segment.out_ticks + shrink,
        });
        commands.push(EditCommand::SetLayout {
            segment_id: segment_id.clone(),
            state: if rng.below(2) == 0 {
                LayoutState::Fit
            } else {
                LayoutState::SpeakerFill
            },
        });
        let local =
            i64::try_from(rng.below(u64::try_from(segment.duration_ticks().max(1)).unwrap_or(1)))
                .unwrap_or(0);
        commands.push(EditCommand::SetCropKeyframe {
            segment_id: segment_id.clone(),
            t_ticks: local,
            rect: CropRect {
                x: i64::try_from(rng.below(200)).unwrap_or(0),
                y: 0,
                width: 608,
                height: 1080,
            },
        });
        if let Some(first) = segment.layout.crop_path.first() {
            commands.push(EditCommand::RemoveCropKeyframe {
                segment_id,
                t_ticks: first.t_ticks,
            });
        }
    }
    if let Some(cue_id) = pick(rng, &cue_ids) {
        let index = document.cue_index(&cue_id).unwrap_or(0);
        let cue = &document.captions.cues[index];
        commands.push(EditCommand::EditCaptionText {
            cue_id: cue_id.clone(),
            word_index: usize::try_from(rng.below(cue.word_count().max(1) as u64)).unwrap_or(0),
            text: "corrected".to_owned(),
        });
        if cue.word_count() >= 2 {
            commands.push(EditCommand::SplitCue {
                cue_id: cue_id.clone(),
                at_word_index: 1,
                new_cue_id: format!("{cue_id}_split"),
            });
            commands.push(EditCommand::SetCueLines {
                cue_id: cue_id.clone(),
                line_word_counts: vec![1, cue.word_count() - 1],
            });
        }
        if index + 1 < document.captions.cues.len() {
            commands.push(EditCommand::MergeCues {
                first_cue_id: cue_id,
                second_cue_id: document.captions.cues[index + 1].cue_id.clone(),
            });
        }
    }
    let start = i64::try_from(rng.below(u64::try_from(duration).unwrap_or(1))).unwrap_or(0);
    let span = i64::try_from(rng.below(u64::try_from(duration).unwrap_or(1))).unwrap_or(0) + 1;
    commands.push(EditCommand::RippleDelete {
        start_ticks: start,
        end_ticks: (start + span).min(duration),
    });
    commands.push(EditCommand::SetGain {
        t_ticks: start,
        gain_db: -6.0,
    });
    commands.push(EditCommand::RemoveGainPoint { t_ticks: 150_000 });
    commands
}

#[test]
fn every_applied_command_inverts_exactly() {
    let mut checked = 0_u32;
    for seed in 0..400_u64 {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1));
        let document = sample_document();
        for command in candidate_commands(&mut rng, &document) {
            let before = document.to_canonical_json().expect("canonical before");
            let mut working = document.clone();
            let Ok(inverse) = command.apply(&mut working) else {
                continue;
            };
            let after = working.to_canonical_json().expect("canonical after");
            inverse
                .apply(&mut working)
                .unwrap_or_else(|error| panic!("seed {seed}: inverse rejected: {error}"));
            let restored = working.to_canonical_json().expect("canonical restored");
            assert_eq!(
                String::from_utf8_lossy(&restored),
                String::from_utf8_lossy(&before),
                "seed {seed}: {command:?} did not invert exactly"
            );
            if before != after {
                checked += 1;
            }
        }
    }
    assert!(
        checked > 500,
        "the generator produced too few effective commands ({checked})"
    );
}

#[test]
fn command_logs_replay_byte_identically() {
    for seed in 0..120_u64 {
        let mut rng = Rng(seed.wrapping_mul(0x2545_F491_4F6C_DD1D).wrapping_add(7));
        let initial = sample_document();
        let mut live = initial.clone();
        let mut log: Vec<Vec<u8>> = Vec::new();
        for _ in 0..8 {
            let candidates = candidate_commands(&mut rng, &live);
            let index = usize::try_from(rng.below(candidates.len() as u64)).unwrap_or(0);
            let command = &candidates[index];
            if command.apply(&mut live).is_ok() {
                log.push(command.to_canonical_json().expect("serialize command"));
            }
        }
        let mut replayed = initial;
        for entry in &log {
            let command = EditCommand::from_canonical_json(entry).expect("parse logged command");
            command.apply(&mut replayed).unwrap_or_else(|error| {
                panic!("seed {seed}: replay rejected a logged step: {error}")
            });
        }
        assert_eq!(
            live.to_canonical_json().expect("live"),
            replayed.to_canonical_json().expect("replayed"),
            "seed {seed}: replaying the log did not reproduce the live document"
        );
    }
}

#[test]
fn a_batch_undoes_as_one_step() {
    let mut document = sample_document();
    let before = document.to_canonical_json().expect("before");
    let batch = EditCommand::Batch {
        commands: vec![
            EditCommand::SetLayout {
                segment_id: "seg_a".to_owned(),
                state: LayoutState::Fit,
            },
            EditCommand::EditCaptionText {
                cue_id: "cue_1".to_owned(),
                word_index: 0,
                text: "The".to_owned(),
            },
            EditCommand::RippleDelete {
                start_ticks: 10_000,
                end_ticks: 40_000,
            },
        ],
    };
    let inverse = batch.apply(&mut document).expect("batch applies");
    assert_ne!(
        document.to_canonical_json().expect("after"),
        before,
        "the batch must actually change the document"
    );
    inverse.apply(&mut document).expect("batch inverts");
    assert_eq!(
        document.to_canonical_json().expect("restored"),
        before,
        "a batch must undo as a single transactional step"
    );
}

#[test]
fn a_rejected_command_leaves_the_document_untouched() {
    let mut document = sample_document();
    let before = document.to_canonical_json().expect("before");
    let batch = EditCommand::Batch {
        commands: vec![
            EditCommand::SetLayout {
                segment_id: "seg_a".to_owned(),
                state: LayoutState::Fit,
            },
            EditCommand::SetLayout {
                segment_id: "seg_missing".to_owned(),
                state: LayoutState::Fit,
            },
        ],
    };
    assert!(batch.apply(&mut document).is_err());
    assert_eq!(
        document.to_canonical_json().expect("after"),
        before,
        "a partially applied batch must not survive its own failure"
    );
}

#[test]
fn ripple_delete_through_a_segment_splits_it_and_closes_the_gap() {
    let mut document = sample_document();
    let original_duration = document.program_duration_ticks();
    let command = EditCommand::RippleDelete {
        start_ticks: 30_000,
        end_ticks: 60_000,
    };
    let inverse = command.apply(&mut document).expect("ripple applies");
    assert_eq!(
        document.program_duration_ticks(),
        original_duration - 30_000,
        "the program shortens by exactly the deleted span"
    );
    assert_eq!(
        document.video.segments.len(),
        4,
        "deleting inside a segment splits it in two"
    );
    assert_eq!(document.video.segments[1].segment_id, "seg_a~1");
    assert_eq!(document.video.segments[0].out_ticks, 30_000);
    assert_eq!(document.video.segments[1].in_ticks, 60_000);
    inverse.apply(&mut document).expect("ripple inverts");
    assert_eq!(document.program_duration_ticks(), original_duration);
    assert_eq!(document.video.segments.len(), 3);
}

#[test]
fn splitting_and_merging_a_cue_preserves_stored_line_breaks() {
    let mut document = sample_document();
    let reflow = EditCommand::SetCueLines {
        cue_id: "cue_1".to_owned(),
        line_word_counts: vec![2, 1],
    };
    reflow.apply(&mut document).expect("reflow applies");
    let before = document.to_canonical_json().expect("before");

    let split = EditCommand::SplitCue {
        cue_id: "cue_1".to_owned(),
        at_word_index: 1,
        new_cue_id: "cue_1b".to_owned(),
    };
    let inverse = split.apply(&mut document).expect("split applies");
    assert_eq!(document.captions.cues[0].word_count(), 1);
    assert_eq!(document.captions.cues[1].cue_id, "cue_1b");
    assert_eq!(document.captions.cues[1].word_count(), 2);

    inverse.apply(&mut document).expect("split inverts");
    assert_eq!(
        document.to_canonical_json().expect("restored"),
        before,
        "splitting across a stored line break must still undo exactly"
    );
}

#[test]
fn trimming_retimes_the_crop_path_and_drops_orphaned_keyframes() {
    let mut document = sample_document();
    let command = EditCommand::Trim {
        segment_id: "seg_a".to_owned(),
        in_ticks: 45_000,
        out_ticks: 90_000,
    };
    command.apply(&mut document).expect("trim applies");
    let path = &document.video.segments[0].layout.crop_path;
    assert_eq!(
        path.iter().map(|key| key.t_ticks).collect::<Vec<_>>(),
        vec![0, 45_000],
        "keyframes follow the source, and the one before the new in-point is gone"
    );
}

#[test]
fn program_and_source_time_round_trip_across_variable_frame_boundaries() {
    let document = sample_document();
    // Frame-boundary ticks at 30000/1001 are exactly where float pipelines
    // drift; integer ticks must survive the round trip untouched.
    for frame in 0..600_i64 {
        let program = frame * FRAME_TICKS;
        let Some((index, source)) = document.program_to_source(program) else {
            continue;
        };
        let restored = document
            .source_to_program(index, source)
            .expect("source maps back into the program");
        assert_eq!(
            restored, program,
            "frame {frame} did not survive the program/source round trip"
        );
    }
    assert!(document.program_to_source(-1).is_none());
    assert!(
        document
            .program_to_source(document.program_duration_ticks())
            .is_none(),
        "the program end is exclusive"
    );
}

#[test]
fn canonical_json_round_trips_frame_exact_ticks() {
    let mut document = sample_document();
    document.video.segments[0].out_ticks = 601 * FRAME_TICKS;
    let bytes = document.to_canonical_json().expect("serialize");
    let parsed = EditDocument::from_canonical_json(&bytes).expect("parse");
    assert_eq!(
        parsed.to_canonical_json().expect("reserialize"),
        bytes,
        "canonical serialization must be a fixed point"
    );
    assert_eq!(parsed.video.segments[0].out_ticks, 601 * FRAME_TICKS);
}

#[test]
fn rationale_is_outside_everything_the_renderer_sees() {
    let mut with_reason = sample_document();
    let mut without_reason = sample_document();
    without_reason.rationale = None;
    with_reason.rationale = Some(Rationale {
        candidate_id: Some("cand_99".to_owned()),
        decisions: vec!["completely different reasoning".to_owned()],
    });
    assert_eq!(
        with_reason.render_projection().expect("projection"),
        without_reason.render_projection().expect("projection"),
        "explaining an edit must never be able to move a pixel"
    );
    assert_ne!(
        with_reason.to_canonical_json().expect("json"),
        without_reason.to_canonical_json().expect("json"),
        "the rationale is still stored, just never rendered"
    );
}

#[test]
fn invalid_documents_are_refused() {
    let mut overlapping = sample_document();
    overlapping.captions.cues[1].start_ticks = 0;
    assert!(overlapping.validate().is_err(), "cues may not overlap");

    let mut backwards = sample_document();
    backwards.video.segments[0].out_ticks = 0;
    assert!(backwards.validate().is_err(), "segments may not be empty");

    let mut wrong_timebase = sample_document();
    wrong_timebase.timebase.den = 48_000;
    assert!(
        wrong_timebase.validate().is_err(),
        "only the 1/90000 edit timebase is allowed"
    );

    let mut stray_keyframe = sample_document();
    stray_keyframe.video.segments[0]
        .layout
        .crop_path
        .push(CropKeyframe {
            t_ticks: 10_000_000,
            rect: CropRect {
                x: 0,
                y: 0,
                width: 10,
                height: 10,
            },
        });
    assert!(
        stray_keyframe.validate().is_err(),
        "crop keyframes live inside their own segment"
    );
}

/// The published contract and the type the daemon actually edits must be the
/// same document. CI validates these fixtures against the JSON Schema; this
/// test proves the operational type reads them and writes them back
/// unchanged, so the two descriptions cannot drift apart unnoticed.
#[test]
fn published_contract_fixtures_load_into_the_operational_document() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for name in ["clip.json", "minimal.json"] {
        let path = repo.join("contracts/fixtures/edit_ir/valid").join(name);
        let raw = std::fs::read(&path).unwrap_or_else(|error| {
            panic!("cannot read {}: {error}", path.display());
        });
        let document = EditDocument::from_canonical_json(&raw)
            .unwrap_or_else(|error| panic!("{name} rejected by the operational type: {error}"));
        let sorted: serde_json::Value =
            serde_json::to_value(&document).expect("re-serialize into a sorted map");
        let pretty = format!(
            "{}\n",
            serde_json::to_string_pretty(&sorted).expect("pretty-print")
        );
        assert_eq!(
            pretty,
            String::from_utf8(raw).expect("fixture is UTF-8"),
            "{name} must round-trip through the operational document unchanged"
        );
    }
    for name in [
        "wrong-timebase.json",
        "float-ticks.json",
        "empty-caption-line.json",
    ] {
        let path = repo.join("contracts/fixtures/edit_ir/invalid").join(name);
        let raw = std::fs::read(&path).unwrap_or_else(|error| {
            panic!("cannot read {}: {error}", path.display());
        });
        assert!(
            EditDocument::from_canonical_json(&raw).is_err(),
            "{name} must be refused by the operational type too"
        );
    }
}
