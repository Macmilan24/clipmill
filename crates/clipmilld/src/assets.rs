//! The person's own pictures and sounds — a logo, a music bed — kept by
//! content hash in one folder beside the artifact store.
//!
//! An asset is the file's bytes under their SHA-256, and a small record
//! beside it saying what they are: a kind decided from the bytes themselves,
//! never from a file name, the size or length the decoder read, the name the
//! person knew it by, and the licence they said they hold. A document names
//! an asset by its hash, so a logo is the same logo in every clip, and an
//! edit to a file somewhere else changes nothing that was made with it.
//!
//! Nothing is served from here by path. The shell derives an asset's file
//! from its hash and asks the daemon only whether that hash is one of these.

use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::media::MediaRunner;

/// The largest picture accepted, in bytes.
const MAX_IMAGE_BYTES: u64 = 20 * 1024 * 1024;
/// The largest sound accepted, in bytes: a long music bed, well encoded.
const MAX_AUDIO_BYTES: u64 = 200 * 1024 * 1024;
/// The largest side a picture may have.
const MAX_IMAGE_SIDE: i64 = 8_192;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AssetKind {
    Image,
    Audio,
}

impl AssetKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Audio => "audio",
        }
    }
}

/// What the store knows about one asset.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AssetRecord {
    /// `sha256:<hex>` of the bytes.
    pub hash: String,
    pub kind: AssetKind,
    /// The file name it was imported from, for the person to recognise it.
    pub name: String,
    pub media_type: String,
    pub bytes: u64,
    /// A picture's size; zero for a sound.
    pub width: i64,
    pub height: i64,
    /// A sound's length; zero for a picture.
    pub duration_ticks: i64,
    /// What the person said about their right to use it.
    pub license: String,
    pub added_unix_millis: u64,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum AssetError {
    #[error("that file cannot be read: {0}")]
    Unreadable(String),
    #[error("{0}")]
    Refused(String),
    #[error("the asset folder cannot be written: {0}")]
    Store(String),
}

/// The folder, and the decoder that reads what goes in it.
#[derive(Clone, Debug)]
pub(crate) struct AssetStore {
    dir: PathBuf,
    media: MediaRunner,
}

/// The licences a person may state for an asset.
pub(crate) const LICENSES: [&str; 4] = ["own_content", "licensed", "royalty_free", "public_domain"];

impl AssetStore {
    pub(crate) fn new(dir: PathBuf, media: MediaRunner) -> Result<Self, AssetError> {
        fs::create_dir_all(&dir).map_err(|error| AssetError::Store(error.to_string()))?;
        Ok(Self { dir, media })
    }

    /// Where an asset's bytes are, when it is one of these.
    pub(crate) fn file(&self, hash: &str) -> Option<PathBuf> {
        let hex = hex_of(hash)?;
        let path = self.dir.join(hex);
        (path.is_file() && self.dir.join(format!("{hex}.json")).is_file()).then_some(path)
    }

    pub(crate) fn get(&self, hash: &str) -> Option<AssetRecord> {
        let hex = hex_of(hash)?;
        let bytes = fs::read(self.dir.join(format!("{hex}.json"))).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    /// Every asset, newest first.
    pub(crate) fn list(&self) -> Vec<AssetRecord> {
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut records: Vec<AssetRecord> = entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let name = entry.file_name().into_string().ok()?;
                let hex = name.strip_suffix(".json")?;
                self.get(&format!("sha256:{hex}"))
            })
            .filter(|record| self.file(&record.hash).is_some())
            .collect();
        records.sort_by(|a, b| {
            b.added_unix_millis
                .cmp(&a.added_unix_millis)
                .then(a.name.cmp(&b.name))
        });
        records
    }

    /// Take a copy of a file the person chose. The same bytes imported twice
    /// are one asset; the later name and licence are kept.
    pub(crate) async fn import(
        &self,
        source: &Path,
        license: &str,
    ) -> Result<AssetRecord, AssetError> {
        if !LICENSES.contains(&license) {
            return Err(AssetError::Refused(
                "say whether you made it, licensed it, or it is royalty-free or public domain"
                    .to_owned(),
            ));
        }
        let metadata =
            fs::metadata(source).map_err(|error| AssetError::Unreadable(error.to_string()))?;
        if !metadata.is_file() {
            return Err(AssetError::Refused("that is not a file".to_owned()));
        }
        let mut head = [0_u8; 16];
        let read = fs::File::open(source)
            .and_then(|mut file| file.read(&mut head))
            .map_err(|error| AssetError::Unreadable(error.to_string()))?;
        let (kind, media_type) = sniff(&head[..read]).ok_or_else(|| {
            AssetError::Refused(
                "only PNG, JPEG or WebP pictures and MP3, WAV, FLAC, AAC, M4A or Ogg sounds can be used"
                    .to_owned(),
            )
        })?;
        let limit = match kind {
            AssetKind::Image => MAX_IMAGE_BYTES,
            AssetKind::Audio => MAX_AUDIO_BYTES,
        };
        if metadata.len() > limit {
            return Err(AssetError::Refused(format!(
                "that file is larger than the {} MB a {} may be",
                limit / 1024 / 1024,
                if kind == AssetKind::Image {
                    "picture"
                } else {
                    "sound"
                }
            )));
        }
        let (hex, staged) = self.stage(source)?;
        let final_path = self.dir.join(&hex);
        let (width, height, duration_ticks) = match self.measure(kind, &staged).await {
            Ok(measured) => measured,
            Err(error) => {
                let _ = fs::remove_file(&staged);
                return Err(error);
            }
        };
        if final_path.is_file() {
            let _ = fs::remove_file(&staged);
        } else {
            fs::rename(&staged, &final_path)
                .map_err(|error| AssetError::Store(error.to_string()))?;
        }
        let record = AssetRecord {
            hash: format!("sha256:{hex}"),
            kind,
            name: source
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("asset")
                .chars()
                .take(120)
                .collect(),
            media_type: media_type.to_owned(),
            bytes: metadata.len(),
            width,
            height,
            duration_ticks,
            license: license.to_owned(),
            added_unix_millis: unix_millis(),
        };
        self.write_record(&hex, &record)?;
        Ok(record)
    }

    /// Copy the file in while hashing it, under a name no reader looks for.
    fn stage(&self, source: &Path) -> Result<(String, PathBuf), AssetError> {
        let staged = self.dir.join(format!(".incoming-{}", ulid::Ulid::new()));
        let mut input =
            fs::File::open(source).map_err(|error| AssetError::Unreadable(error.to_string()))?;
        let mut output =
            fs::File::create(&staged).map_err(|error| AssetError::Store(error.to_string()))?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0_u8; 1024 * 1024];
        loop {
            let read = input
                .read(&mut buffer)
                .map_err(|error| AssetError::Unreadable(error.to_string()))?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
            output
                .write_all(&buffer[..read])
                .map_err(|error| AssetError::Store(error.to_string()))?;
        }
        output
            .sync_all()
            .map_err(|error| AssetError::Store(error.to_string()))?;
        Ok((hex::encode(hasher.finalize()), staged))
    }

    /// Its size or length, as the decoder reads it — which also proves the
    /// decoder can read it before any clip depends on it.
    async fn measure(&self, kind: AssetKind, path: &Path) -> Result<(i64, i64, i64), AssetError> {
        let probed = self
            .media
            .run_ffprobe_json(
                path.to_path_buf(),
                ["-show_format", "-show_streams"]
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            )
            .await
            .map_err(|_| AssetError::Refused("the decoder cannot read that file".to_owned()))?;
        let streams = probed["streams"].as_array().cloned().unwrap_or_default();
        match kind {
            AssetKind::Image => {
                let stream = streams
                    .iter()
                    .find(|stream| stream["codec_type"] == "video")
                    .ok_or_else(|| {
                        AssetError::Refused("that picture has no image in it".to_owned())
                    })?;
                let width = stream["width"].as_i64().unwrap_or(0);
                let height = stream["height"].as_i64().unwrap_or(0);
                if width <= 0 || height <= 0 || width > MAX_IMAGE_SIDE || height > MAX_IMAGE_SIDE {
                    return Err(AssetError::Refused(format!(
                        "a picture may be at most {MAX_IMAGE_SIDE} pixels on a side"
                    )));
                }
                Ok((width, height, 0))
            }
            AssetKind::Audio => {
                if !streams.iter().any(|stream| stream["codec_type"] == "audio") {
                    return Err(AssetError::Refused(
                        "that file has no sound in it".to_owned(),
                    ));
                }
                let seconds = probed["format"]["duration"]
                    .as_str()
                    .and_then(|value| value.parse::<f64>().ok())
                    .filter(|seconds| seconds.is_finite() && *seconds > 0.0)
                    .ok_or_else(|| AssetError::Refused("that sound has no length".to_owned()))?;
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "a sound's length in ticks, far inside an i64"
                )]
                let ticks = (seconds * 90_000.0).round() as i64;
                Ok((0, 0, ticks))
            }
        }
    }

    fn write_record(&self, hex: &str, record: &AssetRecord) -> Result<(), AssetError> {
        let staged = self.dir.join(format!(".record-{}", ulid::Ulid::new()));
        let bytes = serde_json::to_vec_pretty(record)
            .map_err(|error| AssetError::Store(error.to_string()))?;
        fs::write(&staged, bytes).map_err(|error| AssetError::Store(error.to_string()))?;
        fs::rename(&staged, self.dir.join(format!("{hex}.json")))
            .map_err(|error| AssetError::Store(error.to_string()))
    }
}

/// The hex of a `sha256:` address, when it is one.
fn hex_of(hash: &str) -> Option<&str> {
    let hex = hash.strip_prefix("sha256:")?;
    (hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()))
    .then_some(hex)
}

/// What a file is, from its first bytes.
fn sniff(head: &[u8]) -> Option<(AssetKind, &'static str)> {
    let starts = |magic: &[u8]| head.starts_with(magic);
    let at = |offset: usize, magic: &[u8]| head.get(offset..offset + magic.len()) == Some(magic);
    if starts(b"\x89PNG\r\n\x1a\n") {
        Some((AssetKind::Image, "image/png"))
    } else if starts(&[0xFF, 0xD8, 0xFF]) {
        Some((AssetKind::Image, "image/jpeg"))
    } else if starts(b"RIFF") && at(8, b"WEBP") {
        Some((AssetKind::Image, "image/webp"))
    } else if starts(b"RIFF") && at(8, b"WAVE") {
        Some((AssetKind::Audio, "audio/wav"))
    } else if starts(b"fLaC") {
        Some((AssetKind::Audio, "audio/flac"))
    } else if starts(b"OggS") {
        Some((AssetKind::Audio, "audio/ogg"))
    } else if starts(b"ID3") {
        Some((AssetKind::Audio, "audio/mpeg"))
    } else if at(4, b"ftyp")
        && (at(8, b"M4A ") || at(8, b"M4B ") || at(8, b"mp42") || at(8, b"isom"))
    {
        Some((AssetKind::Audio, "audio/mp4"))
    } else if head.len() >= 2 && head[0] == 0xFF && head[1] & 0xF6 == 0xF0 {
        Some((AssetKind::Audio, "audio/aac"))
    } else if head.len() >= 2 && head[0] == 0xFF && head[1] & 0xE0 == 0xE0 {
        Some((AssetKind::Audio, "audio/mpeg"))
    } else {
        None
    }
}

fn unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::{AssetKind, hex_of, sniff};

    #[test]
    fn a_file_is_what_its_bytes_say_whatever_it_is_called() {
        assert_eq!(
            sniff(b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR"),
            Some((AssetKind::Image, "image/png"))
        );
        assert_eq!(
            sniff(&[0xFF, 0xD8, 0xFF, 0xE0]),
            Some((AssetKind::Image, "image/jpeg"))
        );
        assert_eq!(
            sniff(b"RIFF\0\0\0\0WEBPVP8 "),
            Some((AssetKind::Image, "image/webp"))
        );
        assert_eq!(
            sniff(b"RIFF\0\0\0\0WAVEfmt "),
            Some((AssetKind::Audio, "audio/wav"))
        );
        assert_eq!(
            sniff(b"ID3\x04\0\0\0\0"),
            Some((AssetKind::Audio, "audio/mpeg"))
        );
        assert_eq!(
            sniff(b"\0\0\0\x20ftypM4A \0\0"),
            Some((AssetKind::Audio, "audio/mp4"))
        );
        assert_eq!(
            sniff(b"fLaC\0\0\0\x22"),
            Some((AssetKind::Audio, "audio/flac"))
        );
        assert_eq!(sniff(b"<svg xmlns="), None, "not a picture this can draw");
        assert_eq!(sniff(b"GIF89a"), None);
    }

    /// A picture and a sound made by the pinned FFmpeg, brought in, listed
    /// and found again by hash; the same bytes twice are one asset; a file
    /// named like a picture that is not one is refused.
    #[tokio::test]
    #[ignore = "needs the pinned FFmpeg sidecar in .cache/bin"]
    async fn pictures_and_sounds_come_in_by_their_bytes() {
        use super::{AssetStore, LICENSES};
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.cache/bin");
        let temp = tempfile::tempdir().expect("temp");
        let media =
            crate::media::MediaRunner::new(root.join("ffprobe"), temp.path().join("scratch"))
                .expect("media runner");
        let store = AssetStore::new(temp.path().join("assets"), media).expect("store");
        let make = |args: &[&str]| {
            let status = std::process::Command::new(root.join("ffmpeg"))
                .current_dir(temp.path())
                .args(["-hide_banner", "-loglevel", "error", "-y"])
                .args(args)
                .status()
                .expect("ffmpeg");
            assert!(status.success());
        };
        make(&[
            "-f",
            "lavfi",
            "-i",
            "color=c=0x00FF00:s=300x120:d=1",
            "-frames:v",
            "1",
            "logo.png",
        ]);
        make(&[
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=220:duration=3",
            "-c:a",
            "libmp3lame",
            "bed.mp3",
        ]);
        std::fs::write(
            temp.path().join("fake.png"),
            b"<svg xmlns='http://www.w3.org/2000/svg'/>",
        )
        .expect("fake");

        let logo = store
            .import(&temp.path().join("logo.png"), LICENSES[0])
            .await
            .expect("a picture");
        assert_eq!(
            (logo.kind, logo.width, logo.height),
            (AssetKind::Image, 300, 120)
        );
        assert_eq!(logo.media_type, "image/png");
        let bed = store
            .import(&temp.path().join("bed.mp3"), "royalty_free")
            .await
            .expect("a sound");
        assert_eq!(bed.kind, AssetKind::Audio);
        assert!(
            (bed.duration_ticks - 3 * 90_000).abs() < 9_000,
            "{}",
            bed.duration_ticks
        );

        let again = store
            .import(&temp.path().join("logo.png"), "licensed")
            .await
            .expect("the same picture");
        assert_eq!(again.hash, logo.hash);
        assert_eq!(store.list().len(), 2);
        assert_eq!(store.get(&logo.hash).expect("found").license, "licensed");
        assert!(store.file(&logo.hash).is_some());

        assert!(
            store
                .import(&temp.path().join("fake.png"), LICENSES[0])
                .await
                .is_err()
        );
        assert!(
            store
                .import(&temp.path().join("logo.png"), "who knows")
                .await
                .is_err()
        );
        assert_eq!(store.list().len(), 2, "nothing refused was kept");
    }

    #[test]
    fn only_a_lowercase_sha256_address_names_an_asset() {
        let hex = "a".repeat(64);
        assert_eq!(hex_of(&format!("sha256:{hex}")), Some(hex.as_str()));
        assert_eq!(hex_of(&format!("sha256:{}", "A".repeat(64))), None);
        assert_eq!(hex_of("sha256:../../etc/passwd"), None);
        assert_eq!(hex_of(&hex), None);
    }
}
