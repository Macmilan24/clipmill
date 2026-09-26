//! The `clipmill-media://` protocol for seekable media outside the control socket.
//!
//! The daemon authorizes project ownership, artifact kind, and descriptor files.
//! This host derives object paths from content addresses and serves only listed
//! files with declared media types; it never accepts filesystem paths from the
//! daemon. Invalid requests are rejected before opening a file.
//!
//! Cache authorized inventories because artifacts and project ownership are
//! immutable, avoiding one control-socket round trip per filmstrip tile.

use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    sync::Arc,
};

use tauri::http::{Request, Response, StatusCode, header};
use tokio::sync::Mutex;

use crate::DaemonSupervisor;

/// The scheme the renderer addresses media by.
pub const SCHEME: &str = "clipmill-media";

/// The most one response carries when the caller asked for everything.
///
/// A WebView asking for a whole video without a Range header gets the first
/// slice and a `content-length` for the rest, which is what makes it ask again
/// with a range. Serving hundreds of megabytes into one buffer would be the
/// alternative.
const MAX_SPAN_BYTES: u64 = 4 * 1024 * 1024;

/// One artifact's authorized inventory: what may be served, and how large.
#[derive(Clone, Debug)]
struct Inventory {
    files: HashMap<String, FileEntry>,
}

#[derive(Clone, Debug)]
struct FileEntry {
    bytes: u64,
    media_type: String,
}

/// Serves the scheme, holding the store root and what the daemon has authorized.
pub struct MediaProtocol {
    supervisor: Arc<DaemonSupervisor>,
    artifacts_dir: PathBuf,
    /// The pinned caption fonts, served so the player draws captions with
    /// the faces the render burns in.
    fonts_dir: Option<PathBuf>,
    /// The person's own pictures and sounds, by content hash.
    assets_dir: Option<PathBuf>,
    /// Assets the daemon has said are its own, and how to serve each.
    assets: Mutex<HashMap<String, FileEntry>>,
    authorized: Mutex<HashMap<(String, String), Inventory>>,
}

/// The path prefix caption fonts are served under.
const FONTS_PREFIX: &str = "/fonts/";
/// The path prefix assets are served under, by the hex of their hash.
const ASSETS_PREFIX: &str = "/assets/";

impl std::fmt::Debug for MediaProtocol {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MediaProtocol")
            .field("artifacts_dir", &self.artifacts_dir)
            .finish_non_exhaustive()
    }
}

impl MediaProtocol {
    pub fn new(supervisor: Arc<DaemonSupervisor>, artifacts_dir: PathBuf) -> Self {
        Self {
            supervisor,
            artifacts_dir,
            fonts_dir: None,
            assets_dir: None,
            assets: Mutex::new(HashMap::new()),
            authorized: Mutex::new(HashMap::new()),
        }
    }

    /// Serve the caption fonts pinned in this directory.
    #[must_use]
    pub fn with_fonts(mut self, fonts_dir: PathBuf) -> Self {
        self.fonts_dir = Some(fonts_dir);
        self
    }

    /// Serve the person's assets from this folder.
    #[must_use]
    pub fn with_assets(mut self, assets_dir: PathBuf) -> Self {
        self.assets_dir = Some(assets_dir);
        self
    }

    /// One asset, by the hex of its hash, once the daemon says it is one.
    /// The file is derived from the hash; nothing else in the folder — the
    /// records beside the files, a file being brought in — can be named.
    async fn serve_asset(&self, hex: &str, range: Option<&str>) -> Response<Vec<u8>> {
        if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return refuse(StatusCode::BAD_REQUEST, "malformed asset address");
        }
        let Some(dir) = &self.assets_dir else {
            return refuse(StatusCode::NOT_FOUND, "no asset folder");
        };
        let known = self.assets.lock().await.get(hex).cloned();
        let entry = match known {
            Some(entry) => entry,
            None => match self
                .supervisor
                .client()
                .resolve_asset(format!("sha256:{hex}"))
                .await
            {
                Ok(resolved) => {
                    let entry = FileEntry {
                        bytes: resolved.bytes,
                        media_type: resolved.media_type,
                    };
                    self.assets
                        .lock()
                        .await
                        .insert(hex.to_owned(), entry.clone());
                    entry
                }
                Err(_) => return refuse(StatusCode::NOT_FOUND, "not an asset"),
            },
        };
        let range = range.and_then(|value| Range::parse(value, entry.bytes));
        read_span(&dir.join(hex), &entry, range)
    }

    /// One caption font, by the file name its catalogue entry pins. Nothing
    /// else in the directory is reachable: the name must be a catalogued
    /// face's, so a URL cannot ask for a licence text or another file.
    fn serve_font(&self, name: &str) -> Response<Vec<u8>> {
        let Some(face) = clipmill_captions::FONTS
            .iter()
            .find(|face| face.file == name)
        else {
            return refuse(StatusCode::NOT_FOUND, "not a caption font");
        };
        let Some(dir) = &self.fonts_dir else {
            return refuse(StatusCode::NOT_FOUND, "caption fonts are not installed");
        };
        let Ok(bytes) = std::fs::read(dir.join(face.file)) else {
            return refuse(StatusCode::NOT_FOUND, "this caption font is not installed");
        };
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "font/ttf")
            .header(header::CONTENT_LENGTH, bytes.len().to_string())
            .header(header::CACHE_CONTROL, "public, max-age=86400")
            .body(bytes)
            .unwrap_or_else(|_| refuse(StatusCode::INTERNAL_SERVER_ERROR, "cannot answer"))
    }

    /// Answer one media request.
    pub async fn serve(&self, request: Request<Vec<u8>>) -> Response<Vec<u8>> {
        let origin = request.headers().get(header::ORIGIN).cloned();
        if let Some(origin) = &origin
            && !origin.to_str().is_ok_and(allowed_origin)
        {
            return refuse(
                StatusCode::FORBIDDEN,
                "media is available only to the application",
            );
        }
        let mut response = self.serve_inner(request).await;
        if let Some(origin) = origin {
            response
                .headers_mut()
                .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
            response.headers_mut().insert(
                header::VARY,
                tauri::http::HeaderValue::from_static("Origin"),
            );
            response.headers_mut().insert(
                header::ACCESS_CONTROL_EXPOSE_HEADERS,
                tauri::http::HeaderValue::from_static(
                    "Content-Length, Content-Range, Accept-Ranges",
                ),
            );
        }
        response
    }

    async fn serve_inner(&self, request: Request<Vec<u8>>) -> Response<Vec<u8>> {
        if request.method() != tauri::http::Method::GET {
            return refuse(StatusCode::METHOD_NOT_ALLOWED, "media supports GET only");
        }
        if let Some(name) = request.uri().path().strip_prefix(FONTS_PREFIX) {
            return self.serve_font(name);
        }
        if let Some(hex) = request.uri().path().strip_prefix(ASSETS_PREFIX) {
            let range = request
                .headers()
                .get(header::RANGE)
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned);
            return self.serve_asset(hex, range.as_deref()).await;
        }
        let Some(target) = Target::parse(request.uri().path()) else {
            return refuse(StatusCode::BAD_REQUEST, "malformed media path");
        };
        let entry = match self.authorize(&target).await {
            Ok(entry) => entry,
            Err(response) => return response,
        };
        let range = request
            .headers()
            .get(header::RANGE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| Range::parse(value, entry.bytes));
        if request
            .headers()
            .get(header::RANGE)
            .is_some_and(|value| value.to_str().is_ok())
            && range.is_none()
        {
            // A range the file cannot satisfy is answered as such rather than
            // silently served whole, because a player that asked to seek and got
            // the beginning would play the wrong thing.
            return Response::builder()
                .status(StatusCode::RANGE_NOT_SATISFIABLE)
                .header(header::CONTENT_RANGE, format!("bytes */{}", entry.bytes))
                .body(Vec::new())
                .unwrap_or_else(|_| refuse(StatusCode::INTERNAL_SERVER_ERROR, "cannot answer"));
        }
        self.read(&target, &entry, range)
    }

    /// The daemon's answer about this artifact, cached because it cannot change.
    async fn authorize(&self, target: &Target) -> Result<FileEntry, Response<Vec<u8>>> {
        let key = (target.project_id.clone(), target.artifact_id.clone());
        if let Some(inventory) = self.authorized.lock().await.get(&key)
            && let Some(entry) = inventory.files.get(&target.file)
        {
            return Ok(entry.clone());
        }
        let resolved = self
            .supervisor
            .client()
            .resolve_media(&target.project_id, &target.artifact_id)
            .await
            .map_err(|error| {
                // The daemon's refusals are policy — not this project's artifact,
                // not a streamable kind — and they are not this layer's to
                // reinterpret. Forbidden, with the daemon's own words.
                tracing::debug!(%error, "media resolve refused");
                refuse(
                    StatusCode::FORBIDDEN,
                    "not a media artifact for this project",
                )
            })?;
        let inventory = Inventory {
            files: resolved
                .files
                .into_iter()
                .map(|file| {
                    (
                        file.path,
                        FileEntry {
                            bytes: file.bytes,
                            media_type: file.media_type,
                        },
                    )
                })
                .collect(),
        };
        let entry = inventory.files.get(&target.file).cloned();
        self.authorized.lock().await.insert(key, inventory);
        // Named by the URL but not by the descriptor: the artifact exists and the
        // project owns it, and this file is still not one of its own.
        entry.ok_or_else(|| refuse(StatusCode::NOT_FOUND, "the artifact names no such file"))
    }

    /// Open the one authorized file and answer the requested span.
    fn read(&self, target: &Target, entry: &FileEntry, range: Option<Range>) -> Response<Vec<u8>> {
        // Derived, never received. The daemon hands out no paths, so there is
        // nothing in its answer that could point outside the store — and the
        // file name came from the artifact's own descriptor by way of the
        // inventory, so it cannot traverse either.
        let Some(digest) = target.artifact_id.strip_prefix("sha256:") else {
            return refuse(StatusCode::BAD_REQUEST, "malformed artifact address");
        };
        let path = self
            .artifacts_dir
            .join("objects/sha256")
            .join(&digest[..2])
            .join(digest)
            .join(&target.file);
        read_span(&path, entry, range)
    }
}

/// The span of a file a request asked for, or its first slice when it asked
/// for everything.
fn read_span(path: &std::path::Path, entry: &FileEntry, range: Option<Range>) -> Response<Vec<u8>> {
    {
        let Ok(mut file) = File::open(path) else {
            return refuse(StatusCode::NOT_FOUND, "the media file is not in the store");
        };
        let (start, end) = match range {
            Some(range) => (range.start, range.end),
            None => (0, entry.bytes.saturating_sub(1).min(MAX_SPAN_BYTES - 1)),
        };
        let length = end.saturating_sub(start).saturating_add(1);
        if file.seek(SeekFrom::Start(start)).is_err() {
            return refuse(
                StatusCode::INTERNAL_SERVER_ERROR,
                "cannot position the file",
            );
        }
        let mut body = vec![0_u8; usize::try_from(length).unwrap_or(0)];
        if file.read_exact(&mut body).is_err() {
            return refuse(StatusCode::INTERNAL_SERVER_ERROR, "the file ended early");
        }
        let partial = start > 0 || end + 1 < entry.bytes;
        let mut builder = Response::builder()
            .status(if partial {
                StatusCode::PARTIAL_CONTENT
            } else {
                StatusCode::OK
            })
            .header(header::CONTENT_TYPE, &entry.media_type)
            .header(header::CONTENT_LENGTH, length.to_string())
            // Seeking needs the player to know ranges are honoured.
            .header(header::ACCEPT_RANGES, "bytes")
            // Content-addressed: the bytes behind this URL can never change.
            .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable");
        if partial {
            builder = builder.header(
                header::CONTENT_RANGE,
                format!("bytes {start}-{end}/{}", entry.bytes),
            );
        }
        builder
            .body(body)
            .unwrap_or_else(|_| refuse(StatusCode::INTERNAL_SERVER_ERROR, "cannot answer"))
    }
}

/// What a media URL names: `/<project>/<artifact>/<file>`.
#[derive(Debug, Eq, PartialEq)]
struct Target {
    project_id: String,
    artifact_id: String,
    file: String,
}

impl Target {
    fn parse(path: &str) -> Option<Self> {
        let mut parts = path.trim_start_matches('/').splitn(3, '/');
        // The renderer builds these URLs with each segment percent-encoded —
        // an artifact address carries a colon, and `sha256%3A…` is what the
        // WebView sends. Decoded here, segment by segment, before anything is
        // matched: taken as written, every address failed to parse at the
        // daemon and every proxy and thumbnail was a 403 in the window.
        let project_id = percent_decode(parts.next().filter(|part| !part.is_empty())?)?;
        let artifact_id = percent_decode(parts.next().filter(|part| !part.is_empty())?)?;
        let file = parts
            .next()
            .filter(|part| !part.is_empty())?
            .split('/')
            .map(|segment| {
                // A slash that was encoded to hide inside one segment is a
                // segment nobody named, not a directory.
                percent_decode(segment).filter(|decoded| !decoded.contains('/'))
            })
            .collect::<Option<Vec<_>>>()?
            .join("/");
        // Refused rather than normalised. A file name is matched against the
        // inventory further down, so traversal could not escape anyway — but a
        // path that tries is a request nobody should have made, and answering it
        // at all would make the inventory the only thing standing in the way.
        if project_id.contains('/')
            || artifact_id.contains('/')
            || file.is_empty()
            || file.contains("..")
            || file.contains('\\')
            || file.starts_with('/')
        {
            return None;
        }
        Some(Self {
            project_id,
            artifact_id,
            file,
        })
    }
}

/// One path segment with its `%XX` escapes resolved; none for a malformed
/// escape or bytes that are not UTF-8.
fn percent_decode(segment: &str) -> Option<String> {
    let bytes = segment.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hex = std::str::from_utf8(bytes.get(index + 1..index + 3)?).ok()?;
            decoded.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

/// An inclusive byte span, resolved against a known length.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Range {
    start: u64,
    end: u64,
}

impl Range {
    /// Parse the one Range form a media player actually sends.
    ///
    /// `bytes=start-end`, `bytes=start-`, and `bytes=-suffix`. Multi-range is not
    /// answered: it requires a multipart body, no player asks for it while
    /// seeking, and pretending to support it would mean returning one range and
    /// calling it all of them.
    fn parse(header: &str, total: u64) -> Option<Self> {
        let spec = header.trim().strip_prefix("bytes=")?;
        if spec.contains(',') || total == 0 {
            return None;
        }
        let (from, to) = spec.split_once('-')?;
        let (start, end) = match (from.trim(), to.trim()) {
            ("", "") => return None,
            // A suffix range: the last N bytes.
            ("", suffix) => {
                let suffix: u64 = suffix.parse().ok()?;
                if suffix == 0 {
                    return None;
                }
                (total.saturating_sub(suffix), total - 1)
            }
            (start, "") => (start.parse().ok()?, total - 1),
            (start, end) => (start.parse().ok()?, end.parse::<u64>().ok()?.min(total - 1)),
        };
        if start > end || start >= total {
            return None;
        }
        // Bounded here as well as at the source: a player asking for a whole
        // file in one range should get a first slice, not the whole thing in
        // memory.
        Some(Self {
            start,
            end: end.min(start.saturating_add(MAX_SPAN_BYTES - 1)),
        })
    }
}

fn refuse(status: StatusCode, reason: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(reason.as_bytes().to_vec())
        .unwrap_or_else(|_| Response::new(Vec::new()))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{MAX_SPAN_BYTES, Range, Target};

    #[test]
    fn a_media_url_names_a_project_an_artifact_and_a_file() {
        let target = Target::parse("/prj_01/sha256:aa/proxy.mp4").expect("a target");
        assert_eq!(target.project_id, "prj_01");
        assert_eq!(target.artifact_id, "sha256:aa");
        assert_eq!(target.file, "proxy.mp4");
    }

    /// The URL as the renderer actually builds it: every segment through
    /// `encodeURIComponent`, so the address arrives as `sha256%3Aaa`. That
    /// is the proxy behind every black player and the thumbnail behind every
    /// blank card until this decoded it.
    #[test]
    fn a_media_url_arrives_percent_encoded_and_is_decoded_segment_by_segment() {
        let target = Target::parse("/prj_01/sha256%3Aaa/proxy.mp4").expect("a target");
        assert_eq!(target.artifact_id, "sha256:aa");
        assert_eq!(target.file, "proxy.mp4");
        // An encoded slash inside a segment is not a directory: no segment is
        // named "tiles/000.jpg", and the request is refused rather than read
        // as tiles/000.jpg.
        assert!(Target::parse("/prj_01/sha256%3Aaa/tiles%2F000.jpg").is_none());
        let nested = Target::parse("/prj_01/sha256%3Aaa/tiles/000.jpg").expect("a target");
        assert_eq!(nested.file, "tiles/000.jpg");
        let spaced = Target::parse("/prj_01/sha256%3Aaa/proxy%20final.mp4").expect("a target");
        assert_eq!(spaced.file, "proxy final.mp4");
        for malformed in [
            "/prj/sha256%3/proxy.mp4",
            "/prj/sha256%zz/proxy.mp4",
            "/prj/%ff/x",
        ] {
            assert!(
                Target::parse(malformed).is_none(),
                "{malformed} was accepted"
            );
        }
    }

    /// Traversal is refused at the door even though the inventory would catch it
    /// anyway. Two locks, because the inner one is a list of names and the outer
    /// one is about what a request is allowed to look like.
    #[test]
    fn a_path_that_tries_to_escape_is_refused() {
        for path in [
            "/prj/sha256:aa/../../../etc/passwd",
            "/prj/sha256:aa/..%2fsecret",
            "/prj/sha256:aa/sub\\dir",
            "/prj/sha256:aa//etc/passwd",
            "/prj/sha256:aa",
            "/prj",
            "/",
            "",
            "/prj//proxy.mp4",
        ] {
            assert!(Target::parse(path).is_none(), "{path} was accepted");
        }
    }

    /// A file name with a directory in it is kept whole rather than split: the
    /// inventory holds exactly the names the descriptor gave, so a nested name
    /// either matches one of them or matches nothing.
    #[test]
    fn a_nested_file_name_stays_one_name() {
        let target = Target::parse("/prj/sha256:aa/tiles/strip_00001.jpg").expect("a target");
        assert_eq!(target.file, "tiles/strip_00001.jpg");
    }

    #[test]
    fn the_three_range_forms_a_player_sends_all_resolve() {
        assert_eq!(
            Range::parse("bytes=0-99", 1000),
            Some(Range { start: 0, end: 99 })
        );
        assert_eq!(
            Range::parse("bytes=500-", 1000),
            Some(Range {
                start: 500,
                end: 999
            })
        );
        assert_eq!(
            Range::parse("bytes=-100", 1000),
            Some(Range {
                start: 900,
                end: 999
            })
        );
        // An end past the file is clamped rather than refused: that is what a
        // player sends when it does not know the length yet.
        assert_eq!(
            Range::parse("bytes=900-99999", 1000),
            Some(Range {
                start: 900,
                end: 999
            })
        );
    }

    #[test]
    fn a_range_the_file_cannot_satisfy_is_refused() {
        for header in [
            "bytes=1000-",       // starts at the end
            "bytes=2000-",       // starts past it
            "bytes=500-100",     // backwards
            "bytes=-0",          // an empty suffix
            "bytes=-",           // nothing at all
            "items=0-10",        // not bytes
            "0-10",              // no unit
            "bytes=0-10, 20-30", // multi-range, which this does not answer
            "",
        ] {
            assert!(
                Range::parse(header, 1000).is_none(),
                "{header} was accepted"
            );
        }
        // A zero-length file has no satisfiable range at all.
        assert!(Range::parse("bytes=0-", 0).is_none());
    }

    /// A player asking for everything gets a bounded slice. Without this a
    /// single request for a two-hour proxy would be read into one buffer.
    #[test]
    fn an_unbounded_range_is_capped_rather_than_read_whole() {
        let huge = MAX_SPAN_BYTES * 10;
        let range = Range::parse("bytes=0-", huge).expect("a range");
        assert_eq!(range.start, 0);
        assert_eq!(range.end, MAX_SPAN_BYTES - 1);
    }
}

/// No wildcard CORS: only the production WebView and the fixed local dev server.
fn allowed_origin(origin: &str) -> bool {
    matches!(
        origin,
        "tauri://localhost" | "http://tauri.localhost" | "https://tauri.localhost"
    ) || (cfg!(debug_assertions)
        && matches!(origin, "http://localhost:5173" | "http://127.0.0.1:5173"))
}

#[cfg(test)]
mod cors_tests {
    #[test]
    fn only_the_application_origin_can_use_decoded_media() {
        assert!(super::allowed_origin("tauri://localhost"));
        assert!(!super::allowed_origin(
            "https://tauri.localhost.evil.example"
        ));
        assert!(!super::allowed_origin("null"));
        assert!(!super::allowed_origin("https://example.com"));
    }
}
