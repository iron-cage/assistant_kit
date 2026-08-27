//! `clr daemon` — the session daemon's lifecycle, from both sides.
//!
//! Two roles live here. The client side (`clr daemon start|status|stop|log`)
//! talks to the daemon over its socket. The server side is the hidden
//! `__daemon_serve` entry point, which is what a started daemon actually runs.
//!
//! # Why the daemon is this same binary
//!
//! `clr query` already establishes the pattern: `std::env::current_exe()` plus a
//! hidden `__`-prefixed token. The alternative — a separate `claude_daemon`
//! binary — has to be *found*, by `PATH` or by guessing at a sibling of the
//! current executable, and an older copy found that way speaks an older protocol
//! to a newer client. `current_exe()` cannot be the wrong version of itself.
//!
//! # Detachment without unsafe
//!
//! The daemon must outlive the shell that started it. Three things make that so,
//! none of which needs a `setsid` FFI call:
//!
//! - **Its own process group** (`CommandExt::process_group( 0 )`). The terminal
//!   sends `SIGINT`/`SIGQUIT`/`SIGTSTP` to its *foreground* process group; a
//!   daemon in its own group is not in it.
//! - **Not a job of the shell.** `clr daemon start` exits immediately, so the
//!   daemon is reparented to init and is no longer anything the shell will send
//!   `SIGHUP` to on exit.
//! - **No terminal to write to.** stdin is `/dev/null`, stdout and stderr are
//!   appended to the daemon log — so nothing it prints lands in a session that
//!   has moved on, and nothing it reads blocks on a terminal nobody is at.
//!
//! Unsafe stays confined to `claude_pty_core`, which is the crate that owns it.
//!
//! # Stopping is a request, not a signal
//!
//! `SIGTERM` tells the sender nothing: not whether it reached this daemon, not
//! whether the sessions came down cleanly, not whether there was a daemon at
//! all. `clr daemon stop` sends `stop_daemon` over the socket, gets an answer,
//! and then waits for the socket to stop answering.

use core::time::Duration;
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::time::{ Instant, SystemTime, UNIX_EPOCH };

use claude_daemon_core::
{
  acquire, client, BackgroundReporting, Daemon, DaemonPaths, Error, Listener, Request, Result,
  BG_TASKS_REPORT_RUNNING_ENV,
};
use claude_pty_core::{ PtySession, SessionConfig };

/// Hidden token that turns this binary into the daemon.
pub( crate ) const SERVE_TOKEN : &str = "__daemon_serve";

/// How long a probe waits for the daemon to answer before calling it absent.
///
/// Short on purpose. A probe is asking "is anything there?", and a daemon wedged
/// badly enough not to answer in two seconds is not a daemon a client should go
/// on waiting for.
const PROBE_TIMEOUT : Duration = Duration::from_secs( 2 );

/// How long `start` waits for a freshly spawned daemon to answer.
const START_TIMEOUT : Duration = Duration::from_secs( 10 );

/// How long `stop` waits for the socket to go quiet after it is acknowledged.
const STOP_TIMEOUT : Duration = Duration::from_secs( 10 );

/// Gap between polls while waiting for a daemon to appear or disappear.
const POLL : Duration = Duration::from_millis( 50 );

/// How much of the daemon log to show when startup fails.
const LOG_TAIL_LINES : usize = 20;

/// `clr daemon [start|status|stop|log]`.
///
/// Bare `clr daemon` is `status`, which is the question being asked most of the
/// time and the only one with no side effect.
pub( crate ) fn dispatch_daemon( tokens : &[ String ] ) -> !
{
  match tokens.get( 1 ).map_or( "status", String::as_str )
  {
    "status" => cmd_status(),
    "start" => cmd_start(),
    "stop" => cmd_stop(),
    "log" => cmd_log(),
    "help" | "-h" | "--help" => print_daemon_help(),
    other =>
    {
      eprintln!( "Error: unknown daemon subcommand {other:?}" );
      eprintln!( "Run `clr daemon help` for usage." );
      std::process::exit( 1 )
    },
  }
}

/// Usage text for the `daemon` subcommand group.
fn print_daemon_help() -> !
{
  println!( "clr daemon — manage the session daemon" );
  println!();
  println!( "USAGE" );
  println!( "  clr daemon [status]     Report whether the daemon is running" );
  println!( "  clr daemon start        Start it if it is not already running" );
  println!( "  clr daemon stop         Shut every session down and stop it" );
  println!( "  clr daemon log          Print the path of the daemon's log file" );
  println!( "  clr daemon help         Show this help" );
  println!();
  println!( "EXIT CODES" );
  println!( "  status  0 running, 1 not running" );
  println!( "  start   0 running when the command returned, 1 it could not be started" );
  println!( "  stop    0 not running when the command returned, 1 it would not stop" );
  println!();
  println!( "NOTES" );
  println!( "  The daemon survives the shell that starts it: it runs in its own" );
  println!( "  process group, so neither Ctrl-C nor closing the terminal reaches it." );
  println!();
  println!( "  Log timestamps are epoch seconds — `date -d @<n>` reads one back." );
  println!( "  To watch it live:  tail -f \"$( clr daemon log )\"" );
  std::process::exit( 0 )
}

/// Resolve the daemon's locations, or explain why they cannot be resolved.
pub( crate ) fn daemon_paths() -> DaemonPaths
{
  let Some( paths ) = DaemonPaths::new() else
  {
    eprintln!( "Error: cannot resolve the Claude home — HOME is not set" );
    std::process::exit( 1 )
  };
  paths
}

/// Ask the daemon for its version, or `None` if nothing answers.
///
/// This is the liveness test everywhere in this module. A pid file would report
/// a process; this reports a socket that answers, which is the thing a client
/// actually needs to be true.
pub( crate ) fn probe( socket : &Path ) -> Option< String >
{
  let response = client::request_within( socket, &Request::Ping, PROBE_TIMEOUT ).ok()?;
  let claude_daemon_core::Response::Ok { result, .. } = response else { return None };
  Some( result[ "version" ].as_str().unwrap_or( "unknown" ).to_string() )
}

/// `clr daemon status`.
fn cmd_status() -> !
{
  let paths = daemon_paths();
  let socket = paths.socket_file();

  let Some( version ) = probe( &socket ) else
  {
    println!( "daemon   : not running" );
    println!( "socket   : {}", socket.display() );
    std::process::exit( 1 )
  };

  println!( "daemon   : running (version {version})" );
  println!( "socket   : {}", socket.display() );
  println!( "log      : {}", paths.log_file().display() );

  match client::call( &socket, &Request::ListSessions )
  {
    Ok( sessions ) =>
    {
      let listed = sessions.as_array().map_or( 0, Vec::len );
      println!( "sessions : {listed}" );
      for session in sessions.as_array().unwrap_or( &Vec::new() )
      {
        println!
        (
          "  {}  pid {}  {}",
          session[ "session_id" ].as_str().unwrap_or( "?" ),
          session[ "pid" ].as_u64().unwrap_or( 0 ),
          session[ "cwd" ].as_str().unwrap_or( "?" ),
        );
      }
    },
    // It answered the ping and then failed to list. Worth saying out loud rather
    // than printing a zero that reads as "no sessions".
    Err( error ) => println!( "sessions : unavailable ({error})" ),
  }

  std::process::exit( 0 )
}

/// Which way a daemon came to be answering.
enum Started
{
  /// This call started it.
  Fresh( String ),
  /// Something else got there first. Distinguished from [`Started::Fresh`] only
  /// so the message is honest — either way, a daemon is running.
  Adopted( String ),
}

/// Spawn a daemon and wait for it to answer.
///
/// The waiting is the substance: a spawn that returned an alive `Child` proves a
/// process exists, not that a socket answers, and the socket is the thing every
/// caller after this actually needs.
fn start_and_wait( paths : &DaemonPaths ) -> core::result::Result< Started, String >
{
  let socket = paths.socket_file();
  let mut child = spawn_daemon( paths )
    .map_err( | error | format!( "cannot start the daemon: {error}" ) )?;

  let started = Instant::now();
  while started.elapsed() < START_TIMEOUT
  {
    if let Some( version ) = probe( &socket )
    {
      return Ok( Started::Fresh( version ) );
    }

    // A daemon that has already exited is never going to answer. Losing the
    // race to another starter looks exactly like this, which is why the probe
    // above is retried once before giving up.
    if matches!( child.try_wait(), Ok( Some( _ ) ) )
    {
      return probe( &socket )
        .map( Started::Adopted )
        .ok_or_else( || "the daemon exited during startup".to_string() );
    }

    std::thread::sleep( POLL );
  }

  Err( "the daemon did not answer in time".to_string() )
}

/// Make sure a daemon is answering, starting one if none is.
///
/// What a client command calls before its first request. Auto-starting is right
/// here and wrong for `clr daemon start`: a client asked for something a daemon
/// provides, and the daemon is an implementation detail of providing it.
///
/// # Errors
///
/// Returns the same reason `clr daemon start` would have printed.
pub( crate ) fn ensure_running( paths : &DaemonPaths ) -> core::result::Result< (), String >
{
  if probe( &paths.socket_file() ).is_some()
  {
    return Ok( () );
  }
  start_and_wait( paths ).map( | _ | () )
}

/// `clr daemon start`.
fn cmd_start() -> !
{
  let paths = daemon_paths();
  let socket = paths.socket_file();

  if let Some( version ) = probe( &socket )
  {
    println!( "daemon already running (version {version})" );
    std::process::exit( 0 )
  }

  match start_and_wait( &paths )
  {
    Ok( Started::Fresh( version ) ) =>
    {
      println!( "daemon started (version {version})" );
      println!( "socket : {}", socket.display() );
      std::process::exit( 0 )
    },
    Ok( Started::Adopted( version ) ) =>
    {
      println!( "daemon already running (version {version})" );
      std::process::exit( 0 )
    },
    Err( reason ) => report_failed_start( &paths, &reason ),
  }
}

/// Explain a failed start, with whatever the daemon managed to write down.
fn report_failed_start( paths : &DaemonPaths, reason : &str ) -> !
{
  eprintln!( "Error: {reason}" );
  let log = paths.log_file();
  match std::fs::read_to_string( &log )
  {
    Ok( text ) =>
    {
      eprintln!( "--- {} (last {LOG_TAIL_LINES} lines) ---", log.display() );
      let lines : Vec< &str > = text.lines().collect();
      for line in lines.iter().rev().take( LOG_TAIL_LINES ).rev()
      {
        eprintln!( "{line}" );
      }
    },
    Err( error ) => eprintln!( "(the log at {} could not be read: {error})", log.display() ),
  }
  std::process::exit( 1 )
}

/// Launch the detached daemon process. See the module docs on detachment.
fn spawn_daemon( paths : &DaemonPaths ) -> std::io::Result< std::process::Child >
{
  std::fs::create_dir_all( paths.runtime_dir() )?;

  // Appended, never truncated: the interesting case is a daemon that keeps
  // dying at startup, and truncating erases the evidence on every restart.
  let log = std::fs::OpenOptions::new()
    .create( true )
    .append( true )
    .open( paths.log_file() )?;

  let mut command = std::process::Command::new( std::env::current_exe()? );
  command.arg( SERVE_TOKEN );
  command.stdin( std::process::Stdio::null() );
  command.stdout( std::process::Stdio::from( log.try_clone()? ) );
  command.stderr( std::process::Stdio::from( log ) );
  command.process_group( 0 );

  command.spawn()
}

/// `clr daemon stop`.
fn cmd_stop() -> !
{
  let paths = daemon_paths();
  let socket = paths.socket_file();

  if probe( &socket ).is_none()
  {
    println!( "daemon not running" );
    std::process::exit( 0 )
  }

  if let Err( error ) = client::call( &socket, &Request::StopDaemon )
  {
    eprintln!( "Error: the daemon refused to stop: {error}" );
    std::process::exit( 1 )
  }

  // Acknowledged is not stopped. The daemon answers first and tears its sessions
  // down afterwards, so this waits for the socket to actually go quiet — a
  // command that returned while children were still dying would be lying.
  let started = Instant::now();
  while started.elapsed() < STOP_TIMEOUT
  {
    if probe( &socket ).is_none()
    {
      println!( "daemon stopped" );
      std::process::exit( 0 )
    }
    std::thread::sleep( POLL );
  }

  eprintln!( "Error: the daemon acknowledged the stop but is still answering" );
  eprintln!( "Its log is at {}", paths.log_file().display() );
  std::process::exit( 1 )
}

/// `clr daemon log`.
///
/// The path and nothing else, so it composes: `tail -f "$( clr daemon log )"`.
fn cmd_log() -> !
{
  println!( "{}", daemon_paths().log_file().display() );
  std::process::exit( 0 )
}

/// Hidden entry point (`clr __daemon_serve`) — this process *is* the daemon.
///
/// Everything it prints goes to the log its parent opened for it, so each line
/// carries an epoch-second timestamp; `date -d @<n>` reads one back.
pub( crate ) fn run_daemon_serve() -> !
{
  let paths = daemon_paths();

  if let Err( error ) = std::fs::create_dir_all( paths.runtime_dir() )
  {
    log_line( &format!( "cannot create {}: {error}", paths.runtime_dir().display() ) );
    std::process::exit( 1 )
  }

  let lock = match acquire( &paths.lock_file() )
  {
    Ok( lock ) => lock,
    // Somebody else won the race. Whoever started this one wanted a daemon
    // running, and there is one — so this is a success, quietly.
    Err( Error::AlreadyRunning { .. } ) =>
    {
      log_line( "another daemon already holds the instance lock; exiting" );
      std::process::exit( 0 )
    },
    Err( error ) =>
    {
      log_line( &format!( "cannot take the instance lock: {error}" ) );
      std::process::exit( 1 )
    },
  };

  let socket = paths.socket_file();
  let listener = match Listener::bind( &socket, &lock )
  {
    Ok( listener ) => listener,
    Err( error ) =>
    {
      log_line( &format!( "cannot bind {}: {error}", socket.display() ) );
      std::process::exit( 1 )
    },
  };

  // `Enabled` because `spawn_claude` sets the variable that earns it. The claim
  // and the thing that makes it true are both in this file, on purpose.
  let mut daemon = Daemon::new( paths.sessions_dir(), spawn_claude )
    .with_background_reporting( BackgroundReporting::Enabled )
    // Read-only, and empty until something has measured a baseline into it — a
    // context summary reports the overhead split as null until then.
    .with_baselines( paths.runtime_dir() );
  log_line( &format!( "listening on {} (pid {})", socket.display(), std::process::id() ) );

  loop
  {
    // One client's connection failing concerns that client. Ending the loop over
    // it would take every other session down with it.
    if let Err( error ) = claude_daemon_core::serve_once( &listener, &mut daemon )
    {
      log_line( &format!( "connection error: {error}" ) );
    }
    if daemon.stop_requested()
    {
      break;
    }
  }

  log_line( "stopping" );
  if let Err( error ) = daemon.shutdown_all()
  {
    log_line( &format!( "shutting sessions down: {error}" ) );
  }

  // Explicit, and in this order: the socket goes before the lock, so there is no
  // moment where the lock is free while a socket is still there to connect to.
  drop( listener );
  drop( lock );

  log_line( "stopped" );
  std::process::exit( 0 )
}

/// Start an interactive `claude` on a terminal of its own.
///
/// Deliberately not routed through `claude_runner_core::build_command()`, which
/// owns the single execution point for `Command::new( "claude" )`. That builds a
/// print-mode invocation over pipes and has no way to express a PTY — a session
/// here needs a terminal, because Claude Code's interactive REPL only exists on
/// one. Different mechanism, different knowledge, so this is not that constant
/// duplicated; the flag surface still belongs to `build_command()`.
///
/// The environment variable is what makes an observed `idle` mean "the turn is
/// over". Without it a session waiting on a background task reports `idle` too,
/// and there is no way to tell the two apart after the fact — so it is set here,
/// at the only place that starts one, and declared to [`Daemon`] below.
fn spawn_claude( cwd : &Path ) -> Result< PtySession >
{
  let config = SessionConfig::new( "claude" )
    .cwd( cwd )
    .env( BG_TASKS_REPORT_RUNNING_ENV, "1" );
  PtySession::spawn( &config ).map_err( Error::Pty )
}

/// Write one timestamped line to the daemon's log.
///
/// Epoch seconds rather than a formatted date: it needs no dependency, sorts
/// correctly, and `date -d @<n>` turns it back into something readable.
fn log_line( message : &str )
{
  let stamp = SystemTime::now()
    .duration_since( UNIX_EPOCH )
    .map_or( 0, | since | since.as_secs() );
  let mut err = std::io::stderr();
  drop( writeln!( err, "[{stamp}] {message}" ) );
  drop( err.flush() );
}
