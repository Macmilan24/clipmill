//! A recording with the answers written down first.
//!
//! Every other test here checks that a candidate set is *well formed* — legal
//! boundaries, walkable evidence, a cluster for everything. None of them checks
//! that it found anything worth finding, because on a real recording nobody in
//! this repository can say what the right answer is.
//!
//! So the right answer is planted. This builds a recording whose moments are
//! known because they were put there: a question with its answer, a claim
//! carrying numbers and a strong verb, and a topic that opens and closes. Those
//! are the three things the three proposers are built to notice, one each, and
//! the moments are far enough apart that a candidate covering one cannot be
//! credited with another.
//!
//! What this proves is narrow and worth having: the mesh, the lattice, the
//! scorer, the boundary optimizer and the selector, run end to end, put the
//! planted moments in front of a user. It says nothing about a real recording —
//! that is what the private corpus and an editor's annotations are for — and it
//! needs no recognizer, which is why it can run on every push.
//!
//! The documents are written where the export drill's are: a gate outside this
//! crate scores them, so the recall arithmetic has exactly one implementation
//! and it is not this one.

use clipmill_contracts::schemas::index_transcript as index;
use clipmill_contracts::schemas::speech_transcript as transcript;

use crate::fixture::{SECOND, indexed, sentence, silence, spoken, topic, utterance, word};

/// One planted moment: where it is, and why it should be found.
pub(crate) struct Planted {
    pub moment_id: &'static str,
    pub start_ticks: u64,
    pub end_ticks: u64,
    /// What a miss would cost. Every planted moment is `essential`: the point
    /// of planting one is that failing to find it is a failure.
    pub importance: &'static str,
}

pub(crate) struct PlantedRecording {
    pub index: index::IndexTranscript,
    pub transcript: transcript::SpeechTranscript,
    pub moments: Vec<Planted>,
}

/// A block of sentences that belongs together, and whether it is an answer.
struct Block {
    moment_id: &'static str,
    lines: &'static [&'static str],
}

/// The three blocks, each written to be the kind of thing one proposer sees.
///
/// The wording is not decoration. The question mark is what the Q&A proposer
/// keys on; the numerals and the claim verb are what the insight proposer
/// scores; the open-and-close shape is what the narrative proposer reads as a
/// topic. A fixture written in bland prose would test the plumbing and nothing
/// else.
const BLOCKS: &[Block] = &[
    Block {
        moment_id: "question-and-answer",
        lines: &[
            "So how much does a bad thumbnail actually cost you?",
            "It costs about forty percent of your click-through rate.",
            "We measured it across two hundred uploads on the same channel.",
            "The videos were identical and only the thumbnail changed.",
            "Forty percent is the difference between a living and a hobby.",
        ],
    },
    Block {
        moment_id: "strong-claim",
        lines: &[
            "Charging less is lying to yourself about what the work is worth.",
            "I raised my rate by sixty percent and lost exactly one client.",
            "That client was costing me eleven hours a month in revisions.",
            "The arithmetic was never close once I actually wrote it down.",
            "Nobody ever told me that, so I am telling you.",
        ],
    },
    Block {
        moment_id: "narrative-arc",
        lines: &[
            "The first year I shipped nothing at all.",
            "Every week I rewrote the same three features from scratch.",
            "A friend finally asked me what I was actually afraid of.",
            "I published the broken version that Friday and nobody died.",
            "That is the whole reason anything exists now.",
        ],
    },
];

/// Between two planted moments. Long enough that a silence edge sits between
/// them and no legal boundary can span the gap, so a candidate cannot be
/// credited with two moments at once.
const BETWEEN: u64 = SECOND * 6;
/// Inside a block, a conversational beat.
const WITHIN: u64 = SECOND * 2 / 5;
/// Ticks a spoken word occupies. Slow enough that five sentences clear the
/// fifteen-second floor a clip has to reach.
const PER_WORD: u64 = SECOND * 3 / 5;

pub(crate) fn recording() -> PlantedRecording {
    let mut sentences = Vec::new();
    let mut utterances = Vec::new();
    let mut words = Vec::new();
    let mut topics = Vec::new();
    let mut edges = Vec::new();
    let mut moments = Vec::new();
    let mut next_word = 0u64;
    let mut at = 0u64;

    for (block_index, block) in BLOCKS.iter().enumerate() {
        if block_index > 0 {
            let gap_start = at;
            at += BETWEEN;
            // A real pause between moments, which is what closes an answer
            // window and what the lattice may cut on.
            edges.push(silence((gap_start, at)));
        }
        let first_sentence = sentences.len() as u64;
        let block_start = at;
        for (line_index, text) in block.lines.iter().enumerate() {
            if line_index > 0 {
                at += WITHIN;
            }
            let tokens = text.split_whitespace().count() as u64;
            let start = at;
            let end = start + tokens * PER_WORD;
            let position = sentences.len();
            sentences.push(sentence(
                position,
                position,
                (next_word, tokens),
                (start, end),
                text,
            ));
            utterances.push(utterance(position, (next_word, tokens), (start, end), text));
            for (offset, token) in text.split_whitespace().enumerate() {
                let token_at = start + offset as u64 * PER_WORD;
                words.push(word(
                    next_word + offset as u64,
                    (token_at, token_at + PER_WORD),
                    token,
                ));
            }
            next_word += tokens;
            at = end;
        }
        topics.push(topic(
            block_index,
            (first_sentence, block.lines.len() as u64),
            (block_start, at),
            // A depth the segmenter would report at a real boundary. Zero at
            // the opening, because nothing precedes the first topic.
            if block_index == 0 { 0.0 } else { 0.8 },
        ));
        moments.push(Planted {
            moment_id: block.moment_id,
            start_ticks: block_start,
            end_ticks: at,
            importance: "essential",
        });
    }

    let coverage = (0, at + SECOND);
    PlantedRecording {
        index: indexed(sentences, utterances, topics, edges, coverage),
        transcript: spoken(words, coverage),
        moments,
    }
}

#[cfg(test)]
mod tests;
