//! The `clipmill` command and the MCP server it runs for agents.
//!
//! Both speak the engine API v2 (docs/engine-api.md) to the ClipMill already
//! running on this computer; neither does any work of its own.

pub mod client;
pub mod mcp;
pub mod tools;
