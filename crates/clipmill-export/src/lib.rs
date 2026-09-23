//! Pure export validation, naming, metadata, and archive generation.
//!
//! The daemon performs delivery I/O. Naming previews and delivery both use
//! [`naming::Pattern`], so displayed names match written files. Time and byte counts
//! arrive as arguments rather than clock or filesystem reads.

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
