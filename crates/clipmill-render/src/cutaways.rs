//! B-roll: another picture over a span of the program while the voice
//! carries on.
//!
//! A cutaway is laid over the program's own picture and under everything
//! drawn on top of it — the brand, the emoji, the captions — so what a
//! viewer reads stays put when the picture behind it changes. A picture is
//! read from the clip's assets, staged by its hash; footage is one more
//! decoder on a recording of the project, seeked to a keyframe and trimmed
//! exactly, like a section. Neither has sound: the program's carries on.

use clipmill_edit_ir::{Cutaway, CutawayContent, CutawayFit, EditDocument};

use crate::{
    plan::{RenderError, SourceInput},
    profile::RenderProfile,
    timing::ticks_to_seconds,
};

/// Where cutaway pictures are staged for `movie` to read, each named by the
/// hex of its hash.
pub const CUTAWAY_DIR: &str = "cutaways";

/// How much closer a picture that pushes in is by its last frame.
const PUSH_IN: f64 = Cutaway::PUSH_IN as f64 / 1_000.0;

/// Resampler slack for footage, trimmed to the cutaway's frames as a
/// section's is.
const TAIL_PAD: &str = "tpad=stop_mode=clone:stop_duration=1";

/// One footage cutaway's decoder: its recording seeked to a keyframe at or
/// before where it starts, and read for as long as it shows.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FootageInput {
    pub cutaway_id: String,
    pub source_fingerprint: String,
    pub seek_ticks: i64,
    pub trim_start_ticks: i64,
    pub trim_end_ticks: i64,
}

/// The decoders a document's footage cutaways need, in their order.
pub(crate) fn footage_inputs(
    document: &EditDocument,
    sources: &[SourceInput],
) -> Result<Vec<FootageInput>, RenderError> {
    document
        .video
        .cutaways
        .iter()
        .filter_map(|cutaway| match &cutaway.content {
            CutawayContent::Footage {
                source_fingerprint,
                in_ticks,
            } => Some((cutaway, source_fingerprint, *in_ticks)),
            CutawayContent::Picture { .. } => None,
        })
        .map(|(cutaway, fingerprint, in_ticks)| {
            let source = sources
                .iter()
                .find(|source| &source.fingerprint == fingerprint)
                .ok_or_else(|| RenderError::UnresolvedSource(fingerprint.clone()))?;
            let out_ticks = in_ticks + cutaway.end_ticks - cutaway.start_ticks;
            if source.duration_ticks > 0 && out_ticks > source.duration_ticks {
                return Err(RenderError::CutawayPastEndOfSource(
                    cutaway.cutaway_id.clone(),
                ));
            }
            let seek_ticks = source.seek_target(in_ticks);
            Ok(FootageInput {
                cutaway_id: cutaway.cutaway_id.clone(),
                source_fingerprint: fingerprint.clone(),
                seek_ticks,
                trim_start_ticks: in_ticks - seek_ticks,
                trim_end_ticks: out_ticks - seek_ticks,
            })
        })
        .collect()
}

/// The file a cutaway picture is staged as: the hex of its hash, and nothing
/// else, so a document cannot name a path.
pub fn cutaway_picture_file(asset: &str) -> Option<String> {
    asset
        .strip_prefix("sha256:")
        .filter(|hex| hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .map(str::to_ascii_lowercase)
}

/// The pictures a document's cutaways show, once each, by hash, for the
/// render to stage.
pub fn cutaway_pictures(document: &EditDocument) -> Vec<String> {
    let mut pictures: Vec<String> = document
        .video
        .cutaways
        .iter()
        .filter_map(|cutaway| match &cutaway.content {
            CutawayContent::Picture { asset, .. } => Some(asset.clone()),
            CutawayContent::Footage { .. } => None,
        })
        .collect();
    pictures.sort();
    pictures.dedup();
    pictures
}

/// A cutaway starting after the program ends was left behind by an edit.
pub(crate) fn check_cutaways(
    document: &EditDocument,
    duration_ticks: i64,
) -> Result<(), RenderError> {
    match document
        .video
        .cutaways
        .iter()
        .find(|cutaway| cutaway.start_ticks >= duration_ticks)
    {
        Some(cutaway) => Err(RenderError::CutawayOutsideProgram(
            cutaway.cutaway_id.clone(),
        )),
        None => Ok(()),
    }
}

/// The chains laying each cutaway over `picture`, and the label of the
/// result. Footage cutaways read from the inputs after the sections', the
/// first of them `first_input`.
pub(crate) fn cutaway_chains(
    document: &EditDocument,
    profile: &RenderProfile,
    inputs: &[FootageInput],
    first_input: usize,
    picture: &str,
) -> Result<(Vec<String>, String), RenderError> {
    let rate = profile.rate();
    let total = rate.frame_count(document.program_duration_ticks());
    let (num, den) = (rate.num, rate.den);
    let mut chains = Vec::new();
    let mut picture = picture.to_owned();
    let mut footage = 0;
    for (index, cutaway) in document.video.cutaways.iter().enumerate() {
        let first = rate.frame_ceil(cutaway.start_ticks);
        let end = rate.frame_ceil(cutaway.end_ticks).min(total);
        if end <= first {
            continue;
        }
        let frames = end - first;
        let raw = format!("cut{index}raw");
        let laid = format!("cut{index}");
        // Its first frame falls on the program frame it starts on.
        let at = format!("setpts=PTS-STARTPTS+{first}*{den}/{num}/TB");
        let still = match &cutaway.content {
            CutawayContent::Picture { asset, push_in } => {
                let file = cutaway_picture_file(asset)
                    .ok_or_else(|| RenderError::UnresolvedAsset(asset.clone()))?;
                chains.push(format!(
                    "movie=filename={CUTAWAY_DIR}/{file},format=yuv420p[{raw}]"
                ));
                if *push_in {
                    // Composed at twice the size, so the move lands on half
                    // pixels rather than stepping a whole one at a time.
                    let big = format!("cut{index}big");
                    chains.extend(fitted(
                        cutaway.fit,
                        (profile.width * 2, profile.height * 2),
                        profile.fit_background_sigma * 2,
                        &raw,
                        &big,
                    ));
                    chains.push(format!(
                        "[{big}]zoompan=z='1+{PUSH_IN}*on/{span}':\
                         x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)':d={frames}:\
                         s={width}x{height}:fps={num}/{den},{at},format=yuv420p[{laid}]",
                        span = (frames - 1).max(1),
                        width = profile.width,
                        height = profile.height,
                    ));
                    false
                } else {
                    chains.extend(fitted(
                        cutaway.fit,
                        (profile.width, profile.height),
                        profile.fit_background_sigma,
                        &raw,
                        &laid,
                    ));
                    true
                }
            }
            CutawayContent::Footage { .. } => {
                let input = inputs
                    .get(footage)
                    .filter(|input| input.cutaway_id == cutaway.cutaway_id)
                    .ok_or_else(|| RenderError::UnresolvedSource(cutaway.cutaway_id.clone()))?;
                chains.push(format!(
                    "[{decoder}:v]trim=start={start}:end={stop},setpts=PTS-STARTPTS,\
                     fps={num}/{den}:start_time=0,{TAIL_PAD},trim=end_frame={frames},\
                     setpts=PTS-STARTPTS,format=yuv420p[{raw}]",
                    decoder = first_input + footage,
                    start = ticks_to_seconds(input.trim_start_ticks),
                    stop = ticks_to_seconds(input.trim_end_ticks),
                ));
                footage += 1;
                let fitted_label = format!("cut{index}fitted");
                chains.extend(fitted(
                    cutaway.fit,
                    (profile.width, profile.height),
                    profile.fit_background_sigma,
                    &raw,
                    &fitted_label,
                ));
                chains.push(format!("[{fitted_label}]{at}[{laid}]"));
                false
            }
        };
        // A still is one frame, held; a moving one runs out and lets the
        // program through. Either shows on its frames alone.
        let output = format!("[vcut{index}]");
        chains.push(format!(
            "{picture}[{laid}]overlay=x=0:y=0:eof_action={held}:\
             enable='between(n\\,{first}\\,{last})',format=yuv420p{output}",
            held = if still { "repeat" } else { "pass" },
            last = end - 1,
        ));
        picture = output;
    }
    Ok((chains, picture))
}

/// A picture made the frame's size: covering it, what does not fit cropped
/// away; or the whole of it over a blurred copy that fills the rest.
fn fitted(
    fit: CutawayFit,
    (width, height): (i64, i64),
    sigma: u32,
    input: &str,
    output: &str,
) -> Vec<String> {
    let cover = format!(
        "scale={width}:{height}:force_original_aspect_ratio=increase,crop={width}:{height}"
    );
    match fit {
        CutawayFit::Fill => vec![format!("[{input}]{cover},setsar=1[{output}]")],
        CutawayFit::Fit => vec![
            format!("[{input}]split=2[{input}bg][{input}fg]"),
            format!("[{input}bg]{cover},gblur=sigma={sigma}[{input}bgb]"),
            format!(
                "[{input}fg]scale={width}:{height}:force_original_aspect_ratio=decrease[{input}fgs]"
            ),
            format!(
                "[{input}bgb][{input}fgs]overlay=x=(W-w)/2:y=(H-h)/2,setsar=1,format=yuv420p[{output}]"
            ),
        ],
    }
}
