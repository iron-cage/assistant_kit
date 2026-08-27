//! The daemon's table of hosted sessions.
//!
//! Keyed by conversation id rather than PID. Claude Code's own daemon re-hosts a
//! session with `--fork-session` on auto-update or recovery, producing a process
//! with a different PID, no inherited environment, and a new conversation id — so
//! a PID key detaches silently at exactly the moment recovery is meant to help.
//! The conversation id survives as the handle a client keeps using.

use core::time::Duration;
use std::collections::HashMap;
use std::path::{ Path, PathBuf };
use std::process::ExitStatus;
use std::time::Instant;

use claude_pty_core::{ PtySession, WinSize };

use crate::error::{ Error, Result };
use crate::output::{ OutputPump, OutputSlice, DEFAULT_OUTPUT_CAP };
use crate::protocol::SessionSummary;

/// End-of-transmission — the byte a terminal sends for `Ctrl-D`.
const EOT : u8 = 0x04;

/// How long a session gets to exit on its own before it is killed.
///
/// Long enough for an interactive program to flush a transcript and release its
/// locks; short enough that a wedged one does not hold up a client indefinitely.
const SHUTDOWN_GRACE : Duration = Duration::from_secs( 5 );

/// How often the shutdown path re-checks whether the child has exited.
const SHUTDOWN_POLL : Duration = Duration::from_millis( 20 );

/// One session hosted by the daemon.
///
/// Fields are private because two of them have an invariant between them: the
/// [`OutputPump`] holds a clone of the pty master, so a session cannot be
/// constructed without a pump draining it, nor torn down without stopping that
/// pump first. Public fields would make both mistakes expressible — and both are
/// silent, presenting as a session that appears to think forever.
#[ derive( Debug ) ]
pub struct HostedSession
{
  session_id : String,
  cwd : PathBuf,
  pty : PtySession,
  pump : OutputPump,
  busy : bool,
}

impl HostedSession
{
  /// Adopt a freshly spawned `pty` as the session named `session_id`, and start
  /// draining its output.
  ///
  /// # Errors
  ///
  /// Returns [`Error::ReaderTaken`] if something has already taken the session's
  /// reader — without it there is no way to drain the master, and an undrained
  /// master stalls the child as soon as the kernel's buffer fills.
  #[ inline ]
  pub fn adopt
  (
    session_id : impl Into< String >,
    cwd : impl Into< PathBuf >,
    mut pty : PtySession,
  )
  -> Result< Self >
  {
    let reader = pty.take_reader().ok_or( Error::ReaderTaken )?;
    Ok( Self
    {
      session_id : session_id.into(),
      cwd : cwd.into(),
      pump : OutputPump::spawn( reader, DEFAULT_OUTPUT_CAP ),
      pty,
      busy : false,
    })
  }

  /// The client-facing handle.
  #[ inline ]
  #[ must_use ]
  pub fn session_id( &self ) -> &str
  {
    &self.session_id
  }

  /// Working directory the session runs in.
  #[ inline ]
  #[ must_use ]
  pub fn cwd( &self ) -> &Path
  {
    &self.cwd
  }

  /// Current process id.
  ///
  /// Diagnostic only — for correlating against `ps` or a registry scan. Never an
  /// address: a re-host changes it while the conversation id stays put.
  #[ inline ]
  #[ must_use ]
  pub fn pid( &self ) -> u32
  {
    self.pty.pid()
  }

  /// Whether the daemon currently believes a turn is in flight.
  #[ inline ]
  #[ must_use ]
  pub const fn busy( &self ) -> bool
  {
    self.busy
  }

  /// Record whether a turn is in flight.
  ///
  /// Maintained from `claude_session_core`'s turn watcher, not sampled from the
  /// registry directly — the difference is the whole point of turn detection
  /// being its own feature rather than a field read.
  #[ inline ]
  pub fn set_busy( &mut self, busy : bool )
  {
    self.busy = busy;
  }

  /// Deliver `bytes` to the session's terminal.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Pty`] if the write queue is full or its thread is gone.
  #[ inline ]
  pub fn write( &self, bytes : &[ u8 ] ) -> Result< () >
  {
    self.pty.write( bytes ).map_err( Error::Pty )
  }

  /// Change the session's terminal dimensions.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Pty`] if the session is already closed or the ioctl fails.
  #[ inline ]
  pub fn resize( &self, rows : u16, cols : u16 ) -> Result< () >
  {
    self.pty.resize( WinSize::new( rows, cols ) ).map_err( Error::Pty )
  }

  /// Read the session's output since `cursor`.
  #[ inline ]
  #[ must_use ]
  pub fn read_from( &self, cursor : u64 ) -> OutputSlice
  {
    self.pump.read_from( cursor )
  }

  /// Absolute position just past the newest byte of output.
  ///
  /// Taken immediately before writing a prompt, this is exactly where that
  /// prompt's output begins — the daemon is single-threaded, so nothing can
  /// have written to this session in between.
  #[ inline ]
  #[ must_use ]
  pub fn output_end( &self ) -> u64
  {
    self.pump.end()
  }

  /// Summarize for [`crate::protocol::Request::ListSessions`].
  #[ inline ]
  #[ must_use ]
  pub fn summary( &self ) -> SessionSummary
  {
    SessionSummary
    {
      session_id : self.session_id.clone(),
      pid : self.pty.pid(),
      cwd : self.cwd.clone(),
      busy : self.busy,
    }
  }

  /// End the session and reap it, in the only order that terminates.
  ///
  /// Three steps, each of which the next one depends on:
  ///
  /// 1. **`Ctrl-D`.** An interactive program handed end-of-input exits through
  ///    its own shutdown path — flushing a transcript, releasing locks. Nothing
  ///    below gives it that chance, so it goes first.
  /// 2. **Kill on grace expiry.** A wedged child would otherwise hold the daemon
  ///    here forever, because step 3 cannot proceed while the child lives.
  /// 3. **Join the pump, then shut the pty down.** The pump holds a master
  ///    descriptor that [`PtySession::shutdown`] cannot reach; while it lives the
  ///    child never sees a hangup and `shutdown` waits for a child that is
  ///    waiting for it. The pump releases that descriptor only when its read
  ///    ends, which happens when the child's own descriptors close — which is
  ///    what steps 1 and 2 exist to bring about.
  ///
  /// Idempotent: a second call finds an already-exited child and returns the
  /// status recorded by the first.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Pty`] if waiting on or reaping the child fails.
  #[ inline ]
  pub fn shutdown( &mut self ) -> Result< ExitStatus >
  {
    // Twice, because a terminal in canonical mode only reads `Ctrl-D` as
    // end-of-input at the start of a line: with a partial line pending, the first
    // one flushes it and the second lands where it means EOF. Sending a newline
    // to clear the line instead would submit whatever the user had half-typed.
    //
    // Best effort: a child that has already exited has no terminal left to write
    // to, and that failure means the request has been satisfied, not refused.
    drop( self.pty.write( &[ EOT, EOT ] ) );

    let deadline = Instant::now() + SHUTDOWN_GRACE;
    while self.pty.try_wait().map_err( Error::Pty )?.is_none()
    {
      if Instant::now() >= deadline
      {
        self.pty.kill().map_err( Error::Pty )?;
        break;
      }
      std::thread::sleep( SHUTDOWN_POLL );
    }

    self.pump.join();
    self.pty.shutdown().map_err( Error::Pty )
  }
}

/// Every session the daemon owns.
#[ derive( Debug, Default ) ]
pub struct SessionTable
{
  sessions : HashMap< String, HostedSession >,
}

impl SessionTable
{
  /// An empty table.
  #[ inline ]
  #[ must_use ]
  pub fn new() -> Self
  {
    Self::default()
  }

  /// Number of hosted sessions.
  #[ inline ]
  #[ must_use ]
  pub fn len( &self ) -> usize
  {
    self.sessions.len()
  }

  /// Whether the table hosts no sessions.
  #[ inline ]
  #[ must_use ]
  pub fn is_empty( &self ) -> bool
  {
    self.sessions.is_empty()
  }

  /// Add a session, replacing any existing entry with the same conversation id.
  ///
  /// A replaced session is returned rather than dropped: dropping one silently
  /// would leak its pump thread and leave its child running with nobody holding
  /// the handle, so the caller has to decide what happens to it.
  #[ inline ]
  #[ must_use = "a replaced session still has a live child and pump thread" ]
  pub fn insert( &mut self, session : HostedSession ) -> Option< HostedSession >
  {
    self.sessions.insert( session.session_id.clone(), session )
  }

  /// Borrow a session by conversation id.
  ///
  /// # Errors
  ///
  /// Returns [`Error::UnknownSession`] when no session carries `session_id`.
  #[ inline ]
  pub fn get( &self, session_id : &str ) -> Result< &HostedSession >
  {
    self.sessions
      .get( session_id )
      .ok_or_else( || Error::UnknownSession( session_id.to_string() ) )
  }

  /// Borrow a session mutably by conversation id.
  ///
  /// # Errors
  ///
  /// Returns [`Error::UnknownSession`] when no session carries `session_id`.
  #[ inline ]
  pub fn get_mut( &mut self, session_id : &str ) -> Result< &mut HostedSession >
  {
    self.sessions
      .get_mut( session_id )
      .ok_or_else( || Error::UnknownSession( session_id.to_string() ) )
  }

  /// Remove a session, returning it.
  ///
  /// # Errors
  ///
  /// Returns [`Error::UnknownSession`] when no session carries `session_id`.
  #[ inline ]
  pub fn remove( &mut self, session_id : &str ) -> Result< HostedSession >
  {
    self.sessions
      .remove( session_id )
      .ok_or_else( || Error::UnknownSession( session_id.to_string() ) )
  }

  /// Summarize every hosted session, ordered by conversation id.
  #[ inline ]
  #[ must_use ]
  pub fn summaries( &self ) -> Vec< SessionSummary >
  {
    let mut out : Vec< SessionSummary > = self.sessions.values().map( HostedSession::summary ).collect();
    out.sort_by( | a, b | a.session_id.cmp( &b.session_id ) );
    out
  }

  /// Conversation ids of every hosted session, ordered.
  #[ inline ]
  #[ must_use ]
  pub fn session_ids( &self ) -> Vec< String >
  {
    let mut out : Vec< String > = self.sessions.keys().cloned().collect();
    out.sort();
    out
  }
}
