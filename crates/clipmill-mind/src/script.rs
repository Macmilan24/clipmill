//! The annotated script: a recording as the text a language model reads
//! (architecture §4.3, tier 1).
//!
//! Who speaks, what they say, where they pause, where the audience laughs and
//! where the picture cuts are all facts in the Timeline Index. Written into one
//! text with unit ids, they are something a model can read, cite and plan
//! from, and textualised cues are what the 2026 work on laughter found models
//! reason over best.
//!
//! Three levels trade detail for length. `Full` is every sentence with every
//! cue; `Compact` folds a speaker's turn into one line and keeps the cues that
//! matter for editing; `Outline` keeps each chapter's opening and the lines
//! that drew a reaction. The text is a pure function of its source: the same
//! index gives the same bytes, so a script is a stable prompt prefix that a
//! model host can cache.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::SECOND;

/// A sentence of the transcript, as the Timeline Index holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sentence {
    /// Its index in the recording; the script cites it as `s{index}`.
    pub index: u64,
    pub start_ticks: u64,
    pub end_ticks: u64,
    /// The diarised voice, when the recording has more than one.
    pub speaker: Option<String>,
    pub text: String,
}

/// Something heard or seen that is not words: laughter, applause, music, an
/// emphasis, a description of what is on screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cue {
    pub kind: String,
    pub start_ticks: u64,
    pub end_ticks: u64,
    /// A few words the cue carries, such as a scene description.
    pub label: Option<String>,
}

/// Where a topic starts, and the words that name it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chapter {
    pub start_ticks: u64,
    pub keywords: Vec<String>,
}

/// Everything a script is written from. Sentences are in time order; cues,
/// cuts and chapters may come in any order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Source {
    pub duration_ticks: u64,
    pub sentences: Vec<Sentence>,
    pub cues: Vec<Cue>,
    /// Shot boundaries.
    pub cuts: Vec<u64>,
    pub chapters: Vec<Chapter>,
}

/// How much of the recording a script spells out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Full,
    Compact,
    Outline,
}

impl Level {
    pub const ALL: [Self; 3] = [Self::Full, Self::Compact, Self::Outline];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Compact => "compact",
            Self::Outline => "outline",
        }
    }
}

/// A written script and what it costs to read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Script {
    pub level: Level,
    pub text: String,
    pub tokens: u64,
}

/// A gap between sentences worth showing at `Full`: longer than a breath.
pub const FULL_PAUSE_TICKS: u64 = 63_000;
/// A gap inside a turn worth showing at `Compact`: a deliberate silence.
pub const COMPACT_PAUSE_TICKS: u64 = 2 * SECOND;
/// The most sentences one `Compact` line folds together, so every line stays
/// short enough to cite.
pub const COMPACT_TURN_SENTENCES: usize = 6;
/// An outline line is a sentence's first words, not all of it.
pub const OUTLINE_WORDS: usize = 14;
/// Lines an outline keeps per chapter besides its opening.
pub const OUTLINE_REACTIONS: usize = 3;
/// Where a recording has no topics, the outline groups it by this much time.
pub const OUTLINE_BLOCK_TICKS: u64 = 180 * SECOND;

/// The cues that mean the room responded to what was just said.
const REACTIONS: [&str; 5] = ["laughter", "applause", "cheering", "shout", "gasp"];

/// A conservative token count for text a model will read.
///
/// No tokenizer is loaded here, and the models differ, so this errs long:
/// English prose runs near four characters a token in the tokenizers ClipMill
/// uses, while times and unit ids run nearer two or three. Three keeps a script
/// that is said to fit a budget inside it.
pub fn estimate_tokens(text: &str) -> u64 {
    (text.chars().count() as u64).div_ceil(3)
}

/// The script of a recording, or of the part of it inside `range`, at `level`.
///
/// Speaker letters and shares are taken from the whole recording, so the same
/// voice has the same letter in every range a model reads.
pub fn render(source: &Source, level: Level, range: Option<(u64, u64)>) -> Script {
    let (lo, hi) = range.unwrap_or((0, source.duration_ticks.max(1)));
    let view = View::new(source, lo, hi);
    let mut text = header(source, &view, level);
    match level {
        Level::Full => full(&view, &mut text),
        Level::Compact => compact(&view, &mut text),
        Level::Outline => outline(&view, &mut text),
    }
    let tokens = estimate_tokens(&text);
    Script {
        level,
        text,
        tokens,
    }
}

/// The most detailed script of `range` that fits in `budget` tokens, or
/// `None` when not even the outline does.
pub fn fit(source: &Source, budget: u64, range: Option<(u64, u64)>) -> Option<Script> {
    Level::ALL
        .into_iter()
        .map(|level| render(source, level, range))
        .find(|script| script.tokens <= budget)
}

/// A stretch of the script read in one call, and the sentences it may cite.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Window {
    /// Ticks from the window's first sentence to the next window's first.
    pub range: (u64, u64),
    /// The sentences this window owns. Every sentence belongs to exactly one
    /// window, so nothing the model reads twice can be cited twice.
    pub units: Vec<u64>,
    pub script: Script,
}

/// Allowance per sentence on top of its words: the unit id, the time, the
/// voice, and its share of pauses, cues and topic lines.
const LINE_OVERHEAD_TOKENS: u64 = 12;
/// Allowance for a window's two header lines.
const HEADER_TOKENS: u64 = 40;

/// The script cut into windows of about `budget` tokens each, for a model
/// that cannot or should not read the whole recording at once.
///
/// A window ends where a voice changes when there is such a place in its
/// second half, so a question and its answer are rarely split; a single
/// sentence longer than the budget is a window of its own.
pub fn windows(source: &Source, level: Level, budget: u64) -> Vec<Window> {
    let room = budget.saturating_sub(HEADER_TOKENS).max(1);
    let cost = |sentence: &Sentence| estimate_tokens(&sentence.text) + LINE_OVERHEAD_TOKENS;
    let mut groups: Vec<Vec<&Sentence>> = Vec::new();
    let mut current: Vec<&Sentence> = Vec::new();
    let mut spent = 0;
    for sentence in &source.sentences {
        if !current.is_empty() && spent + cost(sentence) > room {
            let cut = (current.len().div_ceil(2).max(1)..current.len())
                .rev()
                .find(|&k| current[k].speaker != current[k - 1].speaker)
                .unwrap_or(current.len());
            let tail = current.split_off(cut);
            groups.push(current);
            spent = tail.iter().map(|s| cost(s)).sum();
            current = tail;
        }
        spent += cost(sentence);
        current.push(sentence);
    }
    if !current.is_empty() {
        groups.push(current);
    }
    let starts: Vec<u64> = groups
        .iter()
        .enumerate()
        .map(|(i, group)| if i == 0 { 0 } else { group[0].start_ticks })
        .collect();
    groups
        .iter()
        .enumerate()
        .map(|(i, group)| {
            let range = (
                starts[i],
                starts
                    .get(i + 1)
                    .copied()
                    .unwrap_or(source.duration_ticks.max(1)),
            );
            Window {
                range,
                units: group.iter().map(|s| s.index).collect(),
                script: render(source, level, Some(range)),
            }
        })
        .collect()
}

/// The source cut down to one range, with what every line needs to know.
struct View<'a> {
    sentences: Vec<&'a Sentence>,
    cues: Vec<&'a Cue>,
    cuts: Vec<u64>,
    chapters: Vec<&'a Chapter>,
    letters: BTreeMap<&'a str, String>,
    long: bool,
    lo: u64,
    hi: u64,
}

/// The letter the script gives each voice, by speaker id: A, B, … in order
/// of first appearance, the same in every range of the recording.
pub fn speaker_letters(source: &Source) -> BTreeMap<&str, String> {
    let mut letters = BTreeMap::new();
    for speaker in source.sentences.iter().filter_map(|s| s.speaker.as_deref()) {
        let next = letters.len();
        letters.entry(speaker).or_insert_with(|| letter(next));
    }
    letters
}

impl<'a> View<'a> {
    fn new(source: &'a Source, lo: u64, hi: u64) -> Self {
        let letters = speaker_letters(source);
        let mut cues: Vec<&Cue> = source
            .cues
            .iter()
            .filter(|cue| cue.end_ticks > lo && cue.start_ticks < hi)
            .collect();
        cues.sort_by(|a, b| {
            (a.start_ticks, a.end_ticks, &a.kind).cmp(&(b.start_ticks, b.end_ticks, &b.kind))
        });
        let mut shot_cuts: Vec<u64> = source
            .cuts
            .iter()
            .copied()
            .filter(|cut| (lo..hi).contains(cut))
            .collect();
        shot_cuts.sort_unstable();
        shot_cuts.dedup();
        let mut chapters: Vec<&Chapter> = source
            .chapters
            .iter()
            .filter(|chapter| (lo..hi).contains(&chapter.start_ticks))
            .collect();
        chapters.sort_by_key(|chapter| chapter.start_ticks);
        Self {
            sentences: source
                .sentences
                .iter()
                .filter(|s| s.end_ticks > lo && s.start_ticks < hi)
                .collect(),
            cues,
            cuts: shot_cuts,
            chapters,
            letters,
            long: source.duration_ticks >= 3600 * SECOND,
            lo,
            hi,
        }
    }

    fn who(&self, sentence: &Sentence) -> Option<&str> {
        sentence
            .speaker
            .as_deref()
            .and_then(|speaker| self.letters.get(speaker))
            .map(String::as_str)
    }

    fn stamp(&self, ticks: u64, tenths: bool) -> String {
        stamp(ticks, self.long, tenths)
    }
}

/// A, B, … Z, then A2, B2, …, in order of first appearance.
fn letter(position: usize) -> String {
    let base = char::from(b'A' + u8::try_from(position % 26).unwrap_or(0));
    match position / 26 {
        0 => base.to_string(),
        round => format!("{base}{}", round + 1),
    }
}

/// `mm:ss.d`, or `h:mm:ss.d` in a recording an hour or longer.
fn stamp(ticks: u64, long: bool, tenths: bool) -> String {
    let total_tenths = ticks / (SECOND / 10);
    let seconds = total_tenths / 10;
    let (hours, minutes, seconds) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    let mut out = if long {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    };
    if tenths {
        let _ = write!(out, ".{}", total_tenths % 10);
    }
    out
}

/// `1.8 s` under ten seconds, whole seconds above.
fn duration(ticks: u64) -> String {
    let tenths = (ticks + SECOND / 20) / (SECOND / 10);
    if tenths >= 100 {
        format!("{} s", (tenths + 5) / 10)
    } else {
        format!("{}.{} s", tenths / 10, tenths % 10)
    }
}

fn header(source: &Source, view: &View<'_>, level: Level) -> String {
    let mut text = format!(
        "# Recording {} · {} sentences · script {}",
        stamp(source.duration_ticks, view.long, false),
        source.sentences.len(),
        level.as_str(),
    );
    if (view.lo, view.hi) != (0, source.duration_ticks.max(1)) {
        let _ = write!(
            text,
            " · range {}-{}",
            view.stamp(view.lo, true),
            view.stamp(view.hi, true)
        );
    }
    text.push('\n');
    if !view.letters.is_empty() {
        let mut talk: BTreeMap<&str, u64> = BTreeMap::new();
        for sentence in &source.sentences {
            if let Some(speaker) = sentence.speaker.as_deref() {
                *talk.entry(speaker).or_default() +=
                    sentence.end_ticks.saturating_sub(sentence.start_ticks);
            }
        }
        let total = talk.values().sum::<u64>().max(1);
        let mut voices: Vec<(&String, u64)> = view
            .letters
            .iter()
            .map(|(speaker, letter)| (letter, talk.get(speaker).copied().unwrap_or(0)))
            .collect();
        voices.sort_by(|a, b| a.0.len().cmp(&b.0.len()).then(a.0.cmp(b.0)));
        let shares: Vec<String> = voices
            .iter()
            .map(|(letter, ticks)| format!("{letter} {}%", (ticks * 100 + total / 2) / total))
            .collect();
        let _ = writeln!(text, "# Speakers: {}", shares.join(" · "));
    }
    text
}

fn chapter_line(view: &View<'_>, chapter: &Chapter, text: &mut String) {
    let _ = writeln!(
        text,
        "## [{}] topic: {}",
        view.stamp(chapter.start_ticks, false),
        chapter.keywords.join(", ")
    );
}

fn cue_line(view: &View<'_>, cue: &Cue, text: &mut String) {
    let _ = write!(
        text,
        "[{}] [{} {}",
        view.stamp(cue.start_ticks, true),
        cue.kind,
        duration(cue.end_ticks.saturating_sub(cue.start_ticks))
    );
    if let Some(label) = &cue.label {
        let _ = write!(text, ": {label}");
    }
    text.push_str("]\n");
}

/// Chapters and cues interleaved with the lines they fall between.
///
/// A cue that starts during a line is written after it, because what an
/// audience does follows what was said; one that starts between lines is
/// written before the next.
struct Interleave {
    cue: usize,
    chapter: usize,
}

impl Interleave {
    /// Everything that starts before a line at `at`: chapters that open at or
    /// before it and cues that begin before it, in time order.
    fn before(&mut self, view: &View<'_>, at: u64, text: &mut String) {
        while let Some(chapter) = view.chapters.get(self.chapter) {
            if chapter.start_ticks > at {
                break;
            }
            self.cues_until(view, chapter.start_ticks, text);
            chapter_line(view, chapter, text);
            self.chapter += 1;
        }
        self.cues_until(view, at, text);
    }

    fn cues_until(&mut self, view: &View<'_>, until: u64, text: &mut String) {
        while let Some(cue) = view.cues.get(self.cue) {
            if cue.start_ticks >= until {
                break;
            }
            cue_line(view, cue, text);
            self.cue += 1;
        }
    }

    fn rest(&mut self, view: &View<'_>, text: &mut String) {
        self.before(view, u64::MAX, text);
    }
}

fn full(view: &View<'_>, text: &mut String) {
    let mut around = Interleave { cue: 0, chapter: 0 };
    for (position, sentence) in view.sentences.iter().enumerate() {
        around.before(view, sentence.start_ticks, text);
        let _ = write!(
            text,
            "s{} [{}] ",
            sentence.index,
            view.stamp(sentence.start_ticks, true)
        );
        if let Some(who) = view.who(sentence) {
            let _ = write!(text, "{who}: ");
        }
        text.push_str(sentence.text.trim());
        let cuts = view
            .cuts
            .iter()
            .filter(|&&cut| cut > sentence.start_ticks && cut <= sentence.end_ticks)
            .count();
        match cuts {
            0 => {}
            1 => text.push_str(" [cut]"),
            many => {
                let _ = write!(text, " [{many} cuts]");
            }
        }
        if let Some(next) = view.sentences.get(position + 1) {
            let gap = next.start_ticks.saturating_sub(sentence.end_ticks);
            if gap >= FULL_PAUSE_TICKS {
                let _ = write!(text, " [pause {}]", duration(gap));
            }
        }
        text.push('\n');
        around.cues_until(view, sentence.end_ticks, text);
    }
    around.rest(view, text);
}

/// Consecutive sentences of one voice, up to the line limit.
fn turns<'v, 'a>(view: &'v View<'a>) -> Vec<&'v [&'a Sentence]> {
    let mut turns = Vec::new();
    let mut first = 0;
    for position in 1..=view.sentences.len() {
        let ends = match view.sentences.get(position) {
            None => true,
            Some(next) => {
                next.speaker != view.sentences[first].speaker
                    || position - first >= COMPACT_TURN_SENTENCES
            }
        };
        if ends {
            turns.push(&view.sentences[first..position]);
            first = position;
        }
    }
    turns
}

fn compact(view: &View<'_>, text: &mut String) {
    let mut around = Interleave { cue: 0, chapter: 0 };
    for turn in turns(view) {
        let (Some(first), Some(last)) = (turn.first(), turn.last()) else {
            continue;
        };
        around.before(view, first.start_ticks, text);
        if first.index == last.index {
            let _ = write!(text, "s{}", first.index);
        } else {
            let _ = write!(text, "s{}-s{}", first.index, last.index);
        }
        let _ = write!(text, " [{}] ", view.stamp(first.start_ticks, false));
        if let Some(who) = view.who(first) {
            let _ = write!(text, "{who}: ");
        }
        for (position, sentence) in turn.iter().enumerate() {
            if position > 0 {
                let gap = sentence
                    .start_ticks
                    .saturating_sub(turn[position - 1].end_ticks);
                if gap >= COMPACT_PAUSE_TICKS {
                    let _ = write!(text, " [pause {}]", duration(gap));
                }
                text.push(' ');
            }
            text.push_str(sentence.text.trim());
        }
        text.push('\n');
        around.cues_until(view, last.end_ticks, text);
    }
    around.rest(view, text);
}

/// A sentence's opening words.
fn opening(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() <= OUTLINE_WORDS {
        words.join(" ")
    } else {
        format!("{}…", words[..OUTLINE_WORDS].join(" "))
    }
}

fn outline(view: &View<'_>, text: &mut String) {
    // Sections: the chapters, or fixed blocks where the index has none.
    let mut starts: Vec<u64> = if view.chapters.is_empty() {
        (view.lo..view.hi)
            .step_by(usize::try_from(OUTLINE_BLOCK_TICKS).unwrap_or(usize::MAX))
            .collect()
    } else {
        view.chapters
            .iter()
            .map(|chapter| chapter.start_ticks)
            .collect()
    };
    if starts.first().is_none_or(|first| *first > view.lo) {
        starts.insert(0, view.lo);
    }
    for (position, &start) in starts.iter().enumerate() {
        let end = starts.get(position + 1).copied().unwrap_or(view.hi);
        let sentences: Vec<&Sentence> = view
            .sentences
            .iter()
            .copied()
            .filter(|s| s.start_ticks >= start && s.start_ticks < end)
            .collect();
        let cues: Vec<&Cue> = view
            .cues
            .iter()
            .copied()
            .filter(|cue| cue.start_ticks >= start && cue.start_ticks < end)
            .collect();
        if sentences.is_empty() && cues.is_empty() {
            continue;
        }
        section(view, start, end, &sentences, &cues, text);
    }
}

fn section(
    view: &View<'_>,
    start: u64,
    end: u64,
    sentences: &[&Sentence],
    cues: &[&Cue],
    text: &mut String,
) {
    let _ = write!(
        text,
        "## [{}-{}]",
        view.stamp(start, false),
        view.stamp(end, false)
    );
    if let (Some(first), Some(last)) = (sentences.first(), sentences.last()) {
        let _ = write!(text, " s{}-s{}", first.index, last.index);
    }
    let mut talk: BTreeMap<&str, u64> = BTreeMap::new();
    for sentence in sentences {
        if let Some(who) = view.who(sentence) {
            *talk.entry(who).or_default() +=
                sentence.end_ticks.saturating_sub(sentence.start_ticks);
        }
    }
    let total = talk.values().sum::<u64>().max(1);
    let mut voices: Vec<(&str, u64)> = talk.into_iter().collect();
    voices.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    for (who, ticks) in &voices {
        let _ = write!(text, " · {who} {}%", (ticks * 100 + total / 2) / total);
    }
    if let Some(chapter) = view.chapters.iter().find(|c| c.start_ticks == start)
        && !chapter.keywords.is_empty()
    {
        let _ = write!(text, " · {}", chapter.keywords.join(", "));
    }
    let mut kinds: BTreeMap<&str, usize> = BTreeMap::new();
    for cue in cues {
        *kinds.entry(cue.kind.as_str()).or_default() += 1;
    }
    for (kind, count) in &kinds {
        let _ = write!(text, " · {kind} {count}");
    }
    text.push('\n');

    // The opening line, then the lines the room answered.
    let mut kept: Vec<(&Sentence, Option<&str>)> = Vec::new();
    if let Some(first) = sentences.first() {
        kept.push((first, None));
    }
    for cue in cues
        .iter()
        .filter(|cue| REACTIONS.contains(&cue.kind.as_str()))
    {
        if kept.len() > OUTLINE_REACTIONS {
            break;
        }
        // The line said as, or just before, the reaction began.
        let Some(cause) = sentences
            .iter()
            .rev()
            .find(|s| s.start_ticks <= cue.start_ticks)
        else {
            continue;
        };
        match kept.iter_mut().find(|(s, _)| s.index == cause.index) {
            Some(entry) => entry.1 = entry.1.or(Some(cue.kind.as_str())),
            None => kept.push((cause, Some(cue.kind.as_str()))),
        }
    }
    kept.sort_by_key(|(sentence, _)| sentence.index);
    for (sentence, reaction) in kept {
        let _ = write!(text, "s{} ", sentence.index);
        if let Some(who) = view.who(sentence) {
            let _ = write!(text, "{who}: ");
        }
        text.push_str(&opening(&sentence.text));
        if let Some(kind) = reaction {
            let _ = write!(text, " [{kind}]");
        }
        text.push('\n');
    }
}

#[cfg(test)]
#[path = "script_tests.rs"]
mod tests;
