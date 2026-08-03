//! Delivery: the part where a render stops being an artifact and becomes files
//! a person keeps.
//!
//! Four things live here and they share one property — none of them touches the
//! world. The strip that decides whether an export may start, the pattern that
//! decides what the files are called, the documents that describe what was
//! delivered, and the archive writer that packs a project's work into a format
//! that outlives this application. The daemon does the I/O; this crate decides
//! what the I/O should be.
//!
//! That split is what makes the naming preview honest. The shell shows a user
//! what their files will be called by asking the daemon to resolve the pattern,
//! and the daemon resolves it with [`naming::Pattern`] — the same function that
//! names the files. There is no second implementation to drift, which is the
//! same rule the editor's preview follows (`docs/preview-parity.md`).
//!
//! Nothing here reads a clock. Where a time or a byte count is needed it
//! arrives as an argument, so every function is one whose output can be
//! reproduced from its inputs.

pub mod archive;
pub mod naming;
pub mod package;
pub mod validate;
pub mod zip;

pub use archive::{
    ARCHIVE_INDEX_FILE, ARCHIVE_SCHEMA_VERSION, ArchiveEntry, ArchiveIndex, ArchivedSource,
    EntryKind,
};
pub use naming::{Fields, Pattern, PatternError, Token};
pub use package::{
    AudioSummary, CHECKSUMS_SUFFIX, DeliveredFile, Disclosure, ExportPackage, FileRole,
    PACKAGE_SCHEMA_VERSION, PACKAGE_SUFFIX, THUMBNAIL_SUFFIX, VideoSummary, checksum_file,
};
pub use validate::{
    Context, DURATION_GATE, Finding, RIGHTS_GATE_SECONDS, Report, Severity, estimate_bytes,
    validate,
};
pub use zip::{ZipError, ZipWriter};

use clipmill_core::Sha256Digest;
use sha2::{Digest, Sha256};

/// The hex digest of some bytes, in the form every document here records.
///
/// One helper rather than five call sites agreeing on a format: a checksum
/// written uppercase in one file and lowercase in another is a checksum a
/// verifier reports as a mismatch. It goes through the project's own digest
/// type rather than formatting the hasher's output directly, so "lower-case hex
/// without a prefix" is decided in one place for the whole codebase.
pub fn digest_of(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Sha256Digest::from_bytes(hasher.finalize().into()).to_hex()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::digest_of;

    #[test]
    fn the_digest_is_lowercase_hex_of_the_expected_length() {
        let digest = digest_of(b"");
        assert_eq!(
            digest,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(digest.len(), 64);
    }
}
