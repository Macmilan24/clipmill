//! The strip a clip has to get through before it becomes files.
//!
//! Four questions, and each one is a thing a user would otherwise discover
//! after uploading: has anybody said what this footage is, does a cut land
//! inside a word, can the sidecars actually be read at the speed they run, and
//! is there room on the disk. Every answer is a [`Finding`] carrying its own
//! reason, because "export failed" is not an answer anyone can act on.
//!
//! Two severities and the line between them is deliberate. **Blocking** means
//! the file would be wrong or would not fit; **advisory** means a person might
//! have meant it. The burn-in caption track is the case that proves the line is
//! real: it runs deliberately hot — a few words held briefly is the whole point
//! of the kinetic intent — so a reading-rate finding against it is advisory,
//! while the same finding against the accessibility cues is blocking, because
//! those are what leave the building as SRT and VTT and they are held to the
//! profile the caption engine exists to guarantee.
//!
//! Nothing here does any I/O. Disk headroom is checked against numbers the
//! caller measured, so this function is the same function in a pre-flight
//! preview and at delivery.

use clipmill_captions::{Profile, Violation, validate as validate_cues};
use clipmill_edit_ir::{CaptionCue, EditDocument};
use serde::{Deserialize, Serialize};

/// The gate token a render carries when a user has confirmed the rights on a
/// clip long enough to need it. The same string the render manifest records.
pub const DURATION_GATE: &str = "duration_60s";

/// Above this, the rights gate has to have been shown and passed.
///
/// Sixty seconds is where the platforms stop treating a clip as a short, which
/// is where a rights claim starts being worth something to somebody.
pub const RIGHTS_GATE_SECONDS: i64 = 60;

const TICKS_PER_SECOND: i64 = 90_000;

/// Bytes a second of delivered video is assumed to want.
///
/// 1080x1920 at CRF 18 is content-dependent by a factor of several, so this is
/// a ceiling drawn from the noisiest material rather than an average: a
/// pre-flight check that under-estimates is a check that lets an export start
/// and fail halfway, which is the failure this exists to prevent. The real byte
/// count replaces it the moment the render exists.
const ESTIMATED_BYTES_PER_SECOND: u64 = 2_500_000;

/// Free space that must remain after the export, beyond the export itself.
///
/// Filling a volume to the last byte breaks the machine, not just this feature.
const HEADROOM_RESERVE_BYTES: u64 = 512 * 1024 * 1024;

/// A generous ceiling on what a clip of this length will occupy on disk,
/// counting the sidecars as negligible because they are.
pub fn estimate_bytes(duration_ticks: i64) -> u64 {
    let seconds = duration_ticks.max(0) / TICKS_PER_SECOND + 1;
    u64::try_from(seconds).unwrap_or(0) * ESTIMATED_BYTES_PER_SECOND
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// The export does not start.
    Blocking,
    /// The export starts, and the user was told.
    Advisory,
}

/// What a check found, in the words the user reads.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Finding {
    /// Stable identifier a surface can key off, e.g. `rights.missing`.
    pub code: String,
    pub severity: Severity,
    /// One sentence, naming the thing and the number.
    pub detail: String,
}

impl Finding {
    fn blocking(code: &str, detail: String) -> Self {
        Self {
            code: code.to_owned(),
            severity: Severity::Blocking,
            detail,
        }
    }

    fn advisory(code: &str, detail: String) -> Self {
        Self {
            code: code.to_owned(),
            severity: Severity::Advisory,
            detail,
        }
    }
}

/// Everything the strip checks that is not in the document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Context<'a> {
    /// What the user attested about the footage. Empty means nobody said.
    pub source_attestation: &'a str,
    /// The gates the user has passed, e.g. [`DURATION_GATE`].
    pub gates_passed: &'a [String],
    /// Bytes this export is expected to write. [`estimate_bytes`] before a
    /// render exists; the measured size once one does.
    pub estimated_bytes: u64,
    /// Free space on the destination volume, or `None` when it could not be
    /// read — which is reported as its own finding rather than assumed to be
    /// fine.
    pub available_bytes: Option<u64>,
}

/// Everything the strip found, and whether that stops the export.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Report {
    pub findings: Vec<Finding>,
}

impl Report {
    /// True when nothing blocking was found. Advisories do not stop an export.
    pub fn passes(&self) -> bool {
        !self
            .findings
            .iter()
            .any(|finding| finding.severity == Severity::Blocking)
    }

    pub fn blocking(&self) -> impl Iterator<Item = &Finding> {
        self.findings
            .iter()
            .filter(|finding| finding.severity == Severity::Blocking)
    }
}

/// Run the strip.
pub fn validate(document: &EditDocument, context: &Context<'_>) -> Report {
    let mut findings = Vec::new();
    check_rights(document, context, &mut findings);
    check_boundaries(document, &mut findings);
    check_captions(document, &mut findings);
    check_headroom(context, &mut findings);
    Report { findings }
}

/// Nobody can infer what footage is. Somebody has to have said.
fn check_rights(document: &EditDocument, context: &Context<'_>, findings: &mut Vec<Finding>) {
    if context.source_attestation.trim().is_empty() {
        findings.push(Finding::blocking(
            "rights.missing",
            "No rights attestation. An export carries a claim about the footage, and \
             this one would carry a blank."
                .to_owned(),
        ));
    }
    let seconds = document.program_duration_ticks() / TICKS_PER_SECOND;
    if seconds > RIGHTS_GATE_SECONDS
        && !context
            .gates_passed
            .iter()
            .any(|gate| gate == DURATION_GATE)
    {
        findings.push(Finding::blocking(
            "rights.gate_not_passed",
            format!(
                "This clip runs {seconds} s, past the {RIGHTS_GATE_SECONDS} s mark where the \
                 rights confirmation applies, and that confirmation has not been given."
            ),
        ));
    }
}

/// A cut inside a word is a cut a viewer hears.
///
/// The same rule the boundary optimizer follows upstream, checked again here
/// because a boundary can be dragged in the editor after the optimizer chose
/// one — and the drag snaps, but a document can also arrive from elsewhere.
fn check_boundaries(document: &EditDocument, findings: &mut Vec<Finding>) {
    let mut boundaries: Vec<i64> = document.segment_program_starts();
    boundaries.push(document.program_duration_ticks());
    for boundary in boundaries {
        for cue in &document.captions.cues {
            for word in cue.words() {
                if word.start_ticks < boundary && boundary < word.end_ticks {
                    findings.push(Finding::blocking(
                        "boundary.inside_word",
                        format!(
                            "A cut at {:.2} s lands inside “{}”, which a viewer hears as a \
                             clipped word.",
                            seconds(boundary),
                            word.text
                        ),
                    ));
                }
            }
        }
    }
}

/// The sidecars have to be readable at the speed they run.
fn check_captions(document: &EditDocument, findings: &mut Vec<Finding>) {
    // The accessibility track is what becomes SRT and VTT, so it is held to the
    // profile without exception.
    for violation in run_profile(&document.captions.cues, Profile::ACCESSIBILITY_EN) {
        findings.push(Finding::blocking(
            &format!("captions.{}", code_of(&violation)),
            format!("Sidecar caption: {}", violation.message()),
        ));
    }
    // The burn-in track runs hot on purpose. A finding against it is worth
    // showing and is not worth refusing an export over — the alternative would
    // be a strip that blocks every kinetic caption the caption engine was
    // asked to produce.
    if !document.captions.burn_in.is_empty() {
        for violation in run_profile(&document.captions.burn_in, Profile::BURN_IN_EN) {
            findings.push(Finding::advisory(
                &format!("captions.burn_in.{}", code_of(&violation)),
                format!("Burned-in caption: {}", violation.message()),
            ));
        }
    }
}

fn run_profile(cues: &[CaptionCue], profile: Profile) -> Vec<Violation> {
    let lines: Vec<Vec<usize>> = cues
        .iter()
        .map(|cue| {
            cue.lines
                .iter()
                .map(|line| {
                    line.words
                        .iter()
                        .map(|word| word.text.chars().count())
                        .sum::<usize>()
                        + line.words.len().saturating_sub(1)
                })
                .collect()
        })
        .collect();
    let facts: Vec<clipmill_captions::CueFacts<'_>> = cues
        .iter()
        .zip(&lines)
        .map(|(cue, widths)| clipmill_captions::CueFacts {
            cue_id: &cue.cue_id,
            start_ticks: cue.start_ticks,
            end_ticks: cue.end_ticks,
            lines: widths,
        })
        .collect();
    // No shot cuts: the check that needs them belongs to the caption engine,
    // which has the shot index. Passing an empty list would silently pass that
    // check, so it is the one violation kind this strip does not claim to run.
    validate_cues(&facts, profile, &[])
}

fn code_of(violation: &Violation) -> &'static str {
    match violation {
        Violation::ReadingRate { .. } => "reading_rate",
        Violation::LineTooWide { .. } => "line_too_wide",
        Violation::TooManyLines { .. } => "too_many_lines",
        Violation::TooBrief { .. } => "too_brief",
        Violation::HeldTooLong { .. } => "held_too_long",
        Violation::Crowds { .. } => "crowds",
        Violation::SpansCut { .. } => "spans_cut",
        Violation::OutOfOrder { .. } => "out_of_order",
    }
}

fn check_headroom(context: &Context<'_>, findings: &mut Vec<Finding>) {
    let Some(available) = context.available_bytes else {
        findings.push(Finding::advisory(
            "disk.unknown",
            "Free space on the destination could not be read, so headroom was not checked."
                .to_owned(),
        ));
        return;
    };
    let needed = context.estimated_bytes;
    if available < needed {
        findings.push(Finding::blocking(
            "disk.insufficient",
            format!(
                "This export wants about {} and the destination has {} free.",
                megabytes(needed),
                megabytes(available)
            ),
        ));
    } else if available - needed < HEADROOM_RESERVE_BYTES {
        findings.push(Finding::advisory(
            "disk.tight",
            format!(
                "This export would leave under {} free on the destination.",
                megabytes(HEADROOM_RESERVE_BYTES)
            ),
        ));
    }
}

/// Both casts below are for a sentence a person reads. A double holds every
/// integer to 2^53, which is more bytes than any volume this ships on and more
/// ticks than three thousand years of recording, so the lint is right in
/// general and wrong here.
#[allow(
    clippy::cast_precision_loss,
    reason = "a byte count for display stays far inside a double's exact range"
)]
fn megabytes(bytes: u64) -> String {
    format!("{:.0} MB", bytes as f64 / (1024.0 * 1024.0))
}

#[allow(
    clippy::cast_precision_loss,
    reason = "ticks stay far inside a double's exact integer range"
)]
fn seconds(ticks: i64) -> f64 {
    ticks as f64 / TICKS_PER_SECOND as f64
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use clipmill_edit_ir::EditDocument;

    use super::{Context, DURATION_GATE, Report, Severity, estimate_bytes, validate};

    fn document() -> EditDocument {
        let json = include_str!("../tests/fixtures/short.json");
        EditDocument::from_canonical_json(json.as_bytes()).expect("the fixture is a document")
    }

    fn context(gates: &[String]) -> Context<'_> {
        Context {
            source_attestation: "own_content",
            gates_passed: gates,
            estimated_bytes: 10 * 1024 * 1024,
            available_bytes: Some(64 * 1024 * 1024 * 1024),
        }
    }

    fn codes(report: &Report) -> Vec<String> {
        report
            .findings
            .iter()
            .map(|finding| finding.code.clone())
            .collect()
    }

    #[test]
    fn a_clean_document_passes_with_nothing_to_say() {
        let report = validate(&document(), &context(&[]));
        assert!(report.passes(), "{:?}", report.findings);
        assert!(report.findings.is_empty(), "{:?}", report.findings);
    }

    #[test]
    fn an_export_nobody_attested_is_blocked() {
        let gates = Vec::new();
        let mut without = context(&gates);
        without.source_attestation = "   ";
        let report = validate(&document(), &without);
        assert!(!report.passes());
        assert_eq!(codes(&report), ["rights.missing"]);
    }

    #[test]
    fn a_long_clip_needs_the_gate_and_a_short_one_does_not() {
        let mut long = document();
        // Push the program past a minute by moving the only segment's out.
        long.video.segments[0].out_ticks = long.video.segments[0].in_ticks + 90_000 * 75;
        let gates = Vec::new();
        let report = validate(&long, &context(&gates));
        assert!(!report.passes());
        assert!(codes(&report).contains(&"rights.gate_not_passed".to_owned()));

        let passed = vec![DURATION_GATE.to_owned()];
        let report = validate(&long, &context(&passed));
        assert!(
            !codes(&report).contains(&"rights.gate_not_passed".to_owned()),
            "{:?}",
            report.findings
        );
    }

    #[test]
    fn a_cut_inside_a_word_blocks_and_names_the_word() {
        let mut clipped = document();
        // Move the program's end into the middle of the first caption word.
        let word = clipped.captions.cues[0].lines[0].words[0].clone();
        let inside = i64::midpoint(word.start_ticks, word.end_ticks);
        clipped.video.segments[0].out_ticks = clipped.video.segments[0].in_ticks + inside;
        let gates = Vec::new();
        let report = validate(&clipped, &context(&gates));
        assert!(!report.passes());
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.code == "boundary.inside_word")
            .expect("the boundary finding");
        assert!(finding.detail.contains(&word.text), "{}", finding.detail);
    }

    #[test]
    fn a_sidecar_that_cannot_be_read_at_speed_blocks() {
        let mut fast = document();
        // Same words, a third of the time.
        for cue in &mut fast.captions.cues {
            cue.end_ticks = cue.start_ticks + (cue.end_ticks - cue.start_ticks) / 8;
            for line in &mut cue.lines {
                for word in &mut line.words {
                    word.end_ticks = word.end_ticks.min(cue.end_ticks);
                    word.start_ticks = word.start_ticks.min(word.end_ticks);
                }
            }
        }
        let gates = Vec::new();
        let report = validate(&fast, &context(&gates));
        assert!(!report.passes(), "{:?}", report.findings);
        assert!(
            report
                .blocking()
                .any(|finding| finding.code.starts_with("captions.")),
            "{:?}",
            report.findings
        );
    }

    #[test]
    fn the_burn_in_track_running_hot_is_advice_rather_than_a_refusal() {
        let mut kinetic = document();
        // The kinetic intent: the same words, two at a time, held briefly. This
        // is what the caption engine produces on purpose.
        kinetic.captions.burn_in = kinetic
            .captions
            .cues
            .iter()
            .map(|cue| {
                let mut hot = cue.clone();
                hot.end_ticks = hot.start_ticks + (hot.end_ticks - hot.start_ticks) / 6;
                hot
            })
            .collect();
        let gates = Vec::new();
        let report = validate(&kinetic, &context(&gates));
        assert!(
            report.passes(),
            "the burn-in track must not block: {:?}",
            report.findings
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.code.starts_with("captions.burn_in.")
                    && finding.severity == Severity::Advisory),
            "{:?}",
            report.findings
        );
    }

    #[test]
    fn a_full_disk_blocks_and_a_nearly_full_one_warns() {
        let gates = Vec::new();
        let mut full = context(&gates);
        full.estimated_bytes = 4 * 1024 * 1024 * 1024;
        full.available_bytes = Some(1024 * 1024 * 1024);
        let report = validate(&document(), &full);
        assert!(!report.passes());
        assert!(codes(&report).contains(&"disk.insufficient".to_owned()));

        let mut tight = context(&gates);
        tight.estimated_bytes = 1024 * 1024 * 1024;
        tight.available_bytes = Some(1024 * 1024 * 1024 + 1024);
        let report = validate(&document(), &tight);
        assert!(report.passes(), "{:?}", report.findings);
        assert!(codes(&report).contains(&"disk.tight".to_owned()));
    }

    #[test]
    fn a_disk_that_could_not_be_read_says_so_rather_than_passing_quietly() {
        let gates = Vec::new();
        let mut unknown = context(&gates);
        unknown.available_bytes = None;
        let report = validate(&document(), &unknown);
        assert!(report.passes());
        assert_eq!(codes(&report), ["disk.unknown"]);
    }

    #[test]
    fn the_size_estimate_grows_with_the_clip_and_is_never_zero() {
        assert!(estimate_bytes(0) > 0);
        assert!(estimate_bytes(90_000 * 60) > estimate_bytes(90_000 * 10));
        // A negative duration is not a small file; it is a broken document, and
        // the estimate must not underflow into a number that passes headroom.
        assert!(estimate_bytes(-1) > 0);
    }
}
