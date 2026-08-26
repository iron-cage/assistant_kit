#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

//! # `claude_daemon_core` — single-instance session daemon
//!
//! Owns disowned interactive Claude Code sessions and answers clients over a
//! Unix domain socket. Exactly one daemon runs at a time; it hosts any number of
//! sessions.
//!
//! Composes the two layers below it rather than reimplementing them:
//! `claude_pty_core` for terminal mechanics, `claude_session_core` for observing
//! whether a session is alive and whether a turn has actually finished.
//!
//! ## Core types
//!
//! - [`DaemonPaths`] — lock, socket, and registry locations
//! - [`InstanceLock`] — the single-instance guarantee
//! - [`Listener`] — the socket, bound and cleaned up after
//! - [`SessionTable`] — hosted sessions, keyed by conversation id
//! - [`OutputPump`] / [`OutputSlice`] — output kept drained and read by cursor
//! - [`Request`] / [`Response`] — the wire protocol
//! - [`Daemon`] / [`serve_once`] — what a request means, and the body of a loop
//! - [`client::call`] — the other end of that exchange
//! - [`to_plain_text`] — a terminal's output stream, made readable
//!
//! ## Two decisions worth knowing
//!
//! **Sessions are named by conversation id, not PID.** Claude Code re-hosts a
//! session with `--fork-session` on auto-update or recovery: new process, new
//! PID, no inherited environment. A PID-keyed table detaches silently at exactly
//! the moment recovery was supposed to help.
//!
//! **Protocol lines are capped.** The `query.rs` prototype this generalizes reads
//! its socket with an unbounded `read_line`; with one daemon hosting every
//! session, an unterminated line is no longer one session's problem. See
//! [`ipc::MAX_IPC_LINE_BYTES`].

#![ deny( missing_docs ) ]
#![ warn( rust_2018_idioms ) ]

pub mod client;
mod error;
pub mod ipc;
pub mod listener;
pub mod lock;
pub mod output;
pub mod paths;
pub mod protocol;
pub mod registration;
pub mod render;
pub mod serve;
pub mod table;

pub use error::{ Error, Result };
pub use ipc::{ read_capped_line, MAX_IPC_LINE_BYTES };
pub use listener::Listener;
pub use lock::{ acquire, InstanceLock };
pub use output::{ OutputBuffer, OutputPump, OutputSlice, DEFAULT_OUTPUT_CAP };
pub use paths::DaemonPaths;
pub use protocol::{ Request, Response, SessionSummary };
pub use registration::{ await_session_id, REGISTRATION_TIMEOUT };
pub use render::to_plain_text;
pub use serve::{ serve_connection, serve_once, Daemon };
// Re-exported rather than left to the caller to depend on `claude_session_core`
// for: [`Daemon::with_background_reporting`] takes it, and an argument type a
// caller cannot name without adding a dependency is not really public.
pub use claude_session_core::BackgroundReporting;
pub use claude_session_core::turn::BG_TASKS_REPORT_RUNNING_ENV;
pub use table::{ HostedSession, SessionTable };
