//! A table of supervised child sessions, keyed by a caller-chosen id rather
//! than PID.
//!
//! PID is a poor key for anything that outlives one process incarnation: a
//! caller that re-hosts a child under a fresh process — same conversation, new
//! PID, no inherited environment — needs the table to keep recognizing it. A
//! PID key detaches silently at exactly the moment re-hosting was meant to
//! help. Whatever id the caller mints instead survives that, as long as it is
//! durable across the replacement.

use core::time::Duration;
use std::collections::HashMap;
use std::path::{ Path, PathBuf };
use std::process::ExitStatus;
use std::time::Instant;

use claude_pty_core::{ PtySession, WinSize };

use crate::error::{ Error, Result };
use crate::output::{ OutputPump, OutputSlice, DEFAULT_OUTPUT_CAP };

/// End-of-transmission — the byte a terminal sends for `Ctrl-D`.
const EOT : u8 = 0x04;

/// How long a session gets to exit on its own before it is killed.
///
/// Long enough for an interactive program to flush a transcript and release its
/// locks; short enough that a wedged one does not hold up a client indefinitely.
const SHUTDOWN_GRACE : Duration = Duration::from_secs( 5 );

/// How often the shutdown path re-checks whether the child has exited.
const SHUTDOWN_POLL : Duration = Duration::from_millis( 20 );

/// One session hosted by the supervisor.
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
  last_active : Instant,
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
      last_active : Instant::now(),
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
  /// address: a re-host changes it while the session id stays put.
  #[ inline ]
  #[ must_use ]
  pub fn pid( &self ) -> u32
  {
    self.pty.pid()
  }

  /// Whether the caller currently believes this session is doing work.
  ///
  /// Purely a flag this type stores and reports — nothing here decides when it
  /// flips. Deciding that is the job of whatever caller can observe the child's
  /// own progress; see [`Self::set_busy`].
  #[ inline ]
  #[ must_use ]
  pub const fn busy( &self ) -> bool
  {
    self.busy
  }

  /// When this session last saw activity, per [`Self::touch`], or was last
  /// reported busy, per [`Self::set_busy`].
  ///
  /// What an idle-reaping policy built on this table measures against.
  #[ inline ]
  #[ must_use ]
  pub const fn last_active( &self ) -> Instant
  {
    self.last_active
  }

  /// Record whether the session is doing work.
  ///
  /// Deliberately does not touch [`Self::last_active`] itself: a long-running
  /// turn needs the clock refreshed on *every* tick it is still busy, not only
  /// at the moment it started, and repeating that refresh is the caller's own
  /// job — the caller is what knows how often "still busy" gets re-observed.
  #[ inline ]
  pub fn set_busy( &mut self, busy : bool )
  {
    self.busy = busy;
  }

  /// Record activity, resetting the idle clock [`Self::last_active`] reports.
  #[ inline ]
  pub fn touch( &mut self )
  {
    self.last_active = Instant::now();
  }

  /// Whether the child has already exited, without waiting for it.
  ///
  /// `None` while it is still running. A session that answers `Some` is dead
  /// weight and cannot become live again: it still occupies a session id, still
  /// reports a pid, and still holds a pump thread, but every write to it fails.
  /// Nothing else notices — a caller learns about it only by writing to a
  /// session that cannot answer.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Pty`] if reaping the child fails.
  #[ inline ]
  pub fn exited( &mut self ) -> Result< Option< ExitStatus > >
  {
    self.pty.try_wait().map_err( Error::Pty )
  }

  /// Deliver `bytes` to the session's terminal.
  ///
  /// Counts as activity: touches [`Self::last_active`].
  ///
  /// # Errors
  ///
  /// Returns [`Error::Pty`] if the write queue is full or its thread is gone.
  #[ inline ]
  pub fn write( &mut self, bytes : &[ u8 ] ) -> Result< () >
  {
    self.touch();
    self.pty.write( bytes ).map_err( Error::Pty )
  }

  /// Change the session's terminal dimensions.
  ///
  /// Counts as activity: touches [`Self::last_active`].
  ///
  /// # Errors
  ///
  /// Returns [`Error::Pty`] if the session is already closed or the ioctl fails.
  #[ inline ]
  pub fn resize( &mut self, rows : u16, cols : u16 ) -> Result< () >
  {
    self.touch();
    self.pty.resize( WinSize::new( rows, cols ) ).map_err( Error::Pty )
  }

  /// Read the session's output since `cursor`.
  ///
  /// Counts as activity: touches [`Self::last_active`].
  #[ inline ]
  #[ must_use ]
  pub fn read_from( &mut self, cursor : u64 ) -> OutputSlice
  {
    self.touch();
    self.pump.read_from( cursor )
  }

  /// Absolute position just past the newest byte of output.
  ///
  /// Taken immediately before writing a prompt, this is exactly where that
  /// prompt's output begins — a single-threaded caller has nothing else that
  /// could have written to this session in between.
  #[ inline ]
  #[ must_use ]
  pub fn output_end( &self ) -> u64
  {
    self.pump.end()
  }

  /// End the session and reap it, in the only order that terminates.
  ///
  /// Three steps, each of which the next one depends on:
  ///
  /// 1. **`Ctrl-D`.** An interactive program handed end-of-input exits through
  ///    its own shutdown path — flushing a transcript, releasing locks. Nothing
  ///    below gives it that chance, so it goes first.
  /// 2. **Kill on grace expiry.** A wedged child would otherwise hold the caller
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

/// Every session this table owns.
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

  /// Add a session, replacing any existing entry with the same id.
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

  /// Borrow a session by id.
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

  /// Borrow a session mutably by id.
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

  /// Ids of every hosted session, ordered.
  #[ inline ]
  #[ must_use ]
  pub fn session_ids( &self ) -> Vec< String >
  {
    let mut out : Vec< String > = self.sessions.keys().cloned().collect();
    out.sort();
    out
  }

  /// Remove every session whose child has already exited, ordered by id.
  ///
  /// A session outlives its child: the caller holds the handle, the pump keeps
  /// its thread, and the table keeps the row. Nothing here notices, so the row
  /// stays listed as hosted until something happens to write to it and gets an
  /// error for a session that died an hour ago.
  ///
  /// Returned rather than dropped, for the same reason [`SessionTable::insert`]
  /// returns what it replaced: each one still owns a pump thread, and dropping it
  /// silently would leak that thread. Call [`HostedSession::shutdown`] on each —
  /// it costs nothing on a child that has already exited, because the grace loop
  /// finds it gone on the first check.
  ///
  /// A failure to determine liveness leaves the session in place. `try_wait`
  /// erroring means the caller does not *know* whether the child is alive, and
  /// evicting on "do not know" would take a working session out of the table.
  #[ inline ]
  #[ must_use = "each removed session still holds a pump thread until it is shut down" ]
  pub fn take_exited( &mut self ) -> Vec< HostedSession >
  {
    let mut dead = Vec::new();
    for ( session_id, session ) in &mut self.sessions
    {
      if matches!( session.exited(), Ok( Some( _ ) ) )
      {
        dead.push( session_id.clone() );
      }
    }
    dead.sort();

    dead.iter().filter_map( | session_id | self.sessions.remove( session_id ) ).collect()
  }
}
