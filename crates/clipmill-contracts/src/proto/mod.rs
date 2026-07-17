//! IPC and worker-protocol messages (from `contracts/proto/`). Module
//! nesting mirrors the proto packages (`clipmill.<pkg>.v1` → `proto::<pkg>::v1`).

pub mod ipc;
pub mod shm;
pub mod time;
pub mod worker;
