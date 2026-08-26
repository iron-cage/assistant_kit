//! The daemon's table of hosted sessions.
//!
//! Keyed by conversation id rather than PID. Claude Code's own daemon re-hosts a
//! session with `--fork-session` on auto-update or recovery, producing a process
//! with a different PID, no inherited environment, and a new conversation id — so
//! a PID key detaches silently at exactly the moment recovery is meant to help.
//! The conversation id survives as the handle a client keeps using.

use std::collections::HashMap;
use std::path::PathBuf;

use claude_pty_core::PtySession;

use crate::error::{ Error, Result };
use crate::protocol::SessionSummary;

/// One session hosted by the daemon.
#[ derive( Debug ) ]
pub struct HostedSession
{
  /// Conversation id — the client-facing handle.
  pub session_id : String,
  /// Working directory the session runs in.
  pub cwd : PathBuf,
  /// The PTY-attached child process.
  pub pty : PtySession,
  /// Whether the daemon currently believes a turn is in flight.
  pub busy : bool,
}

impl HostedSession
{
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
  #[ inline ]
  pub fn insert( &mut self, session : HostedSession )
  {
    self.sessions.insert( session.session_id.clone(), session );
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
}
