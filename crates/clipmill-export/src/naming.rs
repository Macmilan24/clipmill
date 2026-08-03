//! What the files are called, decided once.
//!
//! A naming pattern is the sort of thing that gets implemented twice — once
//! where the preview is drawn and once where the file is written — and then
//! drifts, so the preview a user approved is not the name they got. This is the
//! only implementation. The shell asks the daemon what a pattern resolves to
//! and shows the answer; it does not compute one.
//!
//! Everything a token expands to arrives in [`Fields`]. Nothing here reads a
//! clock, a filesystem, or an environment: a date in a filename is a fact the
//! caller supplies, because a pure function that reads the day is a function
//! whose output nobody can reproduce.

use std::fmt;

/// The longest a single resolved name may be, before the extension.
///
/// 120 rather than the 255 most filesystems allow, because the name is joined
/// to a folder path a user chose and the limit that bites is the whole path.
/// Truncation is by character so a multi-byte name cannot be cut mid-scalar.
const MAX_STEM_CHARACTERS: usize = 120;

/// What a pattern may refer to.
///
/// Adding a token is a compatibility question — a pattern saved by a user must
/// keep resolving — so they are an enum rather than free-form lookup, and an
/// unknown one is an error at validation rather than an empty string at
/// delivery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Token {
    /// The project the clip belongs to.
    Project,
    /// The clip's own title, which is the candidate's headline when it has one.
    Clip,
    /// Ordinal within this export, one-based.
    Index,
    /// Whole seconds of program duration.
    Duration,
    /// The calendar date the export was requested, `YYYY-MM-DD`, supplied.
    Date,
    /// First eight characters of the render's content address.
    Address,
}

impl Token {
    const ALL: [Self; 6] = [
        Self::Project,
        Self::Clip,
        Self::Index,
        Self::Duration,
        Self::Date,
        Self::Address,
    ];

    /// The word between the braces.
    pub fn key(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Clip => "clip",
            Self::Index => "index",
            Self::Duration => "duration",
            Self::Date => "date",
            Self::Address => "address",
        }
    }

    fn parse(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|token| token.key() == key)
    }

    /// Every token, for a surface that has to list them.
    pub fn all() -> [Self; 6] {
        Self::ALL
    }
}

impl fmt::Display for Token {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{{{}}}", self.key())
    }
}

/// What the tokens stand for, for one clip.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fields {
    pub project: String,
    pub clip: String,
    pub index: u32,
    pub duration_seconds: u64,
    /// `YYYY-MM-DD`, supplied by the caller that has a clock.
    pub date: String,
    /// The render's content address, full; the token takes its first eight.
    pub address: String,
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum PatternError {
    #[error("`{{{key}}}` is not a name this pattern can use")]
    UnknownToken { key: String },
    #[error("a `{{` in the pattern is never closed")]
    UnclosedToken,
    #[error("the pattern resolves to an empty name")]
    Empty,
    #[error("the pattern names no clip, so every file in an export would collide")]
    NotUnique,
}

/// A validated naming pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pattern {
    parts: Vec<Part>,
    source: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Part {
    Literal(String),
    Token(Token),
}

impl Pattern {
    /// What a user gets before they have an opinion.
    ///
    /// Index first so a folder sorts in the order the results board showed, and
    /// the clip's own words after it so the file is recognisable without being
    /// opened.
    pub const DEFAULT: &'static str = "{index}-{clip}";

    /// Parse a pattern, refusing the two ways one can be wrong: a token nobody
    /// implements, and a pattern that would name every clip the same thing.
    pub fn parse(pattern: &str) -> Result<Self, PatternError> {
        let mut parts = Vec::new();
        let mut literal = String::new();
        let mut rest = pattern;
        while let Some(open) = rest.find('{') {
            literal.push_str(&rest[..open]);
            let after = &rest[open + 1..];
            let Some(close) = after.find('}') else {
                return Err(PatternError::UnclosedToken);
            };
            let key = &after[..close];
            let Some(token) = Token::parse(key) else {
                return Err(PatternError::UnknownToken {
                    key: key.to_owned(),
                });
            };
            if !literal.is_empty() {
                parts.push(Part::Literal(std::mem::take(&mut literal)));
            }
            parts.push(Part::Token(token));
            rest = &after[close + 1..];
        }
        literal.push_str(rest);
        if !literal.is_empty() {
            parts.push(Part::Literal(literal));
        }
        if parts.is_empty() {
            return Err(PatternError::Empty);
        }
        // A pattern with neither an index nor the clip's own words gives every
        // clip in one export the same stem, and the second delivery would
        // overwrite the first. Refusing is kinder than de-duplicating silently.
        let unique = parts.iter().any(|part| {
            matches!(
                part,
                Part::Token(Token::Index | Token::Clip | Token::Address)
            )
        });
        if !unique {
            return Err(PatternError::NotUnique);
        }
        Ok(Self {
            parts,
            source: pattern.to_owned(),
        })
    }

    /// The pattern as it was written, for round-tripping through storage.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// The filename stem for one clip. Never empty, never a path.
    pub fn resolve(&self, fields: &Fields) -> String {
        let mut out = String::new();
        for part in &self.parts {
            match part {
                Part::Literal(text) => out.push_str(text),
                Part::Token(token) => out.push_str(&expand(*token, fields)),
            }
        }
        let cleaned = sanitize(&out);
        if cleaned.is_empty() {
            // Every token can expand to nothing — an untitled clip, a project
            // named only in an alphabet the sanitizer strips. A name is still
            // required, so the address answers, and it is always there.
            return sanitize(&short_address(&fields.address));
        }
        cleaned
    }
}

fn expand(token: Token, fields: &Fields) -> String {
    match token {
        Token::Project => fields.project.clone(),
        Token::Clip => fields.clip.clone(),
        Token::Index => format!("{:02}", fields.index),
        Token::Duration => format!("{}s", fields.duration_seconds),
        Token::Date => fields.date.clone(),
        Token::Address => short_address(&fields.address),
    }
}

fn short_address(address: &str) -> String {
    address.chars().take(8).collect()
}

/// Reduce a resolved name to something every target filesystem accepts.
///
/// Deliberately narrow rather than clever: letters, digits, and three
/// separators. Windows is a Phase 2 target and its reserved names (`CON`,
/// `NUL`, trailing dots) are the reason a name is never allowed to be pure
/// punctuation or to end in one — dealing with that here costs nothing and
/// dealing with it later would mean renaming files a user already has.
fn sanitize(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut last_was_separator = false;
    for character in name.chars() {
        // Letters and digits outside ASCII are kept: a title in another script
        // is a title, and every filesystem this ships on stores UTF-8.
        let mapped = if character.is_alphanumeric() {
            last_was_separator = false;
            character
        } else if matches!(character, '-' | '_' | '.') {
            // Kept as written. A user who typed an underscore into a pattern
            // meant an underscore, and rewriting it to a hyphen would mean the
            // preview and the pattern disagree about what the user asked for.
            if last_was_separator {
                continue;
            }
            last_was_separator = true;
            character
        } else {
            // Everything else — whitespace, path separators, punctuation — was
            // standing between two words, so it becomes the one separator that
            // every filesystem takes. Dropping it instead would run the words
            // together: "A/B testing" is not "AB-testing".
            if last_was_separator {
                continue;
            }
            last_was_separator = true;
            '-'
        };
        out.push(mapped);
        if out.chars().count() >= MAX_STEM_CHARACTERS {
            break;
        }
    }
    out.trim_matches(['-', '_', '.', ' ']).to_owned()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::{Fields, Pattern, PatternError, Token};

    fn fields() -> Fields {
        Fields {
            project: "Pricing Talk".to_owned(),
            clip: "Charging less is lying to yourself".to_owned(),
            index: 3,
            duration_seconds: 52,
            date: "2026-08-03".to_owned(),
            address: "7f3c9ab21d5e4408aa".to_owned(),
        }
    }

    #[test]
    fn the_default_pattern_names_a_clip_by_its_place_and_its_words() {
        let pattern = Pattern::parse(Pattern::DEFAULT).expect("the default parses");
        assert_eq!(
            pattern.resolve(&fields()),
            "03-Charging-less-is-lying-to-yourself"
        );
    }

    #[test]
    fn every_token_expands_to_something_a_filesystem_takes() {
        let pattern =
            Pattern::parse("{project}_{clip}_{index}_{duration}_{date}_{address}").expect("parses");
        assert_eq!(
            pattern.resolve(&fields()),
            "Pricing-Talk_Charging-less-is-lying-to-yourself_03_52s_2026-08-03_7f3c9ab2"
        );
    }

    #[test]
    fn a_token_nobody_implements_is_refused_rather_than_left_blank() {
        assert_eq!(
            Pattern::parse("{clip}-{episode}"),
            Err(PatternError::UnknownToken {
                key: "episode".to_owned()
            })
        );
    }

    #[test]
    fn an_unclosed_brace_is_refused() {
        assert_eq!(Pattern::parse("{clip"), Err(PatternError::UnclosedToken));
    }

    #[test]
    fn a_pattern_that_would_name_every_clip_the_same_is_refused() {
        // Two clips exported under this would collide on the second write.
        assert_eq!(Pattern::parse("{project}"), Err(PatternError::NotUnique));
        assert_eq!(
            Pattern::parse("{date}-{duration}"),
            Err(PatternError::NotUnique)
        );
        assert!(Pattern::parse("{project}-{index}").is_ok());
    }

    #[test]
    fn separators_are_a_path_the_pattern_cannot_escape() {
        let pattern = Pattern::parse("{clip}").expect("parses");
        let mut escaping = fields();
        escaping.clip = "../../etc/passwd".to_owned();
        let resolved = pattern.resolve(&escaping);
        assert!(!resolved.contains('/'), "resolved to {resolved}");
        assert!(!resolved.starts_with('.'), "resolved to {resolved}");
        assert_eq!(resolved, "etc-passwd");
    }

    #[test]
    fn a_name_that_sanitizes_to_nothing_falls_back_to_the_address() {
        let pattern = Pattern::parse("{clip}").expect("parses");
        let mut untitled = fields();
        untitled.clip = "!!!".to_owned();
        assert_eq!(pattern.resolve(&untitled), "7f3c9ab2");
    }

    #[test]
    fn a_long_title_is_cut_at_a_character_rather_than_a_byte() {
        let pattern = Pattern::parse("{clip}").expect("parses");
        let mut long = fields();
        long.clip = "é".repeat(400);
        let resolved = pattern.resolve(&long);
        assert!(resolved.chars().count() <= 120);
        // The proof it was not cut mid-scalar: it is still valid UTF-8 of the
        // character we put in, all the way to the end.
        assert!(resolved.chars().all(|character| character == 'é'));
    }

    #[test]
    fn a_pattern_round_trips_through_its_own_text() {
        let pattern = Pattern::parse("{index}_{clip}").expect("parses");
        assert_eq!(pattern.source(), "{index}_{clip}");
        assert_eq!(Pattern::parse(pattern.source()), Ok(pattern));
    }

    #[test]
    fn every_token_is_listed_and_every_listed_token_parses() {
        for token in Token::all() {
            let written = token.to_string();
            let pattern = Pattern::parse(&format!("{written}-{{index}}"));
            assert!(pattern.is_ok(), "{written} did not parse");
        }
    }
}
