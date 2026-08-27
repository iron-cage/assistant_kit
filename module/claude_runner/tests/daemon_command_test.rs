//! Integration tests for `clr daemon` — the session daemon's lifecycle.
//!
//! Test spec: [`tests/docs/cli/command/13_daemon.md`](docs/cli/command/13_daemon.md).
//!
//! Every test runs against its own `HOME` in a temporary directory, so a daemon
//! started here can never collide with the one a developer has running, and two
//! tests can never contend for the same instance lock.
//!
//! No `claude` binary is needed anywhere in this file: `start`, `status`, `stop`
//! and `log` never spawn a session. Sessions are `claude_daemon_core`'s own test
//! surface (`serve_test.rs`), against real PTY-attached children.
//!
//! # Test Case Index
//!
//! | ID    | Name                                                       | Category      |
//! |-------|------------------------------------------------------------|---------------|
//! | IT-1  | `clr daemon help` documents all four subcommands            | Documentation |
//! | IT-2  | `clr daemon --help` and `-h` match the positional form      | Documentation |
//! | IT-3  | `clr daemon nonsense` → exit 1, names the token             | Validation    |
//! | IT-4  | `clr daemon status` with nothing running → exit 1           | Absent daemon |
//! | IT-5  | bare `clr daemon` is `status`                               | Defaulting    |
//! | IT-6  | `clr daemon log` prints the path under the runtime dir      | Path contract |
//! | IT-7  | `clr help` lists `daemon`                                   | Help listing  |
//! | IT-8  | `clr daemn` (typo) → exit 1, Did you mean                    | Typo guard    |
//! | IT-9  | start → status → start → stop → status → stop               | Lifecycle     |
//! | IT-10 | the started daemon is in a process group of its own         | Detachment    |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ DaemonGuard, exit_code, run_cli, run_cli_with_env, stderr_str, stdout_str };

/// Run `clr daemon <args>` against an isolated `HOME`.
fn daemon_in( home : &std::path::Path, args : &[ &str ] ) -> std::process::Output
{
  let home = home.to_str().expect( "home path is not UTF-8" );
  let mut full = vec![ "daemon" ];
  full.extend_from_slice( args );
  run_cli_with_env( &full, &[ ( "HOME", home ) ] )
}

// ── IT-1, IT-2: help ──────────────────────────────────────────────────────────

/// IT-1: `clr daemon help` documents every subcommand it accepts.
#[ test ]
fn it_01_help_documents_all_subcommands()
{
  let out = run_cli( &[ "daemon", "help" ] );
  let stdout = stdout_str( &out );

  assert_eq!( exit_code( &out ), 0, "help must succeed, stderr: {}", stderr_str( &out ) );
  // `[status]` bracketed, because bare `clr daemon` is it.
  for usage in [ "clr daemon [status]", "clr daemon start", "clr daemon stop", "clr daemon log" ]
  {
    assert!( stdout.contains( usage ), "help omits {usage:?}, got: {stdout}" );
  }
}

/// IT-2: the flag spellings of help agree with the positional one.
///
/// Three spellings that disagree is three chances to document a subcommand in
/// one of them and not the others.
#[ test ]
fn it_02_help_flag_forms_match_the_positional_form()
{
  let positional = stdout_str( &run_cli( &[ "daemon", "help" ] ) );

  for flag in [ "--help", "-h" ]
  {
    let out = run_cli( &[ "daemon", flag ] );
    assert_eq!( exit_code( &out ), 0, "`clr daemon {flag}` must succeed" );
    assert_eq!( stdout_str( &out ), positional, "`clr daemon {flag}` printed something else" );
  }
}

// ── IT-3: unknown subcommand ──────────────────────────────────────────────────

/// IT-3: an unknown daemon subcommand fails, and says which one it was.
#[ test ]
fn it_03_unknown_subcommand_is_rejected()
{
  let out = run_cli( &[ "daemon", "restart" ] );
  let stderr = stderr_str( &out );

  assert_eq!( exit_code( &out ), 1, "an unknown subcommand must exit 1" );
  assert!( stderr.contains( "restart" ), "the rejected token is not named: {stderr}" );
  assert!( stderr.contains( "clr daemon help" ), "no pointer to help: {stderr}" );
}

// ── IT-4, IT-5: status with nothing running ───────────────────────────────────

/// IT-4: `status` against an untouched home reports no daemon, and exits 1.
///
/// The exit code is the part scripts read — `clr daemon status || clr daemon start`
/// only works because "not running" is a failure and not a 0 with a message.
#[ test ]
fn it_04_status_without_a_daemon_exits_one()
{
  let home = tempfile::TempDir::new().expect( "create isolated home" );
  let out = daemon_in( home.path(), &[ "status" ] );
  let stdout = stdout_str( &out );

  assert_eq!( exit_code( &out ), 1, "no daemon must be a failure, stdout: {stdout}" );
  assert!( stdout.contains( "not running" ), "unexpected report: {stdout}" );
}

/// IT-5: bare `clr daemon` is `clr daemon status`.
#[ test ]
fn it_05_bare_daemon_is_status()
{
  let home = tempfile::TempDir::new().expect( "create isolated home" );

  let bare = daemon_in( home.path(), &[] );
  let explicit = daemon_in( home.path(), &[ "status" ] );

  assert_eq!( exit_code( &bare ), exit_code( &explicit ), "exit codes differ" );
  assert_eq!( stdout_str( &bare ), stdout_str( &explicit ), "output differs" );
}

// ── IT-6: log path ────────────────────────────────────────────────────────────

/// IT-6: `log` prints the path and nothing else, so it composes.
///
/// `tail -f "$( clr daemon log )"` is the intended use, and it only works if the
/// whole of stdout is the path — a heading or a trailing note would break it.
#[ test ]
fn it_06_log_prints_only_the_path()
{
  let home = tempfile::TempDir::new().expect( "create isolated home" );
  let out = daemon_in( home.path(), &[ "log" ] );
  let stdout = stdout_str( &out );

  assert_eq!( exit_code( &out ), 0, "log must succeed, stderr: {}", stderr_str( &out ) );

  let printed = std::path::PathBuf::from( stdout.trim() );
  let expected = home.path().join( ".claude" ).join( "-daemon" ).join( "daemon.log" );
  assert_eq!( printed, expected, "unexpected log path" );
  assert_eq!( stdout.lines().count(), 1, "stdout is not a bare path: {stdout}" );
}

// ── IT-7, IT-8: discoverability ───────────────────────────────────────────────

/// IT-7: `clr help` lists the daemon command.
#[ test ]
fn it_07_top_level_help_lists_daemon()
{
  let stdout = stdout_str( &run_cli( &[ "help" ] ) );

  assert!
  (
    stdout.contains( "clr daemon [start | status | stop | log]" ),
    "the usage line is missing: {stdout}"
  );
}

/// IT-8: a typo is caught by the known-subcommand guard rather than being run.
///
/// Without the guard `clr daemn "..."` would be a `run` with a stray positional,
/// which starts a real session instead of reporting a typo.
#[ test ]
fn it_08_typo_is_caught_by_the_subcommand_guard()
{
  let out = run_cli( &[ "daemn" ] );
  let stderr = stderr_str( &out );

  assert_eq!( exit_code( &out ), 1, "a typo must exit 1" );
  assert!( stderr.contains( "Did you mean" ), "no suggestion offered: {stderr}" );
  assert!( stderr.contains( "daemon" ), "the suggestion is not `daemon`: {stderr}" );
}

// ── IT-9, IT-10: the real lifecycle ───────────────────────────────────────────

/// IT-9: start, status, start again, stop, status again, stop again.
///
/// One test rather than six, because each one would otherwise pay for its own
/// daemon — and the sequence is the contract: `start` and `stop` are idempotent,
/// and `status` agrees with whichever ran last.
#[ test ]
fn it_09_lifecycle_start_status_stop()
{
  let home = tempfile::TempDir::new().expect( "create isolated home" );
  let _guard = DaemonGuard::for_home( home.path() );

  let started = daemon_in( home.path(), &[ "start" ] );
  assert_eq!
  (
    exit_code( &started ), 0,
    "start failed: {} / {}", stdout_str( &started ), stderr_str( &started )
  );
  assert!( stdout_str( &started ).contains( "daemon started" ), "{}", stdout_str( &started ) );

  let running = daemon_in( home.path(), &[ "status" ] );
  assert_eq!( exit_code( &running ), 0, "status disagrees with start" );
  let report = stdout_str( &running );
  assert!( report.contains( "daemon   : running" ), "unexpected report: {report}" );
  assert!( report.contains( "sessions : 0" ), "a fresh daemon hosts no sessions: {report}" );

  // Starting a second time is a no-op that succeeds, not a lock error: the caller
  // asked for a daemon to be running, and one is.
  let again = daemon_in( home.path(), &[ "start" ] );
  assert_eq!( exit_code( &again ), 0, "the second start must succeed" );
  assert!( stdout_str( &again ).contains( "already running" ), "{}", stdout_str( &again ) );

  let stopped = daemon_in( home.path(), &[ "stop" ] );
  assert_eq!( exit_code( &stopped ), 0, "stop failed: {}", stderr_str( &stopped ) );
  assert!( stdout_str( &stopped ).contains( "daemon stopped" ), "{}", stdout_str( &stopped ) );

  let gone = daemon_in( home.path(), &[ "status" ] );
  assert_eq!( exit_code( &gone ), 1, "status still reports a daemon after stop" );

  // Stopping nothing is a success too, for the same reason: the requested state
  // is the state.
  let stop_again = daemon_in( home.path(), &[ "stop" ] );
  assert_eq!( exit_code( &stop_again ), 0, "the second stop must succeed" );
  assert!( stdout_str( &stop_again ).contains( "not running" ), "{}", stdout_str( &stop_again ) );

  // The socket goes; the log stays. A daemon that keeps dying at startup is only
  // debuggable if stopping does not erase what it wrote.
  let runtime = home.path().join( ".claude" ).join( "-daemon" );
  assert!( !runtime.join( "daemon.sock" ).exists(), "the socket outlived the daemon" );
  assert!( runtime.join( "daemon.log" ).exists(), "the log was removed" );
}

/// IT-10: the daemon runs in a process group of its own.
///
/// This is the property that makes Ctrl-C in the starting shell not reach it: a
/// terminal signals its *foreground process group*, and the daemon is not in it.
/// Read from `/proc/<pid>/stat` rather than inferred, because
/// `CommandExt::process_group( 0 )` is the whole of the mechanism and a silent
/// regression there looks exactly like a working daemon until the day a terminal
/// closes.
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn it_10_the_daemon_has_its_own_process_group()
{
  let home = tempfile::TempDir::new().expect( "create isolated home" );
  let _guard = DaemonGuard::for_home( home.path() );

  let started = daemon_in( home.path(), &[ "start" ] );
  assert_eq!( exit_code( &started ), 0, "start failed: {}", stderr_str( &started ) );

  let log = std::fs::read_to_string( home.path().join( ".claude/-daemon/daemon.log" ) )
    .expect( "the daemon wrote no log" );
  let pid = daemon_pid_from( &log ).unwrap_or_else( || panic!( "no pid in the log: {log}" ) );

  let stat = std::fs::read_to_string( format!( "/proc/{pid}/stat" ) )
    .unwrap_or_else( | error | panic!( "pid {pid} is not running: {error}" ) );

  // `comm` is parenthesised and may itself contain spaces and parens, so the
  // numeric fields start after its *last* closing paren, never at a fixed index.
  let tail = &stat[ stat.rfind( ')' ).expect( "malformed stat line" ) + 1.. ];
  let fields : Vec< &str > = tail.split_whitespace().collect();
  let group : u32 = fields.get( 2 )
    .expect( "stat has no pgrp field" )
    .parse()
    .expect( "pgrp is not a number" );

  assert_eq!( group, pid, "the daemon shares its starter's process group" );
}

/// Pull the daemon's pid out of its own `listening on ... (pid N)` line.
#[ cfg( target_os = "linux" ) ]
fn daemon_pid_from( log : &str ) -> Option< u32 >
{
  let line = log.lines().find( | line | line.contains( "listening on" ) )?;
  let digits = line.rsplit_once( "(pid " )?.1;
  digits.trim_end_matches( ')' ).trim().parse().ok()
}
