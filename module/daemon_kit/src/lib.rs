#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

//! # `daemon_kit` — generic single-instance Unix-socket daemon skeleton
//!
//! The parts of building a single-instance daemon that have nothing to do with
//! what the daemon is *for*: exactly one instance running at a time, a socket
//! that cleans up after itself, capped line-oriented framing, a client for the
//! other end, and the accept-serve-respond loop tying them together.
//!
//! Extracted from `claude_daemon_core`, which composes this crate with its own
//! Claude-specific dispatch logic rather than reimplementing any of it. See that
//! crate's `docs/feature/001_single_instance.md`, `002_wire_protocol.md`, and
//! `006_serving_clients.md` for the behavioral rationale behind what lives here.
//!
//! ## Core types
//!
//! - [`InstanceLock`] / [`acquire`] — the single-instance guarantee
//! - [`Listener`] — the socket, bound and cleaned up after
//! - [`read_capped_line`] / [`MAX_IPC_LINE_BYTES`] — line framing with a hard cap
//! - [`Response`] — the `{ok, result}` / `{ok, error}` envelope every answer
//!   travels in, whatever the request type turns out to be
//! - [`serve_connection`] / [`serve_once`] — the body of a daemon's main loop
//! - [`spawn_waker`] — a synthetic client that gives a blocking-`accept` loop a
//!   floor tick rate
//! - [`client::call`] / [`client::request`] — the other end of that exchange

#![ deny( missing_docs ) ]
#![ warn( rust_2018_idioms ) ]

pub mod client;
mod error;
pub mod ipc;
pub mod listener;
pub mod lock;
pub mod response;
pub mod serve;

pub use error::{ Error, Result };
pub use ipc::{ read_capped_line, MAX_IPC_LINE_BYTES };
pub use listener::Listener;
pub use lock::{ acquire, InstanceLock };
pub use response::{ OkFalse, OkTrue, Response };
pub use serve::{ serve_connection, serve_once, spawn_waker, DEFAULT_TICK };
