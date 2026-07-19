use std::{fmt, str::FromStr};

use thiserror::Error;
use ulid::Ulid;

const PROJECT_PREFIX: &str = "prj_";
const STAGING_PREFIX: &str = "stg_";
const SOURCE_PREFIX: &str = "src_";
const JOB_PREFIX: &str = "job_";
const TASK_PREFIX: &str = "tsk_";
const LEASE_PREFIX: &str = "lse_";
const WORKER_PREFIX: &str = "wrk_";

/// A durable task-event replay cursor.
///
/// Cursor zero means "from the beginning". Positive values are SQLite row
/// identifiers, so values above SQLite's signed 64-bit range are rejected at
/// the protocol boundary rather than surfacing as storage failures.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TaskEventCursor(u64);

impl TaskEventCursor {
    pub const BEGINNING: Self = Self(0);

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl TryFrom<u64> for TaskEventCursor {
    type Error = TaskEventCursorError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value > i64::MAX as u64 {
            return Err(TaskEventCursorError::OutOfRange);
        }
        Ok(Self(value))
    }
}

impl From<TaskEventCursor> for u64 {
    fn from(value: TaskEventCursor) -> Self {
        value.0
    }
}

impl fmt::Display for TaskEventCursor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum TaskEventCursorError {
    #[error("task-event cursor exceeds the supported monotonic range")]
    OutOfRange,
}

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

macro_rules! define_prefixed_id {
    (
        $(#[$type_meta:meta])*
        $type_name:ident,
        $error_name:ident,
        $prefix:ident
    ) => {
        $(#[$type_meta])*
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $type_name(String);

        impl $type_name {
            #[must_use]
            pub fn new() -> Self {
                Self(format!("{}{}", $prefix, Ulid::new()))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl Default for $type_name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $type_name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl From<$type_name> for String {
            fn from(value: $type_name) -> Self {
                value.0
            }
        }

        impl FromStr for $type_name {
            type Err = $error_name;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                let suffix = value
                    .strip_prefix($prefix)
                    .ok_or($error_name::WrongPrefix)?;
                let parsed = Ulid::from_string(suffix).map_err(|_| $error_name::InvalidUlid)?;
                if suffix != parsed.to_string() {
                    return Err($error_name::NonCanonical);
                }
                Ok(Self(value.to_owned()))
            }
        }

        #[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
        pub enum $error_name {
            #[error("identifier has the wrong prefix")]
            WrongPrefix,
            #[error("identifier contains an invalid ULID")]
            InvalidUlid,
            #[error("identifier must use canonical uppercase ULID encoding")]
            NonCanonical,
        }
    };
}

define_prefixed_id!(
    /// An immutable source reference identifier serialized as `src_<ULID>`.
    SourceId,
    SourceIdError,
    SOURCE_PREFIX
);
define_prefixed_id!(
    /// A durable job identifier serialized as `job_<ULID>`.
    JobId,
    JobIdError,
    JOB_PREFIX
);
define_prefixed_id!(
    /// A durable task identifier serialized as `tsk_<ULID>`.
    TaskId,
    TaskIdError,
    TASK_PREFIX
);
define_prefixed_id!(
    /// A task lease identifier serialized as `lse_<ULID>`.
    LeaseId,
    LeaseIdError,
    LEASE_PREFIX
);
define_prefixed_id!(
    /// A worker process identifier serialized as `wrk_<ULID>`.
    WorkerId,
    WorkerIdError,
    WORKER_PREFIX
);

#[cfg(test)]
mod tests {
    use super::{
        IdError, JobId, LeaseId, ProjectId, SourceId, StagingId, StagingIdError, TaskEventCursor,
        TaskEventCursorError, TaskId, WorkerId,
    };

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

    #[test]
    fn orchestration_ids_roundtrip_and_are_prefix_separated() {
        let source = SourceId::new();
        let job = JobId::new();
        let task = TaskId::new();
        let lease = LeaseId::new();
        let worker = WorkerId::new();
        assert_eq!(source.as_str().parse::<SourceId>(), Ok(source));
        assert_eq!(job.as_str().parse::<JobId>(), Ok(job.clone()));
        assert_eq!(task.as_str().parse::<TaskId>(), Ok(task));
        assert_eq!(lease.as_str().parse::<LeaseId>(), Ok(lease));
        assert_eq!(worker.as_str().parse::<WorkerId>(), Ok(worker));
        assert!(job.as_str().parse::<TaskId>().is_err());
    }

    #[test]
    fn task_event_cursor_accepts_beginning_and_sqlite_range() {
        assert_eq!(
            TaskEventCursor::try_from(0).map(TaskEventCursor::get),
            Ok(0)
        );
        assert_eq!(
            TaskEventCursor::try_from(i64::MAX as u64).map(TaskEventCursor::get),
            Ok(i64::MAX as u64)
        );
        assert_eq!(
            TaskEventCursor::try_from(i64::MAX as u64 + 1),
            Err(TaskEventCursorError::OutOfRange)
        );
    }
}
