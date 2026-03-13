//! Process scanner: enumerate running Claude Code instances via `/proc`.
//!
//! Re-exported from `claude_common` — the authoritative implementation lives there.

pub use claude_common::process::{ ProcessInfo, find_claude_processes, send_sigterm, send_sigkill };
