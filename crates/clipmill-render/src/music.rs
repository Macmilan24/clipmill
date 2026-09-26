//! Music under the voice, and the voice cleaned before it is mixed.
//!
//! The music drops wherever the clip's own words are said, on an envelope
//! computed here once: the render plays it as a volume curve, and the
//! preview schedules the same points, so what is heard while editing is the
//! mix the export makes. It fades in at the start and out at the end.

use clipmill_edit_ir::{EditDocument, GainPoint, VoiceCleanup};

/// Where the music's file is staged, beside the captions, for `amovie`.
pub const MUSIC_FILE: &str = "music";

/// Silence, as far as a volume curve is concerned.
const SILENT_DB: f64 = -60.0;
/// A pause shorter than this does not bring the music back up.
const HOLD_TICKS: i64 = 36_000;
/// How far before and after the words the music is already down.
const PAD_TICKS: i64 = 9_000;
/// How long the music takes to go down or come back.
const RAMP_TICKS: i64 = 22_500;
const FADE_IN_TICKS: i64 = 45_000;
const FADE_OUT_TICKS: i64 = 90_000;

/// The music's level through the program, in decibels, as points to
/// interpolate between: faded in, down under every run of speech, back up
/// between, faded out. Empty when the clip has no music.
pub fn music_envelope(document: &EditDocument, duration_ticks: i64) -> Vec<GainPoint> {
    let Some(music) = &document.audio.music else {
        return Vec::new();
    };
    let (base, ducked) = (music.level_db, music.level_db + music.duck_db);
    let fade_in = FADE_IN_TICKS.min(duration_ticks / 4);
    let fade_out = FADE_OUT_TICKS.min(duration_ticks / 4);
    let mut points = vec![(0, base)];
    for (start, end) in speech_runs(document, duration_ticks) {
        let down = (start - PAD_TICKS).max(0);
        let up = end + PAD_TICKS;
        points.push(((down - RAMP_TICKS).max(0), base));
        points.push((down, ducked));
        points.push((up, ducked));
        points.push((up + RAMP_TICKS, base));
    }
    points.push((duration_ticks, points.last().map_or(base, |point| point.1)));
    points.sort_by_key(|point| point.0);
    points.dedup_by_key(|point| point.0);
    let at = |t: i64| level_at(&points, t);
    let (full_from, full_until) = (fade_in, duration_ticks - fade_out);
    let mut curve = vec![
        GainPoint {
            t_ticks: 0,
            gain_db: SILENT_DB,
        },
        GainPoint {
            t_ticks: full_from,
            gain_db: at(full_from),
        },
    ];
    curve.extend(
        points
            .iter()
            .filter(|point| point.0 > full_from && point.0 < full_until)
            .map(|&(t_ticks, gain_db)| GainPoint { t_ticks, gain_db }),
    );
    if full_until > full_from {
        curve.push(GainPoint {
            t_ticks: full_until,
            gain_db: at(full_until),
        });
    }
    curve.push(GainPoint {
        t_ticks: duration_ticks,
        gain_db: SILENT_DB,
    });
    curve.dedup_by_key(|point| point.t_ticks);
    curve
}

/// The spans someone is speaking, from the words the captions time, joined
/// across pauses too short to bring the music back up.
fn speech_runs(document: &EditDocument, duration_ticks: i64) -> Vec<(i64, i64)> {
    let mut words: Vec<(i64, i64)> = document
        .captions
        .cues
        .iter()
        .flat_map(|cue| cue.lines.iter().flat_map(|line| &line.words))
        .map(|word| (word.start_ticks.max(0), word.end_ticks.min(duration_ticks)))
        .filter(|(start, end)| end > start)
        .collect();
    words.sort_unstable();
    let join = HOLD_TICKS + 2 * (PAD_TICKS + RAMP_TICKS);
    let mut runs: Vec<(i64, i64)> = Vec::new();
    for (start, end) in words {
        match runs.last_mut() {
            Some(last) if start - last.1 <= join => last.1 = last.1.max(end),
            _ => runs.push((start, end)),
        }
    }
    runs
}

/// The level at a moment, straight between the points either side.
fn level_at(points: &[(i64, f64)], t: i64) -> f64 {
    let Some(first) = points.first() else {
        return 0.0;
    };
    if t <= first.0 {
        return first.1;
    }
    for pair in points.windows(2) {
        let (before, after) = (pair[0], pair[1]);
        if t <= after.0 {
            let span = after.0 - before.0;
            if span <= 0 {
                return after.1;
            }
            #[allow(
                clippy::cast_precision_loss,
                reason = "tick offsets inside a clip, far inside a double's exact integers"
            )]
            let share = (t - before.0) as f64 / span as f64;
            return before.1 + (after.1 - before.1) * share;
        }
    }
    points.last().map_or(0.0, |point| point.1)
}

/// The filters that clean a voice, in order, for a chain to follow.
pub(crate) fn cleanup_filters(cleanup: VoiceCleanup) -> &'static str {
    match cleanup {
        // Rumble below the voice, a little hiss, and an even level.
        VoiceCleanup::Light => {
            "highpass=f=70,afftdn=nr=10:nf=-45,\
             acompressor=threshold=0.063:ratio=2:attack=15:release=200:makeup=1.5"
        }
        // A noisy room: more noise lowered, sibilance softened, level held.
        VoiceCleanup::Strong => {
            "highpass=f=90,afftdn=nr=20:nf=-40,deesser=i=0.5,\
             acompressor=threshold=0.05:ratio=3:attack=10:release=150:makeup=2"
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used, clippy::cast_precision_loss)]

    use clipmill_edit_ir::{
        Asset, CaptionAnimation, CaptionCue, CaptionLine, CaptionRegion, CaptionWord, EditDocument,
        MusicBed,
    };

    use super::music_envelope;

    fn said(document: &mut EditDocument, words: &[(i64, i64)]) {
        document.captions.cues = vec![CaptionCue {
            cue_id: "cue_1".to_owned(),
            start_ticks: words.first().unwrap().0,
            end_ticks: words.last().unwrap().1,
            region: CaptionRegion::LowerSafe,
            anim: CaptionAnimation::Karaoke,
            lines: vec![CaptionLine {
                words: words
                    .iter()
                    .enumerate()
                    .map(|(index, &(start_ticks, end_ticks))| CaptionWord {
                        text: format!("w{index}"),
                        start_ticks,
                        end_ticks,
                        word_id: None,
                        emphasis: false,
                    })
                    .collect(),
            }],
            position: None,
        }];
    }

    #[test]
    fn the_music_drops_under_speech_and_fades_at_both_ends() {
        let mut document = EditDocument::default();
        let hash = format!("sha256:{}", "3".repeat(64));
        document.assets = vec![Asset {
            hash: hash.clone(),
            license: "royalty_free".to_owned(),
        }];
        document.audio.music = Some(MusicBed {
            asset: hash,
            level_db: -18.0,
            duck_db: -12.0,
            offset_ticks: 0,
        });
        // Speech from 2 s to 4 s, and a second run from 6 s to 7 s.
        said(
            &mut document,
            &[(180_000, 270_000), (270_000, 360_000), (540_000, 630_000)],
        );
        let curve = music_envelope(&document, 900_000);
        let at = |t: i64| {
            curve
                .windows(2)
                .find(|pair| pair[0].t_ticks <= t && t <= pair[1].t_ticks)
                .map(|pair| {
                    let share = (t - pair[0].t_ticks) as f64
                        / (pair[1].t_ticks - pair[0].t_ticks).max(1) as f64;
                    pair[0].gain_db + (pair[1].gain_db - pair[0].gain_db) * share
                })
                .unwrap()
        };
        assert!((at(0) - -60.0).abs() < 1e-9, "silent at the very start");
        assert!((at(90_000) - -18.0).abs() < 1e-9, "up before anyone speaks");
        assert!((at(225_000) - -30.0).abs() < 1e-9, "down under the words");
        assert!(
            (at(460_000) - -18.0).abs() < 1e-9,
            "back up in a long pause"
        );
        assert!((at(585_000) - -30.0).abs() < 1e-9, "down again");
        assert!((at(900_000) - -60.0).abs() < 1e-9, "silent at the end");
        assert!(
            curve
                .windows(2)
                .all(|pair| pair[0].t_ticks < pair[1].t_ticks)
        );

        document.audio.music = None;
        assert!(music_envelope(&document, 900_000).is_empty());
    }
}
