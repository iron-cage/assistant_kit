//! Location resolution tests.
//!
//! ## Specification References
//!
//! - `docs/feature/001_single_instance.md` — where the lock and socket live
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | path01 | `with_home` runtime directory | `<home>/.claude/-daemon` |
//! | path02 | Lock, socket, and log files | Inside the runtime directory, all distinct |
//! | path03 | Registry directory | `<home>/.claude/sessions`, outside the runtime directory |
//! | path04 | Runtime directory name is hyphen-prefixed | Git-ignored by the workspace rule |
//! | path05 | Two homes | Fully independent path sets |

use claude_daemon_core::paths::
{
  LOCK_FILE_NAME, LOG_FILE_NAME, RUNTIME_DIR_NAME, SOCKET_FILE_NAME,
};
use claude_daemon_core::DaemonPaths;

/// path01: the runtime directory hangs off the Claude home, not the process cwd.
#[ test ]
fn path01_runtime_dir_is_under_claude_home()
{
  let home = std::path::Path::new( "/somewhere/home" );
  let paths = DaemonPaths::with_home( home );

  assert_eq!( paths.runtime_dir(), home.join( ".claude" ).join( RUNTIME_DIR_NAME ) );
}

/// path02: every file the daemon owns lives in the runtime directory.
///
/// And no two of them share a name. A collision would not fail loudly — the lock
/// would be taken over the socket, or the log appended into the lock — so the
/// distinctness is asserted rather than assumed from three different constants.
#[ test ]
fn path02_lock_socket_and_log_are_in_the_runtime_dir()
{
  let paths = DaemonPaths::with_home( std::path::Path::new( "/somewhere/home" ) );

  assert_eq!( paths.lock_file(), paths.runtime_dir().join( LOCK_FILE_NAME ) );
  assert_eq!( paths.socket_file(), paths.runtime_dir().join( SOCKET_FILE_NAME ) );
  assert_eq!( paths.log_file(), paths.runtime_dir().join( LOG_FILE_NAME ) );

  let owned = [ paths.lock_file(), paths.socket_file(), paths.log_file() ];
  let distinct : std::collections::BTreeSet< _ > = owned.iter().collect();
  assert_eq!( distinct.len(), owned.len(), "two of the daemon's files share a path: {owned:?}" );
}

/// path03: the registry is Claude Code's, not the daemon's.
///
/// It must stay outside the runtime directory: the daemon *reads* it, and a
/// daemon that put its own state there would be writing into a directory another
/// program owns and reaps.
#[ test ]
fn path03_sessions_dir_is_claude_codes_not_the_daemons()
{
  let home = std::path::Path::new( "/somewhere/home" );
  let paths = DaemonPaths::with_home( home );

  assert_eq!( paths.sessions_dir(), home.join( ".claude" ).join( "sessions" ) );
  assert!(
    !paths.sessions_dir().starts_with( paths.runtime_dir() ),
    "the registry is inside the daemon's runtime directory",
  );
}

/// path04: the runtime directory is hyphen-prefixed, so git ignores it.
///
/// These files are machine-local: a socket path, a lock, and whatever state a
/// running daemon accumulates. The workspace's global `-*` rule is what keeps
/// them out of commits, and it only applies to a leading hyphen.
#[ test ]
fn path04_runtime_dir_name_is_hyphen_prefixed()
{
  assert!(
    RUNTIME_DIR_NAME.starts_with( '-' ),
    "runtime dir {RUNTIME_DIR_NAME:?} is not hyphen-prefixed — it would be committed",
  );
}

/// path05: two homes produce entirely disjoint paths.
///
/// The injected-home form is what lets tests run in parallel without mutating a
/// shared `HOME`.
#[ test ]
fn path05_distinct_homes_do_not_share_paths()
{
  let first = DaemonPaths::with_home( std::path::Path::new( "/home/one" ) );
  let second = DaemonPaths::with_home( std::path::Path::new( "/home/two" ) );

  assert_ne!( first.runtime_dir(), second.runtime_dir() );
  assert_ne!( first.lock_file(), second.lock_file() );
  assert_ne!( first.socket_file(), second.socket_file() );
  assert_ne!( first.log_file(), second.log_file() );
  assert_ne!( first.sessions_dir(), second.sessions_dir() );
}
