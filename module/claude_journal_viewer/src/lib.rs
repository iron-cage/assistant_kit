//! `claude_journal_viewer` — CLI and web viewer for CLR journal events.
//!
//! Provides the `clj` binary with `.list`, `.tail`, `.stats`, `.search`,
//! `.serve`, `.prune`, `.status`, and `.export` commands.
//! Journal data is read via `claude_journal::JournalReader`.
//!
//! # Feature: `routines`
//!
//! When built with the `routines` feature, exposes unilang command routines
//! for `ast .journal.*` integration in the super-app.

#![ doc( html_root_url = "https://docs.rs/claude_journal_viewer/0.1.0" ) ]
#![ warn( missing_docs ) ]
#![ warn( missing_debug_implementations ) ]

/// Shared command output logic — each function returns a `String`.
pub mod output;

#[ cfg( feature = "routines" ) ]
/// Unilang command routines for `ast .journal.*` integration.
pub mod routines;

/// Absolute path to this crate's unilang command definitions YAML.
///
/// Used by `assistant/build.rs` for compile-time aggregation.
pub const COMMANDS_YAML : &str = concat!( env!( "CARGO_MANIFEST_DIR" ), "/claude_journal.commands.yaml" );
