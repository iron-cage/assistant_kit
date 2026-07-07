//! Settings I/O: read and write Claude Code's `settings.json` file.
//!
//! Relocated to [`claude_core::settings_io`] as the shared L0 primitive (also
//! used for `~/.clr/prefs.json` by `claude_profile` and `claude_runner_core`).
//! Re-exported here so existing `claude_version_core::settings_io::*` call
//! sites are unaffected.

pub use claude_core::settings_io::*;
