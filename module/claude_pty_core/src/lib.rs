#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

//! # `claude_pty_core` — pseudo-terminal session mechanics
//!
//! Allocates pseudo-terminals and spawns child processes attached to them.
//!
//! This crate knows nothing about Claude Code. It is a generic terminal layer
//! that higher crates compose — `claude_daemon_core` uses it to host disowned
//! interactive sessions, but nothing here depends on that use.
//!
//! **Zero dependencies**: the PTY layer is hand-rolled POSIX FFI. `pty-process`
//! declares `edition = "2024"` (Rust 1.85) against this workspace's 1.75, and
//! `portable-pty` exposes `anyhow::Error` in its public API, which collides with
//! the error_tools-exclusive rule. See `docs/algorithm/001_pty_allocation.md`.
//!
//! ## Core types
//!
//! - [`Pty`] — an allocated master/slave pair
//! - [`PtySession`] — a child process running on a pty, with a non-blocking write path
//! - [`SessionConfig`] — how to spawn that child
//! - [`WinSize`] — terminal dimensions in character cells
//!
//! ## Unsafe containment
//!
//! Every `extern "C"` declaration and every `unsafe` block in this crate lives in
//! one module, `ffi`, under a scoped `#[ allow( unsafe_code ) ]`. The workspace
//! denies unsafe globally; this is the documented exception, and
//! `tests/unsafe_containment_test.rs` enforces the boundary mechanically.

#![ deny( missing_docs ) ]
#![ warn( rust_2018_idioms ) ]

mod error;
mod ffi;
mod pty;
mod session;
mod writer;
pub mod env_scrub;

pub use error::{ Error, Result };
pub use pty::{ Pty, WinSize };
pub use session::{ PtySession, SessionConfig };
pub use writer::{ WriterHandle, DEFAULT_QUEUE_CAPACITY };
