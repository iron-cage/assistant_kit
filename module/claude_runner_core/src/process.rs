//! Process scanner: enumerate running Claude Code instances via `/proc`.
//!
//! Re-exported from `claude_core` — the authoritative implementation lives there.

pub use claude_core::process::
{
  ProcessInfo,
  current_pid,
  find_claude_processes,
  process_is_alive,
  send_sigkill,
  send_sigterm,
  spawn_background_self,
};
