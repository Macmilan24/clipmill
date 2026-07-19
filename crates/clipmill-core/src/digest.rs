use std::{fmt, str::FromStr};

use thiserror::Error;

const SHA256_PREFIX: &str = "sha256:";
const SHA256_HEX_LENGTH: usize = 64;

/// A canonical lower-case SHA-256 digest without a semantic prefix.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Sha256Digest([u8; 32]);

impl Sha256Digest {
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    #[must_use]
    pub fn to_hex(self) -> String {
        hex::encode(self.0)
    }
}

impl fmt::Display for Sha256Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&hex::encode(self.0))
    }
}

impl FromStr for Sha256Digest {
    type Err = DigestError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() != SHA256_HEX_LENGTH {
            return Err(DigestError::WrongLength);
        }
        if value
            .bytes()
            .any(|byte| !byte.is_ascii_digit() && !(b'a'..=b'f').contains(&byte))
        {
            return Err(DigestError::NonCanonicalHex);
        }
        let mut bytes = [0_u8; 32];
        hex::decode_to_slice(value, &mut bytes).map_err(|_| DigestError::InvalidHex)?;
        Ok(Self(bytes))
    }
}

/// The deterministic cache identity of a committed artifact.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ArtifactId(Sha256Digest);

impl ArtifactId {
    #[must_use]
    pub const fn from_digest(digest: Sha256Digest) -> Self {
        Self(digest)
    }

    #[must_use]
    pub const fn digest(self) -> Sha256Digest {
        self.0
    }

    #[must_use]
    pub fn hex(self) -> String {
        self.0.to_hex()
    }
}

impl fmt::Display for ArtifactId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{SHA256_PREFIX}{}", self.0)
    }
}

impl FromStr for ArtifactId {
    type Err = DigestError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let digest = value
            .strip_prefix(SHA256_PREFIX)
            .ok_or(DigestError::MissingPrefix)?
            .parse()?;
        Ok(Self(digest))
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum DigestError {
    #[error("digest must start with 'sha256:'")]
    MissingPrefix,
    #[error("SHA-256 digest must contain exactly 64 hexadecimal characters")]
    WrongLength,
    #[error("SHA-256 digest must use canonical lower-case hexadecimal")]
    NonCanonicalHex,
    #[error("SHA-256 digest contains invalid hexadecimal")]
    InvalidHex,
}

#[cfg(test)]
mod tests {
    use super::{ArtifactId, DigestError, Sha256Digest};

    #[test]
    fn digest_and_artifact_id_roundtrip_canonically() {
        let digest = Sha256Digest::from_bytes([0xab; 32]);
        let artifact = ArtifactId::from_digest(digest);
        assert_eq!(digest.to_string(), "ab".repeat(32));
        assert_eq!(artifact.to_string(), format!("sha256:{}", "ab".repeat(32)));
        assert_eq!(artifact.to_string().parse::<ArtifactId>(), Ok(artifact));
    }

    #[test]
    fn artifact_id_rejects_prefix_length_and_uppercase() {
        assert_eq!(
            "ab".repeat(32).parse::<ArtifactId>(),
            Err(DigestError::MissingPrefix)
        );
        assert_eq!(
            "sha256:abcd".parse::<ArtifactId>(),
            Err(DigestError::WrongLength)
        );
        assert_eq!(
            format!("sha256:{}", "AB".repeat(32)).parse::<ArtifactId>(),
            Err(DigestError::NonCanonicalHex)
        );
    }
}
