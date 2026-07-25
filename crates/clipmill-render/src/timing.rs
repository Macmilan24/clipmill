//! Rational time for the render target.
//!
//! The Edit IR counts ticks at 1/90000 (decision D06); the render target
//! counts frames at a fixed rate. Every conversion between them happens here,
//! in exact integer arithmetic, so that a caption's first frame is a fact
//! rather than the result of whichever float rounding a call site happened to
//! use. Chapter 17's parity invariant is only defensible if there is one
//! answer to "which frame is this tick on".

use clipmill_edit_ir::TICKS_PER_SECOND;

/// A constant frame rate as an exact rational.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameRate {
    pub num: i64,
    pub den: i64,
}

impl FrameRate {
    /// 30000/1001 — the Phase 1 render target.
    pub const NTSC_30: Self = Self {
        num: 30_000,
        den: 1_001,
    };

    /// The frame index containing `ticks`: the largest `f` whose presentation
    /// time is at or before `ticks`.
    pub fn frame_at(self, ticks: i64) -> i64 {
        let ticks = i128::from(ticks.max(0));
        let frames = ticks * i128::from(self.num) / self.divisor();
        i64::try_from(frames).unwrap_or(i64::MAX)
    }

    fn divisor(self) -> i128 {
        i128::from(self.den) * i128::from(TICKS_PER_SECOND)
    }

    /// The first frame at or after `ticks`: the frame a cue starting there is
    /// first drawn on.
    pub fn frame_ceil(self, ticks: i64) -> i64 {
        let ticks = i128::from(ticks.max(0));
        let divisor = self.divisor();
        let scaled = ticks * i128::from(self.num);
        let frames = scaled / divisor + i128::from(scaled % divisor != 0);
        i64::try_from(frames).unwrap_or(i64::MAX)
    }

    /// Number of frames a program of `ticks` occupies. The final partial frame
    /// is still a frame the viewer sees.
    pub fn frame_count(self, ticks: i64) -> i64 {
        self.frame_ceil(ticks)
    }

    /// Presentation time of frame `frame`, floored to whole centiseconds.
    ///
    /// Flooring is what makes the ASS timestamp land on the intended frame:
    /// a centisecond is 3.34 times shorter than a frame at 29.97, so the
    /// floored time is always strictly after the previous frame's time and at
    /// or before this one's. libass draws a frame when `start <= t < end`, so
    /// the cue appears on exactly the frame the IR asked for.
    pub fn frame_centis(self, frame: i64) -> i64 {
        self.frame_scaled(frame, 100)
    }

    /// Presentation time of frame `frame`, floored to whole milliseconds —
    /// the resolution SRT and WebVTT can express.
    pub fn frame_millis(self, frame: i64) -> i64 {
        self.frame_scaled(frame, 1_000)
    }

    fn frame_scaled(self, frame: i64, per_second: i64) -> i64 {
        let scaled = i128::from(frame.max(0)) * i128::from(self.den) * i128::from(per_second)
            / i128::from(self.num);
        i64::try_from(scaled).unwrap_or(i64::MAX)
    }
}

/// Seconds as a decimal string with microsecond resolution, for the FFmpeg
/// arguments that take a duration rather than a timebase count.
///
/// A tick is 1/90000 s, so the sixth decimal place absorbs the whole
/// conversion error — five hundred times finer than the half-frame that would
/// be needed to move a boundary onto a different frame.
pub fn ticks_to_seconds(ticks: i64) -> String {
    let ticks = i128::from(ticks.max(0));
    let micros = ticks * 1_000_000 / i128::from(TICKS_PER_SECOND);
    let whole = micros / 1_000_000;
    let fraction = micros % 1_000_000;
    format!("{whole}.{fraction:06}")
}

/// `h:mm:ss.cc` — the ASS timestamp form.
pub fn centis_to_ass(centis: i64) -> String {
    let centis = centis.max(0);
    let (hours, minutes, seconds, rest) = split_time(centis, 100);
    format!("{hours}:{minutes:02}:{seconds:02}.{rest:02}")
}

/// `hh:mm:ss,mmm` — the SubRip timestamp form.
pub fn millis_to_srt(millis: i64) -> String {
    let (hours, minutes, seconds, rest) = split_time(millis.max(0), 1_000);
    format!("{hours:02}:{minutes:02}:{seconds:02},{rest:03}")
}

/// `hh:mm:ss.mmm` — the WebVTT timestamp form.
pub fn millis_to_vtt(millis: i64) -> String {
    let (hours, minutes, seconds, rest) = split_time(millis.max(0), 1_000);
    format!("{hours:02}:{minutes:02}:{seconds:02}.{rest:03}")
}

fn split_time(value: i64, per_second: i64) -> (i64, i64, i64, i64) {
    let rest = value % per_second;
    let total_seconds = value / per_second;
    (
        total_seconds / 3_600,
        (total_seconds % 3_600) / 60,
        total_seconds % 60,
        rest,
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::{FrameRate, centis_to_ass, millis_to_srt, millis_to_vtt, ticks_to_seconds};

    const RATE: FrameRate = FrameRate::NTSC_30;
    /// 90000 * 1001 / 30000 — exact at the Phase 1 target rate.
    const FRAME_TICKS: i64 = 3_003;

    #[test]
    fn frame_boundaries_are_exact_at_the_target_rate() {
        for frame in 0..1_000 {
            let start = frame * FRAME_TICKS;
            assert_eq!(RATE.frame_at(start), frame);
            assert_eq!(RATE.frame_ceil(start), frame);
            // One tick past a boundary belongs to the same frame, but a cue
            // starting there cannot be drawn until the next one.
            assert_eq!(RATE.frame_at(start + 1), frame);
            assert_eq!(RATE.frame_ceil(start + 1), frame + 1);
        }
    }

    #[test]
    fn a_partial_final_frame_still_counts() {
        assert_eq!(RATE.frame_count(0), 0);
        assert_eq!(RATE.frame_count(1), 1);
        assert_eq!(RATE.frame_count(FRAME_TICKS), 1);
        assert_eq!(RATE.frame_count(FRAME_TICKS + 1), 2);
        assert_eq!(RATE.frame_count(10 * FRAME_TICKS), 10);
    }

    /// The property the caption pipeline rests on: the centisecond stamp of a
    /// frame is strictly later than the previous frame's presentation time and
    /// no later than its own, so libass draws the cue on exactly that frame.
    #[test]
    fn centisecond_stamps_select_their_own_frame() {
        for frame in 1..20_000 {
            let stamp_ticks = RATE.frame_centis(frame) * 900;
            let previous = (frame - 1) * FRAME_TICKS;
            let own = frame * FRAME_TICKS;
            assert!(
                stamp_ticks > previous,
                "frame {frame} stamp fell back onto frame {}",
                frame - 1
            );
            assert!(stamp_ticks <= own, "frame {frame} stamp overshot its frame");
        }
    }

    #[test]
    fn millisecond_stamps_select_their_own_frame() {
        for frame in 1..20_000 {
            let stamp_ticks = RATE.frame_millis(frame) * 90;
            assert!(stamp_ticks > (frame - 1) * FRAME_TICKS);
            assert!(stamp_ticks <= frame * FRAME_TICKS);
        }
    }

    #[test]
    fn seconds_render_with_microsecond_resolution() {
        assert_eq!(ticks_to_seconds(0), "0.000000");
        assert_eq!(ticks_to_seconds(90_000), "1.000000");
        assert_eq!(ticks_to_seconds(45_000), "0.500000");
        assert_eq!(ticks_to_seconds(3_003), "0.033366");
        assert_eq!(ticks_to_seconds(-5), "0.000000");
    }

    #[test]
    fn timestamps_match_their_formats() {
        assert_eq!(centis_to_ass(0), "0:00:00.00");
        assert_eq!(centis_to_ass(360_123), "1:00:01.23");
        assert_eq!(millis_to_srt(3_601_234), "01:00:01,234");
        assert_eq!(millis_to_vtt(3_601_234), "01:00:01.234");
    }
}
