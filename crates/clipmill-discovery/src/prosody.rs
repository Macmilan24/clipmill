//! Delivery, as far as two measurements can see it.
//!
//! A memorable line is usually delivered differently from the sentences around
//! it — louder, or faster, or slower for emphasis. Neither of those is
//! meaning, and this module claims neither: it reports how far a sentence
//! deviates from the recording's own baseline, in loudness and in speaking
//! rate, and the insight proposer weighs that alongside three other proxies.
//!
//! Deviation from *this* recording rather than an absolute bar, because an
//! absolute bar measures the microphone. A quiet speaker's emphatic sentence
//! and a loud speaker's flat one would otherwise score the same way round.
//!
//! Absent audio is absent evidence. A source with no loudness envelope gets a
//! neutral zero rather than a default that would let prosody vote without
//! having measured anything.

use clipmill_contracts::schemas::index_transcript as index;
use clipmill_contracts::schemas::media_loudness_envelope::MediaLoudnessEnvelope;

/// The baseline a sentence is compared against.
#[derive(Debug, Default)]
pub(crate) struct Prosody {
    loudness: Option<Loudness>,
    /// Median words per minute over the recording, and the spread around it.
    rate: Option<(f64, f64)>,
}

#[derive(Debug)]
struct Loudness {
    points: Vec<(u64, f64)>,
    integrated: f64,
}

impl Prosody {
    /// Measure the baseline once, from whatever evidence exists.
    pub(crate) fn measure(
        document: &index::IndexTranscript,
        envelope: Option<&MediaLoudnessEnvelope>,
    ) -> Self {
        let loudness = envelope.map(|envelope| Loudness {
            points: envelope
                .points
                .iter()
                .map(|point| (point.t_ticks, point.momentary_lufs))
                .collect(),
            integrated: envelope.summary.integrated_lufs,
        });
        let mut rates = document
            .sentences
            .iter()
            .map(|sentence| sentence.words_per_minute)
            .filter(|rate| *rate > 0.0)
            .collect::<Vec<_>>();
        rates.sort_by(f64::total_cmp);
        let rate = (!rates.is_empty()).then(|| {
            let median = rates[rates.len() / 2];
            let spread = rates
                .iter()
                .map(|value| (value - median).abs())
                .sum::<f64>()
                / crate::as_f64(rates.len());
            (median, spread)
        });
        Self { loudness, rate }
    }

    /// How much this sentence stands out, in zero to one.
    ///
    /// The two halves are averaged rather than summed, so a recording with no
    /// audio scores the rate deviation on its own instead of being penalised
    /// for the measurement it could not take.
    pub(crate) fn emphasis(&self, sentence: &index::Sentence) -> f64 {
        let mut parts = Vec::new();
        if let Some(loudness) = &self.loudness {
            parts.push(loudness.deviation(sentence.start_ticks, sentence.end_ticks));
        }
        if let Some((median, spread)) = self.rate
            && spread > 0.0
        {
            // Either direction: a sentence delivered notably slowly is
            // emphasis as often as one delivered fast.
            parts.push(((sentence.words_per_minute - median).abs() / (spread * 3.0)).min(1.0));
        }
        if parts.is_empty() {
            return 0.0;
        }
        parts.iter().sum::<f64>() / crate::as_f64(parts.len())
    }
}

impl Loudness {
    /// Mean momentary loudness over a span, against the integrated baseline,
    /// scaled so that six LU above the recording's own level reads as full
    /// emphasis. Six because that is roughly the point at which a listener
    /// hears a sentence as raised rather than merely audible.
    fn deviation(&self, start: u64, end: u64) -> f64 {
        let inside = self
            .points
            .iter()
            .filter(|(at, _)| *at >= start && *at <= end)
            .map(|(_, lufs)| *lufs)
            .collect::<Vec<_>>();
        if inside.is_empty() {
            return 0.0;
        }
        let mean = inside.iter().sum::<f64>() / crate::as_f64(inside.len());
        ((mean - self.integrated) / 6.0).clamp(0.0, 1.0)
    }
}
