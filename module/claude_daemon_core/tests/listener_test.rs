//! Listener tests — binding over wreckage, and not leaving any.
//!
//! Both halves of a socket's life are failure-shaped when they go wrong, and
//! neither announces itself. A stale socket left by a killed daemon makes the
//! next `bind` fail with `AddrInUse`; a socket left behind by a daemon that
//! exited makes the next *client* fail with `ECONNREFUSED`, which reads like a
//! broken daemon rather than an absent one.
//!
//! The lock check (lis04) is the one that would be easy to skip. Removing a
//! socket before binding is only safe because nothing else can be listening on
//! it, and the instance lock is the entire basis for believing that — so a lock
//! held over some other directory has to be refused, not accepted as a formality.
//!
//! ## Specification References
//!
//! - `docs/feature/006_serving_clients.md` — the socket's lifecycle
//! - `docs/feature/001_single_instance.md` — what the lock guarantees
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | lis01 | Binding a fresh path | Socket exists; `path()` reports it |
//! | lis02 | Dropping the listener | Socket file is gone |
//! | lis03 | Binding over a stale socket | Succeeds; the old file is replaced |
//! | lis04 | A lock from another directory | `LockMismatch`, and the file is untouched |
//! | lis05 | A bound socket's permissions | Owner read/write only |
//! | lis06 | A non-socket at the socket path | `Io`, and the file survives |
//! | lis07 | A client connecting to it | `accept` hands back the connection |

use std::io::{ Read, Write };
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{ UnixListener, UnixStream };
use std::path::Path;

use claude_daemon_core::{ acquire, Error, InstanceLock, Listener };

/// Take the instance lock inside `dir`, as a daemon would before binding.
fn lock_in( dir : &Path ) -> InstanceLock
{
  acquire( &dir.join( "instance.lock" ) ).expect( "acquiring the instance lock failed" )
}

/// lis01: a bound listener has a socket at the path it reports.
#[ test ]
fn lis01_bind_creates_the_socket()
{
  let dir = tempfile::tempdir().expect( "tempdir failed" );
  let lock = lock_in( dir.path() );
  let socket = dir.path().join( "daemon.sock" );

  let listener = Listener::bind( &socket, &lock ).expect( "bind failed" );

  assert_eq!( listener.path(), socket );
  assert!( socket.exists(), "bind reported success without creating a socket" );
}

/// lis02: an ordinary exit leaves nothing for the next daemon to explain.
#[ test ]
fn lis02_drop_removes_the_socket()
{
  let dir = tempfile::tempdir().expect( "tempdir failed" );
  let lock = lock_in( dir.path() );
  let socket = dir.path().join( "daemon.sock" );

  drop( Listener::bind( &socket, &lock ).expect( "bind failed" ) );

  assert!( !socket.exists(), "the socket outlived the listener that bound it" );
}

/// lis03: a socket left by a killed daemon does not block the next one.
///
/// `SIGKILL` runs no destructor, so this is the state a crashed daemon leaves
/// behind — and `bind` refuses an existing path whether or not anything is
/// listening on it. Without the unlink, one crash makes the daemon unstartable
/// until somebody deletes a file by hand.
#[ test ]
fn lis03_bind_replaces_a_stale_socket()
{
  let dir = tempfile::tempdir().expect( "tempdir failed" );
  let lock = lock_in( dir.path() );
  let socket = dir.path().join( "daemon.sock" );

  // Exactly what a killed daemon leaves behind: a real socket node with nothing
  // listening on it. `UnixListener` does not unlink its path on drop — which is
  // the reason `Listener` has a destructor at all.
  drop( UnixListener::bind( &socket ).expect( "staging the stale socket failed" ) );
  assert!( socket.exists(), "test premise broken: no stale socket was left" );

  let listener = Listener::bind( &socket, &lock ).expect( "bind over a stale socket failed" );

  assert_eq!( listener.path(), socket );
}

/// lis04: a lock over a different directory proves nothing about this socket.
///
/// The dangerous case, and the one the type signature alone would not catch: a
/// caller holding *some* lock, passing it, and unlinking a socket a live daemon
/// is listening on.
#[ test ]
fn lis04_a_lock_from_elsewhere_is_refused()
{
  let locked = tempfile::tempdir().expect( "tempdir failed" );
  let elsewhere = tempfile::tempdir().expect( "tempdir failed" );
  let lock = lock_in( locked.path() );
  let socket = elsewhere.path().join( "daemon.sock" );

  // Something already at the target path, so a refusal that still unlinked would
  // be visible.
  std::fs::write( &socket, b"not really a socket" ).expect( "writing the decoy failed" );

  let error = Listener::bind( &socket, &lock ).expect_err( "a foreign lock was accepted" );

  assert!( matches!( error, Error::LockMismatch { .. } ), "unexpected error: {error}" );
  assert!( socket.exists(), "a refused bind removed the file anyway" );
}

/// lis05: the socket is not readable by other users.
///
/// It carries prompts and transcript output. The runtime directory is the real
/// boundary; this narrows what gets through it if that boundary is wider than
/// it should be.
#[ test ]
fn lis05_socket_is_owner_only()
{
  let dir = tempfile::tempdir().expect( "tempdir failed" );
  let lock = lock_in( dir.path() );
  let socket = dir.path().join( "daemon.sock" );

  let listener = Listener::bind( &socket, &lock ).expect( "bind failed" );
  let mode = std::fs::metadata( listener.path() )
    .expect( "stat failed" )
    .permissions()
    .mode();

  assert_eq!( mode & 0o777, 0o600, "socket mode is {:o}, not 600", mode & 0o777 );
}

/// lis06: something that is not a socket is refused rather than deleted.
///
/// The lock is what justifies the unlink in lis03, and what it establishes is
/// that no daemon is listening — a claim about processes, not about files. A
/// regular file sitting at that path is outside what the lock covers, so it is
/// an error to bind over, not a casualty of doing so. The lock in this test is
/// the *right* lock, which is what separates this case from lis04.
#[ test ]
fn lis06_a_non_socket_at_the_path_is_refused()
{
  let dir = tempfile::tempdir().expect( "tempdir failed" );
  let lock = lock_in( dir.path() );
  let socket = dir.path().join( "daemon.sock" );
  std::fs::write( &socket, b"not really a socket" ).expect( "writing the decoy failed" );

  let error = Listener::bind( &socket, &lock ).expect_err( "a regular file was bound over" );

  assert!( matches!( error, Error::Io( _ ) ), "unexpected error: {error}" );
  let survived = std::fs::read_to_string( &socket ).expect( "the decoy was deleted" );
  assert_eq!( survived, "not really a socket" );
}

/// lis07: a client can reach it, and `accept` hands back a usable connection.
#[ test ]
fn lis07_accept_returns_a_connected_stream()
{
  let dir = tempfile::tempdir().expect( "tempdir failed" );
  let lock = lock_in( dir.path() );
  let socket = dir.path().join( "daemon.sock" );
  let listener = Listener::bind( &socket, &lock ).expect( "bind failed" );

  let greeting = std::thread::scope( | scope |
  {
    scope.spawn( ||
    {
      let mut client = UnixStream::connect( &socket ).expect( "connect failed" );
      client.write_all( b"hello\n" ).expect( "client write failed" );
    });

    let mut accepted = listener.accept().expect( "accept failed" );
    let mut received = String::new();
    accepted.read_to_string( &mut received ).expect( "read failed" );
    received
  });

  assert_eq!( greeting, "hello\n" );
}
