//! Error type for `claude_pty_core`.
//!
//! Hand-rolled rather than built on `error_tools`, matching the convention of
//! the workspace's other dependency-free core crates (`claude_storage_core`,
//! `claude_core`). `error_tools` enters the graph at Layer 1 and above.

use core::fmt;

/// Result alias for every fallible operation in this crate.
pub type Result< T > = core::result::Result< T, Error >;

/// Errors produced by PTY allocation, child spawning, and PTY I/O.
#[ derive( Debug ) ]
#[ non_exhaustive ]
pub enum Error
{
  /// A POSIX call failed. Carries the failing operation's name so the OS error
  /// — which on its own says only `ENOTTY` or `EINVAL` — can be attributed to a
  /// specific step of the allocation sequence.
  Os
  {
    /// Name of the POSIX call that failed, e.g. `"posix_openpt"`.
    op : &'static str,
    /// The underlying OS error.
    source : std::io::Error,
  },
  /// The slave device path reported by `ptsname_r` was not valid UTF-8.
  ///
  /// Separated from [`Error::Os`] because it is a data-shape failure, not a
  /// syscall failure — `ptsname_r` itself succeeded.
  NonUtf8SlavePath,
  /// A write was refused because the writer queue is at capacity.
  ///
  /// Signals a child that has stopped draining its stdin. The write is dropped,
  /// never buffered without bound — see `docs/feature/002_writer_thread.md`.
  WriterFull,
  /// The writer thread has exited, so no further writes can be delivered.
  WriterGone,
  /// The child process could not be spawned.
  Spawn( std::io::Error ),
  /// The operation needs the pty master, which `shutdown` has already closed.
  ///
  /// Distinct from [`Error::WriterGone`]: that one means the write path ended
  /// while the terminal is still there, this one means the terminal itself is
  /// gone.
  SessionClosed,
}

impl fmt::Display for Error
{
  #[ inline ]
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
    match self
    {
      Self::Os { op, source } => write!( f, "posix call `{op}` failed: {source}" ),
      Self::NonUtf8SlavePath => write!( f, "pty slave device path is not valid UTF-8" ),
      Self::WriterFull => write!( f, "pty writer queue is full — child is not reading stdin" ),
      Self::WriterGone => write!( f, "pty writer thread has exited" ),
      Self::Spawn( source ) => write!( f, "cannot spawn child on pty: {source}" ),
      Self::SessionClosed => write!( f, "pty session is closed — the master is no longer open" ),
    }
  }
}

impl std::error::Error for Error
{
  #[ inline ]
  fn source( &self ) -> Option< &( dyn std::error::Error + 'static ) >
  {
    match self
    {
      Self::Os { source, .. } | Self::Spawn( source ) => Some( source ),
      Self::NonUtf8SlavePath | Self::WriterFull | Self::WriterGone | Self::SessionClosed => None,
    }
  }
}

impl Error
{
  /// Build an [`Error::Os`] from the current `errno`.
  ///
  /// Called immediately after a failing POSIX call, while `errno` still refers
  /// to it — any intervening call may overwrite the value.
  #[ inline ]
  #[ must_use ]
  pub fn last_os( op : &'static str ) -> Self
  {
    Self::Os { op, source : std::io::Error::last_os_error() }
  }
}
