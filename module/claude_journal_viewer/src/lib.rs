//! `claude_journal_viewer` — CLI and web viewer for CLR journal events.
//!
//! Provides the `clj` binary with `.list`, `.tail`, `.stats`, `.search`,
//! `.serve`, `.prune`, `.status`, `.export`, and `.chart` commands.
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

#[ cfg( feature = "routines" ) ]
/// Register `claude_journal_viewer` commands into an existing registry.
///
/// `claude_journal_viewer` commands are defined in [`COMMANDS_YAML`] for compile-time
/// aggregation (used by `assistant/build.rs`). This function is provided for API
/// consistency with other Layer 2 crates; the body is intentionally empty because
/// runtime registration of `.journal.*` commands is handled by the build-time YAML
/// aggregation path in `assistant`.
#[ inline ]
pub fn register_commands( _registry : &mut unilang::registry::CommandRegistry ) {}
