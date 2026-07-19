use std::{fmt, str::FromStr};

use thiserror::Error;
use ulid::Ulid;

const PROJECT_PREFIX: &str = "prj_";
const STAGING_PREFIX: &str = "stg_";

/// A stable project identifier serialized as `prj_<ULID>`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProjectId(String);

impl ProjectId {
    #[must_use]
    pub fn new() -> Self {
        Self(format!("{PROJECT_PREFIX}{}", Ulid::new()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ProjectId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ProjectId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl From<ProjectId> for String {
    fn from(value: ProjectId) -> Self {
        value.0
    }
}

impl FromStr for ProjectId {
    type Err = IdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let suffix = value
            .strip_prefix(PROJECT_PREFIX)
            .ok_or(IdError::WrongPrefix)?;
        let parsed = Ulid::from_string(suffix).map_err(|_| IdError::InvalidUlid)?;
        if suffix != parsed.to_string() {
            return Err(IdError::NonCanonical);
        }
        Ok(Self(value.to_owned()))
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum IdError {
    #[error("project id must start with 'prj_'")]
    WrongPrefix,
    #[error("project id contains an invalid ULID")]
    InvalidUlid,
    #[error("project id must use canonical uppercase ULID encoding")]
    NonCanonical,
}

/// A private artifact staging identifier serialized as `stg_<ULID>`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StagingId(String);

impl StagingId {
    #[must_use]
    pub fn new() -> Self {
        Self(format!("{STAGING_PREFIX}{}", Ulid::new()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for StagingId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for StagingId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for StagingId {
    type Err = StagingIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let suffix = value
            .strip_prefix(STAGING_PREFIX)
            .ok_or(StagingIdError::WrongPrefix)?;
        let parsed = Ulid::from_string(suffix).map_err(|_| StagingIdError::InvalidUlid)?;
        if suffix != parsed.to_string() {
            return Err(StagingIdError::NonCanonical);
        }
        Ok(Self(value.to_owned()))
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum StagingIdError {
    #[error("staging id must start with 'stg_'")]
    WrongPrefix,
    #[error("staging id contains an invalid ULID")]
    InvalidUlid,
    #[error("staging id must use canonical uppercase ULID encoding")]
    NonCanonical,
}

#[cfg(test)]
mod tests {
    use super::{IdError, ProjectId, StagingId, StagingIdError};

    #[test]
    fn generated_id_roundtrips() {
        let id = ProjectId::new();
        assert_eq!(id.as_str().parse::<ProjectId>(), Ok(id));
    }

    #[test]
    fn rejects_wrong_prefix_and_noncanonical_ulid() {
        assert_eq!(
            "job_01ARZ3NDEKTSV4RRFFQ69G5FAV".parse::<ProjectId>(),
            Err(IdError::WrongPrefix)
        );
        assert_eq!(
            "prj_01arz3ndektsv4rrffq69g5fav".parse::<ProjectId>(),
            Err(IdError::NonCanonical)
        );
        assert_eq!(
            "prj_not-a-ulid".parse::<ProjectId>(),
            Err(IdError::InvalidUlid)
        );
        assert_eq!("prj_".parse::<ProjectId>(), Err(IdError::InvalidUlid));
    }

    #[test]
    fn staging_id_roundtrips_and_validates_prefix() {
        let id = StagingId::new();
        assert_eq!(id.as_str().parse::<StagingId>(), Ok(id));
        assert_eq!(
            "prj_01ARZ3NDEKTSV4RRFFQ69G5FAV".parse::<StagingId>(),
            Err(StagingIdError::WrongPrefix)
        );
    }
}
