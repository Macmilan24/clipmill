//! Development aid: the burned-in captions a caption track renders to.
//!
//! The UI preview harness has no daemon, so it posts its fixture's caption
//! track here (through a dev-server route) and the editor preview draws the
//! export's own ASS with libass instead of an approximation. Reads a
//! `captions` object as the edit document stores it on stdin, prints the ASS.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::io::Read;

use clipmill_edit_ir::{CaptionTrack, EditDocument, Layout, LayoutState, VideoSegment, VideoTrack};
use clipmill_render::{FrameRateSpec, RenderProfile, caption_ass};

fn main() -> std::process::ExitCode {
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        eprintln!("caption_ass: stdin is not text");
        return std::process::ExitCode::FAILURE;
    }
    let captions: CaptionTrack = match serde_json::from_str(&input) {
        Ok(captions) => captions,
        Err(error) => {
            eprintln!("caption_ass: {error}");
            return std::process::ExitCode::FAILURE;
        }
    };
    let end = captions
        .cues
        .iter()
        .chain(&captions.burn_in)
        .map(|cue| cue.end_ticks)
        .max()
        .unwrap_or(90_000);
    // A program as long as the captions, so they are all inside it.
    let document = EditDocument {
        captions,
        video: VideoTrack {
            transition_ticks: 0,
            segments: vec![VideoSegment {
                segment_id: "preview".to_owned(),
                source_fingerprint: format!("sha256:{}", "0".repeat(64)),
                in_ticks: 0,
                out_ticks: end + 90_000,
                layout: Layout {
                    state: LayoutState::Fit,
                    crop_path: Vec::new(),
                    secondary_crop_path: Vec::new(),
                    ..Layout::default()
                },
            }],
        },
        ..EditDocument::default()
    };
    let rate = std::env::args()
        .nth(1)
        .and_then(|value| {
            let (num, den) = value.split_once('/')?;
            Some(FrameRateSpec {
                num: num.parse().ok()?,
                den: den.parse().ok()?,
            })
        })
        .unwrap_or(RenderProfile::default().frame_rate);
    let profile = RenderProfile {
        frame_rate: rate,
        ..RenderProfile::default()
    };
    match caption_ass(&document, &profile) {
        Ok(ass) => {
            print!("{ass}");
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("caption_ass: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
