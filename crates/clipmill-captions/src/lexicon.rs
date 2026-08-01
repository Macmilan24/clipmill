//! Word lists, and the normalization that makes matching against them honest.
//!
//! Three lists, each doing one job.
//!
//! **Fillers** are tagged, never removed. A caption is a record of what was
//! said, and a viewer reading "I mean, the thing is" while hearing it is being
//! told the truth; a viewer reading a tidied sentence is being told what a
//! program thought they should have heard. What the tag is for is emphasis: a
//! filler may never carry it, because emphasising "um" is the clearest possible
//! signal that nothing understood the sentence.
//!
//! **Words that may not end a line** are the book's "never orphan an article".
//! An article, preposition or conjunction stranded at the end of a line makes
//! the reader hold it in mind across a break for no reason — the phrase it
//! belongs to is on the next line. This is the cheapest real improvement in
//! captioning and it costs one list.
//!
//! **Terminators** are punctuation, not a list, and they come from the text
//! itself rather than from a model: a break after a full stop is free, a break
//! after a comma is cheap, a break inside a phrase is what the cost function is
//! there to discourage.

/// The lexicon's identity, recorded in the artifact key. A different list is a
/// different reading of the same words.
pub const FILLER_LEXICON: &str = "clipmill.filler.en.v1";

/// English fillers and discourse markers. Kept short and uncontroversial: every
/// entry here is a word whose emphasis would be an obvious mistake, and a word
/// that is merely common does not belong.
const FILLERS: &[&str] = &[
    "ah",
    "eh",
    "er",
    "erm",
    "hmm",
    "huh",
    "mhm",
    "uh",
    "uhm",
    "um",
    "umm",
    "yeah",
    "basically",
    "literally",
    "actually",
    "anyway",
    "kinda",
    "sorta",
];

/// Words a line may not end on. Articles, the common prepositions, the
/// coordinating conjunctions, and the possessives that bind forward.
const NO_LINE_END: &[&str] = &[
    "a", "an", "the", "and", "but", "or", "nor", "for", "so", "yet", "at", "by", "in", "into",
    "of", "off", "on", "onto", "out", "over", "to", "up", "with", "from", "as", "if", "than",
    "that", "this", "these", "those", "my", "your", "his", "her", "its", "our", "their", "is",
    "was", "are", "were", "be", "been",
];

/// A word stripped to what a lexicon can be asked about: lowercase, without the
/// punctuation that surrounds it.
///
/// Kept beside the rendered text rather than replacing it. The two strings want
/// different things — one is matched, one is read — and collapsing them loses
/// the capitalization a speaker's name depends on.
pub fn normalize(text: &str) -> String {
    text.chars()
        .filter(|character| character.is_alphanumeric() || *character == '\'')
        .flat_map(char::to_lowercase)
        .collect()
}

/// Whether a normalized word is a filler.
pub fn is_filler(normalized: &str) -> bool {
    FILLERS.binary_search(&normalized).is_ok() || FILLERS.contains(&normalized)
}

/// Whether a line ending on this normalized word would orphan it.
pub fn orphans_if_last(normalized: &str) -> bool {
    NO_LINE_END.contains(&normalized)
}

/// How good a break after this word would be, read from its own punctuation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Break {
    /// A sentence ended here. The best place there is.
    Sentence,
    /// A clause ended here — a comma, a colon, a dash.
    Clause,
    /// Nothing says a break belongs here.
    Phrase,
}

impl Break {
    /// What the break-quality weight is multiplied by.
    pub(crate) fn penalty(self) -> f64 {
        match self {
            Self::Sentence => 0.0,
            Self::Clause => 0.35,
            Self::Phrase => 1.0,
        }
    }
}

/// The break quality implied by a word's trailing punctuation.
pub fn break_after(text: &str) -> Break {
    let trimmed = text.trim_end_matches(['"', '\'', ')', ']', '»', '”', '’']);
    let last = trimmed.chars().next_back().unwrap_or(' ');
    match last {
        '.' | '!' | '?' | '…' | '。' | '！' | '？' => Break::Sentence,
        ',' | ';' | ':' | '—' | '–' | '、' | '，' => Break::Clause,
        _ => Break::Phrase,
    }
}

#[cfg(test)]
mod tests {
    use super::{Break, break_after, is_filler, normalize, orphans_if_last};

    #[test]
    fn normalization_keeps_the_word_and_drops_what_surrounds_it() {
        assert_eq!(normalize("\"Kubernetes,\""), "kubernetes");
        assert_eq!(normalize("don't"), "don't");
        assert_eq!(normalize("THE"), "the");
        assert_eq!(normalize("—"), "");
    }

    #[test]
    fn fillers_are_tagged_and_ordinary_words_are_not() {
        assert!(is_filler("um"));
        assert!(is_filler("basically"));
        assert!(!is_filler("kubernetes"));
        // Tagged, not removed: the caller still renders it.
        assert!(!is_filler("the"));
    }

    #[test]
    fn a_line_may_not_end_on_a_word_that_binds_forward() {
        assert!(orphans_if_last("the"));
        assert!(orphans_if_last("of"));
        assert!(orphans_if_last("and"));
        assert!(!orphans_if_last("running"));
    }

    #[test]
    fn break_quality_is_read_from_punctuation_not_guessed() {
        assert_eq!(break_after("done."), Break::Sentence);
        assert_eq!(break_after("really?"), Break::Sentence);
        assert_eq!(break_after("first,"), Break::Clause);
        assert_eq!(break_after("running"), Break::Phrase);
        // Through a closing quote, which is where a sentence usually ends in
        // reported speech.
        assert_eq!(break_after("\"stop.\""), Break::Sentence);
    }

    #[test]
    fn a_better_break_never_costs_more_than_a_worse_one() {
        assert!(Break::Sentence.penalty() < Break::Clause.penalty());
        assert!(Break::Clause.penalty() < Break::Phrase.penalty());
    }
}
