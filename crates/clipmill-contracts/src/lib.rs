//! Generated contract types. The sources of truth live in `contracts/`
//! (protobuf for IPC, JSON Schema for artifacts); this crate only re-exports
//! what `just codegen` emits into `src/gen/`. Hand edits are forbidden and
//! CI fails on regeneration drift.
//!
//! The `proto` module tree mirrors the proto package hierarchy exactly, so
//! prost's relative `super::` cross-package references resolve.
#![allow(clippy::pedantic)]
#![allow(clippy::derivable_impls)]

pub mod proto;
pub mod schemas;
