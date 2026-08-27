//! `clr sessions` — listing what the daemon is hosting.
//!
//! Every case here runs against an injected `HOME`, so the daemon these tests
//! start is theirs and not the developer's. The guard shuts it down whatever the
//! outcome — a leaked daemon holding a lock under a deleted temp directory is a
//! confusing thing to inherit in the next test run.
//!
//! What is deliberately *not* tested here is a listing with sessions in it.
//! Filling one needs a real `claude` on `PATH` answering on a real terminal,
//! which is an end-to-end concern rather than a CLI one; the daemon's own
//! `serve_test.rs` covers the table with real children in it.
//!
//! ## Specification References
//!
//! - `docs/cli/command/15_sessions.md` — command contract
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | SC-1 | `clr sessions help` | Usage, all three forms, exit 0 |
//! | SC-2 | Unknown option | Names it, exit 1 |
//! | SC-3 | No daemon running | Says so on stderr, exit 0 |
//! | SC-4 | No daemon running, `--json` | `[]` on stdout, exit 0 |
//! | SC-5 | No daemon running | Nothing on stdout to be counted |
//! | SC-6 | Daemon running, no sessions | "No hosted sessions.", exit 0 |
//! | SC-7 | Daemon running, no sessions, `--json` | Empty JSON array, exit 0 |
//! | SC-8 | Listing does not start a daemon | Socket still absent afterwards |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ DaemonGuard, exit_code, run_cli_with_env, stderr_str, stdout_str };

/// Run a `clr` subcommand against an injected home.
fn in_home( home : &std::path::Path, args : &[ &str ] ) -> std::process::Output
{
  let home = home.to_str().expect( "home path is not UTF-8" );
  run_cli_with_env( args, &[ ( "HOME", home ) ] )
}

/// SC-1: help names every form the command takes.
#[ test ]
fn sc1_help_lists_every_form()
{
  let home = tempfile::tempdir().expect( "tempdir" );
  let out = in_home( home.path(), &[ "sessions", "help" ] );

  assert_eq!( exit_code( &out ), 0, "help must succeed" );
  let stdout = stdout_str( &out );
  for usage in [ "clr sessions", "clr sessions --json", "clr sessions help" ]
  {
    assert!( stdout.contains( usage ), "help must document {usage:?}. Got:\n{stdout}" );
  }
}

/// SC-2: an option that does not exist is named rather than ignored.
#[ test ]
fn sc2_unknown_option_is_rejected()
{
  let home = tempfile::tempdir().expect( "tempdir" );
  let out = in_home( home.path(), &[ "sessions", "--everything" ] );

  assert_eq!( exit_code( &out ), 1, "an unknown option must fail" );
  let stderr = stderr_str( &out );
  assert!( stderr.contains( "--everything" ), "the rejection must name the option. Got:\n{stderr}" );
}

/// SC-3: nothing hosted is an answer, not a failure.
#[ test ]
fn sc3_no_daemon_is_not_an_error()
{
  let home = tempfile::tempdir().expect( "tempdir" );
  let out = in_home( home.path(), &[ "sessions" ] );

  assert_eq!( exit_code( &out ), 0, "no daemon is a complete answer, not a failure" );
  let stderr = stderr_str( &out );
  assert!
  (
    stderr.contains( "No session daemon is running" ),
    "must say why the list is empty. Got:\n{stderr}"
  );
}

/// SC-4: `--json` still produces parseable JSON when there is no daemon.
///
/// A consumer piping this into a parser should not have to special-case the
/// daemon being down — an empty array is the right shape for "nothing hosted".
#[ test ]
fn sc4_no_daemon_json_is_an_empty_array()
{
  let home = tempfile::tempdir().expect( "tempdir" );
  let out = in_home( home.path(), &[ "sessions", "--json" ] );

  assert_eq!( exit_code( &out ), 0 );
  let parsed : serde_json::Value =
    serde_json::from_str( stdout_str( &out ).trim() ).expect( "stdout must be JSON" );
  assert_eq!( parsed, serde_json::json!( [] ), "must be an empty array" );
}

/// SC-5: the explanation goes to stderr, so stdout stays countable.
///
/// `clr sessions | wc -l` should report zero sessions and not one line of prose.
#[ test ]
fn sc5_no_daemon_leaves_stdout_empty()
{
  let home = tempfile::tempdir().expect( "tempdir" );
  let out = in_home( home.path(), &[ "sessions" ] );

  assert!( stdout_str( &out ).trim().is_empty(), "stdout must carry no rows" );
}

/// SC-6: a running daemon with nothing in it says so plainly.
#[ test ]
fn sc6_running_daemon_with_no_sessions()
{
  let home = tempfile::tempdir().expect( "tempdir" );
  let _guard = DaemonGuard::for_home( home.path() );

  let started = in_home( home.path(), &[ "daemon", "start" ] );
  assert_eq!( exit_code( &started ), 0, "daemon must start. stderr:\n{}", stderr_str( &started ) );

  let out = in_home( home.path(), &[ "sessions" ] );
  assert_eq!( exit_code( &out ), 0 );
  assert!
  (
    stdout_str( &out ).contains( "No hosted sessions." ),
    "an empty daemon must say so. Got:\n{}",
    stdout_str( &out )
  );
}

/// SC-7: the same, in JSON.
#[ test ]
fn sc7_running_daemon_json_is_an_empty_array()
{
  let home = tempfile::tempdir().expect( "tempdir" );
  let _guard = DaemonGuard::for_home( home.path() );

  let started = in_home( home.path(), &[ "daemon", "start" ] );
  assert_eq!( exit_code( &started ), 0, "daemon must start. stderr:\n{}", stderr_str( &started ) );

  let out = in_home( home.path(), &[ "sessions", "--json" ] );
  assert_eq!( exit_code( &out ), 0 );
  let parsed : serde_json::Value =
    serde_json::from_str( stdout_str( &out ).trim() ).expect( "stdout must be JSON" );
  assert_eq!( parsed, serde_json::json!( [] ) );
}

/// SC-8: asking what is hosted does not change what is hosted.
///
/// The distinction from `clr chat`, which does auto-start. A question that
/// starts a process to answer itself has changed the thing it was asking about.
#[ test ]
fn sc8_listing_does_not_start_a_daemon()
{
  let home = tempfile::tempdir().expect( "tempdir" );
  let _guard = DaemonGuard::for_home( home.path() );

  let out = in_home( home.path(), &[ "sessions" ] );
  assert_eq!( exit_code( &out ), 0 );

  // The path the daemon binds. Absent means nothing was started to answer this.
  let socket = home.path().join( ".claude" ).join( "-daemon" ).join( "daemon.sock" );
  assert!( !socket.exists(), "listing must not have started a daemon at {}", socket.display() );
}
