#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

//! Shared Claude Code domain primitives.
//!
//! Layer 0 of the workspace dependency hierarchy — zero workspace crate deps.
//!
//! # Modules
//!
//! - [`paths`]: [`ClaudePaths`] — all `~/.claude/` canonical paths from `HOME`
//! - [`process`]: [`ProcessInfo`] and process signal utilities

#![ cfg_attr( not( feature = "enabled" ), allow( unused ) ) ]
#![ warn( missing_docs ) ]

pub mod paths;
pub mod process;

pub use paths::ClaudePaths;
