//! Real pseudo-terminal allocation tests.
//!
//! ## Specification References
//!
//! - `docs/feature/001_pty_allocation.md` — allocation sequence and window size
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | pty01 | `Pty::open()` succeeds | Returns a `Pty` |
//! | pty02 | Slave path shape | Non-empty, starts with `/dev/pts/` |
//! | pty03 | Two allocations get distinct slaves | Paths differ |
//! | pty04 | `open_slave()` twice | Two independent descriptors, distinct fd numbers |
//! | pty05 | `resize()` on a fresh pty | Succeeds |
//! | pty06 | `WinSize::default()` | 24 rows, 80 cols |
//! | pty07 | Child observes the size set before spawn | `stty size` reports it |

use std::io::Write;
use std::os::fd::AsRawFd;
use std::process::Command;

use claude_pty_core::{ Pty, WinSize };

/// pty01, pty02: allocation succeeds and yields a usable slave path.
#[ test ]
fn pty01_open_yields_slave_path()
{
  let pty = Pty::open().expect( "pty allocation failed" );
  let path = pty.slave_path();

  assert!( !path.is_empty(), "slave path is empty" );
  assert!(
    path.starts_with( "/dev/pts/" ),
    "slave path {path:?} is not under /dev/pts/ — ptsname_r returned something unexpected",
  );
}

/// pty03: two concurrent allocations do not collide.
///
/// This is what `ptsname_r` buys over `ptsname`: the latter writes into a static
/// buffer shared process-wide, so two allocations would report the same path and
/// neither caller would know.
#[ test ]
fn pty03_allocations_are_distinct()
{
  let first = Pty::open().expect( "first pty allocation failed" );
  let second = Pty::open().expect( "second pty allocation failed" );

  assert_ne!(
    first.slave_path(),
    second.slave_path(),
    "two allocations reported the same slave path",
  );
}

/// pty04: each `open_slave()` yields an independent descriptor.
///
/// Independence is load-bearing: `PtySession::spawn` gives stdin, stdout, and
/// stderr one each, and a shared open file description would mean closing any
/// one closes all three.
#[ test ]
fn pty04_open_slave_yields_independent_descriptors()
{
  let pty = Pty::open().expect( "pty allocation failed" );

  let first = pty.open_slave().expect( "first open_slave failed" );
  let mut second = pty.open_slave().expect( "second open_slave failed" );

  assert_ne!(
    first.as_raw_fd(),
    second.as_raw_fd(),
    "open_slave returned the same fd twice",
  );

  // Closing one must leave the other usable. If they shared an open file
  // description, this write would fail with EBADF.
  drop( first );
  second.write_all( b"still open\n" ).expect( "surviving slave descriptor was invalidated by its sibling's close" );
}

/// pty05, pty06: resize succeeds and the default is the historical 24x80.
#[ test ]
fn pty05_resize_succeeds()
{
  let pty = Pty::open().expect( "pty allocation failed" );

  assert_eq!( WinSize::default(), WinSize::new( 24, 80 ), "default window size changed" );

  pty.resize( WinSize::new( 40, 132 ) ).expect( "resize failed" );
  pty.resize( WinSize::default() ).expect( "resize back to default failed" );
}

/// pty07: a child on the slave side observes the size set on the master.
///
/// A terminal program reads its window size from the kernel, not from an
/// environment variable — this asserts the `TIOCSWINSZ` actually reached it.
#[ test ]
fn pty07_child_observes_configured_size()
{
  let pty = Pty::open().expect( "pty allocation failed" );
  pty.resize( WinSize::new( 40, 132 ) ).expect( "resize failed" );

  let slave = pty.open_slave().expect( "open_slave failed" );
  let output = Command::new( "stty" )
    .arg( "size" )
    .stdin( slave )
    .output()
    .expect( "cannot run stty — is it on PATH?" );

  let reported = String::from_utf8_lossy( &output.stdout ).trim().to_string();
  assert_eq!(
    reported, "40 132",
    "child reported window size {reported:?}, expected \"40 132\"",
  );
}
