//! Wire protocol between a client and the session daemon.
//!
//! One JSON object per line, in both directions. Generalized from
//! `claude_runner/src/cli/query.rs`, which established the
//! `{ "ok": true, "result": … }` / `{ "ok": false, "error": … }` response shape
//! against a per-PID socket. The differences here are deliberate:
//!
//! - **One socket, many sessions.** `query.rs` keys a socket per PID; a single
//!   daemon hosts every session, so the session is named *inside* the request.
//! - **Sessions are named by `session_id`, not PID.** Claude Code's own daemon
//!   re-hosts a session with `--fork-session` on auto-update or recovery: the
//!   new process has a different PID, no inherited environment, and a new
//!   conversation id. Anything keyed on PID silently detaches at that moment.
//!
//! [`Response`] itself is [`daemon_kit::Response`] — its envelope shape is
//! payload-agnostic, so it lives one layer down and is re-exported here
//! unchanged. [`Request`] and [`SessionSummary`] are what stay Claude-specific.

use std::path::PathBuf;

use serde::{ Deserialize, Serialize };

/// What a client asks the daemon to do.
#[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
#[ serde( tag = "method", rename_all = "snake_case" ) ]
#[ non_exhaustive ]
pub enum Request
{
  /// Liveness probe. Returns the daemon's version.
  Ping,
  /// List every session the daemon currently hosts.
  ListSessions,
  /// Start a new interactive session in `cwd`.
  Spawn
  {
    /// Working directory for the new session.
    cwd : PathBuf,
    /// Optional first prompt, delivered once the session is ready.
    #[ serde( default ) ]
    prompt : Option< String >,
  },
  /// Deliver `text` to a session's stdin, followed by a carriage return.
  Send
  {
    /// Target session's conversation id.
    session_id : String,
    /// Text to deliver.
    text : String,
  },
  /// Read a session's output since `cursor`.
  ///
  /// Non-destructive: the same cursor returns the same bytes again, and two
  /// clients reading one session do not steal each other's output. A `send`
  /// followed by repeated `read` calls is what makes a hosted session look like
  /// print mode from the outside.
  Read
  {
    /// Target session's conversation id.
    session_id : String,
    /// Absolute byte cursor from the previous read; `0` starts at the beginning
    /// of what is still retained.
    #[ serde( default ) ]
    cursor : u64,
  },
  /// Report what a session's context currently holds.
  ///
  /// Deferred tools, agent and skill rosters, remaining token budget, background
  /// tasks — read from the session's own transcript rather than from anything
  /// the daemon tracks. Most of that arrives in the transcript as *deltas*, so
  /// the current state is accumulated by replaying them, not by sampling.
  ///
  /// Read-only and side-effect free: it neither writes to the session nor
  /// disturbs a turn in flight, so it is safe to call against a busy session.
  ContextSummary
  {
    /// Target session's conversation id.
    session_id : String,
  },
  /// Change a session's terminal dimensions.
  Resize
  {
    /// Target session's conversation id.
    session_id : String,
    /// New height in character cells.
    rows : u16,
    /// New width in character cells.
    cols : u16,
  },
  /// Shut a session down and reap it.
  Shutdown
  {
    /// Target session's conversation id.
    session_id : String,
  },
  /// Shut down every session, then stop the daemon itself.
  ///
  /// A request rather than a signal, so the client gets an acknowledgement on
  /// the same connection it asked over — a signal is fire-and-hope, and tells a
  /// client nothing about whether the daemon it was aimed at was even the one
  /// running.
  StopDaemon,
}

/// What the daemon answers.
///
/// The `{ok:true, result}` / `{ok:false, error}` envelope is payload-agnostic,
/// so the type itself lives in [`daemon_kit`] and is re-exported here
/// unchanged — every caller that named it as `claude_daemon_core::protocol::Response`
/// before this crate composed `daemon_kit` keeps resolving to the same shape.
pub use daemon_kit::{ OkFalse, OkTrue, Response };

/// Summary of one hosted session, as returned by [`Request::ListSessions`].
#[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
#[ non_exhaustive ]
pub struct SessionSummary
{
  /// Conversation id — the stable handle across a `--fork-session` re-host.
  pub session_id : String,
  /// Current process id. Advisory only; it changes when the session is re-hosted.
  pub pid : u32,
  /// Working directory the session runs in.
  pub cwd : PathBuf,
  /// Whether the daemon believes a turn is in flight.
  pub busy : bool,
}
