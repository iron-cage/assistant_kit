//! Integration tests: CLI invocation telemetry (task 470).
//!
//! Covers Test Matrix T01-T06: `run_cli()` appends one redacted `Command` event
//! to the `claude_journal` log per invocation, resolving the journal directory via
//! `CLR_JOURNAL_DIR` (if set) else `~/.clr/journal`, and never lets a journal write
//! failure affect the underlying command's own exit code.
//!
//! ## Test Matrix
//!
//! | id  | condition                                     | expected                                          |
//! |-----|-----------------------------------------------|----------------------------------------------------|
//! | T01 | successful invocation                          | one `Command` event, correct user/host/args/exit_code/duration_ms |
//! | T02 | sensitive argument (`token::...`)              | `args` field has the value redacted                |
//! | T03 | `CLR_JOURNAL_DIR` unwritable (blocked by a file) | command still exits 0, no panic                   |
//! | T04 | `CLR_JOURNAL_DIR` set                          | event lands there, not under the default `HOME`   |
//! | T05 | `CLR_JOURNAL_DIR` unset                        | event lands under `~/.clr/journal`                |
//! | T06 | failing command                                | event still written, with the real non-zero exit_code |
//! | M01 | measurement                                    | `JournalReader::query` count before/after → exactly 1 |

use crate::cli_runner::{ assert_exit, run_cs_with_env, run_cs_with_env_removing, stdout };
use claude_journal::{ EventType, JournalFilter, JournalReader };
use tempfile::TempDir;

fn command_events( journal_dir : &std::path::Path ) -> Vec< claude_journal::EventRecord >
{
  let reader = JournalReader::open( journal_dir.to_path_buf() );
  reader.query( &JournalFilter { event_type : Some( EventType::Command ), ..Default::default() } )
}

/// T01 — a successful invocation appends exactly one `Command` event with all fields populated.
#[ test ]
fn t01_successful_invocation_writes_one_event()
{
  let home = TempDir::new().expect( "tempdir" );
  let out  = run_cs_with_env( &[ ".paths" ], &[ ( "HOME", home.path().to_str().unwrap() ) ] );
  assert_exit( &out, 0 );

  let journal_dir = home.path().join( ".clr" ).join( "journal" );
  let events       = command_events( &journal_dir );
  assert_eq!( events.len(), 1, "expected exactly one Command event" );

  let event = &events[ 0 ];
  assert_eq!( event.fields.exit_code, Some( 0 ) );
  assert!( event.fields.duration_ms.is_some(), "duration_ms must be populated" );
  assert!( event.fields.user.as_deref().is_some_and( | u | !u.is_empty() ), "user must be populated" );
  assert!( event.fields.host.as_deref().is_some_and( | h | !h.is_empty() ), "host must be populated" );
  let args = event.fields.args.as_ref().expect( "args must be populated" );
  assert_eq!( args, &vec![ ".paths".to_string() ] );
}

/// T02 — a sensitive argument value is redacted before it reaches the journal.
#[ test ]
fn t02_sensitive_argument_redacted()
{
  let home = TempDir::new().expect( "tempdir" );
  let _out = run_cs_with_env(
    &[ ".paths", "token::supersecretvalue123" ],
    &[ ( "HOME", home.path().to_str().unwrap() ) ],
  );

  let journal_dir = home.path().join( ".clr" ).join( "journal" );
  let events       = command_events( &journal_dir );
  assert_eq!( events.len(), 1 );

  let args   = events[ 0 ].fields.args.as_ref().expect( "args must be populated" );
  let joined = args.join( " " );
  assert!( !joined.contains( "supersecretvalue123" ), "raw secret leaked into journal: {joined}" );
  assert!( joined.contains( "token::***REDACTED***" ), "expected redacted token marker, got: {joined}" );
}

/// T03 — an unwritable journal directory never breaks the underlying command.
#[ test ]
fn t03_unwritable_journal_dir_does_not_break_command()
{
  let tmp  = TempDir::new().expect( "tempdir" );
  let home = tmp.path().join( "home" );
  std::fs::create_dir_all( &home ).unwrap();

  // A regular file blocks any subpath under it from being created as a directory.
  let blocker = tmp.path().join( "blocker" );
  std::fs::write( &blocker, b"not a directory" ).unwrap();
  let bad_journal_dir = blocker.join( "journal" );

  let out = run_cs_with_env(
    &[ ".paths" ],
    &[ ( "HOME", home.to_str().unwrap() ), ( "CLR_JOURNAL_DIR", bad_journal_dir.to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  assert!( !stdout( &out ).is_empty(), "command output must be unaffected by a journal write failure" );
}

/// T04 — `CLR_JOURNAL_DIR`, when set, is used instead of the `~/.clr/journal` default.
#[ test ]
fn t04_clr_journal_dir_set_routes_there()
{
  let journal_tmp = TempDir::new().expect( "tempdir" );
  let home_tmp    = TempDir::new().expect( "tempdir" ); // distinct dir — proves default isn't also used

  let out = run_cs_with_env(
    &[ ".paths" ],
    &[
      ( "HOME", home_tmp.path().to_str().unwrap() ),
      ( "CLR_JOURNAL_DIR", journal_tmp.path().to_str().unwrap() ),
    ],
  );
  assert_exit( &out, 0 );

  let events = command_events( journal_tmp.path() );
  assert_eq!( events.len(), 1, "event must land in CLR_JOURNAL_DIR" );

  let default_dir = home_tmp.path().join( ".clr" ).join( "journal" );
  assert!( !default_dir.exists(), "default directory must not be used when CLR_JOURNAL_DIR is set" );
}

/// T05 — with `CLR_JOURNAL_DIR` genuinely unset, the event lands under `~/.clr/journal`.
#[ test ]
fn t05_clr_journal_dir_unset_routes_to_home_default()
{
  let home = TempDir::new().expect( "tempdir" );
  let out  = run_cs_with_env_removing(
    &[ ".paths" ],
    &[ ( "HOME", home.path().to_str().unwrap() ) ],
    &[ "CLR_JOURNAL_DIR" ],
  );
  assert_exit( &out, 0 );

  let journal_dir = home.path().join( ".clr" ).join( "journal" );
  let events       = command_events( &journal_dir );
  assert_eq!( events.len(), 1 );
}

/// T06 — a failing invocation still logs an event, with its real non-zero exit code.
#[ test ]
fn t06_failing_command_still_logged()
{
  let home = TempDir::new().expect( "tempdir" );
  let out  = run_cs_with_env( &[ ".nonexistent_command_xyz" ], &[ ( "HOME", home.path().to_str().unwrap() ) ] );
  let actual_exit = out.status.code().unwrap_or( -1 );
  assert_ne!( actual_exit, 0, "test requires a genuinely failing invocation" );

  let journal_dir = home.path().join( ".clr" ).join( "journal" );
  let events       = command_events( &journal_dir );
  assert_eq!( events.len(), 1 );
  assert_eq!( events[ 0 ].fields.exit_code, Some( actual_exit ) );
}

/// M01 — `JournalReader::query` count goes from 0 to exactly 1 across a single invocation.
#[ test ]
fn m01_exactly_one_event_per_invocation()
{
  let home        = TempDir::new().expect( "tempdir" );
  let journal_dir = home.path().join( ".clr" ).join( "journal" );

  let before = command_events( &journal_dir ).len();
  assert_eq!( before, 0, "journal must start empty" );

  let out = run_cs_with_env( &[ ".paths" ], &[ ( "HOME", home.path().to_str().unwrap() ) ] );
  assert_exit( &out, 0 );

  let after = command_events( &journal_dir ).len();
  assert_eq!( after, before + 1, "exactly one Command event must be appended per invocation" );
}
