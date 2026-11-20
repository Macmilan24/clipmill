#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;

const SPEECH: f32 = -20.0;
const ROOM: f32 = -60.0;

/// A level track of `frames` frames of room tone with speech in `spans`
/// (frame ranges), and the matching words in ticks.
fn recording(frames: usize, spans: &[(usize, usize)]) -> (Vec<f32>, Vec<(u64, u64)>) {
    let mut levels = vec![ROOM; frames];
    for &(start, end) in spans {
        for level in &mut levels[start..end] {
            *level = SPEECH;
        }
    }
    let words = spans
        .iter()
        .map(|&(start, end)| (ticks(start), ticks(end)))
        .collect();
    (levels, words)
}

fn ticks(frame: usize) -> u64 {
    u64::try_from(frame).unwrap_or(u64::MAX) * FRAME_TICKS
}

/// Words of 300 ms with 500 ms between them, so the floor and speech level
/// have plenty of both to measure, starting at `from` and ending before `to`.
fn chatter(from: usize, to: usize) -> Vec<(usize, usize)> {
    (from..to)
        .step_by(80)
        .filter(|start| start + 30 <= to)
        .map(|start| (start, start + 30))
        .collect()
}

fn point_between(points: &[CutPoint], from: usize, to: usize) -> Option<CutPoint> {
    points
        .iter()
        .copied()
        .find(|point| point.at >= ticks(from) && point.at <= ticks(to))
}

#[test]
fn a_long_pause_is_a_clean_cut_point_clear_of_both_words() {
    let mut spans = chatter(0, 1_000);
    spans.extend([(1_000, 1_030), (1_070, 1_100)]);
    spans.extend(chatter(1_200, 3_000));
    let (raw, words) = recording(3_000, &spans);
    let points = measure(&Levels::new(&raw), &words);
    let point = point_between(&points, 1_030, 1_070).expect("a cut point in the 400 ms pause");
    assert_eq!(point.grade, Grade::Clean);
    assert!(point.at >= ticks(1_030 + MARGIN_FRAMES));
    assert!(point.at <= ticks(1_070 - MARGIN_FRAMES));
}

#[test]
fn a_short_pause_is_usable_and_a_shorter_one_is_no_place_to_cut() {
    let mut spans = chatter(0, 1_000);
    spans.extend([(1_000, 1_030), (1_050, 1_080), (1_090, 1_120)]);
    spans.extend(chatter(1_200, 3_000));
    let (raw, words) = recording(3_000, &spans);
    let points = measure(&Levels::new(&raw), &words);
    let usable = point_between(&points, 1_030, 1_050).expect("the 200 ms pause");
    assert_eq!(usable.grade, Grade::Usable);
    assert!(point_between(&points, 1_080, 1_090).is_none());
}

#[test]
fn a_gap_filled_by_another_voice_is_not_a_pause() {
    // The transcript shows a full second between the two words, but someone
    // else is talking through it.
    let mut spans = chatter(0, 1_000);
    spans.extend([(1_000, 1_030), (1_130, 1_160)]);
    spans.extend(chatter(1_260, 3_000));
    let (mut raw, words) = recording(3_000, &spans);
    for level in &mut raw[1_030..1_130] {
        *level = -24.0;
    }
    let points = measure(&Levels::new(&raw), &words);
    assert!(point_between(&points, 1_030, 1_130).is_none());
}

#[test]
fn a_one_frame_click_does_not_split_a_pause() {
    let mut spans = chatter(0, 1_000);
    spans.extend([(1_000, 1_030), (1_070, 1_100)]);
    spans.extend(chatter(1_200, 3_000));
    let (mut raw, words) = recording(3_000, &spans);
    raw[1_050] = SPEECH;
    let points = measure(&Levels::new(&raw), &words);
    let point = point_between(&points, 1_030, 1_070).expect("the pause survives the click");
    assert_eq!(point.grade, Grade::Clean);
}

#[test]
fn word_edges_reported_late_still_find_the_pause_that_was_heard() {
    // The word really ends at frame 1_030; the recogniser says 1_038.
    let mut spans = chatter(0, 1_000);
    spans.extend([(1_000, 1_030), (1_070, 1_100)]);
    spans.extend(chatter(1_200, 3_000));
    let (raw, mut words) = recording(3_000, &spans);
    let late = words
        .iter()
        .position(|&(start, _)| start == ticks(1_000))
        .expect("the word");
    words[late].1 = ticks(1_038);
    let points = measure(&Levels::new(&raw), &words);
    let point = point_between(&points, 1_030, 1_070).expect("the pause");
    assert_eq!(point.pause_start, ticks(1_030));
    assert_eq!(point.grade, Grade::Clean);
}

#[test]
fn continuous_speech_offers_no_cut_points() {
    // Speech with 50 ms dips: nothing long enough, however the floor reads.
    let spans: Vec<(usize, usize)> = (0..3_000)
        .step_by(35)
        .map(|start| (start, (start + 30).min(3_000)))
        .collect();
    let (raw, words) = recording(3_000, &spans);
    assert!(measure(&Levels::new(&raw), &words).is_empty());
}

#[test]
fn a_recording_with_almost_no_quiet_is_never_all_quiet() {
    // Two voices over each other: the floor reads as speech, and a level a
    // few dB under it must still not count as a pause.
    let spans = vec![(0, 1_500), (1_520, 3_000)];
    let (mut raw, words) = recording(3_000, &spans);
    for level in &mut raw[1_500..1_520] {
        *level = SPEECH - 4.0;
    }
    assert!(measure(&Levels::new(&raw), &words).is_empty());
}

#[test]
fn the_lattice_finds_the_cut_point_a_sentence_edge_faces() {
    let mut spans = chatter(0, 1_000);
    spans.extend([(1_000, 1_030), (1_070, 1_100)]);
    spans.extend(chatter(1_200, 3_000));
    let (raw, words) = recording(3_000, &spans);
    let lattice = Lattice::new(measure(&Levels::new(&raw), &words));
    let after = lattice.edge(&words, ticks(1_030)).expect("after the word");
    let before = lattice.edge(&words, ticks(1_070)).expect("before the next");
    assert_eq!(after, before);
    assert_eq!(lattice.at(after.at), Some(after));
    assert_eq!(lattice.at(after.at + TOLERANCE_TICKS), Some(after));
    assert_eq!(lattice.at(after.at + TOLERANCE_TICKS + 1), None);
    // The very first word has nothing before it to cut in.
    assert_eq!(lattice.edge(&words, 0), None);
}

#[test]
fn frame_levels_are_decibels_of_full_scale() {
    assert!(frame_db(&[0; 160]) < -130.0);
    let square: Vec<i16> = (0..160)
        .map(|n| if n % 2 == 0 { i16::MAX } else { -i16::MAX })
        .collect();
    assert!(frame_db(&square).abs() < 0.01);
    assert!(frame_db(&[]) < -130.0);
}

#[test]
fn a_cut_point_survives_the_round_trip_through_an_index_event() {
    let point = CutPoint {
        at: 1_234_500,
        pause_start: 1_200_600,
        pause_end: 1_260_000,
        grade: Grade::Clean,
    };
    let (start, end, value, label) = point.event_fields();
    assert_eq!(
        CutPoint::from_event(start, end, Some(value), Some(label)),
        Some(point)
    );
    assert_eq!(CutPoint::from_event(start, end, None, Some(label)), None);
    assert_eq!(
        CutPoint::from_event(start, end, Some(value), Some("pause")),
        None
    );
    assert_eq!(
        CutPoint::from_event(start, end, Some(f64::NAN), Some(label)),
        None
    );
    // An offset past the pause's end is not a cut point in it.
    assert_eq!(
        CutPoint::from_event(start, start + 10, Some(value), Some(label)),
        None
    );
}

#[test]
fn grades_read_back_from_their_names() {
    for grade in [Grade::Usable, Grade::Clean] {
        assert_eq!(Grade::parse(grade.as_str()), Some(grade));
    }
    assert_eq!(Grade::parse("none"), None);
}

/// Four one-second words: a pause after the first, none between the second
/// and third, a pause after the third.
fn agent_material() -> (Lattice, Vec<(u64, u64)>) {
    let words = vec![
        (0, SECOND),
        (2 * SECOND, 3 * SECOND),
        (3 * SECOND, 4 * SECOND),
        (5 * SECOND, 6 * SECOND),
    ];
    let point = |pause_start: u64, pause_end: u64| CutPoint {
        at: u64::midpoint(pause_start, pause_end),
        pause_start,
        pause_end,
        grade: Grade::Clean,
    };
    let lattice = Lattice::new(vec![
        point(SECOND, 2 * SECOND),
        point(4 * SECOND, 5 * SECOND),
    ]);
    (lattice, words)
}

fn straight(start: u64, end: u64) -> MindClipPlan {
    serde_json::from_value(serde_json::json!({
        "schema_version": "clipmill.mind.clip_plan.v1",
        "source_fingerprint": format!("sha256:{}", "a".repeat(64)),
        "plan_id": "plan_0000000000000000000000aa",
        "form": "straight",
        "parts": [{ "part_id": "p1", "role": "body", "start_ticks": start, "end_ticks": end }],
        "joins": [],
        "duration_ticks": end - start,
        "provenance": { "by": "agent" },
    }))
    .expect("a plan")
}

#[test]
fn an_agents_word_edges_are_moved_into_the_pauses_beside_them() {
    let (lattice, words) = agent_material();
    // From the second word's start to the third word's end.
    let mut plan = straight(2 * SECOND, 4 * SECOND);
    snap(&mut plan, &lattice, &words);
    let part = &plan.parts[0];
    assert_eq!(part.start_ticks, 3 * SECOND / 2);
    assert_eq!(part.end_ticks.get(), 9 * SECOND / 2);
    assert_eq!(plan.duration_ticks.get(), 3 * SECOND);
    let bounds = crate::Bounds {
        words: &words,
        lattice: Some(&lattice),
        duration: None,
        source_ticks: 6 * SECOND,
    };
    assert!(crate::check(&plan, &bounds).is_empty());
}

#[test]
fn an_edge_inside_a_word_keeps_the_word_and_one_with_no_pause_is_reported() {
    let (lattice, words) = agent_material();
    // Starts inside the second word: moves back to the pause before it.
    let mut plan = straight(2 * SECOND + SECOND / 3, 4 * SECOND);
    snap(&mut plan, &lattice, &words);
    assert_eq!(plan.parts[0].start_ticks, 3 * SECOND / 2);

    // Ends after the second word, where the third follows with no pause.
    let mut plan = straight(2 * SECOND, 3 * SECOND);
    snap(&mut plan, &lattice, &words);
    assert_eq!(plan.parts[0].end_ticks.get(), 3 * SECOND);
    let bounds = crate::Bounds {
        words: &words,
        lattice: Some(&lattice),
        duration: None,
        source_ticks: 6 * SECOND,
    };
    let codes: Vec<&str> = crate::check(&plan, &bounds)
        .iter()
        .map(|v| v.code)
        .collect();
    assert_eq!(codes, ["not_in_pause"]);
}
