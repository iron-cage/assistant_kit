//! Learning a freshly spawned session's conversation id.
//!
//! # Why this is a wait, not a return value
//!
//! Nothing the daemon does assigns a conversation id. Claude Code mints its own
//! and publishes it by writing a record into its session registry, which happens
//! some milliseconds after the process starts — after the spawn call has already
//! returned. So the id cannot be an output of spawning; it can only be observed
//! afterwards.
//!
//! That leaves a window in which the daemon holds a live session it cannot yet
//! name. [`await_session_id`] closes it by polling the registry for the pid it
//! just spawned.
//!
//! # Why the pid is safe to match on here, and only here
//!
//! Everywhere else this crate refuses to key on a pid, for the reason
//! [`claude_session_core`'s liveness invariant](claude_session_core::liveness)
//! documents: a pid number outlives the process that held it, so it names a
//! process only within a known incarnation. This is the one place the incarnation
//! *is* known — the daemon spawned the child itself and holds its handle, so the
//! number cannot have been recycled while the caller is still looking at it.
//!
//! The pid is used to answer one question, once ("which of these records is my
//! child?"), and then discarded in favour of the conversation id.

use core::time::Duration;
use std::path::Path;
use std::time::Instant;

use crate::error::{ Error, Result };

/// How long to wait for a spawned process to publish its conversation id.
///
/// Registration is one file write into a directory the process already knows, so
/// the normal case is well under a second. The margin is for a cold start on a
/// loaded machine, not for a process that is never going to register.
pub const REGISTRATION_TIMEOUT : Duration = Duration::from_secs( 30 );

/// How often the registry is re-scanned while waiting.
///
/// A scan reads one small directory. Frequent enough that the wait does not
/// dominate a spawn, cheap enough that it does not matter if it usually takes
/// two or three tries.
const REGISTRATION_POLL : Duration = Duration::from_millis( 25 );

/// Wait for `pid` to appear in the session registry and return its conversation
/// id.
///
/// `alive` is consulted between scans and reports whether the spawned child is
/// still running. It exists so a child that dies during startup fails
/// immediately with [`Error::NoRegistration`] rather than burning the full
/// [`REGISTRATION_TIMEOUT`] waiting for a record that can no longer be written —
/// the caller holds the child handle, so only the caller can answer that.
///
/// # Errors
///
/// - [`Error::NoRegistration`] — the child exited, or the timeout elapsed with no
///   record naming `pid`.
/// - [`Error::Registry`] — the registry directory could not be read at all. A
///   *missing* directory is not this error: `scan` reports that as an empty
///   result, which is correct here, since a first-ever session creates it.
#[ inline ]
pub fn await_session_id
(
  sessions_dir : &Path,
  pid : u32,
  timeout : Duration,
  mut alive : impl FnMut() -> bool,
)
-> Result< String >
{
  let deadline = Instant::now() + timeout;
  loop
  {
    if let Some( id ) = lookup( sessions_dir, pid )?
    {
      return Ok( id );
    }
    // Checked after the scan, never before: a process that registered and then
    // exited immediately still has a readable record, and refusing to report the
    // id we can plainly see would be a race the caller cannot do anything about.
    if !alive() || Instant::now() >= deadline
    {
      return Err( Error::NoRegistration { pid } );
    }
    std::thread::sleep( REGISTRATION_POLL );
  }
}

/// One scan of the registry for `pid`.
///
/// Separate from the wait so the lookup can be tested against a fixture
/// directory without any timing involved.
///
/// # Errors
///
/// Returns [`Error::Registry`] if the registry directory exists but cannot be
/// read.
#[ inline ]
pub fn lookup( sessions_dir : &Path, pid : u32 ) -> Result< Option< String > >
{
  let found = claude_session_core::scan( sessions_dir )
    .map_err( Error::Registry )?
    .into_iter()
    .find( | record | record.pid == pid )
    .map( | record | record.session_id );
  Ok( found )
}
