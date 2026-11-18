//! The Timeline Index (decision R74): one SQLite file per recording holding
//! what it contains, in time order and searchable.
//!
//! The index is derived, never edited. It is built once from a recording's
//! published artifacts (transcript, transcript index, speaker turns, shots)
//! and published as an artifact itself, so it caches and invalidates like
//! every other stage, and readers open it read-only. The schema is the
//! contract: `meta.schema_version` names it, and a reader refuses a file that
//! names another.
//!
//! Times are integer ticks at 1/90000 s, as everywhere in ClipMill.

use std::path::Path;

use rusqlite::{Connection, OpenFlags, OptionalExtension, params};

/// The version of the tables below. A change to them is a new version.
pub const SCHEMA_VERSION: &str = "clipmill.timeline.index.v1";

/// A pause between utterances at least this long becomes a `pause` event: a
/// beat a person hears, rather than the breath between phrases.
pub const PAUSE_EVENT_TICKS: u64 = 63_000;

#[derive(Debug, thiserror::Error)]
pub enum TimelineError {
    #[error("timeline index storage failed: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("the timeline index is {found}, not {SCHEMA_VERSION}")]
    Version { found: String },
    #[error("the recording's evidence is inconsistent: {0}")]
    Invalid(String),
}

/// One recognised word.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Word {
    pub index: u64,
    pub text: String,
    pub start_ticks: u64,
    pub end_ticks: u64,
}

/// One sentence of the transcript index, over a run of words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sentence {
    pub index: u64,
    pub text: String,
    pub start_ticks: u64,
    pub end_ticks: u64,
    pub first_word_index: u64,
    pub word_count: u64,
    pub utterance_index: u64,
}

/// One utterance: speech between pauses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Utterance {
    pub index: u64,
    pub start_ticks: u64,
    pub end_ticks: u64,
    pub pause_before_ticks: u64,
    pub pause_after_ticks: u64,
}

/// One lexical topic segment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Topic {
    pub index: u64,
    pub start_ticks: u64,
    pub end_ticks: u64,
    pub first_sentence_index: u64,
    pub sentence_count: u64,
    pub keywords: Vec<String>,
}

/// One stretch in which one voice speaks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Turn {
    pub speaker: String,
    pub start_ticks: u64,
    pub end_ticks: u64,
}

/// One shot between cuts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shot {
    pub start_ticks: u64,
    pub end_ticks: u64,
}

/// Something that happens at a time: a pause, laughter, applause, music, an
/// emphasis. Stages add kinds; the table does not enumerate them.
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub kind: String,
    pub start_ticks: u64,
    pub end_ticks: u64,
    pub value: Option<f64>,
    pub label: Option<String>,
    pub producer: String,
}

/// Everything one index is built from.
#[derive(Clone, Debug, Default)]
pub struct Recording {
    pub source_fingerprint: String,
    pub language: String,
    pub words: Vec<Word>,
    pub sentences: Vec<Sentence>,
    pub utterances: Vec<Utterance>,
    pub topics: Vec<Topic>,
    pub turns: Vec<Turn>,
    pub shots: Vec<Shot>,
    pub events: Vec<Event>,
}

/// What a build wrote.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    pub words: u64,
    pub sentences: u64,
    pub events: u64,
    pub shots: u64,
}

const SCHEMA: &str = "
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL) WITHOUT ROWID;
CREATE TABLE sentences (
    id INTEGER PRIMARY KEY,
    start_ticks INTEGER NOT NULL,
    end_ticks INTEGER NOT NULL,
    speaker TEXT,
    utterance INTEGER NOT NULL,
    first_word INTEGER NOT NULL,
    word_count INTEGER NOT NULL,
    text TEXT NOT NULL
);
CREATE INDEX sentences_time ON sentences (start_ticks, end_ticks);
CREATE TABLE words (
    id INTEGER PRIMARY KEY,
    start_ticks INTEGER NOT NULL,
    end_ticks INTEGER NOT NULL,
    sentence INTEGER,
    speaker TEXT,
    text TEXT NOT NULL
);
CREATE INDEX words_time ON words (start_ticks, end_ticks);
CREATE TABLE utterances (
    id INTEGER PRIMARY KEY,
    start_ticks INTEGER NOT NULL,
    end_ticks INTEGER NOT NULL,
    pause_before_ticks INTEGER NOT NULL,
    pause_after_ticks INTEGER NOT NULL
);
CREATE TABLE topics (
    id INTEGER PRIMARY KEY,
    start_ticks INTEGER NOT NULL,
    end_ticks INTEGER NOT NULL,
    first_sentence INTEGER NOT NULL,
    sentence_count INTEGER NOT NULL,
    keywords TEXT NOT NULL
);
CREATE TABLE turns (
    id INTEGER PRIMARY KEY,
    speaker TEXT NOT NULL,
    start_ticks INTEGER NOT NULL,
    end_ticks INTEGER NOT NULL
);
CREATE INDEX turns_time ON turns (start_ticks, end_ticks);
CREATE TABLE shots (
    id INTEGER PRIMARY KEY,
    start_ticks INTEGER NOT NULL,
    end_ticks INTEGER NOT NULL
);
CREATE TABLE events (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL,
    start_ticks INTEGER NOT NULL,
    end_ticks INTEGER NOT NULL,
    value REAL,
    label TEXT,
    producer TEXT NOT NULL
);
CREATE INDEX events_time ON events (kind, start_ticks);
CREATE VIRTUAL TABLE sentence_text USING fts5(
    text, content='sentences', content_rowid='id',
    tokenize='unicode61 remove_diacritics 2'
);
";

/// Write the index for `recording` to a new file at `path`.
///
/// The same recording always produces the same tables in the same order, and
/// the file is compacted before it is closed, so a rebuild matches the
/// published index row for row.
#[allow(
    clippy::too_many_lines,
    reason = "one transaction filling each table in turn; the order is the schema's"
)]
pub fn build(path: &Path, recording: &Recording) -> Result<Summary, TimelineError> {
    check(recording)?;
    let mut connection = Connection::open(path)?;
    // Built once, then published; no journal, no WAL, no temporary file
    // beside it. A failed build is discarded with its staging area.
    connection.execute_batch(
        "PRAGMA page_size = 4096;
         PRAGMA journal_mode = OFF;
         PRAGMA synchronous = OFF;
         PRAGMA temp_store = MEMORY;
         PRAGMA locking_mode = EXCLUSIVE;",
    )?;
    connection.execute_batch(SCHEMA)?;
    let transaction = connection.transaction()?;
    let mut summary = Summary::default();
    {
        let mut meta = transaction.prepare("INSERT INTO meta (key, value) VALUES (?1, ?2)")?;
        for (key, value) in [
            ("schema_version", SCHEMA_VERSION),
            ("source_fingerprint", recording.source_fingerprint.as_str()),
            ("language", recording.language.as_str()),
        ] {
            meta.execute(params![key, value])?;
        }

        let sentence_speakers: Vec<Option<&str>> = recording
            .sentences
            .iter()
            .map(|sentence| speaker_of(&recording.turns, sentence.start_ticks, sentence.end_ticks))
            .collect();
        let mut insert = transaction.prepare(
            "INSERT INTO sentences (id, start_ticks, end_ticks, speaker, utterance, first_word, word_count, text)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;
        for (sentence, speaker) in recording.sentences.iter().zip(&sentence_speakers) {
            insert.execute(params![
                sql(sentence.index),
                sql(sentence.start_ticks),
                sql(sentence.end_ticks),
                speaker,
                sql(sentence.utterance_index),
                sql(sentence.first_word_index),
                sql(sentence.word_count),
                sentence.text,
            ])?;
            summary.sentences += 1;
        }

        let mut insert = transaction.prepare(
            "INSERT INTO words (id, start_ticks, end_ticks, sentence, speaker, text)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;
        let mut owner = recording
            .sentences
            .iter()
            .zip(&sentence_speakers)
            .peekable();
        for word in &recording.words {
            while owner.peek().is_some_and(|(sentence, _)| {
                sentence
                    .first_word_index
                    .saturating_add(sentence.word_count)
                    <= word.index
            }) {
                owner.next();
            }
            let (sentence, speaker) = owner
                .peek()
                .filter(|(sentence, _)| sentence.first_word_index <= word.index)
                .map_or((None, None), |(sentence, speaker)| {
                    (Some(sql(sentence.index)), **speaker)
                });
            insert.execute(params![
                sql(word.index),
                sql(word.start_ticks),
                sql(word.end_ticks),
                sentence,
                speaker,
                word.text,
            ])?;
            summary.words += 1;
        }

        let mut insert = transaction.prepare(
            "INSERT INTO utterances (id, start_ticks, end_ticks, pause_before_ticks, pause_after_ticks)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        for utterance in &recording.utterances {
            insert.execute(params![
                sql(utterance.index),
                sql(utterance.start_ticks),
                sql(utterance.end_ticks),
                sql(utterance.pause_before_ticks),
                sql(utterance.pause_after_ticks),
            ])?;
        }

        let mut insert = transaction.prepare(
            "INSERT INTO topics (id, start_ticks, end_ticks, first_sentence, sentence_count, keywords)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;
        for topic in &recording.topics {
            insert.execute(params![
                sql(topic.index),
                sql(topic.start_ticks),
                sql(topic.end_ticks),
                sql(topic.first_sentence_index),
                sql(topic.sentence_count),
                topic.keywords.join(" "),
            ])?;
        }

        let mut insert = transaction
            .prepare("INSERT INTO turns (speaker, start_ticks, end_ticks) VALUES (?1, ?2, ?3)")?;
        for turn in &recording.turns {
            insert.execute(params![
                turn.speaker,
                sql(turn.start_ticks),
                sql(turn.end_ticks)
            ])?;
        }

        let mut insert =
            transaction.prepare("INSERT INTO shots (start_ticks, end_ticks) VALUES (?1, ?2)")?;
        for shot in &recording.shots {
            insert.execute(params![sql(shot.start_ticks), sql(shot.end_ticks)])?;
            summary.shots += 1;
        }

        let mut insert = transaction.prepare(
            "INSERT INTO events (kind, start_ticks, end_ticks, value, label, producer)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;
        for event in events(recording) {
            insert.execute(params![
                event.kind,
                sql(event.start_ticks),
                sql(event.end_ticks),
                event.value,
                event.label,
                event.producer,
            ])?;
            summary.events += 1;
        }

        transaction.execute(
            "INSERT INTO sentence_text (rowid, text) SELECT id, text FROM sentences ORDER BY id",
            [],
        )?;
    }
    transaction.commit()?;
    connection.execute_batch("PRAGMA locking_mode = NORMAL; VACUUM;")?;
    connection
        .close()
        .map_err(|(_, error)| TimelineError::Sqlite(error))?;
    Ok(summary)
}

/// The pauses the transcript index measured, followed by the recording's own
/// events, in time order: what the index stores as `events`.
fn events(recording: &Recording) -> Vec<Event> {
    let mut events: Vec<Event> = recording
        .utterances
        .iter()
        .filter(|utterance| utterance.pause_after_ticks >= PAUSE_EVENT_TICKS)
        .map(|utterance| Event {
            kind: "pause".to_owned(),
            start_ticks: utterance.end_ticks,
            end_ticks: utterance
                .end_ticks
                .saturating_add(utterance.pause_after_ticks),
            value: Some(seconds(utterance.pause_after_ticks)),
            label: None,
            producer: "clipmill-timeline".to_owned(),
        })
        .chain(recording.events.iter().cloned())
        .collect();
    events.sort_by(|left, right| {
        (left.start_ticks, left.end_ticks, &left.kind).cmp(&(
            right.start_ticks,
            right.end_ticks,
            &right.kind,
        ))
    });
    events
}

/// The voice that speaks for most of `[start, end)`, if any turn overlaps it.
/// Ties go to the earlier turn, so the choice never depends on input order
/// beyond the time order turns arrive in.
fn speaker_of(turns: &[Turn], start: u64, end: u64) -> Option<&str> {
    let mut best: Option<(&str, u64)> = None;
    for turn in turns {
        let overlap = end
            .min(turn.end_ticks)
            .saturating_sub(start.max(turn.start_ticks));
        if overlap > 0 && best.is_none_or(|(_, longest)| overlap > longest) {
            best = Some((turn.speaker.as_str(), overlap));
        }
    }
    best.map(|(speaker, _)| speaker)
}

fn check(recording: &Recording) -> Result<(), TimelineError> {
    let ordered = |pairs: &mut dyn Iterator<Item = (u64, u64)>, what: &str| {
        let mut previous = 0;
        for (start, end) in pairs {
            if end < start || start < previous {
                return Err(TimelineError::Invalid(format!(
                    "{what} are not in time order"
                )));
            }
            previous = start;
        }
        Ok(())
    };
    ordered(
        &mut recording.words.iter().map(|w| (w.start_ticks, w.end_ticks)),
        "words",
    )?;
    ordered(
        &mut recording
            .sentences
            .iter()
            .map(|s| (s.start_ticks, s.end_ticks)),
        "sentences",
    )?;
    ordered(
        &mut recording.turns.iter().map(|t| (t.start_ticks, t.end_ticks)),
        "speaker turns",
    )?;
    ordered(
        &mut recording.shots.iter().map(|s| (s.start_ticks, s.end_ticks)),
        "shots",
    )?;
    Ok(())
}

fn sql(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

#[allow(
    clippy::cast_precision_loss,
    reason = "seconds of a recording are far below 2^52 ticks"
)]
fn seconds(ticks: u64) -> f64 {
    ticks as f64 / 90_000.0
}

fn unsigned(value: i64) -> u64 {
    u64::try_from(value).unwrap_or(0)
}

/// A sentence as the index returns it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SentenceRow {
    pub index: u64,
    pub start_ticks: u64,
    pub end_ticks: u64,
    pub speaker: Option<String>,
    pub text: String,
}

/// An event as the index returns it.
#[derive(Clone, Debug, PartialEq)]
pub struct EventRow {
    pub kind: String,
    pub start_ticks: u64,
    pub end_ticks: u64,
    pub value: Option<f64>,
    pub label: Option<String>,
}

/// A published index, open read-only.
pub struct TimelineIndex {
    connection: Connection,
}

impl std::fmt::Debug for TimelineIndex {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TimelineIndex")
            .field("path", &self.connection.path())
            .finish()
    }
}

impl TimelineIndex {
    /// Open a published index without writing to it or beside it.
    pub fn open(path: &Path) -> Result<Self, TimelineError> {
        let connection = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        let found: String = connection.query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )?;
        if found != SCHEMA_VERSION {
            return Err(TimelineError::Version { found });
        }
        Ok(Self { connection })
    }

    /// One `meta` value, such as `source_fingerprint`.
    pub fn meta(&self, key: &str) -> Result<Option<String>, TimelineError> {
        Ok(self
            .connection
            .query_row("SELECT value FROM meta WHERE key = ?1", [key], |row| {
                row.get(0)
            })
            .optional()?)
    }

    /// The sentences that overlap `[start, end)`, in time order.
    pub fn sentences_between(
        &self,
        start_ticks: u64,
        end_ticks: u64,
    ) -> Result<Vec<SentenceRow>, TimelineError> {
        let mut statement = self.connection.prepare_cached(
            "SELECT id, start_ticks, end_ticks, speaker, text FROM sentences
             WHERE end_ticks > ?1 AND start_ticks < ?2 ORDER BY start_ticks, id",
        )?;
        let rows = statement.query_map(params![sql(start_ticks), sql(end_ticks)], sentence_row)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Sentences matching a full-text query, best match first. Words are
    /// matched as typed, ignoring case and accents; quotes match a phrase.
    pub fn search(&self, query: &str, limit: u32) -> Result<Vec<SentenceRow>, TimelineError> {
        let query = fts_query(query);
        if query.is_empty() {
            return Ok(Vec::new());
        }
        let mut statement = self.connection.prepare_cached(
            "SELECT s.id, s.start_ticks, s.end_ticks, s.speaker, s.text
             FROM sentence_text JOIN sentences s ON s.id = sentence_text.rowid
             WHERE sentence_text MATCH ?1
             ORDER BY bm25(sentence_text), s.start_ticks LIMIT ?2",
        )?;
        let rows = statement.query_map(params![query, limit], sentence_row)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Who is speaking at `ticks`, if anyone.
    pub fn speaker_at(&self, ticks: u64) -> Result<Option<String>, TimelineError> {
        Ok(self
            .connection
            .query_row(
                "SELECT speaker FROM turns WHERE start_ticks <= ?1 AND end_ticks > ?1
                 ORDER BY start_ticks DESC LIMIT 1",
                [sql(ticks)],
                |row| row.get(0),
            )
            .optional()?)
    }

    /// Events of the given kinds (all kinds when empty) overlapping
    /// `[start, end)`, in time order.
    pub fn events_between(
        &self,
        start_ticks: u64,
        end_ticks: u64,
        kinds: &[&str],
    ) -> Result<Vec<EventRow>, TimelineError> {
        let mut statement = self.connection.prepare_cached(
            "SELECT kind, start_ticks, end_ticks, value, label FROM events
             WHERE end_ticks > ?1 AND start_ticks < ?2 ORDER BY start_ticks, kind, id",
        )?;
        let rows = statement.query_map(params![sql(start_ticks), sql(end_ticks)], |row| {
            Ok(EventRow {
                kind: row.get(0)?,
                start_ticks: unsigned(row.get(1)?),
                end_ticks: unsigned(row.get(2)?),
                value: row.get(3)?,
                label: row.get(4)?,
            })
        })?;
        let mut events = rows.collect::<Result<Vec<_>, _>>()?;
        if !kinds.is_empty() {
            events.retain(|event| kinds.contains(&event.kind.as_str()));
        }
        Ok(events)
    }

    /// Where each topic starts and the words that name it, in time order.
    pub fn topics(&self) -> Result<Vec<(u64, Vec<String>)>, TimelineError> {
        let mut statement = self
            .connection
            .prepare_cached("SELECT start_ticks, keywords FROM topics ORDER BY start_ticks, id")?;
        let rows = statement.query_map([], |row| {
            let keywords: String = row.get(1)?;
            Ok((
                unsigned(row.get(0)?),
                keywords.split_whitespace().map(str::to_owned).collect(),
            ))
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Every word as `(start, end)`, in time order: the edges a cut may use.
    pub fn word_times(&self) -> Result<Vec<(u64, u64)>, TimelineError> {
        let mut statement = self
            .connection
            .prepare_cached("SELECT start_ticks, end_ticks FROM words ORDER BY start_ticks, id")?;
        let rows =
            statement.query_map([], |row| Ok((unsigned(row.get(0)?), unsigned(row.get(1)?))))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// The latest end of anything indexed: how far into the recording the
    /// index reaches.
    pub fn end_ticks(&self) -> Result<u64, TimelineError> {
        let end: Option<i64> = self.connection.query_row(
            "SELECT MAX(end_ticks) FROM (
                 SELECT end_ticks FROM sentences UNION ALL SELECT end_ticks FROM words
                 UNION ALL SELECT end_ticks FROM shots UNION ALL SELECT end_ticks FROM events)",
            [],
            |row| row.get(0),
        )?;
        Ok(end.map_or(0, unsigned))
    }

    /// The shots overlapping `[start, end)`, in time order.
    pub fn shots_between(
        &self,
        start_ticks: u64,
        end_ticks: u64,
    ) -> Result<Vec<Shot>, TimelineError> {
        let mut statement = self.connection.prepare_cached(
            "SELECT start_ticks, end_ticks FROM shots
             WHERE end_ticks > ?1 AND start_ticks < ?2 ORDER BY start_ticks",
        )?;
        let rows = statement.query_map(params![sql(start_ticks), sql(end_ticks)], |row| {
            Ok(Shot {
                start_ticks: unsigned(row.get(0)?),
                end_ticks: unsigned(row.get(1)?),
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
}

fn sentence_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SentenceRow> {
    Ok(SentenceRow {
        index: unsigned(row.get(0)?),
        start_ticks: unsigned(row.get(1)?),
        end_ticks: unsigned(row.get(2)?),
        speaker: row.get(3)?,
        text: row.get(4)?,
    })
}

/// A person's words as an FTS5 query: each word or quoted phrase must appear,
/// with every FTS5 operator and special character treated as text. A query is
/// data a person or an agent typed, never syntax they can inject.
fn fts_query(input: &str) -> String {
    let mut terms = Vec::new();
    for (position, part) in input.split('"').enumerate() {
        let phrase = position % 2 == 1;
        let words: Vec<String> = part
            .split(|character: char| !character.is_alphanumeric() && character != '\'')
            .map(|word| word.trim_matches('\''))
            .filter(|word| !word.is_empty())
            .map(str::to_owned)
            .collect();
        if words.is_empty() {
            continue;
        }
        if phrase {
            terms.push(format!("\"{}\"", words.join(" ")));
        } else {
            terms.extend(words.into_iter().map(|word| format!("\"{word}\"")));
        }
    }
    terms.join(" ")
}

#[cfg(test)]
mod tests;
