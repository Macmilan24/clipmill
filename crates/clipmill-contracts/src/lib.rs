//! Generated contract types. The sources of truth live in `contracts/`
//! (protobuf for IPC, JSON Schema for artifacts); this crate only re-exports
//! what `just codegen` emits into `src/gen/`. Hand edits are forbidden and
//! CI fails on regeneration drift.
#![allow(clippy::pedantic)]

// `#[path]` must be declared at file level so the relative path resolves
// against the real `src/` directory rather than virtual inline-module dirs.
// Generated code may unwrap on compile-time-constant patterns; the workspace
// deny applies to handwritten code only.
#[allow(clippy::unwrap_used, clippy::expect_used)]
#[path = "gen/proto/clipmill/ipc/v1/clipmill.ipc.v1.rs"]
mod gen_proto_ipc_v1;
#[allow(clippy::unwrap_used, clippy::expect_used)]
#[path = "gen/proto/clipmill/time/v1/clipmill.time.v1.rs"]
mod gen_proto_time_v1;
#[allow(clippy::unwrap_used, clippy::expect_used)]
#[path = "gen/schemas/artifact_manifest.rs"]
mod gen_schema_artifact_manifest;

/// IPC and worker-protocol messages (from `contracts/proto/`).
pub mod proto {
    pub mod time {
        pub mod v1 {
            pub use crate::gen_proto_time_v1::*;
        }
    }
    pub mod ipc {
        pub mod v1 {
            pub use crate::gen_proto_ipc_v1::*;
        }
    }
}

/// Artifact contracts (from `contracts/schemas/`).
pub mod schemas {
    pub mod artifact_manifest {
        pub use crate::gen_schema_artifact_manifest::*;
    }
}
