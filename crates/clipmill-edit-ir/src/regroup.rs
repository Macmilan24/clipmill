//! Grouping the on-screen captions by how many words a viewer sees at once.
//!
//! Short-form captions are a pace as much as a text: one word at a time
//! punches, three reads like speech. The caption engine groups by characters
//! and reading speed, which is right for a sidecar and says nothing about
//! words on screen, so this is a grouping of its own. It only ever writes the
//! burned-in list: the reading cues are what a viewer who cannot hear is left
//! with, and they keep the conservative grouping whatever the picture shows.
//!
//! It is a pure function of the words, so a command that asks for it replays
//! to the same cues. Words keep their ids, text, timing and emphasis; a cue
//! takes its region, animation and position from the cue its first word was
//! in.

use crate::document::{CaptionCue, CaptionLine, CaptionWord};

/// Speech stops long enough to start a new caption.
const PAUSE_TICKS: i64 = 54_000;
/// A caption is never shown for less than this, pause or not.
const MIN_SHOWN_TICKS: i64 = 30_000;
/// A gap to the next caption shorter than this is closed, so captions do not
/// flicker off between words said together.
const BRIDGE_TICKS: i64 = 36_000;
/// How long a caption stays after its last word when the next is far off.
const TAIL_TICKS: i64 = 18_000;
/// Characters one line holds before a longer group takes two.
const LINE_CHARACTERS: usize = 18;

/// One word with the cue it came from.
struct Placed<'a> {
    word: &'a CaptionWord,
    cue: &'a CaptionCue,
}

/// The cues regrouped into at most `max_words` words each.
pub(crate) fn regroup(cues: &[CaptionCue], max_words: usize) -> Vec<CaptionCue> {
    let max_words = max_words.max(1);
    let placed: Vec<Placed<'_>> = cues
        .iter()
        .flat_map(|cue| cue.words().map(move |word| Placed { word, cue }))
        .collect();
    let mut groups: Vec<Vec<&Placed<'_>>> = Vec::new();
    for item in &placed {
        let start_new = groups.last().is_none_or(|group| {
            let Some(previous) = group.last() else {
                return true;
            };
            group.len() >= max_words
                || ends_sentence(&previous.word.text)
                || item.word.start_ticks - previous.word.end_ticks > PAUSE_TICKS
                // A clause break is a good place to turn over when the group
                // already says something on its own.
                || (group.len() >= 2 && ends_clause(&previous.word.text))
        });
        if start_new {
            groups.push(Vec::new());
        }
        if let Some(group) = groups.last_mut() {
            group.push(item);
        }
    }

    let mut regrouped = Vec::with_capacity(groups.len());
    for (index, group) in groups.iter().enumerate() {
        let (Some(first), Some(last)) = (group.first(), group.last()) else {
            continue;
        };
        let previous_end = regrouped
            .last()
            .map_or(i64::MIN, |cue: &CaptionCue| cue.end_ticks);
        let start = first.word.start_ticks.max(previous_end).max(0);
        let next_start = groups
            .get(index + 1)
            .and_then(|next| next.first())
            .map(|next| next.word.start_ticks);
        let spoken_end = last.word.end_ticks.max(start + 1);
        let mut end = match next_start {
            Some(next) if next - spoken_end < BRIDGE_TICKS => next,
            _ => spoken_end + TAIL_TICKS,
        };
        end = end.max(start + MIN_SHOWN_TICKS);
        if let Some(next) = next_start {
            end = end.min(next.max(spoken_end));
        }
        let words: Vec<CaptionWord> = group
            .iter()
            .map(|item| CaptionWord {
                start_ticks: item.word.start_ticks.max(start),
                end_ticks: item
                    .word
                    .end_ticks
                    .min(end)
                    .max(item.word.start_ticks.max(start) + 1),
                ..item.word.clone()
            })
            .collect();
        regrouped.push(CaptionCue {
            cue_id: cue_id(first.word),
            start_ticks: start,
            end_ticks: end,
            region: first.cue.region,
            anim: first.cue.anim,
            lines: lines(words),
            position: first.cue.position,
        });
    }
    regrouped
}

/// One line, or two balanced ones when the words would not fit on one.
fn lines(words: Vec<CaptionWord>) -> Vec<CaptionLine> {
    let width = words
        .iter()
        .map(|word| word.text.chars().count())
        .sum::<usize>()
        + words.len().saturating_sub(1);
    if words.len() < 2 || width <= LINE_CHARACTERS {
        return vec![CaptionLine { words }];
    }
    // The break that leaves the two lines closest in width.
    let mut best = (usize::MAX, 1);
    for split in 1..words.len() {
        let left = words[..split]
            .iter()
            .map(|word| word.text.chars().count() + 1)
            .sum::<usize>();
        let right = width.saturating_sub(left);
        let difference = left.abs_diff(right);
        if difference < best.0 {
            best = (difference, split);
        }
    }
    let mut first = words;
    let second = first.split_off(best.1);
    vec![CaptionLine { words: first }, CaptionLine { words: second }]
}

fn cue_id(word: &CaptionWord) -> String {
    word.word_id.as_ref().map_or_else(
        || format!("cue_t{}", word.start_ticks),
        |id| format!("cue_{id}"),
    )
}

fn ends_sentence(text: &str) -> bool {
    text.trim_end_matches(['"', '\'', ')', '’', '”'])
        .ends_with(['.', '?', '!', '…'])
}

fn ends_clause(text: &str) -> bool {
    text.trim_end_matches(['"', '\'', ')', '’', '”'])
        .ends_with([',', ';', ':', '—'])
}

#[cfg(test)]
mod tests {
    use super::regroup;
    use crate::document::{CaptionAnimation, CaptionCue, CaptionLine, CaptionRegion, CaptionWord};

    fn word(id: usize, text: &str, start: i64, end: i64) -> CaptionWord {
        CaptionWord {
            text: text.to_owned(),
            start_ticks: start,
            end_ticks: end,
            word_id: Some(format!("w{id}")),
            emphasis: id == 2,
        }
    }

    fn cues() -> Vec<CaptionCue> {
        let words = [
            "So", "this", "is", "the", "part.", "Nobody", "expects", "it,", "really",
        ]
        .iter()
        .enumerate()
        .map(|(index, text)| {
            let start = i64::try_from(index).unwrap_or(0) * 18_000;
            word(index, text, start, start + 15_000)
        })
        .collect::<Vec<_>>();
        vec![CaptionCue {
            cue_id: "cue_1".to_owned(),
            start_ticks: 0,
            end_ticks: 9 * 18_000,
            region: CaptionRegion::LowerSafe,
            anim: CaptionAnimation::Karaoke,
            lines: vec![CaptionLine { words }],
            position: None,
        }]
    }

    #[test]
    fn one_word_at_a_time_keeps_every_word_in_order() {
        let singles = regroup(&cues(), 1);
        assert_eq!(singles.len(), 9);
        assert!(singles.iter().all(|cue| cue.word_count() == 1));
        for pair in singles.windows(2) {
            assert!(pair[0].end_ticks <= pair[1].start_ticks);
        }
        assert!(singles[2].words().next().is_some_and(|word| word.emphasis));
        assert_eq!(singles[0].cue_id, "cue_w0");
    }

    #[test]
    fn groups_turn_over_at_sentences_and_clauses() {
        let threes = regroup(&cues(), 3);
        let texts = threes
            .iter()
            .map(|cue| {
                cue.words()
                    .map(|word| word.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>();
        assert_eq!(
            texts,
            ["So this is", "the part.", "Nobody expects it,", "really"]
        );
    }
}
