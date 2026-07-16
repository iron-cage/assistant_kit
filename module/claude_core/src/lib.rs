#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

//! Shared Claude Code domain primitives.
//!
//! Layer 0 of the workspace dependency hierarchy — zero workspace crate deps.
//!
//! # Modules
//!
//! - [`paths`]: [`ClaudePaths`] — all `~/.claude/` canonical paths from `HOME`
//! - [`process`]: [`ProcessInfo`], [`ProcessMetrics`] (Linux), and process signal utilities
//! - [`settings_io`]: Atomic read/write of flat-JSON key-value files (e.g. `settings.json`)
//! - [`toml_io`]: Tiered (project + user) read/atomic write of flat-TOML key-value files (e.g. `config.toml`)

#![ cfg_attr( not( feature = "enabled" ), allow( unused ) ) ]
#![ warn( missing_docs ) ]

pub mod paths;
pub mod process;
pub mod settings_io;
pub mod toml_io;

pub use paths::ClaudePaths;
