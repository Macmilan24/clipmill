//! The words that carry no topic.
//!
//! Lexical cohesion measures whether two stretches of speech use the same
//! vocabulary. "the" and "and" are used by every stretch of English ever
//! spoken, so leaving them in makes every gap look cohesive and the
//! segmentation finds nothing. Removing them is what makes the score about
//! subject matter.
//!
//! This list is deliberately small and deliberately English. A large list
//! starts making editorial decisions about which words matter, and a
//! multilingual one would need per-language tuning nobody here has measured.
//! Its identifier travels in the published document, so an index built with a
//! different list is a distinguishable observation rather than a silent
//! change of meaning — and an index over a language this does not cover says
//! so by naming the language beside the list.

use std::collections::BTreeSet;
use std::sync::OnceLock;

/// Named in every published index, so the list and the numbers it produced
/// cannot come apart.
pub const IDENTIFIER: &str = "english-minimal.v1";

const WORDS: &[&str] = &[
    "a", "about", "after", "all", "also", "am", "an", "and", "any", "are", "as", "at", "be",
    "because", "been", "before", "being", "but", "by", "can", "could", "did", "do", "does",
    "doing", "done", "down", "each", "even", "for", "from", "get", "got", "had", "has", "have",
    "having", "he", "her", "here", "hers", "him", "his", "how", "i", "if", "in", "into", "is",
    "it", "its", "just", "like", "me", "more", "most", "much", "my", "no", "not", "now", "of",
    "off", "on", "one", "only", "or", "other", "our", "out", "over", "own", "really", "said",
    "same", "she", "should", "so", "some", "such", "than", "that", "the", "their", "them", "then",
    "there", "these", "they", "this", "those", "through", "to", "too", "up", "us", "very", "was",
    "we", "were", "what", "when", "where", "which", "while", "who", "why", "will", "with", "would",
    "you", "your",
];

fn set() -> &'static BTreeSet<&'static str> {
    static SET: OnceLock<BTreeSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| WORDS.iter().copied().collect())
}

/// Whether a lowercased token carries no topic.
#[must_use]
pub fn is_stopword(token: &str) -> bool {
    set().contains(token)
}

#[cfg(test)]
mod tests {
    use super::{IDENTIFIER, WORDS, is_stopword};

    #[test]
    fn the_list_is_sorted_and_has_no_duplicates() {
        // Sorted so a diff of this file reads as an editorial decision rather
        // than a shuffle, and unique so the identifier means one thing.
        let mut sorted = WORDS.to_vec();
        sorted.sort_unstable();
        assert_eq!(sorted, WORDS, "the stopword list is out of order");
        sorted.dedup();
        assert_eq!(sorted.len(), WORDS.len(), "a stopword is listed twice");
    }

    #[test]
    fn membership_is_exact_and_lowercase() {
        assert!(is_stopword("the"));
        assert!(is_stopword("would"));
        // The caller lowercases; this does not, so that a bug there is visible
        // here rather than silently absorbed.
        assert!(!is_stopword("The"));
        assert!(!is_stopword("timestamp"));
    }

    #[test]
    fn the_identifier_names_this_list() {
        assert_eq!(IDENTIFIER, "english-minimal.v1");
    }
}
