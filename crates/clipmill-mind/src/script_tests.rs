#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;

const S: u64 = SECOND;

fn sentence(index: u64, start: u64, end: u64, speaker: &str, text: &str) -> Sentence {
    Sentence {
        index,
        start_ticks: start,
        end_ticks: end,
        speaker: Some(speaker.to_owned()),
        text: text.to_owned(),
    }
}

fn cue(kind: &str, start: u64, end: u64) -> Cue {
    Cue {
        kind: kind.to_owned(),
        start_ticks: start,
        end_ticks: end,
        label: None,
    }
}

/// A short interview: a question, an answer that gets a laugh, a pause, and
/// a cut to the guest.
fn interview() -> Source {
    Source {
        duration_ticks: 60 * S,
        sentences: vec![
            sentence(
                0,
                S,
                4 * S,
                "spk_1",
                "So what happened when the deal fell through?",
            ),
            sentence(1, 4 * S + S / 2, 7 * S, "spk_0", "We lost everything."),
            sentence(2, 9 * S, 11 * S, "spk_0", "No, really, everything."),
            sentence(3, 14 * S, 17 * S, "spk_0", "Even the office plant."),
            sentence(4, 20 * S, 24 * S, "spk_1", "And then you started again?"),
        ],
        cues: vec![
            cue("laughter", 17 * S + S / 5, 19 * S),
            Cue {
                label: Some("guest holds up a contract".to_owned()),
                ..cue("visual", 9 * S, 11 * S)
            },
        ],
        cuts: vec![10 * S, 21 * S, 22 * S],
        chapters: vec![
            Chapter {
                start_ticks: 0,
                keywords: vec!["deal".to_owned(), "money".to_owned()],
            },
            Chapter {
                start_ticks: 20 * S,
                keywords: vec!["restart".to_owned()],
            },
        ],
    }
}

#[test]
fn full_script_writes_every_sentence_and_cue_in_time_order() {
    let script = render(&interview(), Level::Full, None);
    assert_eq!(
        script.text,
        "\
# Recording 01:00 · 5 sentences · script full
# Speakers: A 48% · B 52%
## [00:00] topic: deal, money
s0 [00:01.0] A: So what happened when the deal fell through?
s1 [00:04.5] B: We lost everything. [pause 2.0 s]
s2 [00:09.0] B: No, really, everything. [cut] [pause 3.0 s]
[00:09.0] [visual 2.0 s: guest holds up a contract]
s3 [00:14.0] B: Even the office plant. [pause 3.0 s]
[00:17.2] [laughter 1.8 s]
## [00:20] topic: restart
s4 [00:20.0] A: And then you started again? [2 cuts]
"
    );
    assert_eq!(script.tokens, estimate_tokens(&script.text));
}

#[test]
fn compact_script_folds_a_turn_into_one_line() {
    let script = render(&interview(), Level::Compact, None);
    assert_eq!(
        script.text,
        "\
# Recording 01:00 · 5 sentences · script compact
# Speakers: A 48% · B 52%
## [00:00] topic: deal, money
s0 [00:01] A: So what happened when the deal fell through?
s1-s3 [00:04] B: We lost everything. [pause 2.0 s] No, really, everything. [pause 3.0 s] Even the office plant.
[00:09.0] [visual 2.0 s: guest holds up a contract]
[00:17.2] [laughter 1.8 s]
## [00:20] topic: restart
s4 [00:20] A: And then you started again?
"
    );
}

#[test]
fn outline_keeps_openings_and_the_lines_the_room_answered() {
    let script = render(&interview(), Level::Outline, None);
    assert_eq!(
        script.text,
        "\
# Recording 01:00 · 5 sentences · script outline
# Speakers: A 48% · B 52%
## [00:00-00:20] s0-s3 · B 71% · A 29% · deal, money · laughter 1 · visual 1
s0 A: So what happened when the deal fell through?
s3 B: Even the office plant. [laughter]
## [00:20-01:00] s4-s4 · A 100% · restart
s4 A: And then you started again?
"
    );
}

#[test]
fn a_range_keeps_the_recordings_speaker_letters() {
    let script = render(&interview(), Level::Full, Some((14 * S, 30 * S)));
    assert!(script.text.contains("range 00:14.0-00:30.0"));
    assert!(
        script
            .text
            .contains("s3 [00:14.0] B: Even the office plant.")
    );
    assert!(script.text.contains("s4 [00:20.0] A:"));
    assert!(!script.text.contains("s1 "));
}

/// A synthetic hour of conversation: two voices trading turns of one to four
/// sentences, a laugh now and then, a cut every five seconds, a topic every
/// three minutes.
fn podcast_hour() -> Source {
    const WORDS: [&str; 12] = [
        "pricing",
        "really",
        "customers",
        "because",
        "the",
        "team",
        "we",
        "decided",
        "honestly",
        "product",
        "and",
        "growth",
    ];
    let mut sentences = Vec::new();
    let mut cues = Vec::new();
    let mut at = 0;
    let mut index = 0;
    let mut speaker = 0;
    while at < 3600 * S - 10 * S {
        let turn = 1 + index % 4;
        for _ in 0..turn {
            let length = 8 + (index * 7) % 9;
            let text: Vec<&str> = (0..length)
                .map(|w| WORDS[usize::try_from((index + w) % 12).unwrap()])
                .collect();
            let end = at + length * S * 2 / 5;
            sentences.push(sentence(
                index,
                at,
                end,
                if speaker == 0 { "host" } else { "guest" },
                &(text.join(" ") + "."),
            ));
            if index % 37 == 0 {
                cues.push(cue("laughter", end, end + 2 * S));
            }
            at = end + S / 4;
            index += 1;
        }
        speaker = 1 - speaker;
        at += S / 2;
    }
    Source {
        duration_ticks: 3600 * S,
        sentences,
        cues,
        cuts: (1..720).map(|n| n * 5 * S).collect(),
        chapters: (0..20)
            .map(|n| Chapter {
                start_ticks: n * 180 * S,
                keywords: vec![format!("topic {n}")],
            })
            .collect(),
    }
}

#[test]
fn an_hour_of_conversation_fits_a_reading_budget() {
    let source = podcast_hour();
    let [full, compact, outline] = Level::ALL.map(|level| render(&source, level, None));
    assert!(source.sentences.len() > 600, "{}", source.sentences.len());
    assert!(compact.tokens < full.tokens);
    assert!(outline.tokens * 5 < compact.tokens);
    // The Recording Memory reads the whole hour at once in a 32k context.
    assert!(compact.tokens <= 24_000, "compact {}", compact.tokens);
    assert!(outline.tokens <= 4_000, "outline {}", outline.tokens);
    assert!(full.text.starts_with("# Recording 1:00:00 ·"));

    let chosen = fit(&source, 24_000, None).unwrap();
    assert_eq!(chosen.level, Level::Compact);
    assert_eq!(fit(&source, 100, None), None);
}

#[test]
fn the_same_index_gives_the_same_script() {
    let source = podcast_hour();
    let mut shuffled = source.clone();
    shuffled.cues.reverse();
    shuffled.cuts.reverse();
    shuffled.chapters.reverse();
    for level in Level::ALL {
        assert_eq!(render(&source, level, None), render(&shuffled, level, None));
    }
}

#[test]
fn letters_continue_past_z() {
    assert_eq!(letter(0), "A");
    assert_eq!(letter(25), "Z");
    assert_eq!(letter(26), "A2");
}

#[test]
fn an_outline_without_topics_groups_by_three_minutes() {
    let mut source = podcast_hour();
    source.chapters.clear();
    let script = render(&source, Level::Outline, None);
    let sections = script.text.lines().filter(|l| l.starts_with("## ")).count();
    assert_eq!(sections, 20);
    assert!(script.text.contains("## [0:03:00-0:06:00] s"));
}

#[test]
fn windows_cover_every_sentence_once_within_budget() {
    let source = podcast_hour();
    let windows = windows(&source, Level::Compact, 3_000);
    assert!(windows.len() >= 7, "{}", windows.len());
    let cited: Vec<u64> = windows.iter().flat_map(|w| w.units.clone()).collect();
    let all: Vec<u64> = source.sentences.iter().map(|s| s.index).collect();
    assert_eq!(cited, all);
    for window in &windows {
        assert!(window.script.tokens <= 3_000, "{}", window.script.tokens);
        // A window starts where a voice starts, wherever it could.
        let first = window.units[0];
        if first > 0 {
            let before = &source.sentences[usize::try_from(first - 1).unwrap()];
            let at = &source.sentences[usize::try_from(first).unwrap()];
            assert_ne!(
                before.speaker, at.speaker,
                "window at s{first} splits a turn"
            );
        }
    }
    assert_eq!(windows[0].range.0, 0);
    assert_eq!(windows.last().unwrap().range.1, source.duration_ticks);
}
