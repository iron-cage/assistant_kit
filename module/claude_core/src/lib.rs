#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

//! Shared Claude Code domain primitives.
//!
//! Layer 0 of the workspace dependency hierarchy — zero workspace crate deps.
//!
//! # Modules
//!
//! - [`file_io`]: Atomic write-then-rename (unique temp names, optional `0o600` secret mode) and the secret-redacting trace formatter
//! - [`paths`]: [`ClaudePaths`] — all `~/.claude/` canonical paths from `HOME`
//! - [`process`]: [`ProcessInfo`], [`ProcessMetrics`] (Linux), and process signal utilities
//! - [`settings_io`]: Atomic read/write of flat-JSON key-value files (e.g. `settings.json`)
//! - [`time`]: [`chrono_now_utc`], [`trace_ts`] — pure-stdlib UTC timestamp utilities
//! - [`toml_io`]: Tiered (project + user) read/atomic write of flat-TOML key-value files (e.g. `config.toml`)

#![ cfg_attr( not( feature = "enabled" ), allow( unused ) ) ]
#![ warn( missing_docs ) ]

pub mod file_io;
pub mod paths;
pub mod process;
pub mod settings_io;
pub mod time;
pub mod toml_io;

pub use paths::ClaudePaths;
pub use time::{ chrono_now_utc, trace_ts };
