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

use core::time::Duration;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::Path;

use crate::error::{ Error, Result };
use crate::ipc::read_capped_line;
use crate::protocol::{ Request, Response };

/// How long a request waits for the daemon before giving up.
///
/// Comfortably above [`crate::registration::REGISTRATION_TIMEOUT`], because a
/// `spawn` legitimately takes that long — a client timeout under it would
/// abandon spawns that were about to succeed. Everything else answers in
/// milliseconds, so the margin costs nothing in practice and only matters when
/// the daemon has genuinely stopped answering.
pub const DEFAULT_TIMEOUT : Duration = Duration::from_secs( 60 );

/// Send `request` to the daemon at `socket_path` and return its answer.
///
/// The answer may be [`Response::Err`] — that is the daemon working, not
/// failing. Use [`call`] to treat it as an error instead.
///
/// # Errors
///
/// - [`Error::Io`] — the daemon is not listening, or the exchange timed out.
/// - [`Error::LineTooLong`] / [`Error::NonUtf8Line`] — the reply was not a
///   well-formed protocol line.
/// - [`Error::Malformed`] — the reply parsed as JSON but not as a [`Response`],
///   or the daemon hung up without sending one.
#[ inline ]
pub fn request( socket_path : &Path, request : &Request ) -> Result< Response >
{
  request_within( socket_path, request, DEFAULT_TIMEOUT )
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
  let stream = UnixStream::connect( socket_path ).map_err( Error::Io )?;
  // Both directions: a daemon that accepted the connection and then wedged
  // would otherwise block the write just as easily as the read.
  stream.set_read_timeout( Some( timeout ) ).map_err( Error::Io )?;
  stream.set_write_timeout( Some( timeout ) ).map_err( Error::Io )?;

  let mut line = serde_json::to_vec( request ).map_err( | source |
  {
    Error::Io( std::io::Error::other( source ) )
  } )?;
  line.push( b'\n' );

  let mut writer = &stream;
  writer.write_all( &line ).map_err( Error::Io )?;
  writer.flush().map_err( Error::Io )?;

  let mut reader = std::io::BufReader::new( &stream );
  let reply = read_capped_line( &mut reader )?
    .ok_or_else( || Error::Malformed( "daemon closed the connection without answering".into() ) )?;

  serde_json::from_str( &reply ).map_err( | source | Error::Malformed( source.to_string() ) )
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
/// As [`request`], plus [`Error::Remote`] when the daemon answered with a
/// failure.
#[ inline ]
pub fn call( socket_path : &Path, request : &Request ) -> Result< serde_json::Value >
{
  // Path-qualified: the parameter shadows the function's own name here.
  match self::request( socket_path, request )?
  {
    Response::Ok { result, .. } => Ok( result ),
    Response::Err { error, .. } => Err( Error::Remote( error ) ),
  }
}
