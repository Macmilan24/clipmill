//! Re-grouping the caption a cut fell inside.
//!
//! A trim that lands inside a cue leaves the cue's remaining words as a
//! fragment: "Leave room," on screen for six tenths of a second, which no
//! profile calls readable and the export strip refuses for the sidecar. The
//! words are still said, so they may not be dropped, and there is nothing
//! to hold them longer against — the next cue begins where it begins. What
//! a subtitler does is fold them into the neighbouring caption and break
//! that caption again. This does the same, with the caption engine's own
//! segmenter, over exactly the two cues involved: the fragment and the cue
//! beside it on the side that was kept. Every other cue, and every grouping
//! a person chose elsewhere, is left as it stands.
//!
//! It runs only when the fragment fails the profile on its own — too brief
//! to read, or asking for a reading speed the profile forbids. A cut that
//! leaves a readable caption changes nothing but the caption's window.

use clipmill_captions::{
    Cue as EngineCue, CueFacts, Profile, Span, Token, Violation, Weights, lexicon, segment,
    validate,
};

use crate::document::{CaptionCue, CaptionLine, CaptionWord, Presentation};

/// Which end of the program the cut was at.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Edge {
    Head,
    Tail,
}

/// The profile a presentation's cues are held to.
fn profile_of(presentation: Presentation) -> Profile {
    match presentation {
        Presentation::Reading => Profile::ACCESSIBILITY_EN,
        Presentation::BurnIn => Profile::BURN_IN_EN,
    }
}

/// Whether a cue, on its own, is one a viewer could read.
fn unreadable(cue: &CaptionCue, profile: Profile) -> bool {
    let widths: Vec<usize> = cue
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
    let facts = CueFacts {
        cue_id: &cue.cue_id,
        start_ticks: cue.start_ticks,
        end_ticks: cue.end_ticks,
        speech_end_ticks: cue.words().map(|word| word.end_ticks).max(),
        lines: &widths,
    };
    validate(&[facts], profile, &[]).iter().any(|violation| {
        matches!(
            violation,
            Violation::TooBrief { .. } | Violation::ReadingRate { .. }
        )
    })
}

fn token_of(word: &CaptionWord) -> Token {
    let normalized = lexicon::normalize(&word.text);
    Token {
        text: word.text.clone(),
        start_ticks: word.start_ticks,
        end_ticks: word.end_ticks,
        filler: lexicon::is_filler(&normalized),
        // Emphasis comes from the transcript's index, which a document does
        // not carry; a re-broken cue is plain, as a hand-edited one is.
        emphasis: false,
        break_after: lexicon::break_after(&word.text),
        orphans: lexicon::orphans_if_last(&normalized),
        normalized,
    }
}

/// A cue id not used by any cue in the list, derived from a wanted one.
fn free_id(cues: &[CaptionCue], wanted: &str) -> String {
    if !cues.iter().any(|cue| cue.cue_id == wanted) {
        return wanted.to_owned();
    }
    // At most one more than the cues there are can be taken.
    (1..=cues.len() + 1)
        .map(|n| format!("{wanted}~{n}"))
        .find(|candidate| !cues.iter().any(|cue| cue.cue_id == *candidate))
        .unwrap_or_else(|| unreachable!("more candidates than cues to collide with"))
}

/// The engine's cue as the document's, over the merged word list.
fn cue_from(
    engine: &EngineCue,
    words: &[CaptionWord],
    model: &CaptionCue,
    cue_id: String,
) -> CaptionCue {
    let lines = engine
        .lines
        .iter()
        .map(|line| CaptionLine {
            words: words[line.first_token..line.first_token + line.token_count]
                .iter()
                .map(|word| CaptionWord {
                    // Kept inside the cue's window, as the projection keeps
                    // them: a cue may give up the end of its last word to
                    // leave the blank before the next.
                    start_ticks: word
                        .start_ticks
                        .max(engine.start_ticks)
                        .min(engine.end_ticks - 1),
                    end_ticks: word
                        .end_ticks
                        .min(engine.end_ticks)
                        .max(word.start_ticks.max(engine.start_ticks) + 1),
                    ..word.clone()
                })
                .collect(),
        })
        .collect();
    CaptionCue {
        cue_id,
        start_ticks: engine.start_ticks,
        end_ticks: engine.end_ticks,
        region: model.region,
        anim: model.anim,
        lines,
    }
}

/// Fold an unreadable fragment at `edge` into the cue beside it and break
/// the pair again. True when the list changed.
pub(crate) fn reflow_fragment(
    cues: &mut Vec<CaptionCue>,
    presentation: Presentation,
    edge: Edge,
) -> bool {
    if cues.len() < 2 {
        return false;
    }
    let profile = profile_of(presentation);
    let (fragment_index, neighbour_index) = match edge {
        Edge::Head => (0, 1),
        Edge::Tail => (cues.len() - 1, cues.len() - 2),
    };
    if !unreadable(&cues[fragment_index], profile) {
        return false;
    }
    let (first, second) = (
        fragment_index.min(neighbour_index),
        fragment_index.max(neighbour_index),
    );
    let words: Vec<CaptionWord> = cues[first]
        .words()
        .chain(cues[second].words())
        .cloned()
        .collect();
    let tokens: Vec<Token> = words.iter().map(token_of).collect();
    let span = Span {
        start_ticks: cues[first].start_ticks,
        end_ticks: cues[second].end_ticks,
    };
    let Ok(broken) = segment(&tokens, &[], span, profile, Weights::default()) else {
        return false;
    };
    if broken.is_empty() {
        return false;
    }
    let ids: Vec<String> = {
        let mut taken: Vec<CaptionCue> = cues.clone();
        taken.drain(first..=second);
        let wanted = [cues[first].cue_id.clone(), cues[second].cue_id.clone()];
        let mut ids = Vec::with_capacity(broken.len());
        for (index, _) in broken.iter().enumerate() {
            let base = wanted
                .get(index)
                .cloned()
                .unwrap_or_else(|| wanted[1].clone());
            let id = free_id(&taken, &base);
            taken.push(CaptionCue {
                cue_id: id.clone(),
                ..cues[first].clone()
            });
            ids.push(id);
        }
        ids
    };
    let model = cues[first].clone();
    let replacement: Vec<CaptionCue> = broken
        .iter()
        .zip(ids)
        .map(|(engine, id)| cue_from(engine, &words, &model, id))
        .collect();
    cues.splice(first..=second, replacement);
    true
}
