//! Framing one request/response exchange, and driving it from a socket.
//!
//! [`serve_connection`] is one line in, one line out: read a request, hand it to
//! a caller-supplied closure, write back whatever [`Response`] it returns.
//! [`serve_once`] wires that to an accepted connection. Neither loops — that is
//! the caller's, because only the caller knows what should end it.
//!
//! # One request per connection
//!
//! Not a limitation to work around. A single-threaded daemon serving persistent
//! connections is a single-*client* daemon: whoever connects first decides when
//! everyone else gets served. Closing after one request bounds a client's hold
//! on the daemon to the request it actually sent.
//!
//! # A daemon's own clock
//!
//! A daemon built on this crate spends its life blocked in [`Listener::accept`],
//! so nothing runs between requests unless something manufactures one.
//! [`spawn_waker`] is that something: a synthetic client that sleeps for one
//! tick, connects, and hangs up — purely so `accept` returns on a schedule even
//! when no real client has anything to ask. A synthetic connection is not a
//! special case: [`serve_connection`] already treats a client that hangs up
//! without sending anything as a normal, silent no-op.

use core::time::Duration;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use serde::de::DeserializeOwned;

use crate::error::{ Error, Result };
use crate::ipc::read_capped_line;
use crate::listener::Listener;
use crate::response::Response;

/// Serve exactly one request from `stream`, then leave it to be closed.
///
/// `handle` turns the parsed request into the response to send back.
///
/// A client that hangs up without sending anything is not an error: nothing is
/// read, nothing is written, and this returns `Ok`. Neither is a request that
/// cannot be parsed — that gets a well-formed error response, which is the whole
/// point of having one. Only a failure to *write* the answer is an error here,
/// since at that point there is nothing left to tell the client.
///
/// # Errors
///
/// Returns [`Error::Io`] if the response cannot be written.
#[ inline ]
pub fn serve_connection< Req, H >( stream : &UnixStream, handle : H ) -> Result< () >
where
  Req : DeserializeOwned,
  H : FnOnce( Req ) -> Response,
{
  let mut reader = std::io::BufReader::new( stream );
  let response = match read_capped_line( &mut reader )
  {
    Ok( None ) => return Ok( () ),
    Ok( Some( line ) ) => match serde_json::from_str::< Req >( &line )
    {
      Ok( request ) => handle( request ),
      Err( source ) => Response::err( Error::Malformed( source.to_string() ).to_string() ),
    },
    Err( error ) => Response::err( error.to_string() ),
  };

  let mut line = serde_json::to_vec( &response ).map_err( | source |
  {
    Error::Io( std::io::Error::other( source ) )
  } )?;
  line.push( b'\n' );

  let mut writer = stream;
  writer.write_all( &line ).map_err( Error::Io )?;
  writer.flush().map_err( Error::Io )
}

/// Accept one client and serve its request via `handle`.
///
/// The whole body of a daemon's main loop, minus the loop.
///
/// # Errors
///
/// Returns [`Error::Io`] if accepting the connection or writing the answer
/// fails. A failure here concerns one client; it is not by itself a reason to
/// stop serving the others.
#[ inline ]
pub fn serve_once< Req, H >( listener : &Listener, handle : H ) -> Result< () >
where
  Req : DeserializeOwned,
  H : FnOnce( Req ) -> Response,
{
  let stream = listener.accept()?;
  serve_connection( &stream, handle )
}

/// How often [`spawn_waker`] pokes the daemon when nothing else does.
pub const DEFAULT_TICK : Duration = Duration::from_secs( 30 );

/// Start a thread that sleeps for `tick`, connects to `socket`, and hangs up —
/// forever, once every `tick` — so the main loop's blocking `accept` returns on
/// a schedule even when no real client has anything to ask.
///
/// Runs until the process exits; nothing about it needs joining or stopping
/// explicitly, so the returned handle exists only for a caller that wants to
/// notice if the thread itself ever panics. A connection failure — most likely
/// the daemon shutting down between one tick and the next — is dropped for the
/// same reason: nobody is waiting on this thread's success, only on what its
/// connections cause to happen.
#[ inline ]
pub fn spawn_waker( socket : impl Into< PathBuf >, tick : Duration ) -> std::thread::JoinHandle< () >
{
  let socket = socket.into();
  std::thread::spawn( move ||
  {
    loop
    {
      std::thread::sleep( tick );
      drop( UnixStream::connect( &socket ) );
    }
  } )
}
