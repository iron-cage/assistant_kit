//! Error type for `claude_daemon_core`.

use core::fmt;

/// Result alias for every fallible operation in this crate.
pub type Result< T > = core::result::Result< T, Error >;

/// Errors produced by the daemon, its lock, and its IPC layer.
#[ derive( Debug ) ]
#[ non_exhaustive ]
pub enum Error
{
  /// An I/O operation failed.
  Io( std::io::Error ),
  /// Another daemon instance already holds the lock.
  ///
  /// Not a failure the caller should retry through — exactly one daemon may run,
  /// and the correct response is to talk to the existing one.
  AlreadyRunning
  {
    /// Path of the contended lock file.
    lock_path : std::path::PathBuf,
  },
  /// The instance lock offered as evidence does not cover the socket being bound.
  ///
  /// Removing a stale socket is safe only because no other daemon can be
  /// listening on it, and the instance lock is what establishes that. A lock
  /// held over a different directory establishes nothing about this one.
  LockMismatch
  {
    /// Path of the lock that was offered.
    lock_path : std::path::PathBuf,
    /// Path of the socket it was offered for.
    socket_path : std::path::PathBuf,
  },
  /// A protocol line exceeded [`crate::ipc::MAX_IPC_LINE_BYTES`].
  LineTooLong,
  /// A protocol line was not valid UTF-8.
  NonUtf8Line,
  /// A protocol line was not valid JSON, or not a known request shape.
  Malformed( String ),
  /// No hosted session carries the requested conversation id.
  UnknownSession( String ),
  /// A session's reader was already taken, so its output cannot be drained.
  ///
  /// A pty master that nobody reads stalls its child as soon as the kernel's
  /// buffer fills, and a stalled child is indistinguishable from a thinking one.
  /// Refusing to host the session is better than hosting one that will silently
  /// wedge.
  ReaderTaken,
  /// The spawned process never registered a conversation id.
  ///
  /// Claude Code writes its registry record shortly after start; a process that
  /// has not done so within the timeout either is not Claude Code or failed
  /// before it got that far.
  NoRegistration
  {
    /// Process id that was being waited on.
    pid : u32,
  },
  /// The daemon answered, and its answer was a failure.
  ///
  /// Carries the daemon's own message verbatim. Distinct from every other
  /// variant here, which describe something going wrong on *this* side of the
  /// socket — a `Remote` means the round trip worked and the request did not.
  Remote( String ),
  /// A PTY-layer operation failed.
  Pty( claude_pty_core::Error ),
  /// Reading Claude Code's session registry failed.
  Registry( claude_session_core::Error ),
  /// Reading Claude Code's on-disk conversation storage failed.
  Storage( claude_storage_core::Error ),
  /// The session has no transcript to read.
  ///
  /// Either its working directory will not encode to a storage path, or the
  /// session has not written a transcript yet — Claude Code creates one on the
  /// first turn, so a session spawned moments ago legitimately has none.
  /// Reported rather than answered with an empty summary, which would read as
  /// "this session's context is empty" when the truth is "not known yet".
  NoTranscript
  {
    /// Conversation id whose transcript could not be read.
    session_id : String,
  },
  /// A baseline probe ran but did not produce a usable measurement.
  ///
  /// Distinct from [`Self::Io`], which means `claude` could not be run at all.
  /// This one means it ran and then failed, or answered something that could not
  /// be read as a measurement — a probe that half-worked must not be recorded as
  /// a floor of zero, which would report a session's entire context as
  /// conversation.
  Probe
  {
    /// What went wrong, including `claude`'s own stderr on a non-zero exit.
    reason : String,
  },
}

impl fmt::Display for Error
{
  #[ inline ]
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
    match self
    {
      Self::Io( source ) => write!( f, "io error: {source}" ),
      Self::AlreadyRunning { lock_path } =>
        write!( f, "another daemon already holds {}", lock_path.display() ),
      Self::LockMismatch { lock_path, socket_path } => write!
      (
        f,
        "instance lock {} does not cover socket {}",
        lock_path.display(),
        socket_path.display(),
      ),
      Self::LineTooLong =>
        write!( f, "protocol line exceeds {} bytes", crate::ipc::MAX_IPC_LINE_BYTES ),
      Self::NonUtf8Line => write!( f, "protocol line is not valid UTF-8" ),
      Self::Malformed( detail ) => write!( f, "malformed request: {detail}" ),
      Self::UnknownSession( id ) => write!( f, "no such session: {id}" ),
      Self::ReaderTaken => write!( f, "session output is already being read elsewhere" ),
      Self::NoRegistration { pid } =>
        write!( f, "process {pid} never registered a conversation id" ),
      Self::Remote( message ) => write!( f, "daemon reported: {message}" ),
      Self::Pty( source ) => write!( f, "pty error: {source}" ),
      Self::Registry( source ) => write!( f, "session registry error: {source}" ),
      Self::Storage( source ) => write!( f, "conversation storage error: {source}" ),
      Self::NoTranscript { session_id } =>
        write!( f, "session {session_id} has no readable transcript" ),
      Self::Probe { reason } => write!( f, "baseline probe failed: {reason}" ),
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
      Self::Io( source ) => Some( source ),
      Self::Pty( source ) => Some( source ),
      Self::Registry( source ) => Some( source ),
      Self::Storage( source ) => Some( source ),
      Self::AlreadyRunning { .. }
      | Self::LockMismatch { .. }
      | Self::LineTooLong
      | Self::NonUtf8Line
      | Self::Malformed( _ )
      | Self::UnknownSession( _ )
      | Self::ReaderTaken
      | Self::NoRegistration { .. }
      | Self::NoTranscript { .. }
      | Self::Probe { .. }
      | Self::Remote( _ ) => None,
    }
  }
}

impl From< claude_pty_core::Error > for Error
{
  #[ inline ]
  fn from( source : claude_pty_core::Error ) -> Self
  {
    Self::Pty( source )
  }
}

impl From< std::io::Error > for Error
{
  #[ inline ]
  fn from( source : std::io::Error ) -> Self
  {
    Self::Io( source )
  }
}

impl From< claude_session_core::Error > for Error
{
  #[ inline ]
  fn from( source : claude_session_core::Error ) -> Self
  {
    Self::Registry( source )
  }
}

impl From< claude_storage_core::Error > for Error
{
  #[ inline ]
  fn from( source : claude_storage_core::Error ) -> Self
  {
    Self::Storage( source )
  }
}
