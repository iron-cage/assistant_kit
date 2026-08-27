//! Turning a request into an answer.
//!
//! [`Daemon`] holds the session table and decides what each [`Request`] means.
//! [`serve_connection`] does the framing around it — one line in, one line out —
//! and [`serve_once`] wires the two together over an accepted connection.
//!
//! # One request per connection
//!
//! Not a limitation to work around. A single-threaded daemon serving persistent
//! connections is a single-*client* daemon: whoever connects first decides when
//! everyone else gets served. Closing after one request bounds a client's hold
//! on the daemon to the request it actually sent.
//!
//! # Nothing here blocks on a turn
//!
//! `send` returns as soon as the text is queued, and carries back the output
//! cursor from immediately before the write. That cursor is exact rather than
//! approximate — the daemon is single-threaded, so no other request can have
//! written to that session in between — and it is what lets a client poll `read`
//! from precisely where its own prompt begins.
//!
//! A `send` that waited for the turn to finish would be easier to call and would
//! freeze every other session for the duration.
//!
//! # When turn state is sampled
//!
//! A session's `busy` flag comes from Claude Code's own registry, by way of
//! [`TurnWatcher`] — and it is refreshed while answering [`Request::ListSessions`],
//! not on a timer.
//!
//! The daemon has no timer to hang it on. It is single-threaded and spends its
//! life blocked in `accept`, so between requests there is nobody to sample and
//! nobody to sample *for*: `busy` is only ever observed through the one request
//! that reports it. Refreshing there means every answer is as fresh as the
//! question, and a client polling for a turn boundary is itself the clock.

use core::time::Duration;
use std::collections::HashMap;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::{ Path, PathBuf };

use claude_pty_core::PtySession;
use claude_session_core::{ BackgroundReporting, TurnEvent, TurnWatcher };
use serde_json::json;

use crate::error::{ Error, Result };
use crate::ipc::read_capped_line;
use crate::listener::Listener;
use crate::protocol::{ Request, Response };
use crate::registration::await_session_id;
use crate::table::{ HostedSession, SessionTable };

/// How long to leave between a prompt's text and the return that submits it.
///
/// See [`Daemon::send`] for what this is separating and why. The value is a
/// compromise: long enough that no reader could plausibly still be treating the
/// two as one burst, short enough to be invisible next to the model call it
/// precedes.
const SUBMIT_GAP : Duration = Duration::from_millis( 200 );

/// The daemon's state, and what it does with a request.
///
/// Generic over how a session gets started. The library does not decide what
/// program a session runs — that belongs to whoever is building a daemon out of
/// this, and keeping it out means the dispatch logic can be exercised against
/// any child rather than only against a real `claude`.
#[ derive( Debug ) ]
pub struct Daemon< S >
{
  sessions_dir : PathBuf,
  sessions : SessionTable,
  spawner : S,
  registration_timeout : Duration,
  stop_requested : bool,
  /// One watcher per hosted session, keyed by conversation id.
  ///
  /// Per session because turn detection is edge-triggered: a watcher has to
  /// remember the status it saw last to recognise a transition, and one shared
  /// watcher fed by several sessions would see their statuses interleaved and
  /// call every one of those a transition.
  watchers : HashMap< String, TurnWatcher >,
  reporting : BackgroundReporting,
}

impl< S > Daemon< S >
where
  S : FnMut( &Path ) -> Result< PtySession >,
{
  /// A daemon with no sessions, reading conversation ids from `sessions_dir`.
  ///
  /// `spawner` starts a session in the working directory it is handed and
  /// returns it before it has registered — which is the only thing it can do,
  /// since the conversation id does not exist yet.
  #[ inline ]
  pub fn new( sessions_dir : impl Into< PathBuf >, spawner : S ) -> Self
  {
    Self
    {
      sessions_dir : sessions_dir.into(),
      sessions : SessionTable::new(),
      spawner,
      registration_timeout : crate::registration::REGISTRATION_TIMEOUT,
      stop_requested : false,
      watchers : HashMap::new(),
      // The conservative default. Only `spawner`'s author knows whether the
      // sessions it starts carry the guarantee, and this crate does not own
      // `spawner`.
      reporting : BackgroundReporting::Unknown,
    }
  }

  /// Override how long a spawned session gets to publish its conversation id.
  #[ inline ]
  #[ must_use ]
  pub fn with_registration_timeout( mut self, timeout : Duration ) -> Self
  {
    self.registration_timeout = timeout;
    self
  }

  /// Declare whether this daemon's `spawner` starts sessions with background-task
  /// reporting enabled.
  ///
  /// Claims a guarantee about a child this crate does not start, so it is the
  /// caller's to make: pass [`BackgroundReporting::Enabled`] only if `spawner`
  /// really does set [`claude_session_core::turn::BG_TASKS_REPORT_RUNNING_ENV`].
  /// Claiming it falsely makes `busy` go false while a background task is still
  /// outstanding, which is precisely the failure the flag exists to describe.
  ///
  /// Defaults to [`BackgroundReporting::Unknown`].
  #[ inline ]
  #[ must_use ]
  pub const fn with_background_reporting( mut self, reporting : BackgroundReporting ) -> Self
  {
    self.reporting = reporting;
    self
  }

  /// The sessions currently hosted.
  #[ inline ]
  pub const fn sessions( &self ) -> &SessionTable
  {
    &self.sessions
  }

  /// Whether a client has asked the daemon to stop.
  ///
  /// Checked by the main loop *after* [`serve_once`] returns, so the answer to
  /// the request that set this has already gone out over the wire. A client
  /// asking the daemon to stop still gets told that it will.
  #[ inline ]
  pub const fn stop_requested( &self ) -> bool
  {
    self.stop_requested
  }

  /// Answer `request`.
  ///
  /// Infallible by construction: every failure becomes a [`Response::err`],
  /// because a client that sent a request is owed an answer either way. The
  /// caller writes whatever comes back and moves on.
  #[ inline ]
  pub fn dispatch( &mut self, request : Request ) -> Response
  {
    match self.try_dispatch( request )
    {
      Ok( result ) => Response::ok( result ),
      Err( error ) => Response::err( error.to_string() ),
    }
  }

  /// End every hosted session, in the order they were named.
  ///
  /// Returns the first failure encountered, having already attempted the rest —
  /// stopping at the first would strand every session after it, which is worse
  /// than a partially reported teardown.
  ///
  /// # Errors
  ///
  /// Returns whichever [`Error`] a session's shutdown reported first.
  #[ inline ]
  pub fn shutdown_all( &mut self ) -> Result< () >
  {
    let mut first : Option< Error > = None;
    for id in self.sessions.session_ids()
    {
      let Ok( mut session ) = self.sessions.remove( &id ) else { continue };
      if let Err( error ) = session.shutdown()
      {
        first.get_or_insert( error );
      }
    }
    first.map_or( Ok( () ), Err )
  }

  /// The fallible half of [`Daemon::dispatch`].
  fn try_dispatch( &mut self, request : Request ) -> Result< serde_json::Value >
  {
    match request
    {
      Request::Ping => Ok( json!( { "version" : env!( "CARGO_PKG_VERSION" ) } ) ),
      Request::ListSessions =>
      {
        self.refresh_turns();
        Ok( json!( self.sessions.summaries() ) )
      },
      Request::Spawn { cwd, prompt } => self.spawn( &cwd, prompt.as_deref() ),
      Request::Send { session_id, text } => self.send( &session_id, &text ),
      Request::Read { session_id, cursor } =>
      {
        Ok( json!( self.sessions.get( &session_id )?.read_from( cursor ) ) )
      },
      Request::Resize { session_id, rows, cols } =>
      {
        self.sessions.get( &session_id )?.resize( rows, cols )?;
        Ok( serde_json::Value::Null )
      },
      Request::Shutdown { session_id } =>
      {
        let status = self.sessions.remove( &session_id )?.shutdown()?;
        Ok( json!( { "exit_code" : status.code() } ) )
      },
      Request::StopDaemon =>
      {
        // The flag only. Sessions are torn down by the caller once it leaves its
        // loop — doing it here would spend the teardown inside a request the
        // client is still waiting on, and a slow session would look like a
        // daemon that never answered.
        self.stop_requested = true;
        Ok( json!( { "stopping" : true } ) )
      },
      // No catch-all arm. `Request` is `#[ non_exhaustive ]` for clients, but
      // this crate defines it — so a variant added later stops the build here
      // instead of silently reaching a default that answers it wrongly.
    }
  }

  /// Bring every hosted session's `busy` flag up to date from the registry.
  ///
  /// One scan for all of them, because they share a directory and a per-session
  /// scan would read the same files over again.
  ///
  /// Failures are silent by design. The registry is written by another program
  /// entirely, and a scan that cannot be read means the daemon does not know
  /// whether anything changed — which is exactly what leaving the last known
  /// state in place says. Turning it into an error would fail a `list_sessions`
  /// that has a perfectly good answer to every other part of the question.
  fn refresh_turns( &mut self )
  {
    let Ok( records ) = claude_session_core::scan( &self.sessions_dir ) else { return };
    let hosted = self.sessions.session_ids();

    // Watchers outlive nothing: a session that is gone can never transition
    // again, and keeping its watcher would resurrect stale `last` state if its
    // conversation id ever came back.
    self.watchers.retain( | id, _ | hosted.contains( id ) );

    let reporting = self.reporting;
    for record in records
    {
      let Ok( session ) = self.sessions.get_mut( &record.session_id ) else { continue };
      let watcher = self.watchers
        .entry( record.session_id.clone() )
        .or_insert_with( || TurnWatcher::new( reporting ) );

      match watcher.observe( &record.status )
      {
        Some( TurnEvent::Started ) => session.set_busy( true ),
        // `SettledUnverified` is treated as settled here, and reported as `busy
        // = false`, because there is nothing else a boolean can say. The
        // distinction is not lost — it is `reporting`, which the caller set and
        // can consult. What must not happen is a session stuck at `busy`
        // forever because the only honest answer was "probably".
        Some( TurnEvent::Settled | TurnEvent::SettledUnverified ) => session.set_busy( false ),
        None => {},
      }
    }
  }

  /// Start a session, wait for it to name itself, and host it.
  fn spawn( &mut self, cwd : &Path, prompt : Option< &str > ) -> Result< serde_json::Value >
  {
    let mut pty = ( self.spawner )( cwd )?;
    let pid = pty.pid();

    // The child is borrowed for the wait and free again after it. Liveness comes
    // from the handle rather than from the registry, because the registry cannot
    // tell "has not registered yet" from "died before it could".
    let registered = await_session_id
    (
      &self.sessions_dir,
      pid,
      self.registration_timeout,
      || matches!( pty.try_wait(), Ok( None ) ),
    );

    let session_id = match registered
    {
      Ok( session_id ) => session_id,
      Err( error ) =>
      {
        end_unregistered( &mut pty );
        return Err( error );
      },
    };

    let session = HostedSession::adopt( session_id.clone(), cwd, pty )?;
    if let Some( mut replaced ) = self.sessions.insert( session )
    {
      // Two live children under one conversation id should not be possible — the
      // id is minted per process. If it happens anyway, the older one is the one
      // nobody can address any more.
      drop( replaced.shutdown() );
    }

    if let Some( text ) = prompt
    {
      self.send( &session_id, text )?;
    }
    Ok( json!( { "session_id" : session_id } ) )
  }

  /// Queue `text` for `session_id`, reporting where its output will start.
  fn send( &mut self, session_id : &str, text : &str ) -> Result< serde_json::Value >
  {
    let session = self.sessions.get( session_id )?;
    let cursor = session.output_end();

    // A carriage return, not a newline: the child is on a terminal in canonical
    // mode, where `Enter` is what submits a line, and `Enter` is `\r`.
    session.write( text.as_bytes() )?;

    // The pause is the whole trick, and it was measured rather than guessed.
    // Without it, prompts under about 55 bytes submitted and everything longer
    // silently did not — the text appeared in the input box and stayed there,
    // with the next prompt landing underneath it on a second line.
    //
    // Both writes land in the pty buffer at once, so a reader that has not been
    // scheduled in between sees one chunk of text-then-return. A terminal
    // application reading a burst that size treats it as pasted input, and a
    // newline inside a paste is a newline, not a submission — which is correct
    // behaviour on its part, and exactly wrong for us. Below the threshold the
    // burst was small enough to be read as typing, which is why the bug looked
    // like it was about length.
    //
    // So the return is sent as its own event, far enough behind the text that
    // no arrival-rate heuristic can attach the two. This blocks the daemon, and
    // deliberately: `send` is already the one request whose caller is waiting on
    // the result, and a fifth of a second buys the difference between a prompt
    // that runs and a prompt that sits in a box.
    std::thread::sleep( SUBMIT_GAP );
    session.write( b"\r" )?;

    Ok( json!( { "cursor" : cursor } ) )
  }
}

/// End a child that was spawned but never became a session.
///
/// Dropping it is not enough and never was: `PtySession` has no `Drop`, and
/// `std::process::Child` deliberately does not kill on drop either. A child left
/// this way is reparented to init and holds its terminal for the life of the
/// machine, addressable by nobody — the daemon's handle to it is what is being
/// discarded.
///
/// `kill` before `shutdown`, which inverts the usual preference. `shutdown`
/// alone is the graceful path — closing the master descriptors hangs the child
/// up and lets it exit through its own shutdown code — but it then *waits*, and
/// this daemon is single-threaded: a child that ignores the hangup would freeze
/// every other session behind it. This one has already failed to do the one
/// thing asked of it within the registration timeout, and having never
/// registered it has no conversation to flush. `shutdown` still follows, to
/// close the descriptors and reap what `kill` left.
///
/// Both results are discarded on purpose. The caller is already returning the
/// error that brought it here, and a failure to clean up after a failure is not
/// a better thing to report than the failure itself.
fn end_unregistered( pty : &mut PtySession )
{
  drop( pty.kill() );
  drop( pty.shutdown() );
}

/// Serve exactly one request from `stream`, then leave it to be closed.
///
/// `handle` turns the parsed request into the response to send back.
///
/// A client that hangs up without sending anything is not an error: nothing is
/// read, nothing is written, and this returns `Ok`. Neither is a request that
/// cannot be parsed — that gets a well-formed error response, which is the whole
/// point of having one. Only a failure to *write* the answer is an error here,
/// since at that point there is nothing left to tell the client.
///
/// # Errors
///
/// Returns [`Error::Io`] if the response cannot be written.
#[ inline ]
pub fn serve_connection< H >( stream : &UnixStream, handle : H ) -> Result< () >
where
  H : FnOnce( Request ) -> Response,
{
  let mut reader = std::io::BufReader::new( stream );
  let response = match read_capped_line( &mut reader )
  {
    Ok( None ) => return Ok( () ),
    Ok( Some( line ) ) => match serde_json::from_str::< Request >( &line )
    {
      Ok( request ) => handle( request ),
      Err( source ) => Response::err( Error::Malformed( source.to_string() ).to_string() ),
    },
    Err( error ) => Response::err( error.to_string() ),
  };

  let mut line = serde_json::to_vec( &response ).map_err( | source |
  {
    Error::Io( std::io::Error::other( source ) )
  } )?;
  line.push( b'\n' );

  let mut writer = stream;
  writer.write_all( &line ).map_err( Error::Io )?;
  writer.flush().map_err( Error::Io )
}

/// Accept one client and serve its request against `daemon`.
///
/// The whole body of a daemon's main loop, minus the loop — which is the
/// caller's, because only the caller knows what should end it.
///
/// # Errors
///
/// Returns [`Error::Io`] if accepting the connection or writing the answer
/// fails. A failure here concerns one client; it is not by itself a reason to
/// stop serving the others.
#[ inline ]
pub fn serve_once< S >( listener : &Listener, daemon : &mut Daemon< S > ) -> Result< () >
where
  S : FnMut( &Path ) -> Result< PtySession >,
{
  let stream = listener.accept()?;
  serve_connection( &stream, | request | daemon.dispatch( request ) )
}
