#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

//! # `claude_session_core` — live Claude Code session observation
//!
//! Reads the mutable, PID-keyed registry Claude Code maintains at
//! `~/.claude/sessions/`, decides whether the process behind a record is really
//! running, and turns status transitions into turn boundaries.
//!
//! Distinct from `claude_storage_core`, which owns the *append-only conversation
//! transcripts* under `~/.claude/projects/`. Different directory, different
//! format, different lifecycle; joined by the `sessionId` field.
//!
//! ## Core types
//!
//! - [`SessionRecord`] — one registry entry
//! - [`SessionStatus`] — a session's self-reported status
//! - [`TurnWatcher`] — turn-boundary detection over status transitions
//!
//! ## Two traps this crate exists to encode
//!
//! 1. **A PID is not an identity.** `/proc/{pid}` existence proves a number is in
//!    use, not that the recorded process runs — zombies, non-leader thread ids,
//!    and PID-space wrap all defeat it. [`liveness::pid_alive`] carries the four
//!    clauses and the two production bugs that produced them.
//! 2. **`idle` is not "done".** A session parked on an outstanding background
//!    task reports `idle` unless it was spawned with
//!    [`turn::BG_TASKS_REPORT_RUNNING_ENV`] set. [`TurnWatcher`] makes that
//!    guarantee an explicit constructor argument rather than an assumption.

#![ deny( missing_docs ) ]
#![ warn( rust_2018_idioms ) ]

mod error;
pub mod liveness;
pub mod registry;
pub mod turn;

pub use error::{ Error, Result };
pub use liveness::{ pid_alive, proc_starttime };
pub use registry::{ scan, scan_live, SessionRecord, SessionStatus };
pub use turn::{ BackgroundReporting, TurnEvent, TurnWatcher };
