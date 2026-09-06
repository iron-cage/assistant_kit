#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

//! # `child_supervisor` — generic PTY-child bookkeeping
//!
//! Drained, bounded output and a keyed session table — the parts of hosting a
//! long-lived PTY child that have nothing to do with what the child is *for*.
//!
//! Extracted from `claude_daemon_core` — see its (now relocated)
//! `docs/feature/003_session_table.md` and `004_session_output.md`, reissued
//! here as `001_session_table.md`/`002_session_output.md`, for the full
//! behavioral rationale. Knows nothing about Claude Code specifically: a
//! caller supplies its own session id scheme and drives busy/idle bookkeeping
//! itself.
//!
//! ## Core types
//!
//! - [`OutputBuffer`] / [`OutputPump`] / [`OutputSlice`] — bounded, cursor-addressed
//!   terminal output, kept drained
//! - [`HostedSession`] / [`SessionTable`] — one supervised child and the table
//!   of them, keyed by a caller-chosen id rather than PID

#![ deny( missing_docs ) ]
#![ warn( rust_2018_idioms ) ]

mod error;
pub mod output;
pub mod table;

pub use error::{ Error, Result };
pub use output::{ OutputBuffer, OutputPump, OutputSlice, DEFAULT_OUTPUT_CAP };
pub use table::{ HostedSession, SessionTable };
