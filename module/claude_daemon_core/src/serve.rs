//! Turning a request into an answer.
//!
//! [`Daemon`] holds the session table and decides what each [`Request`] means.
//! [`daemon_kit::serve_connection`] does the framing around it — one line in,
//! one line out — and [`serve_once`] wires the two together over an accepted
//! connection.
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
//! [`TurnWatcher`]. It is refreshed eagerly while answering
//! [`Request::ListSessions`], so that request's own answer is never a tick
//! stale — and, per `docs/feature/010_session_reaping.md`, on every
//! [`Daemon::reap`] as well, which the main loop drives after *every*
//! connection, real or not.
//!
//! The daemon is single-threaded and spends its life blocked in `accept`, so
//! between requests there is nobody to sample unless something manufactures a
//! connection to sample on. [`daemon_kit::spawn_waker`] is that something: a
//! synthetic client that sleeps for one tick, connects, and hangs up, purely so
//! `accept` returns and [`Daemon::reap`] gets to run even when no real client
//! has anything to ask.
//!
//! # Idle and linger
//!
//! [`Daemon::reap`] releases at most one session per tick that is both not
//! busy and untouched for `idle_timeout`, and [`Daemon::should_exit`] answers
//! `true` once the table has stayed continuously empty for `linger`. Full
//! policy, defaults, and the reasoning behind both bounds:
//! `docs/feature/010_session_reaping.md`.

use core::time::Duration;
use std::collections::HashMap;
use std::path::{ Path, PathBuf };
use std::time::Instant;

use claude_pty_core::PtySession;
use claude_session_core::{ BackgroundReporting, TurnEvent, TurnWatcher };
use child_supervisor::{ HostedSession, SessionTable };
use daemon_kit::Listener;
use serde_json::json;

use crate::error::{ Error, Result };
use crate::protocol::{ Request, Response, SessionSummary };
use crate::registration::await_session_id;

/// How long to leave between a prompt's text and the return that submits it.
///
/// See [`Daemon::send`] for what this is separating and why. The value is a
/// compromise: long enough that no reader could plausibly still be treating the
/// two as one burst, short enough to be invisible next to the model call it
/// precedes.
const SUBMIT_GAP : Duration = Duration::from_millis( 200 );

/// How long a session may go untouched and not busy before [`Daemon::reap`]
/// releases it. `0` means never — see `docs/feature/010_session_reaping.md`.
pub const DEFAULT_IDLE_TIMEOUT : Duration = Duration::from_secs( 30 * 60 );

/// How long the table may stay continuously empty before [`Daemon::should_exit`]
/// answers `true`. `0` means never — see `docs/feature/010_session_reaping.md`.
pub const DEFAULT_LINGER : Duration = Duration::from_secs( 5 * 60 );

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
  idle_timeout : Duration,
  linger : Duration,
  /// When the table last became continuously empty, if it is empty now.
  ///
  /// `None` while the table is non-empty. Stamped by [`Daemon::reap`] the
  /// first tick it finds the table empty with no stamp already in place, and
  /// cleared by [`Daemon::spawn`] — so it measures *continuously* empty, per
  /// `docs/feature/010_session_reaping.md`, never cumulatively.
  empty_since : Option< Instant >,
  stop_requested : bool,
  /// One watcher per hosted session, keyed by conversation id.
  ///
  /// Per session because turn detection is edge-triggered: a watcher has to
  /// remember the status it saw last to recognise a transition, and one shared
  /// watcher fed by several sessions would see their statuses interleaved and
  /// call every one of those a transition.
  watchers : HashMap< String, TurnWatcher >,
  reporting : BackgroundReporting,
  /// Where to look for a cached static baseline, if anywhere.
  ///
  /// Read from while answering [`Request::ContextSummary`] and never written to.
  /// Taking a measurement is deliberately not something this daemon does — see
  /// [`Daemon::with_baselines`].
  baselines : Option< PathBuf >,
}

impl< S > Daemon< S >
where
  S : FnMut( &Path, Option< &str > ) -> Result< PtySession >,
{
  /// A daemon with no sessions, reading conversation ids from `sessions_dir`.
  ///
  /// `spawner` starts a session in the working directory it is handed, resuming
  /// the conversation id given as its second argument when `Some` rather than
  /// starting fresh, and returns before the session has registered — which is
  /// the only thing it can do, since a fresh conversation's id does not exist
  /// yet (a resumed one already reuses the id it was given — see
  /// [`Daemon::spawn`]).
  #[ inline ]
  pub fn new( sessions_dir : impl Into< PathBuf >, spawner : S ) -> Self
  {
    Self
    {
      sessions_dir : sessions_dir.into(),
      sessions : SessionTable::new(),
      spawner,
      registration_timeout : crate::registration::REGISTRATION_TIMEOUT,
      idle_timeout : DEFAULT_IDLE_TIMEOUT,
      linger : DEFAULT_LINGER,
      // Empty from the moment it exists, same as a table that has just been
      // emptied out — a daemon that never hosts anything should still be
      // reachable by the linger clock rather than exempt from it forever.
      empty_since : Some( Instant::now() ),
      stop_requested : false,
      watchers : HashMap::new(),
      // The conservative default. Only `spawner`'s author knows whether the
      // sessions it starts carry the guarantee, and this crate does not own
      // `spawner`.
      reporting : BackgroundReporting::Unknown,
      baselines : None,
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

  /// Override how long a session may go untouched and not busy before
  /// [`Daemon::reap`] releases it. `0` disables idle reaping entirely.
  /// Defaults to [`DEFAULT_IDLE_TIMEOUT`]. See
  /// `docs/feature/010_session_reaping.md`.
  #[ inline ]
  #[ must_use ]
  pub fn with_idle_timeout( mut self, timeout : Duration ) -> Self
  {
    self.idle_timeout = timeout;
    self
  }

  /// Override how long the table may stay continuously empty before
  /// [`Daemon::should_exit`] answers `true`. `0` disables it entirely.
  /// Defaults to [`DEFAULT_LINGER`]. See `docs/feature/010_session_reaping.md`.
  #[ inline ]
  #[ must_use ]
  pub fn with_linger( mut self, linger : Duration ) -> Self
  {
    self.linger = linger;
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

  /// Read cached static baselines from `dir` when summarizing context.
  ///
  /// Lets [`Request::ContextSummary`] divide a session's context into fixed
  /// overhead and actual conversation. Without it — or with no measurement on
  /// file for the session's version and model — that split is reported as
  /// `null`, and nothing else in the summary changes.
  ///
  /// # Why the daemon reads these but never takes them
  ///
  /// A measurement is one `--print` call to the API: seconds of latency, and it
  /// spends the user's tokens. This daemon is single-threaded and serves one
  /// request at a time, so measuring here would freeze every other session for
  /// the length of a network round trip — the same reason [`Daemon::send`] does
  /// not wait for a turn to finish.
  ///
  /// So measuring belongs to whoever knows where `claude` is, using
  /// [`crate::baseline::measure`] and [`crate::baseline::store`] directly. The
  /// daemon's half is the cheap half: a local file read on a request that is
  /// already reading files.
  #[ inline ]
  #[ must_use ]
  pub fn with_baselines( mut self, dir : impl Into< PathBuf > ) -> Self
  {
    self.baselines = Some( dir.into() );
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

  /// Give idle-detection a chance to run, whether or not the connection that just
  /// woke the main loop asked for anything.
  ///
  /// Called from the main loop next to [`Daemon::stop_requested`] and
  /// [`Daemon::should_exit`], immediately after [`serve_once`] returns — so
  /// *any* connection drives it, including the synthetic one [`spawn_waker`]
  /// makes on a schedule when no real client has anything to ask. See
  /// `docs/feature/010_session_reaping.md`.
  ///
  /// Refreshes turn state and drops sessions whose child has already exited
  /// (both inside [`Daemon::refresh_turns`]), then releases at most one
  /// session idle long enough to reap.
  #[ inline ]
  pub fn reap( &mut self )
  {
    self.refresh_turns();
    self.reap_idle();
  }

  /// Whether the table has been continuously empty for at least `linger`.
  ///
  /// Checked by the main loop next to [`Daemon::stop_requested`] — both break
  /// the same loop into the same shutdown tail, so an idle exit unlinks the
  /// socket before dropping the lock exactly as a requested one does. See
  /// `docs/feature/010_session_reaping.md`.
  #[ inline ]
  #[ must_use ]
  pub fn should_exit( &self ) -> bool
  {
    if self.linger.is_zero()
    {
      return false;
    }
    self.empty_since.is_some_and( | since | since.elapsed() >= self.linger )
  }

  /// Release at most one session nobody has touched for `idle_timeout` and
  /// that is not currently busy, chosen deterministically by conversation id
  /// when more than one qualifies.
  ///
  /// At most one, never a whole backlog at once: [`HostedSession::shutdown`]
  /// can block for up to `SHUTDOWN_GRACE`, and the daemon is single-threaded,
  /// so releasing several in one tick is that many multiples of
  /// `SHUTDOWN_GRACE` during which no other client is served — from outside
  /// that reads as a hung daemon. `tick` is far shorter than `idle_timeout` by
  /// design, so a backlog drains long before anyone notices it existed. See
  /// `docs/feature/010_session_reaping.md`.
  fn reap_idle( &mut self )
  {
    if self.idle_timeout.is_zero()
    {
      return;
    }

    let idle = self.sessions.session_ids().into_iter().find( | id |
    {
      self.sessions.get( id ).is_ok_and( | session |
        !session.busy() && session.last_active().elapsed() >= self.idle_timeout )
    } );

    if let Some( id ) = idle
    {
      if let Ok( mut session ) = self.sessions.remove( &id )
      {
        drop( session.shutdown() );
      }
    }

    self.note_emptied();
  }

  /// Stamp [`Daemon::empty_since`] the instant the table has nothing left, so
  /// the linger clock starts exactly when a removal causes it rather than
  /// waiting for some later tick to notice. A no-op while the table already
  /// carries a stamp or still hosts something.
  fn note_emptied( &mut self )
  {
    if self.sessions.is_empty() && self.empty_since.is_none()
    {
      self.empty_since = Some( Instant::now() );
    }
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
        first.get_or_insert( error.into() );
      }
    }
    first.map_or( Ok( () ), Err )
  }

  /// Snapshot every hosted session as a [`SessionSummary`].
  ///
  /// `child_supervisor::SessionTable` has no notion of [`SessionSummary`] — that
  /// DTO is this crate's wire shape, not the generic table's — so this crate
  /// builds it from the table's own id list and per-session accessors rather
  /// than from a `summaries()` method the table does not have.
  fn summaries( &self ) -> Vec< SessionSummary >
  {
    self.sessions.session_ids().into_iter().filter_map( | session_id |
    {
      let session = self.sessions.get( &session_id ).ok()?;
      Some( SessionSummary
      {
        session_id : session.session_id().to_string(),
        pid : session.pid(),
        cwd : session.cwd().to_path_buf(),
        busy : session.busy(),
      } )
    } ).collect()
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
        Ok( json!( self.summaries() ) )
      },
      Request::Spawn { cwd, prompt } => self.spawn( &cwd, prompt.as_deref() ),
      Request::Send { session_id, text } => self.send( &session_id, &text ),
      Request::Read { session_id, cursor } =>
      {
        Ok( json!( self.sessions.get_mut( &session_id )?.read_from( cursor ) ) )
      },
      Request::ContextSummary { session_id } =>
      {
        // Resolved through the table so an unknown id is reported as such, and
        // so the cwd comes from the daemon's own record rather than the client.
        let cwd = self.sessions.get( &session_id )?.cwd().to_path_buf();
        crate::context::summary( &cwd, &session_id, self.baselines.as_deref() )
      },
      Request::Resize { session_id, rows, cols } =>
      {
        self.sessions.get_mut( &session_id )?.resize( rows, cols )?;
        Ok( serde_json::Value::Null )
      },
      Request::Shutdown { session_id } =>
      {
        let status = self.sessions.remove( &session_id )?.shutdown()?;
        self.note_emptied();
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

  /// Bring every hosted session's `busy` flag up to date from the registry, and
  /// drop any whose child has already exited.
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

      // Every tick this session is *still* busy, not only the tick it became
      // busy — a forty-minute autonomous turn has no `send`/`read`/`resize` of
      // its own for the whole forty minutes, and would otherwise look idle the
      // instant it settles rather than getting the full `idle_timeout` from
      // that point. See `docs/feature/010_session_reaping.md`.
      if session.busy()
      {
        session.touch();
      }
    }

    // A session whose child died is dead weight nobody else notices: it keeps
    // its row, its pid, its pump thread, and answers every `send` with an
    // error. Collected here rather than only from `Daemon::reap` because this
    // method also runs eagerly on every `list_sessions` — a client asking right
    // now shouldn't have to wait for the next tick to see a dead session drop
    // off. `reap` covers the gap between requests; this covers the request
    // itself. See `docs/feature/010_session_reaping.md`. Shutdown failures are
    // dropped for the same reason a scan failure is: there is no request in
    // flight to report them to, and the session is leaving the table either way.
    for mut session in self.sessions.take_exited()
    {
      drop( session.shutdown() );
    }
    self.note_emptied();
  }

  /// Start a session, wait for it to name itself, and host it.
  ///
  /// Resumes the conversation that last occupied `cwd`, if any, rather than
  /// always starting fresh. Resolved from disk on every call — via
  /// `claude_storage_core`'s own continuation-detection primitive, the same one
  /// an interactive `claude -c` agrees with — never from an in-memory map: a map
  /// would die with the daemon and reintroduce, in the exact window
  /// `docs/feature/010_session_reaping.md`'s idle exit creates, the silent
  /// new-conversation failure this feature exists to prevent. When a directory
  /// has hosted several conversations, the most-recently-modified transcript
  /// wins unconditionally — settled, not a per-daemon preference, so a
  /// directory's "current" conversation means the same thing whether the last
  /// session in it was hosted or interactive.
  fn spawn( &mut self, cwd : &Path, prompt : Option< &str > ) -> Result< serde_json::Value >
  {
    let resume = claude_storage_core::most_recent_session_id( cwd );
    let mut pty = ( self.spawner )( cwd, resume.as_ref().map( claude_storage_core::SessionId::as_str ) )?;
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
    // Any spawn clears it, per `docs/feature/010_session_reaping.md` — the
    // linger clock measures continuously empty, and this table just stopped
    // being that.
    self.empty_since = None;

    if let Some( text ) = prompt
    {
      self.send( &session_id, text )?;
    }
    Ok( json!( { "session_id" : session_id } ) )
  }

  /// Queue `text` for `session_id`, reporting where its output will start.
  fn send( &mut self, session_id : &str, text : &str ) -> Result< serde_json::Value >
  {
    let session = self.sessions.get_mut( session_id )?;
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

/// Accept one client and serve its request against `daemon`.
///
/// The whole body of a daemon's main loop, minus the loop — which is the
/// caller's, because only the caller knows what should end it.
///
/// Thin wrapper over [`daemon_kit::serve_once`]: the framing and dispatch loop
/// are generic over the request type and live there; this crate supplies
/// [`Request`] and [`Daemon::dispatch`].
///
/// # Errors
///
/// Returns [`Error::Io`] if accepting the connection or writing the answer
/// fails. A failure here concerns one client; it is not by itself a reason to
/// stop serving the others.
#[ inline ]
pub fn serve_once< S >( listener : &Listener, daemon : &mut Daemon< S > ) -> Result< () >
where
  S : FnMut( &Path, Option< &str > ) -> Result< PtySession >,
{
  daemon_kit::serve_once( listener, | request | daemon.dispatch( request ) ).map_err( Into::into )
}
