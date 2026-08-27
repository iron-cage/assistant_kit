//! Advisory per-topic exclusion, so two writers do not resume one conversation.
//!
//! # The hazard
//!
//! A fork topic is addressed by a deterministic session id, so two concurrent
//! `claude --resume <id>` invocations target the same transcript file. The
//! concurrency gate does not prevent this: its slots are indexed by the live
//! process *count*, deliberately, so that racing callers collide on one path and
//! `create_new` can arbitrate — which bounds how many sessions run at once and
//! says nothing about *which* sessions they are.
//!
//! Whether Claude Code itself guards against this has not been established here.
//! This module is therefore a mitigation for a hazard, not a fix for a confirmed
//! defect, and it is scoped accordingly:
//!
//! - **Fan-out commands take the lock by default.** Fanning a prompt out over
//!   every topic is what makes a collision likely enough to be worth preventing.
//! - **The ordinary run path does not, unless `CLR_TOPIC_LOCK=1`.** Turning it on
//!   there would make a second concurrent `clr topic --topic x` start failing
//!   where today it runs, which is a behaviour change this hazard does not yet
//!   justify. The switch exists so it can be flipped once it does.
//!
//! # What "advisory" means here
//!
//! Reclaiming a lock whose owner died is a compare-and-delete: the file is removed
//! only if it still holds the exact content that was judged stale. That shrinks the
//! window between "decided the owner is dead" and "deleted the file" from a `/proc`
//! read down to two adjacent filesystem calls — it does not close it. Two processes
//! reclaiming the same stale lock in the same instant can still both proceed. That
//! degrades to the current behaviour for that one invocation rather than to
//! something worse, which is the bar a mitigation has to clear.

use std::path::{ Path, PathBuf };

use crate::enumerate::Topic;

/// Environment switch that opts the ordinary run path into locking. Unset or any
/// value other than `1`/`true` leaves it off; see the module docs.
pub const LOCK_ENV : &str = "CLR_TOPIC_LOCK";

/// Environment override for the lock directory, for tests and for callers who
/// need locks to outlive a temp-dir sweep.
pub const LOCK_DIR_ENV : &str = "CLR_TOPIC_LOCK_DIR";

/// Directory name appended to the system temp dir when [`LOCK_DIR_ENV`] is unset.
const LOCK_DIRNAME : &str = "clr-topic-lock";

/// Why a lock could not be taken.
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub enum LockDenied
{
  /// Another live process holds this topic; its pid is carried so the caller can
  /// name it.
  Held( u32 ),
  /// The lock directory or file could not be worked with at all. The lock is
  /// advisory, so a caller may reasonably proceed anyway — but it should say so.
  Unavailable( String ),
}

impl core::fmt::Display for LockDenied
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    match self
    {
      Self::Held( pid ) => write!( f, "already held by pid {pid}" ),
      Self::Unavailable( why ) => write!( f, "lock unavailable: {why}" ),
    }
  }
}

/// A held topic lock. Released on drop.
///
/// Drop does not run on `SIGKILL`, which is what the dead-owner reclaim in
/// [`try_lock`] is for.
#[ derive( Debug ) ]
pub struct TopicLock
{
  path : PathBuf,
}

impl TopicLock
{
  /// The lock file backing this guard — for diagnostics and tests.
  #[ inline ]
  #[ must_use ]
  pub fn path( &self ) -> &Path
  {
    &self.path
  }
}

impl Drop for TopicLock
{
  #[ inline ]
  fn drop( &mut self )
  {
    let _ = std::fs::remove_file( &self.path );
  }
}

/// Whether the ordinary run path should lock, per [`LOCK_ENV`].
#[ inline ]
#[ must_use ]
pub fn enabled_for_run_path() -> bool
{
  std::env::var( LOCK_ENV ).is_ok_and( | v | v == "1" || v == "true" )
}

/// The directory locks live in: [`LOCK_DIR_ENV`] when non-empty, else
/// `<system temp dir>/clr-topic-lock`.
#[ inline ]
#[ must_use ]
pub fn lock_dir() -> PathBuf
{
  match std::env::var( LOCK_DIR_ENV )
  {
    Ok( v ) if !v.is_empty() => PathBuf::from( v ),
    _ => std::env::temp_dir().join( LOCK_DIRNAME ),
  }
}

/// The lock file for `topic`.
///
/// Keyed on the topic's own resolved path — the session file for a fork topic,
/// the working directory for a dir topic — which is exactly the resource being
/// protected, and is already unique per `( base, name, mode )`. `None` when that
/// path is not encodable.
#[ inline ]
#[ must_use ]
pub fn lock_file( topic : &Topic ) -> Option< PathBuf >
{
  let encoded = claude_storage_core::encode_path( &topic.path ).ok()?;
  Some( lock_dir().join( format!( "{encoded}.lock" ) ) )
}

/// Parse a lock file's `<pid> <starttime>` body.
fn parse_owner( body : &str ) -> Option< ( u32, Option< u64 > ) >
{
  let mut parts = body.split_whitespace();
  let pid = parts.next()?.parse().ok()?;
  let starttime = parts.next().and_then( | s | s.parse().ok() );
  Some( ( pid, starttime ) )
}

/// Try to take `topic`'s lock, reclaiming it from an owner that is no longer running.
///
/// Returns immediately either way — this never waits. A caller that wants to wait
/// owns that policy, because how long to wait for a topic depends entirely on why
/// it is being asked for.
///
/// # Errors
///
/// [`LockDenied::Held`] when a live process already holds the topic, carrying that
/// process's pid. [`LockDenied::Unavailable`] when the lock could not be worked
/// with at all — an unencodable topic path, an unwritable lock directory, or a
/// reclaim lost twice in a row. The lock is advisory, so a caller may reasonably
/// proceed on `Unavailable`; it should say so when it does.
#[ inline ]
pub fn try_lock( topic : &Topic ) -> Result< TopicLock, LockDenied >
{
  let Some( path ) = lock_file( topic ) else
  {
    return Err( LockDenied::Unavailable( format!( "cannot encode {}", topic.path.display() ) ) );
  };
  if let Some( parent ) = path.parent()
  {
    if let Err( e ) = std::fs::create_dir_all( parent )
    {
      return Err( LockDenied::Unavailable( format!( "{}: {e}", parent.display() ) ) );
    }
  }

  // Two attempts: the first may find a stale lock, and reclaiming it earns exactly
  // one retry. A third would only re-race whoever won the reclaim.
  for _ in 0 .. 2
  {
    match claim( &path )
    {
      Ok( lock ) => return Ok( lock ),
      Err( LockDenied::Held( _ ) ) =>
      {
        let Ok( body ) = std::fs::read_to_string( &path ) else { continue };
        match parse_owner( &body )
        {
          // Unparseable content cannot be attributed to a live process, so it is
          // reclaimed on the same compare-and-delete terms as a dead owner's.
          None => reclaim( &path, &body ),
          Some( ( owner, starttime ) ) =>
          {
            if claude_session_core::pid_alive( owner, starttime )
            {
              return Err( LockDenied::Held( owner ) );
            }
            reclaim( &path, &body );
          },
        }
      },
      Err( other ) => return Err( other ),
    }
  }

  // Both attempts lost: the file existed, was judged stale, and somebody else won
  // the reclaim. It is held now — by whoever won, so read who that is rather than
  // reporting the dead owner we displaced.
  match std::fs::read_to_string( &path ).ok().as_deref().and_then( parse_owner )
  {
    Some( ( owner, _ ) ) => Err( LockDenied::Held( owner ) ),
    None => Err( LockDenied::Unavailable( "lock churned during reclaim".to_owned() ) ),
  }
}

/// One `create_new` attempt, stamping this process's identity into the file.
fn claim( path : &Path ) -> Result< TopicLock, LockDenied >
{
  use std::io::Write as _;
  match std::fs::OpenOptions::new().create_new( true ).write( true ).open( path )
  {
    Ok( mut file ) =>
    {
      let pid = std::process::id();
      // The starttime is omitted rather than defaulted when unreadable. A recorded
      // `0` would not match this process's real starttime, so the next caller would
      // read it as a dead owner and take the lock out from under a live one.
      let stamp = claude_session_core::proc_starttime( pid )
        .map_or_else( || pid.to_string(), | t | format!( "{pid} {t}" ) );
      if let Err( e ) = write!( file, "{stamp}" )
      {
        // A lock whose owner cannot be read back cannot be reclaimed correctly.
        let _ = std::fs::remove_file( path );
        return Err( LockDenied::Unavailable( e.to_string() ) );
      }
      Ok( TopicLock { path : path.to_path_buf() } )
    },
    Err( e ) if e.kind() == std::io::ErrorKind::AlreadyExists => Err( LockDenied::Held( 0 ) ),
    Err( e ) => Err( LockDenied::Unavailable( e.to_string() ) ),
  }
}

/// Compare-and-delete: remove `path` only if it still holds `expected`.
///
/// The re-read is what keeps this from deleting a lock somebody else has already
/// taken in the meantime. See the module docs for the residual window.
fn reclaim( path : &Path, expected : &str )
{
  if std::fs::read_to_string( path ).is_ok_and( | now | now == expected )
  {
    let _ = std::fs::remove_file( path );
  }
}
