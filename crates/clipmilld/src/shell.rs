//! What a shell may read, and by which door.
//!
//! The renderer is the least trusted thing that talks to this daemon. It runs
//! web content, it is the part an attacker reaches first, and it asks for
//! artifacts by address. So what it may read is a list rather than a rule: two
//! lists, in fact, because the two doors have different shapes.
//!
//! `ReadArtifact` serves **documents** over the control socket — short JSON
//! observations a screen renders. A kind on that list carries exactly one
//! document, and this table says which, so the request names no path and a
//! caller cannot point somewhere else inside the object.
//!
//! `clipmill-media://` serves **media** — the proxy a player seeks through, the
//! filmstrip a timeline draws, the waveform peaks, a finished render. Those are
//! large, they are byte-ranged, and they must never come through the control
//! socket, where they would occupy a frame each.
//!
//! Neither list is the union of "everything in the store". Model weights are
//! absent because a renderer has no business reading them. The three speech
//! intermediates — voice activity, recognition, alignment — are absent because
//! the assembly that fuses them is what a reader wants, and an allowlist that
//! names everything is not one. Adding a kind is one line here and one line in
//! the test that pins the lists, which is the cost it should have.

/// Documents `ReadArtifact` will serve, as kind → the one file it carries.
///
/// Ordered by where a shell meets them: the source it imported, what ingest
/// derived, the transcript, the structure over it, the nominations, the ranking,
/// the analysis that roots them all, then the edit and its render.
const DOCUMENTS: [(&str, &str); 11] = [
    ("evidence.source_map.v1", "source-map.json"),
    ("media.ingest_manifest.v1", "ingest-manifest.json"),
    // A waveform, and a document despite the `media.` prefix: its peaks are in
    // the JSON, not in a file beside it. A timeline reads it like any other
    // observation.
    ("media.audio_peaks.v1", "peaks.json"),
    ("speech.transcript.v1", "transcript.json"),
    ("evidence.shots.v1", "shots.json"),
    ("index.transcript.v1", "index.json"),
    ("discovery.candidates.v1", "candidates.json"),
    ("ranking.set.v1", "ranking.json"),
    ("analysis.manifest.v1", "analysis.json"),
    ("edit.ir.v1", "edit-ir.json"),
    // A render's manifest, not its video. The video is on the media list under
    // the same kind, because the two are different doors onto one artifact.
    ("render.clip.v1", "render-manifest.json"),
];

/// Where a media descriptor keeps the names of the files beside it.
///
/// Each kind says it differently — a proxy names one container, a filmstrip an
/// array of tiles, a render an array of outputs — so the shape is part of the
/// table rather than something the reader infers. Getting this wrong would mean
/// serving a file the descriptor never named, which is the one thing the
/// descriptor is for.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MediaLayout {
    /// A string at the named key holds the one file. `media.proxy.v1`.
    One { key: &'static str },
    /// An array at the named key, each element naming a file at `file`.
    /// Filmstrip tiles and analysis frames.
    Many { list: &'static str },
    /// An array at the named key, each element naming a file at `path`. A
    /// render's outputs: the video and its sidecars.
    Outputs { list: &'static str },
}

/// Media `clipmill-media://` will serve: kind → its descriptor and that
/// descriptor's shape.
///
/// The protocol reads the descriptor, then serves only a file the descriptor
/// named — so this table covers the payload without having to know how many
/// files there are or what they are called.
const MEDIA: [(&str, &str, MediaLayout); 4] = [
    (
        "media.proxy.v1",
        "proxy.json",
        MediaLayout::One { key: "file" },
    ),
    (
        "media.filmstrip.v1",
        "index.json",
        MediaLayout::Many { list: "tiles" },
    ),
    (
        "media.frames.v1",
        "index.json",
        MediaLayout::Many { list: "frames" },
    ),
    (
        "render.clip.v1",
        "render-manifest.json",
        MediaLayout::Outputs { list: "outputs" },
    ),
];

/// What to answer a media request with, by extension.
///
/// A table rather than a guess, and an extension absent from it is refused
/// rather than served as `application/octet-stream`. A browser told the wrong
/// type does something creative with the bytes, and "creative" is the opposite
/// of what a media surface wants.
const MEDIA_TYPES: [(&str, &str); 4] = [
    ("mp4", "video/mp4"),
    ("jpg", "image/jpeg"),
    ("vtt", "text/vtt"),
    ("srt", "application/x-subrip"),
];

/// The media type for a file this daemon is willing to serve.
pub(crate) fn media_type_for(path: &str) -> Option<&'static str> {
    let extension = path.rsplit_once('.').map(|(_, tail)| tail)?;
    MEDIA_TYPES
        .iter()
        .find(|(named, _)| *named == extension)
        .map(|(_, media_type)| *media_type)
}

/// The document this kind carries, or nothing if a shell may not read it.
pub(crate) fn document_for(kind: &str) -> Option<&'static str> {
    DOCUMENTS
        .iter()
        .find(|(named, _)| *named == kind)
        .map(|(_, file)| *file)
}

/// The descriptor and shape for this kind, or nothing if a shell may not stream
/// it.
pub(crate) fn media_descriptor_for(kind: &str) -> Option<(&'static str, MediaLayout)> {
    MEDIA
        .iter()
        .find(|(named, _, _)| *named == kind)
        .map(|(_, file, layout)| (*file, *layout))
}

/// The files a media descriptor names, in the order it named them.
///
/// Only names: no validation of what they point at, because the manifest is the
/// authority on which files exist and how large they are. What this does is turn
/// three different descriptor shapes into one list, so the caller has one thing
/// to check rather than three.
pub(crate) fn media_files(descriptor: &serde_json::Value, layout: MediaLayout) -> Vec<String> {
    match layout {
        MediaLayout::One { key } => descriptor
            .get(key)
            .and_then(serde_json::Value::as_str)
            .map(|file| vec![file.to_owned()])
            .unwrap_or_default(),
        MediaLayout::Many { list } => collect(descriptor, list, "file"),
        MediaLayout::Outputs { list } => collect(descriptor, list, "path"),
    }
}

fn collect(descriptor: &serde_json::Value, list: &str, key: &str) -> Vec<String> {
    descriptor
        .get(list)
        .and_then(serde_json::Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| entry.get(key).and_then(serde_json::Value::as_str))
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// The most one `ReadArtifact` reply carries.
///
/// Well under the 4 MiB frame the transport enforces, because the frame has to
/// hold the encoded response around this chunk and a limit that only just fits
/// is a limit that stops fitting the first time a field is added. A document
/// longer than this is read in several calls, which is why the response states
/// its total size.
pub(crate) const MAX_CHUNK_BYTES: u64 = 1024 * 1024;

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use std::collections::BTreeSet;

    use super::{
        DOCUMENTS, MAX_CHUNK_BYTES, MEDIA, MEDIA_TYPES, MediaLayout, document_for,
        media_descriptor_for, media_type_for,
    };

    /// The lists are pinned. A kind arriving here without somebody editing this
    /// test is a kind nobody decided to expose — which is the whole point of a
    /// list rather than a rule.
    #[test]
    fn the_readable_kinds_are_exactly_these() {
        assert_eq!(
            DOCUMENTS
                .iter()
                .map(|(kind, _)| *kind)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([
                "analysis.manifest.v1",
                "discovery.candidates.v1",
                "edit.ir.v1",
                "evidence.shots.v1",
                "evidence.source_map.v1",
                "index.transcript.v1",
                "media.audio_peaks.v1",
                "media.ingest_manifest.v1",
                "ranking.set.v1",
                "render.clip.v1",
                "speech.transcript.v1",
            ])
        );
        assert_eq!(
            MEDIA
                .iter()
                .map(|(kind, _, _)| *kind)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([
                "media.filmstrip.v1",
                "media.frames.v1",
                "media.proxy.v1",
                "render.clip.v1",
            ])
        );
    }

    /// Each descriptor's shape has to match the schema that produced it. This
    /// reads the published schemas rather than restating them, because the table
    /// naming the wrong key means serving a file the descriptor never named.
    #[test]
    fn every_media_layout_matches_the_schema_it_reads() {
        for (kind, _, layout) in MEDIA {
            let schema: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(format!(
                    "{}/../../contracts/schemas/clipmill.{kind}.json",
                    env!("CARGO_MANIFEST_DIR")
                ))
                .expect("the schema is published"),
            )
            .expect("the schema parses");
            let properties = &schema["properties"];
            match layout {
                MediaLayout::One { key } => {
                    assert_eq!(
                        properties[key]["type"], "string",
                        "{kind} names its file at {key}"
                    );
                }
                MediaLayout::Many { list } => {
                    assert_eq!(properties[list]["type"], "array", "{kind}.{list} is a list");
                    assert!(
                        !properties[list]["items"]["properties"]["file"].is_null(),
                        "{kind}.{list} elements name a file"
                    );
                }
                MediaLayout::Outputs { list } => {
                    assert_eq!(properties[list]["type"], "array", "{kind}.{list} is a list");
                    assert!(
                        !properties[list]["items"]["properties"]["path"].is_null(),
                        "{kind}.{list} elements name a path"
                    );
                }
            }
        }
    }

    /// The three shapes turn into one list, and a descriptor that names nothing
    /// yields nothing rather than a name the caller would then try to open.
    #[test]
    fn every_descriptor_shape_becomes_one_list_of_names() {
        use serde_json::json;

        assert_eq!(
            super::media_files(
                &json!({"file": "proxy.mp4"}),
                MediaLayout::One { key: "file" }
            ),
            vec!["proxy.mp4"]
        );
        assert_eq!(
            super::media_files(
                &json!({"tiles": [{"file": "a.jpg", "t_ticks": 0}, {"file": "b.jpg"}]}),
                MediaLayout::Many { list: "tiles" }
            ),
            vec!["a.jpg", "b.jpg"]
        );
        assert_eq!(
            super::media_files(
                &json!({"outputs": [{"path": "clip.mp4", "bytes": 1}, {"path": "clip.srt"}]}),
                MediaLayout::Outputs { list: "outputs" }
            ),
            vec!["clip.mp4", "clip.srt"]
        );
        // Nothing named, and nothing invented: an entry missing its key is
        // skipped rather than turned into an empty path.
        for (value, layout) in [
            (json!({}), MediaLayout::One { key: "file" }),
            (json!({"file": 7}), MediaLayout::One { key: "file" }),
            (json!({"tiles": []}), MediaLayout::Many { list: "tiles" }),
            (
                json!({"tiles": [{"t_ticks": 0}]}),
                MediaLayout::Many { list: "tiles" },
            ),
            (
                json!({"tiles": "a.jpg"}),
                MediaLayout::Many { list: "tiles" },
            ),
        ] {
            assert!(super::media_files(&value, layout).is_empty());
        }
    }

    /// An extension nobody listed is refused. A browser handed the wrong type
    /// does something creative with the bytes.
    #[test]
    fn only_listed_extensions_get_a_media_type() {
        assert_eq!(media_type_for("proxy.mp4"), Some("video/mp4"));
        assert_eq!(media_type_for("strip_00001.jpg"), Some("image/jpeg"));
        for path in ["weights.bin", "notes.txt", "proxy", "", ".", "proxy.MP4"] {
            assert!(media_type_for(path).is_none(), "{path} was given a type");
        }
        assert_eq!(
            MEDIA_TYPES.len(),
            MEDIA_TYPES
                .iter()
                .map(|(extension, _)| *extension)
                .collect::<BTreeSet<_>>()
                .len()
        );
    }

    /// The refusals that matter, named. Weights are the one a renderer must
    /// never reach; the speech intermediates are the one a reader does not want.
    #[test]
    fn what_a_shell_may_not_read_is_refused_by_name() {
        for kind in [
            "evidence.device_profile.v1",
            "speech.vad.v1",
            "speech.asr.v1",
            "speech.alignment.v1",
            "media.audio_16k.v1",
            "media.audio_48k.v1",
            "evidence.demo.seed.v1",
            "",
        ] {
            assert!(document_for(kind).is_none(), "{kind} is readable");
        }
        // Media is a different door: a document kind is not streamable and a
        // media kind is not a document, except the render that is honestly both.
        for kind in [
            "speech.transcript.v1",
            "ranking.set.v1",
            "analysis.manifest.v1",
            "media.audio_peaks.v1",
            "evidence.device_profile.v1",
        ] {
            assert!(media_descriptor_for(kind).is_none(), "{kind} is streamable");
        }
        for kind in ["media.proxy.v1", "media.filmstrip.v1", "media.frames.v1"] {
            assert!(document_for(kind).is_none(), "{kind} is a document");
        }
    }

    /// One kind, one document. Two kinds sharing a filename is fine — several
    /// artifacts call their descriptor `index.json` — but one kind resolving two
    /// ways would make the served file depend on iteration order.
    #[test]
    fn every_kind_resolves_to_exactly_one_file() {
        let documents = DOCUMENTS
            .iter()
            .map(|(kind, _)| *kind)
            .collect::<BTreeSet<_>>();
        assert_eq!(documents.len(), DOCUMENTS.len(), "a kind is listed twice");
        let media = MEDIA
            .iter()
            .map(|(kind, _, _)| *kind)
            .collect::<BTreeSet<_>>();
        assert_eq!(media.len(), MEDIA.len(), "a kind is listed twice");
        assert_eq!(document_for("ranking.set.v1"), Some("ranking.json"));
        assert_eq!(
            media_descriptor_for("media.proxy.v1"),
            Some(("proxy.json", MediaLayout::One { key: "file" }))
        );
    }

    /// The chunk has to leave room for the response around it. A limit equal to
    /// the frame would fail on the first document that filled it.
    #[test]
    fn a_chunk_fits_inside_a_frame_with_room_to_spare() {
        let frame = u64::try_from(crate::ipc::MAX_FRAME_BYTES).expect("a frame size");
        assert!(MAX_CHUNK_BYTES < frame);
        assert!(
            MAX_CHUNK_BYTES * 2 <= frame,
            "the margin is thin enough that adding a response field could break it"
        );
    }
}
