//! The emoji a clip can show: a curated set a creator reaches for, each a
//! pinned picture (`bom.toml`, `[emoji]`) and the words that call for it.
//!
//! Colour emoji are drawn as pictures because libass draws glyph outlines.
//! The words are plain and lower-case; a caption word matches after its
//! punctuation is taken off, and the first emoji to list a word wins it.
//! They are words a speaker leans on, never the everyday ones — "think",
//! "time", "see" — which would put an emoji on every sentence.

/// One emoji the app offers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Emoji {
    /// Its code point, as the pinned file names it: `emoji_u{code}.png`.
    pub code: &'static str,
    /// What a person calls it.
    pub label: &'static str,
    /// Words in a caption that call for it.
    pub words: &'static [&'static str],
}

impl Emoji {
    /// The pinned picture's file name.
    pub fn file(&self) -> String {
        format!("emoji_u{}.png", self.code)
    }
}

pub const EMOJI: &[Emoji] = &[
    Emoji {
        code: "1f525",
        label: "Fire",
        words: &["fire", "hot", "lit", "amazing", "insane", "incredible"],
    },
    Emoji {
        code: "1f4a1",
        label: "Idea",
        words: &[
            "idea", "ideas", "realise", "realize", "realised", "realized", "insight",
        ],
    },
    Emoji {
        code: "1f602",
        label: "Laughing",
        words: &["funny", "laugh", "laughing", "joke", "hilarious"],
    },
    Emoji {
        code: "1f92f",
        label: "Mind blown",
        words: &[
            "crazy",
            "blown",
            "wild",
            "unbelievable",
            "shocking",
            "shocked",
        ],
    },
    Emoji {
        code: "1f440",
        label: "Eyes",
        words: &["watch", "notice", "attention"],
    },
    Emoji {
        code: "1f4b0",
        label: "Money",
        words: &[
            "money", "cash", "dollars", "paid", "pay", "price", "revenue", "profit", "income",
            "salary", "budget",
        ],
    },
    Emoji {
        code: "1f680",
        label: "Rocket",
        words: &[
            "launch", "launched", "rocket", "fast", "faster", "scale", "boost",
        ],
    },
    Emoji {
        code: "2705",
        label: "Check",
        words: &["correct", "done", "works", "worked"],
    },
    Emoji {
        code: "274c",
        label: "Cross",
        words: &["wrong", "mistake", "mistakes", "fail", "failed", "failure"],
    },
    Emoji {
        code: "26a0",
        label: "Warning",
        words: &[
            "warning",
            "careful",
            "danger",
            "dangerous",
            "risk",
            "beware",
        ],
    },
    Emoji {
        code: "2764",
        label: "Heart",
        words: &["love", "loved", "heart"],
    },
    Emoji {
        code: "1f44f",
        label: "Clapping",
        words: &["congrats", "congratulations", "bravo", "applause"],
    },
    Emoji {
        code: "1f64c",
        label: "Raised hands",
        words: &["finally", "yay", "hallelujah"],
    },
    Emoji {
        code: "1f4af",
        label: "Hundred",
        words: &["hundred", "absolutely", "exactly", "totally", "definitely"],
    },
    Emoji {
        code: "1f3af",
        label: "Target",
        words: &["goal", "goals", "target", "focus", "aim"],
    },
    Emoji {
        code: "1f4c8",
        label: "Chart up",
        words: &["grow", "growth", "growing", "increase", "higher", "rise"],
    },
    Emoji {
        code: "1f4c9",
        label: "Chart down",
        words: &["decrease", "lower", "drop", "dropped", "fall", "decline"],
    },
    Emoji {
        code: "23f0",
        label: "Alarm clock",
        words: &["deadline", "late", "morning", "hours"],
    },
    Emoji {
        code: "1f914",
        label: "Thinking",
        words: &["question", "wonder", "wondering", "hmm"],
    },
    Emoji {
        code: "1f62e",
        label: "Surprised",
        words: &["wow", "surprised", "surprise", "whoa"],
    },
    Emoji {
        code: "1f605",
        label: "Sweat smile",
        words: &["awkward", "oops", "nervous"],
    },
    Emoji {
        code: "1f60d",
        label: "Heart eyes",
        words: &["beautiful", "gorgeous", "adore", "obsessed"],
    },
    Emoji {
        code: "1f64f",
        label: "Folded hands",
        words: &["thanks", "thank", "grateful", "pray"],
    },
    Emoji {
        code: "1f4aa",
        label: "Strong",
        words: &["strong", "strength", "gym", "discipline", "effort"],
    },
    Emoji {
        code: "1f449",
        label: "Pointing",
        words: &["tip", "tips", "step", "steps"],
    },
    Emoji {
        code: "1f389",
        label: "Party",
        words: &["party", "win", "won", "celebrate", "success", "successful"],
    },
    Emoji {
        code: "2b50",
        label: "Star",
        words: &["star", "favorite", "favourite"],
    },
    Emoji {
        code: "1f9e0",
        label: "Brain",
        words: &["brain", "smart", "learn", "learning", "knowledge", "study"],
    },
    Emoji {
        code: "1f4ac",
        label: "Speech",
        words: &[
            "conversation",
            "podcast",
            "interview",
            "discuss",
            "discussion",
        ],
    },
    Emoji {
        code: "1f511",
        label: "Key",
        words: &["key", "secret", "secrets", "unlock", "trick"],
    },
    Emoji {
        code: "1f6d1",
        label: "Stop",
        words: &["stop", "quit"],
    },
    Emoji {
        code: "1f91d",
        label: "Handshake",
        words: &["deal", "partner", "partnership", "team", "agree"],
    },
    Emoji {
        code: "1f60e",
        label: "Cool",
        words: &["cool", "confident", "chill"],
    },
    Emoji {
        code: "1f976",
        label: "Cold",
        words: &["cold", "freezing", "frozen"],
    },
    Emoji {
        code: "1f911",
        label: "Money face",
        words: &["rich", "millionaire", "million", "millions", "billion"],
    },
    Emoji {
        code: "1f4f1",
        label: "Phone",
        words: &["phone", "app", "apps", "social", "instagram", "tiktok"],
    },
    Emoji {
        code: "1f4bb",
        label: "Laptop",
        words: &["computer", "code", "coding", "software", "laptop"],
    },
    Emoji {
        code: "1f4da",
        label: "Books",
        words: &["book", "books", "reading", "school"],
    },
    Emoji {
        code: "1f3c6",
        label: "Trophy",
        words: &["champion", "winner", "award"],
    },
    Emoji {
        code: "1f480",
        label: "Skull",
        words: &["dead", "dying", "died", "skull"],
    },
];

/// The emoji with this code, when it is one of these.
pub fn emoji(code: &str) -> Option<&'static Emoji> {
    EMOJI.iter().find(|emoji| emoji.code == code)
}

/// The emoji a caption word calls for, after its punctuation is taken off.
pub fn emoji_for_word(word: &str) -> Option<&'static Emoji> {
    let plain = word
        .trim_matches(|character: char| !character.is_alphanumeric())
        .to_lowercase();
    if plain.is_empty() {
        return None;
    }
    EMOJI
        .iter()
        .find(|emoji| emoji.words.contains(&plain.as_str()))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::{EMOJI, emoji, emoji_for_word};

    /// Every emoji offered is one the bill of materials pins, and the other
    /// way round: offering one without its picture would fail a render.
    #[test]
    fn the_catalogue_is_the_pinned_set() {
        let bom = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bom.toml"),
        )
        .expect("bom.toml");
        let ids = bom
            .lines()
            .skip_while(|line| *line != "[emoji]")
            .find_map(|line| line.strip_prefix("ids = "))
            .expect("emoji ids")
            .trim_matches('"')
            .split(',')
            .collect::<Vec<_>>();
        let codes = EMOJI.iter().map(|emoji| emoji.code).collect::<Vec<_>>();
        assert_eq!(codes, ids);
        let mut words = EMOJI
            .iter()
            .flat_map(|emoji| emoji.words.iter())
            .collect::<Vec<_>>();
        let count = words.len();
        words.sort_unstable();
        words.dedup();
        assert_eq!(words.len(), count, "a word calls for one emoji");
    }

    #[test]
    fn a_word_calls_for_its_emoji_whatever_its_case_or_punctuation() {
        assert_eq!(
            emoji_for_word("Money,").map(|found| found.code),
            Some("1f4b0")
        );
        assert_eq!(
            emoji_for_word("“idea”").map(|found| found.code),
            Some("1f4a1")
        );
        assert!(emoji_for_word("...").is_none());
        // Words every sentence has call for nothing.
        for everyday in ["the", "think", "time", "see", "say", "yes", "best", "here"] {
            assert!(emoji_for_word(everyday).is_none(), "{everyday}");
        }
        assert_eq!(emoji("2764").unwrap().file(), "emoji_u2764.png");
    }
}
