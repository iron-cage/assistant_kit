//! Talking to a running daemon.
//!
//! One connection per request, matching what the daemon serves. There is no
//! client handle to hold, because there is no connection worth keeping: a
//! persistent one would let a single client decide when a single-threaded daemon
//! gets round to everyone else.
//!
//! That also removes a whole class of bug from the caller's side. There is no
//! stale socket to reconnect, no half-consumed response to resynchronise, and no
//! state to get wrong across a daemon restart — each call stands alone.
//!
//! Thin wrapper over [`daemon_kit::client`]: the framing, timeout, and
//! connect-write-read mechanics are generic over the request type and live
//! there; this crate supplies [`Request`].

use core::time::Duration;
use std::path::Path;

use crate::error::Result;
use crate::protocol::{ Request, Response };

/// How long a request waits for the daemon before giving up.
///
/// Comfortably above [`crate::registration::REGISTRATION_TIMEOUT`], because a
/// `spawn` legitimately takes that long — a client timeout under it would
/// abandon spawns that were about to succeed. Everything else answers in
/// milliseconds, so the margin costs nothing in practice and only matters when
/// the daemon has genuinely stopped answering.
pub const DEFAULT_TIMEOUT : Duration = daemon_kit::client::DEFAULT_TIMEOUT;

/// Send `request` to the daemon at `socket_path` and return its answer.
///
/// The answer may be [`Response::Err`] — that is the daemon working, not
/// failing. Use [`call`] to treat it as an error instead.
///
/// # Errors
///
/// - [`crate::Error::Io`] — the daemon is not listening, or the exchange timed out.
/// - [`crate::Error::LineTooLong`] / [`crate::Error::NonUtf8Line`] — the reply was
///   not a well-formed protocol line.
/// - [`crate::Error::Malformed`] — the reply parsed as JSON but not as a
///   [`Response`], or the daemon hung up without sending one.
#[ inline ]
pub fn request( socket_path : &Path, request : &Request ) -> Result< Response >
{
  daemon_kit::client::request( socket_path, request ).map_err( Into::into )
}

/// [`request`], with an explicit timeout.
///
/// Worth reaching for on a polling `read`, where waiting a full minute for a
/// daemon that has stopped answering is a minute the caller could have spent
/// reporting it.
///
/// # Errors
///
/// As [`request`].
#[ inline ]
pub fn request_within( socket_path : &Path, request : &Request, timeout : Duration )
-> Result< Response >
{
  daemon_kit::client::request_within( socket_path, request, timeout ).map_err( Into::into )
}

/// Send `request` and unwrap a successful result, turning a failure answer into
/// an error.
///
/// The form to reach for when a failure is a failure — which is most callers.
/// [`request`] is there for the ones that want to render the daemon's own
/// message themselves rather than propagate it.
///
/// # Errors
///
/// As [`request`], plus [`crate::Error::Remote`] when the daemon answered with a
/// failure.
#[ inline ]
pub fn call( socket_path : &Path, request : &Request ) -> Result< serde_json::Value >
{
  daemon_kit::client::call( socket_path, request ).map_err( Into::into )
}
