use std::{fmt, path::Path, str::FromStr};

use thiserror::Error;

const MANIFEST_NAME: &str = "manifest.json";

/// A normalized, portable UTF-8 path relative to an artifact directory.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ArtifactPath(String);

impl ArtifactPath {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn as_path(&self) -> &Path {
        Path::new(&self.0)
    }
}

impl fmt::Display for ArtifactPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for ArtifactPath {
    type Err = ArtifactPathError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty() {
            return Err(ArtifactPathError::Empty);
        }
        let has_windows_drive_prefix = value
            .as_bytes()
            .get(..2)
            .is_some_and(|prefix| prefix[0].is_ascii_alphabetic() && prefix[1] == b':');
        if value.starts_with('/') || value.ends_with('/') || has_windows_drive_prefix {
            return Err(ArtifactPathError::AbsoluteOrEmptyComponent);
        }
        if value.contains('\0') {
            return Err(ArtifactPathError::Nul);
        }
        if value.contains('\\') {
            return Err(ArtifactPathError::Backslash);
        }
        if value == MANIFEST_NAME {
            return Err(ArtifactPathError::Reserved);
        }
        for component in value.split('/') {
            if component.is_empty() || matches!(component, "." | "..") {
                return Err(ArtifactPathError::Traversal);
            }
        }
        Ok(Self(value.to_owned()))
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum ArtifactPathError {
    #[error("artifact path cannot be empty")]
    Empty,
    #[error("artifact path must be relative and cannot contain an empty edge component")]
    AbsoluteOrEmptyComponent,
    #[error("artifact path cannot contain NUL")]
    Nul,
    #[error("artifact path cannot contain backslashes")]
    Backslash,
    #[error("artifact path cannot contain '.', '..', or empty components")]
    Traversal,
    #[error("manifest.json is reserved by the artifact store")]
    Reserved,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{ArtifactPath, ArtifactPathError};

    #[test]
    fn accepts_normalized_nested_paths() {
        let path = "evidence/tracks.arrow"
            .parse::<ArtifactPath>()
            .expect("valid path");
        assert_eq!(path.as_str(), "evidence/tracks.arrow");
    }

    #[test]
    fn rejects_nonportable_and_escaping_paths() {
        for value in ["", "/root", "C:/root", "end/", "a//b", "a/../b", "a/./b"] {
            assert!(value.parse::<ArtifactPath>().is_err(), "accepted {value}");
        }
        assert_eq!(
            "a\\b".parse::<ArtifactPath>(),
            Err(ArtifactPathError::Backslash)
        );
        assert_eq!(
            "manifest.json".parse::<ArtifactPath>(),
            Err(ArtifactPathError::Reserved)
        );
    }
}
